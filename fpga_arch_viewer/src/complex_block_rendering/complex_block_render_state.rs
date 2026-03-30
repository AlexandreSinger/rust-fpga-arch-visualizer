use fpga_arch_parser::{
    ComplexBlockGraph, ComplexBlockNodeId, ComplexBlockPinId, ComplexBlockPortId, StrongIdVec,
};

pub struct ComplexBlockRenderState {
    pub dark_mode: bool,

    pub zoom: f32,

    pub spacing_between_io: f32,

    // If true, the given port is not bit-blasted per-pin.
    pub is_port_collapsed: StrongIdVec<ComplexBlockPortId, bool>,

    // If true, the given complex block is collapsed.
    pub is_complex_block_collapsed: StrongIdVec<ComplexBlockNodeId, bool>,

    pub pin_locations: StrongIdVec<ComplexBlockPinId, egui::Pos2>,

    pub port_locations: StrongIdVec<ComplexBlockPortId, egui::Pos2>,

    pub complex_block_size: StrongIdVec<ComplexBlockNodeId, egui::Vec2>,
}

pub fn create_complex_block_render_state(
    complex_block_graph: &ComplexBlockGraph,
    dark_mode: bool,
    zoom: f32,
    spacing_between_io: f32,
) -> ComplexBlockRenderState {

    ComplexBlockRenderState {
        dark_mode,
        zoom,
        spacing_between_io,
        is_port_collapsed: StrongIdVec::new(vec![true; complex_block_graph.num_ports()]),
        is_complex_block_collapsed: StrongIdVec::new(vec![true; complex_block_graph.num_complex_blocks()]),
        pin_locations: StrongIdVec::new(vec![egui::Pos2::ZERO; complex_block_graph.num_pins()]),
        port_locations: StrongIdVec::new(vec![egui::Pos2::ZERO; complex_block_graph.num_ports()]),
        complex_block_size: StrongIdVec::new(vec![egui::Vec2::ZERO; complex_block_graph.num_complex_blocks()]),
    }
}