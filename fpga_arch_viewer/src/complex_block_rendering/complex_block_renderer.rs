use fpga_arch_parser::{ComplexBlockGraph, ComplexBlockModeId, ComplexBlockNodeId};

use crate::complex_block_rendering::complex_block_render_state::ComplexBlockRenderState;

// This will draw the mode (i.e. the contents of the block).
fn render_selected_mode(
    mode_id: ComplexBlockModeId,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
) -> Vec<egui::Shape> {
    // Use a levelization scheme to place the complex blocks within the mode.

    Vec::new()
}

// This will draw the overall complex block (i.e. bar on top & mode select)
pub fn render_complex_block(
    complex_block_id: ComplexBlockNodeId,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
) -> Vec<egui::Shape> {
    Vec::new()
}