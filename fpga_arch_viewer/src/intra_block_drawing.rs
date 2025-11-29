//! Block-specific Drawing
//!
//! This module contains functions for drawing different types of blocks
//! (LUT, FlipFlop, Memory, Generic, BLIF) and their ports.

use eframe::egui;
use fpga_arch_parser::{PBType, Port};
use std::collections::HashMap;

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
    let stroke_width = if is_highlighted { 2.5 } else { 1.0 };
    let stroke = egui::Stroke::new(stroke_width, stroke_color);

    painter.line_segment([start, end], stroke);

    port_map.insert(pin_name.to_string(), port_pos);

    let square_size = PIN_SQUARE_SIZE;
    let square_rect = egui::Rect::from_center_size(start, egui::vec2(square_size, square_size));
    painter.rect_filled(square_rect, 0.0, stroke_color);

    let hit_rect = square_rect.expand(3.0);
    let response = ui.put(hit_rect, egui::Label::new(""));
    if response.hovered() {
        state.highlighted_positions_next_frame.push(port_pos);
    }
    response.on_hover_ui(|ui| {
        ui.label(pin_name);
    });
}

enum PinSide {
    Left,
    Right,
    Top,
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

    let total_pins = pins.len() as f32;
    let (spacing, start_pos, default_color) = match side {
        PinSide::Left | PinSide::Right => {
            let min_required_height = (total_pins + 1.0) * MIN_PIN_SPACING;
            let spacing = if rect.height() >= min_required_height {
                rect.height() / (total_pins + 1.0)
            } else {
                MIN_PIN_SPACING
            };
            let total_pin_height = spacing * (total_pins - 1.0);
            let start_y = rect.min.y + (rect.height() - total_pin_height) / 2.0;
            let x_pos = match side {
                PinSide::Left => rect.min.x,
                PinSide::Right => rect.max.x,
                _ => unreachable!(),
            };
            (spacing, (x_pos, start_y), egui::Color32::BLACK)
        }
        PinSide::Top => {
            let min_required_width = (total_pins + 1.0) * MIN_PIN_SPACING;
            let spacing = if rect.width() >= min_required_width {
                rect.width() / (total_pins + 1.0)
            } else {
                MIN_PIN_SPACING
            };
            let total_pin_width = spacing * (total_pins - 1.0);
            let start_x = rect.min.x + (rect.width() - total_pin_width) / 2.0;
            (spacing, (start_x, rect.min.y), egui::Color32::RED)
        }
    };

    for (i, pin) in pins.iter().enumerate() {
        let (start, end, port_pos) = match side {
            PinSide::Left => {
                let (x_pos, start_y) = start_pos;
                let y_pos = start_y + spacing * i as f32;
                let start = egui::pos2(x_pos, y_pos);
                let end = egui::pos2(x_pos - PORT_LENGTH, y_pos);
                (start, end, end)
            }
            PinSide::Right => {
                let (x_pos, start_y) = start_pos;
                let y_pos = start_y + spacing * i as f32;
                let start = egui::pos2(x_pos, y_pos);
                let end = egui::pos2(x_pos + PORT_LENGTH, y_pos);
                (start, end, end)
            }
            PinSide::Top => {
                let (start_x, _) = start_pos;
                let x_pos = start_x + spacing * i as f32;
                let start = egui::pos2(x_pos, rect.min.y);
                let end = egui::pos2(x_pos, rect.min.y - PORT_LENGTH);
                (start, end, end)
            }
        };

        let is_highlighted = state
            .highlighted_positions_this_frame
            .iter()
            .any(|p| p.distance(port_pos) < 1.0);

        let stroke_color = if is_highlighted {
            egui::Color32::RED
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
        PinSide::Top,
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
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(240, 240, 240),
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 100, 100)),
    );

    // Title bar
    let title_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT));
    painter.rect(
        title_rect,
        egui::Rounding::ZERO,
        egui::Color32::from_rgb(200, 200, 200),
        egui::Stroke::NONE,
    );

    painter.text(
        rect.min + egui::vec2(5.0, 5.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(14.0),
        egui::Color32::BLACK,
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
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(255, 250, 205),
        egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 180, 0)),
    );

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "LUT",
        egui::FontId::monospace(16.0),
        egui::Color32::from_rgb(180, 180, 0),
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0),
        egui::Color32::BLACK,
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
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(220, 230, 255), // Light Blue
        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 0, 180)),
    );

    let triangle_size = 8.0;
    let bottom_center = rect.center_bottom();

    painter.add(egui::Shape::convex_polygon(
        vec![
            bottom_center + egui::vec2(-triangle_size, 0.0),
            bottom_center + egui::vec2(triangle_size, 0.0),
            bottom_center + egui::vec2(0.0, -triangle_size),
        ],
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "FF",
        egui::FontId::monospace(16.0),
        egui::Color32::from_rgb(0, 0, 180),
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0),
        egui::Color32::BLACK,
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
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(200, 240, 200), // Light Green
        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 100, 0)),
    );

    let grid_spacing = 10.0;
    let mut y = rect.min.y + 20.0;
    while y < rect.max.y - 10.0 {
        painter.line_segment(
            [
                egui::pos2(rect.min.x + 10.0, y),
                egui::pos2(rect.max.x - 10.0, y),
            ],
            egui::Stroke::new(0.5, egui::Color32::from_rgb(0, 100, 0)),
        );
        y += grid_spacing;
    }

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "RAM",
        egui::FontId::monospace(16.0),
        egui::Color32::from_rgb(0, 100, 0),
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0),
        egui::Color32::BLACK,
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
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(255, 220, 220), // Light Pink/Red
        egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 0, 0)),
    );

    // Title bar
    let title_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT));
    painter.rect(
        title_rect,
        egui::Rounding::ZERO,
        egui::Color32::from_rgb(200, 200, 200),
        egui::Stroke::NONE,
    );

    // Display blif_model name in center
    if let Some(blif_model) = &pb_type.blif_model {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            blif_model,
            egui::FontId::monospace(14.0),
            egui::Color32::from_rgb(180, 0, 0),
        );
    }

    painter.text(
        rect.min + egui::vec2(5.0, 5.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(14.0),
        egui::Color32::BLACK,
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}
