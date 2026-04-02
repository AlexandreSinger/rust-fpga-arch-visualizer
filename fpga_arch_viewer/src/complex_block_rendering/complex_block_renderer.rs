use std::collections::HashMap;

use fpga_arch_parser::{ComplexBlockGraph, ComplexBlockModeId, ComplexBlockNodeId};

use crate::{color_scheme, complex_block_rendering::{complex_block_render_state::ComplexBlockRenderState, primitive_renderer::render_primitive}};

const HEADER_HEIGHT: f32 = 35.0;

// This will draw the mode (i.e. the contents of the block).
fn render_selected_mode(
    mode_id: ComplexBlockModeId,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> (Vec<egui::Shape>, egui::Vec2) {
    let mut mode_shapes = Vec::new();
    let mode = &complex_block_graph.complex_block_modes[mode_id];

    // Render and get the size of all blocks in the mode.
    let mut child_complex_block_shapes = HashMap::new();
    for child_complex_block_id in &mode.children_complex_blocks {
        child_complex_block_shapes.insert(*child_complex_block_id, render_complex_block(*child_complex_block_id, complex_block_graph, state, egui_ctx));
    }

    // For now lets just place the shapes in a grid.
    let mut child_complex_block_offsets = HashMap::new();
    let mut current_offset = egui::Vec2::ZERO;
    let mut max_width: f32 = 0.0;
    for child_complex_block_id in &mode.children_complex_blocks {
        child_complex_block_offsets.insert(*child_complex_block_id, current_offset.clone());
        let child_complex_block_size = child_complex_block_shapes[child_complex_block_id].1.clone();
        current_offset += egui::vec2(0.0, child_complex_block_size.y);
        max_width = max_width.max(child_complex_block_size.x);
    }
    let mode_size = egui::vec2(max_width, current_offset.y);
    // TODO: Use a levelization scheme to place the complex blocks within the mode.

    // Add the shapes to the mode
    for child_complex_block_id in &mode.children_complex_blocks {
        let mut child_shapes = child_complex_block_shapes[child_complex_block_id].0.clone();
        for child_shape in &mut child_shapes {
            child_shape.translate(child_complex_block_offsets[child_complex_block_id]);
        }
        mode_shapes.extend(child_shapes);
    }

    (mode_shapes, mode_size)
}

// This will draw the overall complex block (i.e. bar on top & mode select)
// TODO: The return type of this can be made into a more general struct.
pub fn render_complex_block(
    complex_block_id: ComplexBlockNodeId,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> (Vec<egui::Shape>, egui::Vec2) {
    let zoom = state.zoom;
    let mut complex_block_shapes = Vec::new();
    let complex_block = &complex_block_graph.complex_block_nodes[complex_block_id];

    if let Some(primitive_info) = &complex_block.primitive_info {
        return render_primitive(complex_block_id, &primitive_info, egui::Pos2::ZERO, complex_block_graph, state, egui_ctx);
    }

    // Draw the contents (i.e. the mode) and get the size.
    // TODO: Add mode select.
    let selected_mode_id = match complex_block.modes.get(0) {
        Some(id) => id,
        None => {
            panic!("FIXME: HANDLE THIS CASE!");
        }
    };
    let mode_render_info = render_selected_mode(*selected_mode_id, complex_block_graph, state, egui_ctx);
    let complex_block_size = mode_render_info.1 + egui::vec2(0.0, HEADER_HEIGHT * zoom);

    // Draw the background of the complex block.
    let background_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, complex_block_size);
    complex_block_shapes.push(egui::Shape::rect_filled(background_rect, egui::CornerRadius::ZERO, color_scheme::theme_block_bg(state.dark_mode)));
    complex_block_shapes.push(egui::Shape::rect_stroke(background_rect, egui::CornerRadius::ZERO, egui::Stroke::new(1.5 * zoom, color_scheme::theme_border_color(state.dark_mode)), egui::StrokeKind::Inside));

    // Draw the title bar.
    let title_rect = egui::Rect::from_min_size(background_rect.min, egui::vec2(background_rect.width(), HEADER_HEIGHT * zoom));
    complex_block_shapes.push(egui::Shape::rect_filled(title_rect, egui::CornerRadius::ZERO, color_scheme::theme_header_bg(state.dark_mode)));
    egui_ctx.fonts(|fonts| {
        complex_block_shapes.push(egui::Shape::text(fonts, title_rect.min + egui::vec2(5.0, 5.0) * zoom, egui::Align2::LEFT_TOP, &complex_block.name, egui::FontId::proportional(14.0 * zoom), color_scheme::theme_text_color(state.dark_mode)));
    });

    // Push the contents onto the vec to ensure they are drawn last.
    let mut mode_shapes = mode_render_info.0;
    for shape in &mut mode_shapes {
        shape.translate(egui::vec2(0.0, title_rect.height()));
    }
    complex_block_shapes.extend(mode_shapes);

    (complex_block_shapes, complex_block_size)
}