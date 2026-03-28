
use std::ops::{Index, IndexMut};

use crate::{FPGAArchParseError, PBType, PBTypeClass};

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

    let root_id = add_pb_type_recursive(root_pb_type, None, &mut nodes, &mut modes);

    Ok(ComplexBlockGraph {
        root_complex_block_node: root_id,
        complex_block_nodes: nodes,
        complex_block_modes: modes,
        complex_block_ports: Vec::new(),
        complex_block_pins: Vec::new(),
    })
}

fn add_pb_type_recursive(
    pb_type: &PBType,
    parent_mode: Option<ComplexBlockModeId>,
    nodes: &mut Vec<ComplexBlockNode>,
    modes: &mut Vec<ComplexBlockMode>,
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

    // Collect (mode_id, child pb_types) so we can build modes child-first.
    // A pb_type has either explicit modes or direct pb_type children (implicit default mode).
    let mode_sources: Vec<(ComplexBlockModeId, &Vec<PBType>)> = if !pb_type.modes.is_empty() {
        pb_type.modes.iter().map(|mode| {
            let mode_id = ComplexBlockModeId(modes.len());
            modes.push(ComplexBlockMode {
                parent_complex_block: node_id,
                children_complex_blocks: Vec::new(),
                interconnect: Vec::new(),
            });
            (mode_id, &mode.pb_types)
        }).collect()
    } else if !pb_type.pb_types.is_empty() {
        // Implicit single default mode for pb_types with direct children but no named modes.
        let mode_id = ComplexBlockModeId(modes.len());
        modes.push(ComplexBlockMode {
            parent_complex_block: node_id,
            children_complex_blocks: Vec::new(),
            interconnect: Vec::new(),
        });
        vec![(mode_id, &pb_type.pb_types)]
    } else {
        Vec::new()
    };

    // Recurse into children and fill in each mode's children list.
    let mut mode_ids: Vec<ComplexBlockModeId> = Vec::new();
    for (mode_id, children) in mode_sources {
        let child_ids: Vec<ComplexBlockNodeId> = children
            .iter()
            .map(|child| add_pb_type_recursive(child, Some(mode_id), nodes, modes))
            .collect();
        modes[mode_id].children_complex_blocks = child_ids;
        mode_ids.push(mode_id);
    }

    nodes[node_id].modes = mode_ids;
    nodes[node_id].primitive_info = pb_type.blif_model.as_ref().map(|blif_model| {
        ComplexBlockPrimitiveInfo {
            blif_model: blif_model.clone(),
            class: pb_type.class.clone(),
        }
    });

    node_id
}