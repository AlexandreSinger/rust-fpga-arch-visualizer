
use std::ops::Index;

use crate::PBTypeClass;

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
    pub parent_mode: ComplexBlockModeId,
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

impl ComplexBlockGraph {
    pub fn get_complex_block(&self, complex_block_node_id: ComplexBlockNodeId) -> &ComplexBlockNode {
        &self.complex_block_nodes[complex_block_node_id]
    }
}