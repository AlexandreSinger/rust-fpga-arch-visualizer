//! Block-specific Drawing
//!
//! This module contains functions for drawing different types of blocks
//! (LUT, FlipFlop, Memory, Generic, BLIF) and their ports.

use eframe::egui;
use fpga_arch_parser::{PBType, Port};
use std::collections::HashMap;

use super::color_scheme;
use super::intra_tile::IntraTileState;

// Constants
const HEADER_HEIGHT: f32 = 35.0;
const PORT_LENGTH: f32 = 15.0;
const MIN_PIN_SPACING: f32 = 25.0;
const PIN_SQUARE_SIZE: f32 = 6.0;

//-----------------------------------------------------------
// Draw Pin
//-----------------------------------------------------------
fn draw_pin(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    port_pos: egui::Pos2,
    pin_name: &str,
    port_map: &mut HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    stroke_color: egui::Color32,
    is_highlighted: bool,
) {
    let zoom = state.zoom_clamped();
    let stroke_width = if is_highlighted {
        2.5 * zoom
    } else {
        1.0 * zoom
    };
    let stroke = egui::Stroke::new(stroke_width, stroke_color);

    painter.line_segment([start, end], stroke);

    port_map.insert(pin_name.to_string(), port_pos);

    let square_size = PIN_SQUARE_SIZE * zoom;
    let square_rect = egui::Rect::from_center_size(start, egui::vec2(square_size, square_size));
    painter.rect_filled(square_rect, 0.0, stroke_color);

    let hit_rect = square_rect.expand(3.0 * zoom);
    let response = ui.put(hit_rect, egui::Label::new(""));
    if response.hovered() {
        state.highlighted_positions_next_frame.push(port_pos);
    }
    response.on_hover_ui(|ui| {
        ui.label(pin_name);
    });
}

#[allow(dead_code)]
enum PinSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Draws pins along a side of the rectangle.
fn draw_pins_on_side(
    pins: &[PinInfo],
    rect: egui::Rect,
    side: PinSide,
    painter: &egui::Painter,
    port_map: &mut HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) {
    if pins.is_empty() {
        return;
    }

    let zoom = state.zoom_clamped();
    let total_pins = pins.len() as f32;
    let (spacing, start_pos, default_color) = match side {
        PinSide::Left | PinSide::Right => {
            let min_required_height = (total_pins + 1.0) * (MIN_PIN_SPACING * zoom);
            let spacing = if rect.height() >= min_required_height {
                rect.height() / (total_pins + 1.0)
            } else {
                MIN_PIN_SPACING * zoom
            };
            let total_pin_height = spacing * (total_pins - 1.0);
            let start_y = rect.min.y + (rect.height() - total_pin_height) / 2.0;
            let x_pos = match side {
                PinSide::Left => rect.min.x,
                PinSide::Right => rect.max.x,
                _ => unreachable!(),
            };
            (spacing, (x_pos, start_y), color_scheme::PIN_COLOR)
        }
        PinSide::Top => {
            let min_required_width = (total_pins + 1.0) * (MIN_PIN_SPACING * zoom);
            let spacing = if rect.width() >= min_required_width {
                rect.width() / (total_pins + 1.0)
            } else {
                MIN_PIN_SPACING * zoom
            };
            let total_pin_width = spacing * (total_pins - 1.0);
            let start_x = rect.min.x + (rect.width() - total_pin_width) / 2.0;
            (
                spacing,
                (start_x, rect.min.y),
                color_scheme::CLOCK_PIN_COLOR,
            )
        }
        PinSide::Bottom => {
            let min_required_width = (total_pins + 1.0) * (MIN_PIN_SPACING * zoom);
            let spacing = if rect.width() >= min_required_width {
                rect.width() / (total_pins + 1.0)
            } else {
                MIN_PIN_SPACING * zoom
            };
            let total_pin_width = spacing * (total_pins - 1.0);
            let start_x = rect.min.x + (rect.width() - total_pin_width) / 2.0;
            (
                spacing,
                (start_x, rect.max.y),
                color_scheme::CLOCK_PIN_COLOR,
            )
        }
    };

    for (i, pin) in pins.iter().enumerate() {
        let (start, end, port_pos) = match side {
            PinSide::Left => {
                let (x_pos, start_y) = start_pos;
                let y_pos = start_y + spacing * i as f32;
                let start = egui::pos2(x_pos, y_pos);
                let end = egui::pos2(x_pos - PORT_LENGTH * zoom, y_pos);
                (start, end, end)
            }
            PinSide::Right => {
                let (x_pos, start_y) = start_pos;
                let y_pos = start_y + spacing * i as f32;
                let start = egui::pos2(x_pos, y_pos);
                let end = egui::pos2(x_pos + PORT_LENGTH * zoom, y_pos);
                (start, end, end)
            }
            PinSide::Top => {
                let (start_x, _) = start_pos;
                let x_pos = start_x + spacing * i as f32;
                let start = egui::pos2(x_pos, rect.min.y);
                let end = egui::pos2(x_pos, rect.min.y - PORT_LENGTH * zoom);
                (start, end, end)
            }
            PinSide::Bottom => {
                let (start_x, _) = start_pos;
                let x_pos = start_x + spacing * i as f32;
                let start = egui::pos2(x_pos, rect.max.y);
                let end = egui::pos2(x_pos, rect.max.y + PORT_LENGTH * zoom);
                (start, end, end)
            }
        };

        let is_highlighted = state
            .highlighted_positions_this_frame
            .iter()
            .any(|p| p.distance(port_pos) < 1.0);

        let stroke_color = if is_highlighted {
            color_scheme::HIGHLIGHT_COLOR
        } else {
            default_color
        };

        let pin_name = format!("{}[{}]", pin.name, pin.index);
        draw_pin(
            painter,
            start,
            end,
            port_pos,
            &pin_name,
            port_map,
            state,
            ui,
            stroke_color,
            is_highlighted,
        );
    }
}

struct PinInfo<'a> {
    name: &'a str,
    index: i32,
}

//-----------------------------------------------------------
// Draw Ports
//-----------------------------------------------------------
pub fn draw_ports(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    port_map: &mut HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) {
    let mut input_pins: Vec<PinInfo> = Vec::new();
    let mut output_pins: Vec<PinInfo> = Vec::new();
    let mut clock_pins: Vec<PinInfo> = Vec::new();

    for port in &pb_type.ports {
        match port {
            Port::Input(p) => {
                for i in 0..p.num_pins {
                    input_pins.push(PinInfo {
                        name: &p.name,
                        index: i,
                    });
                }
            }
            Port::Output(p) => {
                for i in 0..p.num_pins {
                    output_pins.push(PinInfo {
                        name: &p.name,
                        index: i,
                    });
                }
            }
            Port::Clock(p) => {
                for i in 0..p.num_pins {
                    clock_pins.push(PinInfo {
                        name: &p.name,
                        index: i,
                    });
                }
            }
        }
    }

    draw_pins_on_side(
        &input_pins,
        rect,
        PinSide::Left,
        painter,
        port_map,
        state,
        ui,
    );
    draw_pins_on_side(
        &output_pins,
        rect,
        PinSide::Right,
        painter,
        port_map,
        state,
        ui,
    );
    draw_pins_on_side(
        &clock_pins,
        rect,
        PinSide::Bottom,
        painter,
        port_map,
        state,
        ui,
    );
}

//-----------------------------------------------------------
// Draw Generic Block
//-----------------------------------------------------------
pub fn draw_generic_block(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let zoom = state.zoom_clamped();
    painter.rect(
        rect,
        egui::CornerRadius::ZERO,
        color_scheme::theme_block_bg(dark_mode),
        egui::Stroke::new(1.5 * zoom, color_scheme::theme_border_color(dark_mode)),
        egui::epaint::StrokeKind::Inside,
    );

    // Title bar
    let title_rect =
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT * zoom));
    painter.rect(
        title_rect,
        egui::CornerRadius::ZERO,
        color_scheme::theme_header_bg(dark_mode),
        egui::Stroke::NONE,
        egui::epaint::StrokeKind::Inside,
    );

    painter.text(
        rect.min + egui::vec2(5.0, 5.0) * zoom,
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(14.0 * zoom),
        color_scheme::theme_text_color(dark_mode),
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

//-----------------------------------------------------------
// Draw LUT
//-----------------------------------------------------------
pub fn draw_lut(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let zoom = state.zoom_clamped();
    let colors = color_scheme::lut_colors(dark_mode);
    painter.rect(
        rect,
        egui::CornerRadius::ZERO,
        colors.bg,
        egui::Stroke::new(1.5 * zoom, colors.border),
        egui::epaint::StrokeKind::Inside,
    );

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "LUT",
        egui::FontId::monospace(16.0 * zoom),
        colors.text,
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0) * zoom,
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0 * zoom),
        color_scheme::theme_text_color(dark_mode),
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

//-----------------------------------------------------------
// Draw Flip Flop
//-----------------------------------------------------------
pub fn draw_flip_flop(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let zoom = state.zoom_clamped();
    let colors = color_scheme::flip_flop_colors(dark_mode);
    painter.rect(
        rect,
        egui::CornerRadius::ZERO,
        colors.bg,
        egui::Stroke::new(1.5 * zoom, colors.border),
        egui::epaint::StrokeKind::Inside,
    );

    let triangle_size = 8.0 * zoom;
    let bottom_center = rect.center_bottom();

    painter.add(egui::Shape::convex_polygon(
        vec![
            bottom_center + egui::vec2(-triangle_size, 0.0),
            bottom_center + egui::vec2(triangle_size, 0.0),
            bottom_center + egui::vec2(0.0, -triangle_size),
        ],
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(1.5 * zoom, color_scheme::theme_text_color(dark_mode)),
    ));

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "FF",
        egui::FontId::monospace(16.0 * zoom),
        colors.text,
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0) * zoom,
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0 * zoom),
        color_scheme::theme_text_color(dark_mode),
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

//-----------------------------------------------------------
// Draw Memory
//-----------------------------------------------------------
pub fn draw_memory(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let zoom = state.zoom_clamped();
    let colors = color_scheme::memory_colors(dark_mode);
    painter.rect(
        rect,
        egui::CornerRadius::ZERO,
        colors.bg,
        egui::Stroke::new(1.5 * zoom, colors.border),
        egui::epaint::StrokeKind::Inside,
    );

    let grid_spacing = 10.0 * zoom;
    let mut y = rect.min.y + 20.0 * zoom;
    while y < rect.max.y - 10.0 * zoom {
        painter.line_segment(
            [
                egui::pos2(rect.min.x + 10.0 * zoom, y),
                egui::pos2(rect.max.x - 10.0 * zoom, y),
            ],
            egui::Stroke::new(0.5 * zoom, colors.grid),
        );
        y += grid_spacing;
    }

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "RAM",
        egui::FontId::monospace(16.0 * zoom),
        colors.text,
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0) * zoom,
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0 * zoom),
        color_scheme::theme_text_color(dark_mode),
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

//-----------------------------------------------------------
// Draw BLIF Block
//-----------------------------------------------------------
pub fn draw_blif_block(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let zoom = state.zoom_clamped();
    let colors = color_scheme::blif_colors(dark_mode);
    painter.rect(
        rect,
        egui::CornerRadius::ZERO,
        colors.bg,
        egui::Stroke::new(1.5 * zoom, colors.border),
        egui::epaint::StrokeKind::Inside,
    );

    // Title bar
    let title_rect =
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT * zoom));
    painter.rect(
        title_rect,
        egui::CornerRadius::ZERO,
        color_scheme::theme_header_bg(dark_mode),
        egui::Stroke::NONE,
        egui::epaint::StrokeKind::Inside,
    );

    // Display blif_model name in center
    if let Some(blif_model) = &pb_type.blif_model {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            blif_model,
            egui::FontId::monospace(14.0 * zoom),
            colors.text,
        );
    }

    painter.text(
        rect.min + egui::vec2(5.0, 5.0) * zoom,
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(14.0 * zoom),
        color_scheme::theme_text_color(dark_mode),
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}
