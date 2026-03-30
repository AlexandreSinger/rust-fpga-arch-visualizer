use fpga_arch_parser::{ComplexBlockGraph, ComplexBlockNode, ComplexBlockNodeId, ComplexBlockPinId, ComplexBlockPortId, ComplexBlockPrimitiveInfo, PBTypeClass};

use crate::{color_scheme, complex_block_rendering::complex_block_render_state::ComplexBlockRenderState};

const MIN_BLOCK_SIZE: egui::Vec2 = egui::vec2(80.0, 120.0);

// TODO: Move this into a utils method.
fn count_num_visible_io(
    ports: &Vec<ComplexBlockPortId>,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
) -> usize {
    let mut num_visible_ports_pins = 0;
    for port_id in ports {
        if state.is_port_collapsed[*port_id] {
            num_visible_ports_pins += 1;
        } else {
            num_visible_ports_pins += complex_block_graph.complex_block_ports[*port_id].pins.len();
        }
    }

    num_visible_ports_pins
}

fn get_primitive_block_size(
    complex_block: &ComplexBlockNode,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
) -> egui::Vec2 {
    // The size of the primitive is proportional to the number of io that
    // will be drawn (down to a minimum block size).
    let num_input_points = count_num_visible_io(&complex_block.input_ports, complex_block_graph, state);
    let num_output_points = count_num_visible_io(&complex_block.output_ports, complex_block_graph, state);
    let num_clock_points = count_num_visible_io(&complex_block.clock_ports, complex_block_graph, state);
    let prim_height = (num_input_points.max(num_output_points) + 2) as f32 * state.spacing_between_io;
    let prim_width = (num_clock_points + 2) as f32 * state.spacing_between_io;
    let block_size = MIN_BLOCK_SIZE.max(egui::vec2(prim_width, prim_height));

    block_size
}

fn render_primitive_body(
    rect: &egui::Rect,
    zoom: f32,
    bg_color: &egui::Color32,
    border_color: &egui::Color32,
) -> Vec<egui::Shape> {
    vec![
        egui::Shape::rect_filled(*rect, egui::CornerRadius::ZERO, *bg_color),
        egui::Shape::rect_stroke(*rect, egui::CornerRadius::ZERO, egui::Stroke::new(1.5 * zoom, *border_color), egui::epaint::StrokeKind::Inside,)
    ]
}

fn render_primitive_labels(
    rect: &egui::Rect,
    model: &str,
    name: &str,
    zoom: f32,
    text_color: &egui::Color32,
    dark_mode: bool,
    egui_ctx: &egui::Context,
) -> Vec<egui::Shape> {
    egui_ctx.fonts(|fonts| {
        vec![
            egui::Shape::text(
                fonts,
                rect.center(),
                egui::Align2::CENTER_CENTER,
                model,
                egui::FontId::monospace(16.0 * zoom),
                *text_color,
            ),
            egui::Shape::text(
                fonts,
                rect.min + egui::vec2(5.0, 2.0) * zoom,
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(10.0 * zoom),
                color_scheme::theme_text_color(dark_mode),
            ),
        ]
    })
}

fn render_lut(
    rect: egui::Rect,
    name: &str,
    state: &ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    let zoom = state.zoom;
    let colors = color_scheme::lut_colors(state.dark_mode);

    // Add the body shapes.
    shapes.extend(render_primitive_body(&rect, zoom, &colors.bg, &colors.border));

    // Add text labels
    shapes.extend(render_primitive_labels(&rect, "LUT", name, zoom, &colors.text, state.dark_mode, egui_ctx));

    shapes
}

fn render_ff(
    rect: egui::Rect,
    name: &str,
    state: &ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    let zoom = state.zoom;
    let colors = color_scheme::flip_flop_colors(state.dark_mode);

    // Add the body shapes.
    shapes.extend(render_primitive_body(&rect, zoom, &colors.bg, &colors.border));

    // Add the clock triangle
    let triangle_size = 8.0 * zoom;
    let bottom_center = rect.center_bottom();
    shapes.push(egui::Shape::convex_polygon(
        vec![
            bottom_center + egui::vec2(-triangle_size, 0.0),
            bottom_center + egui::vec2(triangle_size, 0.0),
            bottom_center + egui::vec2(0.0, -triangle_size),
        ],
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(1.5 * zoom, color_scheme::theme_text_color(state.dark_mode)),
    ));

    // Add text labels
    shapes.extend(render_primitive_labels(&rect, "FF", name, zoom, &colors.text, state.dark_mode, egui_ctx));

    shapes
}

fn render_ram(
    rect: egui::Rect,
    name: &str,
    state: &ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    let zoom = state.zoom;
    let colors = color_scheme::memory_colors(state.dark_mode);

    // Add the body shapes.
    shapes.extend(render_primitive_body(&rect, zoom, &colors.bg, &colors.border));

    // Add horizontal lines
    let grid_spacing = 10.0 * zoom;
    let mut y = rect.min.y + 20.0 * zoom;
    while y < rect.max.y - 10.0 * zoom {
        shapes.push(egui::Shape::line_segment(
            [
                egui::pos2(rect.min.x + 10.0 * zoom, y),
                egui::pos2(rect.max.x - 10.0 * zoom, y),
            ],
            egui::Stroke::new(0.5 * zoom, colors.grid),
        ));
        y += grid_spacing;
    }

    // Add text labels
    shapes.extend(render_primitive_labels(&rect, "RAM", name, zoom, &colors.text, state.dark_mode, egui_ctx));

    shapes
}

fn render_generic_blif(
    rect: egui::Rect,
    name: &str,
    model: &str,
    state: &ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    let zoom = state.zoom;
    let colors = color_scheme::blif_colors(state.dark_mode);

    // Add the body shapes.
    shapes.extend(render_primitive_body(&rect, zoom, &colors.bg, &colors.border));

    // Add text labels
    shapes.extend(render_primitive_labels(&rect, model, name, zoom, &colors.text, state.dark_mode, egui_ctx));

    shapes
}

fn render_complete_interconnect(
    rect: egui::Rect,
    state: &ComplexBlockRenderState,
) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    let zoom = state.zoom;
    // TODO: Handle highlighting interconnect.
    // TODO: Add a color scheme for the interconnect.
    let stroke_color = color_scheme::theme_border_color(state.dark_mode);
    let fill_color = color_scheme::theme_block_bg(state.dark_mode);

    // Add the body shapes.
    shapes.extend(render_primitive_body(&rect, zoom, &fill_color, &stroke_color));

    // Draw a large X across the block.
    let x_stroke = egui::Stroke::new(1.5 * zoom, stroke_color);
    shapes.extend([
        egui::Shape::line_segment([rect.min, rect.max], x_stroke),
        egui::Shape::line_segment([egui::pos2(rect.min.x, rect.max.y), egui::pos2(rect.max.x, rect.min.y)], x_stroke),
    ]);

    shapes
}

fn render_mux_interconnect(
    rect: egui::Rect,
    state: &ComplexBlockRenderState,
) -> Vec<egui::Shape> {
    let mut shapes = Vec::new();
    let zoom = state.zoom;
    // TODO: Handle highlighting interconnect.
    // TODO: Add a color scheme for the interconnect.
    let stroke_color = color_scheme::theme_border_color(state.dark_mode);
    let fill_color = color_scheme::theme_block_bg(state.dark_mode);

    // Draw a trapezoid
    let w = rect.width();
    let h = rect.height();
    let c = rect.center();
    let trap_points = vec![
        c + egui::vec2(-w / 2.0, -h / 2.0),
        c + egui::vec2(w / 2.0, -h / 4.0),
        c + egui::vec2(w / 2.0, h / 4.0),
        c + egui::vec2(-w / 2.0, h / 2.0),
    ];
    let trap_stroke = egui::Stroke::new(1.5 * zoom, stroke_color);
    shapes.push(egui::Shape::convex_polygon(trap_points, fill_color, trap_stroke));

    shapes
}

pub fn render_primitive(
    complex_block_id: ComplexBlockNodeId,
    primitive_info: &ComplexBlockPrimitiveInfo,
    offset: egui::Pos2,
    complex_block_graph: &ComplexBlockGraph,
    state: &mut ComplexBlockRenderState,
    egui_ctx: &egui::Context,
) -> Vec<egui::Shape> {
    let complex_block = &complex_block_graph.complex_block_nodes[complex_block_id];

    // Get the shape of the primitive.
    // TODO: The block size may change based on the length of the name.
    // TODO: The block size will certainly be different for interconnect.
    let block_size = get_primitive_block_size(complex_block, complex_block_graph, state);
    let block_rect = egui::Rect::from_min_size(offset, block_size);

    // Render the block.
    let primitive_shapes = match primitive_info.class {
        PBTypeClass::Lut => render_lut(block_rect, &complex_block.name, state, egui_ctx),
        PBTypeClass::FlipFlop => render_ff(block_rect, &complex_block.name, state, egui_ctx),
        PBTypeClass::Memory => render_ram(block_rect, &complex_block.name, state, egui_ctx),
        PBTypeClass::InterconnectComplete => render_complete_interconnect(block_rect, state),
        PBTypeClass::InterconnectMux => render_mux_interconnect(block_rect, state),
        // FIXME: Directs should not be nodes.
        PBTypeClass::InterconnectDirect => render_complete_interconnect(block_rect, state),
        PBTypeClass::None => render_generic_blif(block_rect, &complex_block.name, &primitive_info.blif_model, state, egui_ctx),
    };

    // Render the io.

    primitive_shapes
}