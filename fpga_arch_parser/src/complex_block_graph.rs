use std::collections::HashMap;
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
    pub name: String,
    pub parent_complex_block: ComplexBlockNodeId,
    pub children_complex_blocks: Vec<ComplexBlockNodeId>,
    pub interconnect: Vec<ComplexBlockNet>,
}

pub struct ComplexBlockPrimitiveInfo {
    pub blif_model: String,
    pub class: PBTypeClass,
}

pub struct ComplexBlockNode {
    pub name: String,
    pub parent_mode: Option<ComplexBlockModeId>,
    pub modes: Vec<ComplexBlockModeId>,
    pub primitive_info: Option<ComplexBlockPrimitiveInfo>,
    pub input_ports: Vec<ComplexBlockPortId>,
    pub output_ports: Vec<ComplexBlockPortId>,
    pub clock_ports: Vec<ComplexBlockPortId>,
}

pub struct ComplexBlockPort {
    pub name: String,
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

    pub complex_block_nodes: Vec<ComplexBlockNode>,
    pub complex_block_modes: Vec<ComplexBlockMode>,
    pub complex_block_ports: Vec<ComplexBlockPort>,
    pub complex_block_pins: Vec<ComplexBlockPin>,
}

pub fn build_complex_block_graph(
    root_pb_type: &PBType,
) -> Result<ComplexBlockGraph, FPGAArchParseError> {
    // Traverse the pb-type heirarchy using a child-first-order traversal, building the children complex blocks first
    // and then constructing their parents.
    let mut nodes: Vec<ComplexBlockNode> = Vec::new();
    let mut modes: Vec<ComplexBlockMode> = Vec::new();
    let mut ports: Vec<ComplexBlockPort> = Vec::new();
    let mut pins: Vec<ComplexBlockPin> = Vec::new();

    let root_id = add_pb_type_recursive(
        root_pb_type,
        None,
        &mut nodes,
        &mut modes,
        &mut ports,
        &mut pins,
    )?;

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
) -> Result<ComplexBlockNodeId, FPGAArchParseError> {
    // Reserve the node slot so its ID is known before processing children.
    let node_id = ComplexBlockNodeId(nodes.len());
    nodes.push(ComplexBlockNode {
        name: pb_type.name.clone(),
        parent_mode,
        modes: Vec::new(),
        primitive_info: None,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
        clock_ports: Vec::new(),
    });

    // Build ports and pins for this node.
    for port in &pb_type.ports {
        let (port_name, num_pins) = match port {
            Port::Input(p) => (p.name.clone(), p.num_pins),
            Port::Output(p) => (p.name.clone(), p.num_pins),
            Port::Clock(p) => (p.name.clone(), p.num_pins),
        };
        let port_id = ComplexBlockPortId(ports.len());
        let pin_ids: Vec<ComplexBlockPinId> = (0..num_pins as usize)
            .map(|_| {
                let pin_id = ComplexBlockPinId(pins.len());
                pins.push(ComplexBlockPin {
                    parent_port: port_id,
                });
                pin_id
            })
            .collect();
        ports.push(ComplexBlockPort {
            name: port_name,
            parent_complex_block: node_id,
            pins: pin_ids,
        });
        match port {
            Port::Input(_) => nodes[node_id].input_ports.push(port_id),
            Port::Output(_) => nodes[node_id].output_ports.push(port_id),
            Port::Clock(_) => nodes[node_id].clock_ports.push(port_id),
        }
    }

    // Collect (mode_id, child pb_types, interconnects) so we can build modes child-first.
    // A pb_type has either explicit modes or direct pb_type children (implicit default mode).
    let mode_sources: Vec<(ComplexBlockModeId, &[PBType], &[Interconnect])> =
        if !pb_type.modes.is_empty() {
            pb_type
                .modes
                .iter()
                .map(|mode| {
                    let mode_id = ComplexBlockModeId(modes.len());
                    modes.push(ComplexBlockMode {
                        name: mode.name.clone(),
                        parent_complex_block: node_id,
                        children_complex_blocks: Vec::new(),
                        interconnect: Vec::new(),
                    });
                    (
                        mode_id,
                        mode.pb_types.as_slice(),
                        mode.interconnects.as_slice(),
                    )
                })
                .collect()
        } else if !pb_type.pb_types.is_empty() {
            // Implicit single default mode for pb_types with direct children but no named modes.
            let mode_id = ComplexBlockModeId(modes.len());
            modes.push(ComplexBlockMode {
                name: pb_type.name.clone(),
                parent_complex_block: node_id,
                children_complex_blocks: Vec::new(),
                interconnect: Vec::new(),
            });
            vec![(
                mode_id,
                pb_type.pb_types.as_slice(),
                pb_type.interconnects.as_slice(),
            )]
        } else {
            Vec::new()
        };

    // Recurse into children and fill in each mode's children list.
    // Each child pb_type is instantiated num_pb times, producing independent nodes.
    for (mode_id, children, interconnects) in mode_sources {
        let mut child_ids: Vec<ComplexBlockNodeId> = Vec::new();
        let mut children_by_name: HashMap<String, Vec<ComplexBlockNodeId>> = HashMap::new();
        for child in children {
            for _ in 0..child.num_pb as usize {
                let child_id =
                    add_pb_type_recursive(child, Some(mode_id), nodes, modes, ports, pins)?;
                child_ids.push(child_id);
                children_by_name
                    .entry(child.name.clone())
                    .or_default()
                    .push(child_id);
            }
        }
        let mut mode_nets: Vec<ComplexBlockNet> = Vec::new();
        for interconnect in interconnects {
            let (inter_id, nets) = build_interconnect_node_and_nets(
                interconnect,
                mode_id,
                node_id,
                &pb_type.name,
                &children_by_name,
                nodes,
                ports,
                pins,
            )?;
            child_ids.push(inter_id);
            mode_nets.extend(nets);
        }
        modes[mode_id].children_complex_blocks = child_ids;
        modes[mode_id].interconnect = mode_nets;
        nodes[node_id].modes.push(mode_id);
    }

    nodes[node_id].primitive_info =
        pb_type
            .blif_model
            .as_ref()
            .map(|blif_model| ComplexBlockPrimitiveInfo {
                blif_model: blif_model.clone(),
                class: pb_type.class.clone(),
            });

    Ok(node_id)
}

type NameWithRange<'a> = (&'a str, Option<(usize, usize)>);

// Parses "name", "name[idx]", or "name[high:low]" into (name, range).
// The range preserves order: (3, 1) means high-to-low, (1, 3) means low-to-high.
fn parse_name_with_range(s: &str) -> Result<NameWithRange<'_>, FPGAArchParseError> {
    match s.find('[') {
        None => Ok((s, None)),
        Some(open) => {
            let name = &s[..open];
            let close = s.find(']').unwrap_or(s.len());
            let inner = &s[open + 1..close];
            let range = if let Some(colon) = inner.find(':') {
                let a: usize = inner[..colon].trim().parse().map_err(|_| {
                    FPGAArchParseError::PinParsingError(format!(
                        "Failed to parse index '{}' in range expression '[{}]'",
                        inner[..colon].trim(),
                        inner
                    ))
                })?;
                let b: usize = inner[colon + 1..].trim().parse().map_err(|_| {
                    FPGAArchParseError::PinParsingError(format!(
                        "Failed to parse index '{}' in range expression '[{}]'",
                        inner[colon + 1..].trim(),
                        inner
                    ))
                })?;
                (a, b)
            } else {
                let a: usize = inner.trim().parse().map_err(|_| {
                    FPGAArchParseError::PinParsingError(format!(
                        "Failed to parse index '{}' in '[{}]'",
                        inner.trim(),
                        inner
                    ))
                })?;
                (a, a)
            };
            Ok((name, Some(range)))
        }
    }
}

type PortRef<'a> = (
    &'a str,
    Option<(usize, usize)>,
    &'a str,
    Option<(usize, usize)>,
);

// Parses "block[range].port[range]" into (block_name, inst_range, port_name, bit_range).
fn parse_port_ref(s: &str) -> Result<PortRef<'_>, FPGAArchParseError> {
    let dot = s.find('.').unwrap_or(s.len());
    let (block_name, inst_range): NameWithRange<'_> = parse_name_with_range(&s[..dot])?;
    let (port_name, bit_range): NameWithRange<'_> = if dot < s.len() {
        parse_name_with_range(&s[dot + 1..])?
    } else {
        ("", None)
    };
    Ok((block_name, inst_range, port_name, bit_range))
}

// Resolves a single port-reference token to the ComplexBlockPinIds it names.
// Preserves the order implied by any ranges (e.g. [3:1] yields pins 3, 2, 1).
fn resolve_pins_for_ref(
    ref_str: &str,
    parent_pb_name: &str,
    parent_node_id: ComplexBlockNodeId,
    children_by_name: &HashMap<String, Vec<ComplexBlockNodeId>>,
    nodes: &[ComplexBlockNode],
    ports: &[ComplexBlockPort],
) -> Result<Vec<ComplexBlockPinId>, FPGAArchParseError> {
    let (block_name, inst_range, port_name, bit_range) = parse_port_ref(ref_str)?;
    if port_name.is_empty() {
        return Err(FPGAArchParseError::PinParsingError(format!(
            "No port name in reference '{}' (missing '.')",
            ref_str
        )));
    }

    // Collect the node IDs indicated by the block reference.
    let node_ids: Vec<ComplexBlockNodeId> = if block_name == parent_pb_name {
        vec![parent_node_id]
    } else {
        match children_by_name.get(block_name) {
            None => {
                return Err(FPGAArchParseError::PinParsingError(format!(
                    "Unknown block '{}' in interconnect reference '{}'",
                    block_name, ref_str
                )));
            }
            Some(instances) => match inst_range {
                None => instances.clone(),
                Some((a, b)) => {
                    let lo = a.min(b);
                    let hi = a.max(b);
                    // Validate that all requested indices exist.
                    if hi >= instances.len() {
                        return Err(FPGAArchParseError::PinParsingError(format!(
                            "Instance index {} out of range for block '{}' (has {} instance(s)) in reference '{}'",
                            hi,
                            block_name,
                            instances.len(),
                            ref_str
                        )));
                    }
                    if a > b {
                        (lo..=hi)
                            .rev()
                            .filter_map(|i| instances.get(i).copied())
                            .collect()
                    } else {
                        (lo..=hi)
                            .filter_map(|i| instances.get(i).copied())
                            .collect()
                    }
                }
            },
        }
    };

    // For each node find the named port, then collect the indicated pins.
    // Use .0 for plain slice indexing since the custom Index trait is Vec-only.
    let mut result = Vec::new();
    for nid in node_ids {
        let node = &nodes[nid.0];
        let port_id = node
            .input_ports
            .iter()
            .chain(node.output_ports.iter())
            .chain(node.clock_ports.iter())
            .find(|&&pid| ports[pid.0].name == port_name)
            .copied();
        let pid = port_id.ok_or_else(|| {
            FPGAArchParseError::PinParsingError(format!(
                "Port '{}' not found on block '{}' in reference '{}'",
                port_name, block_name, ref_str
            ))
        })?;
        let port_pins = &ports[pid.0].pins;
        match bit_range {
            None => result.extend_from_slice(port_pins),
            Some((a, b)) => {
                let lo = a.min(b);
                let hi = a.max(b);
                if hi >= port_pins.len() {
                    return Err(FPGAArchParseError::PinParsingError(format!(
                        "Bit index {} out of range for port '{}' on block '{}' (has {} pin(s)) in reference '{}'",
                        hi,
                        port_name,
                        block_name,
                        port_pins.len(),
                        ref_str
                    )));
                }
                if a > b {
                    result.extend(port_pins[lo..=hi].iter().rev().copied());
                } else {
                    result.extend_from_slice(&port_pins[lo..=hi]);
                }
            }
        }
    }
    Ok(result)
}

fn build_interconnect_node_and_nets(
    interconnect: &Interconnect,
    parent_mode_id: ComplexBlockModeId,
    parent_node_id: ComplexBlockNodeId,
    parent_pb_name: &str,
    children_by_name: &HashMap<String, Vec<ComplexBlockNodeId>>,
    nodes: &mut Vec<ComplexBlockNode>,
    ports: &mut Vec<ComplexBlockPort>,
    pins: &mut Vec<ComplexBlockPin>,
) -> Result<(ComplexBlockNodeId, Vec<ComplexBlockNet>), FPGAArchParseError> {
    let class = match interconnect.interconnect_type {
        InterconnectType::Direct => PBTypeClass::InterconnectDirect,
        InterconnectType::Mux => PBTypeClass::InterconnectMux,
        InterconnectType::Complete => PBTypeClass::InterconnectComplete,
    };

    // Resolve all pin references before mutating nodes/ports.
    // Both input and output may contain multiple whitespace-separated port references
    // (e.g. complete interconnects can fan out to multiple output ports).
    let input_groups: Vec<&str> = interconnect.input.split_whitespace().collect();
    let mut input_pin_groups: Vec<Vec<ComplexBlockPinId>> = Vec::new();
    for &group in &input_groups {
        input_pin_groups.push(resolve_pins_for_ref(
            group,
            parent_pb_name,
            parent_node_id,
            children_by_name,
            nodes,
            ports,
        )?);
    }
    let mut output_pins: Vec<ComplexBlockPinId> = Vec::new();
    for token in interconnect.output.split_whitespace() {
        output_pins.extend(resolve_pins_for_ref(
            token,
            parent_pb_name,
            parent_node_id,
            children_by_name,
            nodes,
            ports,
        )?);
    }

    // Create the interconnect node.
    let node_id = ComplexBlockNodeId(nodes.len());
    nodes.push(ComplexBlockNode {
        name: interconnect.name.clone(),
        parent_mode: Some(parent_mode_id),
        modes: Vec::new(),
        primitive_info: None,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
        clock_ports: Vec::new(),
    });

    let num_input_groups = input_pin_groups.len();
    let mut input_port_ids: Vec<ComplexBlockPortId> = Vec::new();
    let mut nets: Vec<ComplexBlockNet> = Vec::new();

    // One input port per group; each source pin gets a paired pin on the port and a connecting net.
    for (group_idx, source_pins) in input_pin_groups.into_iter().enumerate() {
        let port_id = ComplexBlockPortId(ports.len());
        let mut port_pin_ids = Vec::new();
        for source_pin in source_pins {
            let inter_pin = ComplexBlockPinId(pins.len());
            pins.push(ComplexBlockPin {
                parent_port: port_id,
            });
            port_pin_ids.push(inter_pin);
            nets.push(ComplexBlockNet {
                pins: vec![source_pin, inter_pin],
            });
        }
        let port_name = if num_input_groups == 1 {
            "input".to_string()
        } else {
            format!("input_{}", group_idx)
        };
        ports.push(ComplexBlockPort {
            name: port_name,
            parent_complex_block: node_id,
            pins: port_pin_ids,
        });
        input_port_ids.push(port_id);
    }

    // Single output port; each sink pin gets a paired pin on the port and a connecting net.
    let output_port_id = ComplexBlockPortId(ports.len());
    let mut output_pin_ids = Vec::new();
    for sink_pin in output_pins {
        let inter_pin = ComplexBlockPinId(pins.len());
        pins.push(ComplexBlockPin {
            parent_port: output_port_id,
        });
        output_pin_ids.push(inter_pin);
        nets.push(ComplexBlockNet {
            pins: vec![inter_pin, sink_pin],
        });
    }
    ports.push(ComplexBlockPort {
        name: "output".to_string(),
        parent_complex_block: node_id,
        pins: output_pin_ids,
    });

    nodes[node_id].primitive_info = Some(ComplexBlockPrimitiveInfo {
        blif_model: interconnect.name.clone(),
        class,
    });
    nodes[node_id].input_ports = input_port_ids;
    nodes[node_id].output_ports = vec![output_port_id];

    Ok((node_id, nets))
}
