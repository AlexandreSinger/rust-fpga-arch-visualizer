use std::collections::HashMap;

use egui::{Color32, epaint::QuadraticBezierShape};
use fpga_arch_parser::{FPGAArch, Model};

// Visual style constants
const BLOCK_FILL: Color32 = Color32::from_rgb(228, 238, 255);
const BLOCK_STROKE_COLOR: Color32 = Color32::from_rgb(55, 85, 150);
const BLOCK_STROKE_WIDTH: f32 = 2.0;

const CLOCK_COLOR: Color32 = Color32::from_rgb(120, 50, 175);
const CLOCK_FILL: Color32 = Color32::from_rgb(210, 185, 245);
const INPUT_COLOR: Color32 = Color32::from_rgb(25, 105, 190);
const OUTPUT_COLOR: Color32 = Color32::from_rgb(195, 80, 15);

const SETUP_COLOR: Color32 = Color32::from_rgb(210, 40, 40);
const HOLD_COLOR: Color32 = Color32::from_rgb(35, 80, 210);
const COMB_PATH_COLOR: Color32 = Color32::from_rgb(25, 155, 60);

const CONSTRAINT_STROKE_WIDTH: f32 = 1.5;
const SIGNAL_STROKE_WIDTH: f32 = 3.0;

pub struct PrimitiveView {
    selected_model_name: Option<String>,
    show_setup_constraints: bool,
    show_hold_constraints: bool,
    show_combinational_paths: bool,
}

impl Default for PrimitiveView {
    fn default() -> Self {
        Self {
            selected_model_name: None,
            show_setup_constraints: true,
            show_hold_constraints: true,
            show_combinational_paths: true,
        }
    }
}

impl PrimitiveView {
    pub fn render(&mut self, arch: &FPGAArch, ctx: &egui::Context) {
        egui::SidePanel::right("primitive_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_side_panel(arch, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(arch, ui);
        });
    }

    fn render_side_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        ui.heading("Primitive View");

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if !arch.models.is_empty() {
            ui.label("Select Model:");
            ui.add_space(5.0);

            let mut selected_model_name_str = self.selected_model_name.as_deref().unwrap_or("");

            egui::ComboBox::from_id_salt("model_selector_combobox")
                .selected_text(if !selected_model_name_str.is_empty() {
                    selected_model_name_str
                } else {
                    "Select a model"
                })
                .show_ui(ui, |ui| {
                    for model in &arch.models {
                        ui.selectable_value(&mut selected_model_name_str, &model.name, &model.name);
                    }
                });

            if selected_model_name_str != self.selected_model_name.as_deref().unwrap_or("") {
                self.selected_model_name = Some(selected_model_name_str.to_string());
            }
        } else {
            ui.label("No models available in architecture");
        }
        ui.add_space(10.0);

        ui.separator();
        ui.add_space(10.0);
        ui.label("Legend:");
        ui.add_space(6.0);

        ui.label("Signals:");
        ui.add_space(4.0);
        legend_entry(ui, "Input", INPUT_COLOR);
        ui.add_space(4.0);
        legend_entry(ui, "Output", OUTPUT_COLOR);
        ui.add_space(4.0);
        legend_entry(ui, "Clock", CLOCK_COLOR);
        ui.add_space(10.0);

        ui.label("Timing Constraints:");
        ui.add_space(4.0);
        constraint_checkbox(
            ui,
            &mut self.show_setup_constraints,
            "Setup Constraints",
            SETUP_COLOR,
        );
        ui.add_space(4.0);
        constraint_checkbox(
            ui,
            &mut self.show_hold_constraints,
            "Hold Constraints",
            HOLD_COLOR,
        );
        ui.add_space(4.0);
        constraint_checkbox(
            ui,
            &mut self.show_combinational_paths,
            "Combinational Paths",
            COMB_PATH_COLOR,
        );
    }

    fn render_central_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        ui.label("Primitive View");
        if let Some(selected_model_name) = &self.selected_model_name {
            ui.label(format!("Selected Model: {}", selected_model_name));
            if let Some(model) = arch
                .models
                .iter()
                .find(|&model| model.name == *selected_model_name)
            {
                self.render_model(model, ui);
            }
        }
    }

    fn render_model(&mut self, model: &Model, ui: &mut egui::Ui) {
        let mut input_ports = Vec::new();
        let mut output_ports = Vec::new();
        let mut clock_ports = Vec::new();
        for input_port in &model.input_ports {
            if input_port.is_clock {
                clock_ports.push(input_port);
            } else {
                input_ports.push(input_port);
            }
        }
        for output_port in &model.output_ports {
            if output_port.is_clock {
                clock_ports.push(output_port);
            } else {
                output_ports.push(output_port);
            }
        }

        if is_sequential_block(model) {
            // If there are no combinatorial paths, then this acts like
            // a sequential block.
            let block_outline = egui::Rect::from_center_size(
                ui.min_rect().center(),
                egui::vec2(250.0, ui.available_height() / 2.0),
            );
            ui.painter().rect(
                block_outline,
                egui::CornerRadius::same(8),
                BLOCK_FILL,
                egui::Stroke::new(BLOCK_STROKE_WIDTH, BLOCK_STROKE_COLOR),
                egui::epaint::StrokeKind::Middle,
            );

            // Draw the clock triangles
            let mut clock_triangle_top: HashMap<String, egui::Pos2> = HashMap::new();
            let triangle_step_size = block_outline.width() / (clock_ports.len() + 1) as f32;
            for (clk_idx, clock_port) in clock_ports.iter().enumerate() {
                let triangle_base_center = block_outline.left_bottom()
                    + egui::vec2(triangle_step_size * (clk_idx + 1) as f32, 0.0);
                let triangle_top = triangle_base_center + egui::vec2(0.0, -20.0);
                let triangle_left = triangle_base_center + egui::vec2(-10.0, 0.0);
                let triangle_right = triangle_base_center + egui::vec2(10.0, 0.0);
                ui.painter().add(egui::Shape::convex_polygon(
                    vec![triangle_left, triangle_top, triangle_right],
                    CLOCK_FILL,
                    egui::Stroke::new(1.5, CLOCK_COLOR),
                ));

                let arrow_steiner_point =
                    triangle_base_center + egui::vec2(0.0, 50.0 * (clk_idx + 1) as f32);
                let arrow_start_point =
                    egui::pos2(block_outline.left() - 100.0, arrow_steiner_point.y);
                // TODO: Clocks may be inputs or outputs. They should be drawn accordingly.
                ui.painter().line(
                    vec![arrow_start_point, arrow_steiner_point, triangle_base_center],
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH, CLOCK_COLOR),
                );
                ui.painter().text(
                    arrow_start_point - egui::vec2(10.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &clock_port.name,
                    egui::FontId::proportional(24.0),
                    CLOCK_COLOR,
                );
                clock_triangle_top.insert(clock_port.name.clone(), triangle_top);
            }

            // Draw the inputs
            let input_step_size = block_outline.height() / (input_ports.len() + 2) as f32;
            for (input_idx, input_port) in input_ports.iter().enumerate() {
                let arrow_end_point = block_outline.left_top()
                    + egui::vec2(0.0, input_step_size * (input_idx + 1) as f32);
                let arrow_start_point = arrow_end_point - egui::vec2(100.0, 0.0);
                ui.painter().arrow(
                    arrow_start_point,
                    arrow_end_point - arrow_start_point,
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH, INPUT_COLOR),
                );
                ui.painter().text(
                    arrow_start_point - egui::vec2(10.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &input_port.name,
                    egui::FontId::proportional(24.0),
                    INPUT_COLOR,
                );
                // Draw the setup constraints.
                if self.show_setup_constraints
                    && let Some(setup_clock_name) = &input_port.clock
                {
                    let setup_source_point = match clock_triangle_top.get(setup_clock_name) {
                        Some(p) => p,
                        None => {
                            ui.painter().debug_text(
                                arrow_end_point,
                                egui::Align2::LEFT_CENTER,
                                egui::Color32::RED,
                                format!("Cannot find clock: {}", setup_clock_name),
                            );
                            continue;
                        }
                    };
                    let setup_sink_point = arrow_end_point;
                    let bezier_shape = QuadraticBezierShape::from_points_stroke(
                        [
                            *setup_source_point,
                            egui::pos2(setup_source_point.x, setup_sink_point.y),
                            setup_sink_point,
                        ],
                        false,
                        Color32::TRANSPARENT,
                        egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, SETUP_COLOR),
                    );
                    ui.painter().add(bezier_shape);
                }
            }

            // Draw the outputs
            let output_step_size = block_outline.height() / (output_ports.len() + 2) as f32;
            for (output_idx, output_port) in output_ports.iter().enumerate() {
                let arrow_start_point = block_outline.right_top()
                    + egui::vec2(0.0, output_step_size * (output_idx + 1) as f32);
                let arrow_end_point = arrow_start_point + egui::vec2(100.0, 0.0);
                ui.painter().arrow(
                    arrow_start_point,
                    arrow_end_point - arrow_start_point,
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH, OUTPUT_COLOR),
                );
                ui.painter().text(
                    arrow_end_point + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &output_port.name,
                    egui::FontId::proportional(24.0),
                    OUTPUT_COLOR,
                );
                // Draw the hold constraints.
                if self.show_hold_constraints
                    && let Some(hold_clock_name) = &output_port.clock
                {
                    let hold_source_point = match clock_triangle_top.get(hold_clock_name) {
                        Some(p) => p,
                        None => {
                            ui.painter().debug_text(
                                arrow_start_point,
                                egui::Align2::RIGHT_CENTER,
                                egui::Color32::RED,
                                format!("Cannot find clock: {}", hold_clock_name),
                            );
                            continue;
                        }
                    };
                    let hold_sink_point = arrow_start_point;
                    let bezier_shape = QuadraticBezierShape::from_points_stroke(
                        [
                            *hold_source_point,
                            egui::pos2(hold_source_point.x, hold_sink_point.y),
                            hold_sink_point,
                        ],
                        false,
                        Color32::TRANSPARENT,
                        egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, HOLD_COLOR),
                    );
                    ui.painter().add(bezier_shape);
                }
            }
        } else {
            let block_outline = egui::Rect::from_center_size(
                ui.min_rect().center(),
                egui::vec2(500.0, ui.available_height() / 2.0),
            );
            ui.painter().rect(
                block_outline,
                egui::CornerRadius::same(8),
                BLOCK_FILL,
                egui::Stroke::new(BLOCK_STROKE_WIDTH, BLOCK_STROKE_COLOR),
                egui::epaint::StrokeKind::Middle,
            );

            // Draw the clock names
            let mut clock_start_point: HashMap<String, egui::Pos2> = HashMap::new();
            for (clk_idx, clock_port) in clock_ports.iter().enumerate() {
                let signal_start_point = egui::pos2(
                    block_outline.left() - 100.0,
                    block_outline.bottom() + 50.0 * (clk_idx + 1) as f32,
                );
                // TODO: Clocks may be inputs or outputs. They should be drawn accordingly.
                ui.painter().text(
                    signal_start_point - egui::vec2(10.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &clock_port.name,
                    egui::FontId::proportional(24.0),
                    CLOCK_COLOR,
                );
                clock_start_point.insert(clock_port.name.clone(), signal_start_point);
            }

            // Draw the outputs
            let mut output_start_point: HashMap<String, egui::Pos2> = HashMap::new();
            let output_step_size = block_outline.height() / (output_ports.len() + 2) as f32;
            for (output_idx, output_port) in output_ports.iter().enumerate() {
                let arrow_start_point = block_outline.right_top()
                    + egui::vec2(-50.0, output_step_size * (output_idx + 1) as f32);
                let arrow_end_point = arrow_start_point + egui::vec2(150.0, 0.0);
                ui.painter().arrow(
                    arrow_start_point,
                    arrow_end_point - arrow_start_point,
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH, OUTPUT_COLOR),
                );
                ui.painter().text(
                    arrow_end_point + egui::vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &output_port.name,
                    egui::FontId::proportional(24.0),
                    OUTPUT_COLOR,
                );

                let mut signal_start_point = arrow_start_point;
                if let Some(associated_clock_name) = &output_port.clock {
                    // Create a FF
                    let ff_outline = egui::Rect::from_min_size(
                        egui::pos2(signal_start_point.x - 50.0, signal_start_point.y - 20.0),
                        egui::vec2(50.0, 75.0),
                    );
                    draw_flip_flop(
                        &ff_outline,
                        self.show_setup_constraints,
                        self.show_hold_constraints,
                        ui,
                    );
                    signal_start_point -= egui::vec2(ff_outline.width(), 0.0);

                    // Draw clock path.
                    if let Some(clock_drive_point) = clock_start_point.get(associated_clock_name) {
                        let steiner_point = egui::pos2(ff_outline.center().x, clock_drive_point.y);
                        ui.painter().line(
                            vec![
                                *clock_drive_point,
                                steiner_point,
                                ff_outline.center_bottom(),
                            ],
                            egui::Stroke::new(SIGNAL_STROKE_WIDTH, CLOCK_COLOR),
                        );
                    } else {
                        ui.painter().debug_text(
                            ff_outline.center_bottom(),
                            egui::Align2::CENTER_CENTER,
                            egui::Color32::RED,
                            format!("Unable to find clock: {}", associated_clock_name),
                        );
                    }
                }

                output_start_point.insert(output_port.name.clone(), signal_start_point);
            }

            // Draw the inputs
            let input_step_size = block_outline.height() / (input_ports.len() + 2) as f32;
            for (input_idx, input_port) in input_ports.iter().enumerate() {
                let arrow_end_point = block_outline.left_top()
                    + egui::vec2(50.0, input_step_size * (input_idx + 1) as f32);
                let arrow_start_point = arrow_end_point - egui::vec2(150.0, 0.0);
                ui.painter().arrow(
                    arrow_start_point,
                    arrow_end_point - arrow_start_point,
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH, INPUT_COLOR),
                );
                ui.painter().text(
                    arrow_start_point - egui::vec2(10.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &input_port.name,
                    egui::FontId::proportional(24.0),
                    INPUT_COLOR,
                );

                let mut signal_start_point = arrow_end_point;
                if let Some(associated_clock_name) = &input_port.clock {
                    // Create a FF
                    let ff_outline = egui::Rect::from_min_size(
                        egui::pos2(signal_start_point.x, signal_start_point.y - 20.0),
                        egui::vec2(50.0, 75.0),
                    );
                    draw_flip_flop(
                        &ff_outline,
                        self.show_setup_constraints,
                        self.show_hold_constraints,
                        ui,
                    );
                    signal_start_point += egui::vec2(ff_outline.width(), 0.0);

                    // Draw clock path.
                    if let Some(clock_drive_point) = clock_start_point.get(associated_clock_name) {
                        let steiner_point = egui::pos2(ff_outline.center().x, clock_drive_point.y);
                        ui.painter().line(
                            vec![
                                *clock_drive_point,
                                steiner_point,
                                ff_outline.center_bottom(),
                            ],
                            egui::Stroke::new(SIGNAL_STROKE_WIDTH, CLOCK_COLOR),
                        );
                    } else {
                        ui.painter().debug_text(
                            ff_outline.center_bottom(),
                            egui::Align2::CENTER_CENTER,
                            egui::Color32::RED,
                            format!("Unable to find clock: {}", associated_clock_name),
                        );
                    }
                }

                // Draw the combinational timing paths.
                if self.show_combinational_paths {
                    for combinational_sink_port_name in &input_port.combinational_sink_ports {
                        let sink_port_point = match output_start_point
                            .get(combinational_sink_port_name)
                        {
                            Some(p) => p,
                            None => {
                                ui.painter().debug_text(
                                    arrow_end_point,
                                    egui::Align2::RIGHT_CENTER,
                                    egui::Color32::RED,
                                    format!("Cannot find port: {}", combinational_sink_port_name),
                                );
                                continue;
                            }
                        };

                        ui.painter().line_segment(
                            [signal_start_point, *sink_port_point],
                            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, COMB_PATH_COLOR),
                        );
                    }
                }
            }
        }
    }
}

fn draw_flip_flop(
    ff_outline: &egui::Rect,
    show_setup_constraints: bool,
    show_hold_constraints: bool,
    ui: &mut egui::Ui,
) {
    // Draw the outline of the flop.
    ui.painter().rect(
        *ff_outline,
        egui::CornerRadius::same(4),
        Color32::from_rgb(255, 248, 215),
        egui::Stroke::new(1.5, Color32::from_rgb(110, 85, 20)),
        egui::epaint::StrokeKind::Middle,
    );

    // Draw the clock triangle.
    let triangle_base_length = ff_outline.width() * 2.0 / 5.0;
    let triangle_bl = ff_outline.center_bottom() - egui::vec2(triangle_base_length / 2.0, 0.0);
    let triangle_t = ff_outline.center_bottom() - egui::vec2(0.0, triangle_base_length);
    let triangle_br = ff_outline.center_bottom() + egui::vec2(triangle_base_length / 2.0, 0.0);
    ui.painter().add(egui::Shape::convex_polygon(
        vec![triangle_bl, triangle_t, triangle_br],
        CLOCK_FILL,
        egui::Stroke::new(1.5, CLOCK_COLOR),
    ));

    // Draw the timing constraints.
    let d_port_point = ff_outline.left_top() + egui::vec2(0.0, 20.0);
    let q_port_point = ff_outline.right_top() + egui::vec2(0.0, 20.0);
    let control_point = egui::pos2(triangle_t.x, d_port_point.y);
    if show_setup_constraints {
        let bezier_shape = QuadraticBezierShape::from_points_stroke(
            [triangle_t, control_point, d_port_point],
            false,
            Color32::TRANSPARENT,
            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, SETUP_COLOR),
        );
        ui.painter().add(bezier_shape);
    }
    if show_hold_constraints {
        let bezier_shape = QuadraticBezierShape::from_points_stroke(
            [triangle_t, control_point, q_port_point],
            false,
            Color32::TRANSPARENT,
            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, HOLD_COLOR),
        );
        ui.painter().add(bezier_shape);
    }
}

fn legend_entry(ui: &mut egui::Ui, label: &str, color: Color32) {
    ui.horizontal(|ui| {
        // Indent to align with constraint checkboxes (checkbox width ~20px + spacing)
        ui.add_space(24.0);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(28.0, 14.0), egui::Sense::empty());
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            egui::Stroke::new(SIGNAL_STROKE_WIDTH, color),
        );
        ui.add_space(4.0);
        ui.label(label);
    });
}

fn constraint_checkbox(
    ui: &mut egui::Ui,
    value: &mut bool,
    label: &str,
    color: Color32,
) {
    ui.horizontal(|ui| {
        ui.checkbox(value, "");
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(28.0, 14.0), egui::Sense::empty());
        let dimmed = if *value { color } else { color.gamma_multiply(0.35) };
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, dimmed),
        );
        ui.add_space(4.0);
        let text_color = if *value {
            ui.visuals().text_color()
        } else {
            ui.visuals().weak_text_color()
        };
        ui.colored_label(text_color, label);
    });
}

fn is_sequential_block(model: &Model) -> bool {
    // A sequential block is a block with no internal timing paths between their
    // input and output ports.
    for port_group in [&model.input_ports, &model.output_ports] {
        for port in port_group {
            if !port.combinational_sink_ports.is_empty() {
                return false;
            }
        }
    }

    true
}
