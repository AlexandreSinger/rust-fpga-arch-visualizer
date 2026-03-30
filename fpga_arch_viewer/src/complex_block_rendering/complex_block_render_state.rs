use fpga_arch_parser::ComplexBlockGraph;

pub struct ComplexBlockRenderState {
    pub dark_mode: bool,

    // If true, the given port is not bit-blasted per-pin.
    //      [ComplexBlockPinId] -> bool
    is_port_collapsed: Vec<bool>,

    // If true, the given complex block is collapsed.
    //      [ComplexBlockNodeId] -> bool
    is_complex_block_collapsed: Vec<bool>,

    pin_locations: Vec<egui::Pos2>,

    port_locations: Vec<egui::Pos2>,

    complex_block_size: Vec<egui::Vec2>,
}

pub fn create_complex_block_render_state(
    complex_block_graph: &ComplexBlockGraph,
    dark_mode: bool,
) -> ComplexBlockRenderState {

    ComplexBlockRenderState {
        dark_mode,
        is_port_collapsed: vec![true; complex_block_graph.num_ports()],
        is_complex_block_collapsed: vec![true; complex_block_graph.num_complex_blocks()],
        pin_locations: vec![egui::Pos2::ZERO; complex_block_graph.num_pins()],
        port_locations: vec![egui::Pos2::ZERO; complex_block_graph.num_ports()],
        complex_block_size: vec![egui::Vec2::ZERO; complex_block_graph.num_complex_blocks()],
    }
}