use fpga_arch_parser::{ComplexBlockGraph, ComplexBlockNodeId};

use crate::complex_block_rendering::complex_block_render_state::ComplexBlockRenderState;

pub fn render_primitive(
    complex_block_id: ComplexBlockNodeId,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
) -> Vec<egui::Shape> {
    Vec::new()
}