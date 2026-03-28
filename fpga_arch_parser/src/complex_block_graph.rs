
use std::ops::{Index, IndexMut};

use crate::{FPGAArchParseError, Interconnect, InterconnectType, PBType, PBTypeClass, Port};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComplexBlockNodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComplexBlockPortId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComplexBlockPinId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComplexBlockModeId(usize);

macro_rules! impl_vec_index {
    ($id:ty, $item:ty) => {
        impl Index<$id> for Vec<$item> {
            type Output = $item;
            fn index(&self, id: $id) -> &Self::Output {
                &self[id.0]
            }
        }
        impl IndexMut<$id> for Vec<$item> {
            fn index_mut(&mut self, id: $id) -> &mut Self::Output {
                &mut self[id.0]
            }
        }
    };
}

impl_vec_index!(ComplexBlockNodeId, ComplexBlockNode);
impl_vec_index!(ComplexBlockPortId, ComplexBlockPort);
impl_vec_index!(ComplexBlockPinId, ComplexBlockPin);
impl_vec_index!(ComplexBlockModeId, ComplexBlockMode);

pub struct ComplexBlockMode {
    // FIXME: Primitives probably would make sense as a specialization of the mode.
    pub parent_complex_block: ComplexBlockNodeId,
    pub children_complex_blocks: Vec<ComplexBlockNodeId>,
    pub interconnect: Vec<ComplexBlockNet>,
}

pub struct ComplexBlockPrimitiveInfo {
    pub blif_model: String,
    pub class: PBTypeClass,
}

pub struct ComplexBlockNode {
    pub parent_mode: Option<ComplexBlockModeId>,
    pub modes: Vec<ComplexBlockModeId>,
    pub primitive_info: Option<ComplexBlockPrimitiveInfo>,
    pub input_ports: Vec<ComplexBlockPortId>,
    pub output_ports: Vec<ComplexBlockPortId>,
    pub clock_ports: Vec<ComplexBlockPortId>,
}

pub struct ComplexBlockPort {
    pub parent_complex_block: ComplexBlockNodeId,
    pub pins: Vec<ComplexBlockPinId>,
}

pub struct ComplexBlockPin {
    pub parent_port: ComplexBlockPortId,
}

pub struct ComplexBlockNet {
    pub pins: Vec<ComplexBlockPinId>,
}

pub struct ComplexBlockGraph {
    pub root_complex_block_node: ComplexBlockNodeId,

    complex_block_nodes: Vec<ComplexBlockNode>,
    complex_block_modes: Vec<ComplexBlockMode>,
    complex_block_ports: Vec<ComplexBlockPort>,
    complex_block_pins: Vec<ComplexBlockPin>,
}

pub fn build_complex_block_graph(root_pb_type: &PBType) -> Result<ComplexBlockGraph, FPGAArchParseError> {
    // Traverse the pb-type heirarchy using a child-first-order traversal, building the children complex blocks first
    // and then constructing their parents.
    let mut nodes: Vec<ComplexBlockNode> = Vec::new();
    let mut modes: Vec<ComplexBlockMode> = Vec::new();
    let mut ports: Vec<ComplexBlockPort> = Vec::new();
    let mut pins: Vec<ComplexBlockPin> = Vec::new();

    let root_id = add_pb_type_recursive(root_pb_type, None, &mut nodes, &mut modes, &mut ports, &mut pins);

    Ok(ComplexBlockGraph {
        root_complex_block_node: root_id,
        complex_block_nodes: nodes,
        complex_block_modes: modes,
        complex_block_ports: ports,
        complex_block_pins: pins,
    })
}

fn add_pb_type_recursive(
    pb_type: &PBType,
    parent_mode: Option<ComplexBlockModeId>,
    nodes: &mut Vec<ComplexBlockNode>,
    modes: &mut Vec<ComplexBlockMode>,
    ports: &mut Vec<ComplexBlockPort>,
    pins: &mut Vec<ComplexBlockPin>,
) -> ComplexBlockNodeId {
    // Reserve the node slot so its ID is known before processing children.
    let node_id = ComplexBlockNodeId(nodes.len());
    nodes.push(ComplexBlockNode {
        parent_mode,
        modes: Vec::new(),
        primitive_info: None,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
        clock_ports: Vec::new(),
    });

    // Build ports and pins for this node.
    let mut input_port_ids: Vec<ComplexBlockPortId> = Vec::new();
    let mut output_port_ids: Vec<ComplexBlockPortId> = Vec::new();
    let mut clock_port_ids: Vec<ComplexBlockPortId> = Vec::new();
    for port in &pb_type.ports {
        let num_pins = match port {
            Port::Input(p) => p.num_pins,
            Port::Output(p) => p.num_pins,
            Port::Clock(p) => p.num_pins,
        };
        let port_id = ComplexBlockPortId(ports.len());
        let pin_ids: Vec<ComplexBlockPinId> = (0..num_pins as usize).map(|_| {
            let pin_id = ComplexBlockPinId(pins.len());
            pins.push(ComplexBlockPin { parent_port: port_id });
            pin_id
        }).collect();
        ports.push(ComplexBlockPort { parent_complex_block: node_id, pins: pin_ids });
        match port {
            Port::Input(_)  => input_port_ids.push(port_id),
            Port::Output(_) => output_port_ids.push(port_id),
            Port::Clock(_)  => clock_port_ids.push(port_id),
        }
    }

    // Collect (mode_id, child pb_types, interconnects) so we can build modes child-first.
    // A pb_type has either explicit modes or direct pb_type children (implicit default mode).
    let mode_sources: Vec<(ComplexBlockModeId, &[PBType], &[Interconnect])> = if !pb_type.modes.is_empty() {
        pb_type.modes.iter().map(|mode| {
            let mode_id = ComplexBlockModeId(modes.len());
            modes.push(ComplexBlockMode {
                parent_complex_block: node_id,
                children_complex_blocks: Vec::new(),
                interconnect: Vec::new(),
            });
            (mode_id, mode.pb_types.as_slice(), mode.interconnects.as_slice())
        }).collect()
    } else if !pb_type.pb_types.is_empty() {
        // Implicit single default mode for pb_types with direct children but no named modes.
        let mode_id = ComplexBlockModeId(modes.len());
        modes.push(ComplexBlockMode {
            parent_complex_block: node_id,
            children_complex_blocks: Vec::new(),
            interconnect: Vec::new(),
        });
        vec![(mode_id, pb_type.pb_types.as_slice(), pb_type.interconnects.as_slice())]
    } else {
        Vec::new()
    };

    // Recurse into children and fill in each mode's children list.
    // Each child pb_type is instantiated num_pb times, producing independent nodes.
    let mut mode_ids: Vec<ComplexBlockModeId> = Vec::new();
    for (mode_id, children, interconnects) in mode_sources {
        let mut child_ids: Vec<ComplexBlockNodeId> = Vec::new();
        for child in children {
            for _ in 0..child.num_pb as usize {
                child_ids.push(add_pb_type_recursive(child, Some(mode_id), nodes, modes, ports, pins));
            }
        }
        for interconnect in interconnects {
            child_ids.push(add_interconnect(interconnect, mode_id, nodes, ports));
        }
        modes[mode_id].children_complex_blocks = child_ids;
        mode_ids.push(mode_id);
    }

    nodes[node_id].modes = mode_ids;
    nodes[node_id].input_ports = input_port_ids;
    nodes[node_id].output_ports = output_port_ids;
    nodes[node_id].clock_ports = clock_port_ids;
    nodes[node_id].primitive_info = pb_type.blif_model.as_ref().map(|blif_model| {
        ComplexBlockPrimitiveInfo {
            blif_model: blif_model.clone(),
            class: pb_type.class.clone(),
        }
    });

    node_id
}

fn add_interconnect(
    interconnect: &Interconnect,
    parent_mode: ComplexBlockModeId,
    nodes: &mut Vec<ComplexBlockNode>,
    ports: &mut Vec<ComplexBlockPort>,
) -> ComplexBlockNodeId {
    let node_id = ComplexBlockNodeId(nodes.len());
    nodes.push(ComplexBlockNode {
        parent_mode: Some(parent_mode),
        modes: Vec::new(),
        primitive_info: None,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
        clock_ports: Vec::new(),
    });

    let class = match interconnect.interconnect_type {
        InterconnectType::Direct   => PBTypeClass::InterconnectDirect,
        InterconnectType::Mux      => PBTypeClass::InterconnectMux,
        InterconnectType::Complete => PBTypeClass::InterconnectComplete,
    };

    // Mux has 2 input ports; direct and complete have 1. All types have 1 output port.
    let num_input_ports = match interconnect.interconnect_type {
        InterconnectType::Mux => 2,
        _                     => 1,
    };
    let input_port_ids: Vec<ComplexBlockPortId> = (0..num_input_ports).map(|_| {
        let port_id = ComplexBlockPortId(ports.len());
        ports.push(ComplexBlockPort { parent_complex_block: node_id, pins: Vec::new() });
        port_id
    }).collect();
    let output_port_id = ComplexBlockPortId(ports.len());
    ports.push(ComplexBlockPort { parent_complex_block: node_id, pins: Vec::new() });

    nodes[node_id].primitive_info = Some(ComplexBlockPrimitiveInfo {
        blif_model: interconnect.name.clone(),
        class,
    });
    nodes[node_id].input_ports = input_port_ids;
    nodes[node_id].output_ports = vec![output_port_id];

    node_id
}