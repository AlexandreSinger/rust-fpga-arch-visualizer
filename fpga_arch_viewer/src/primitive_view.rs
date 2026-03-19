use std::collections::HashMap;

use egui::{epaint::QuadraticBezierShape, Color32};
use fpga_arch_parser::{FPGAArch, Model};

pub struct PrimitiveView {
    selected_model_name: Option<String>,
    show_setup_constraints: bool,
    show_hold_constraints: bool,
}

impl Default for PrimitiveView {
    fn default() -> Self {
        Self {
            selected_model_name: None,
            show_setup_constraints: true,
            show_hold_constraints: true,
        }
    }
}

impl PrimitiveView {
    pub fn render(
        &mut self,
        arch: &FPGAArch,
        ctx: &egui::Context,
    ) {
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

        ui.checkbox(&mut self.show_setup_constraints, "Show Setup Constraints");
        ui.add_space(10.0);
        ui.checkbox(&mut self.show_hold_constraints, "Show Hold Constraints");
    }

    fn render_central_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        ui.label("Primitive View");
        if let Some(selected_model_name) = &self.selected_model_name {
            ui.label(format!("Selected Model: {}", selected_model_name));
            if let Some(model) = arch.models.iter().find(|&model| {
                model.name == *selected_model_name
            }) {
                self.render_model(model, ui);
            }
        }
    }

    fn render_model(&mut self, model: &Model, ui: &mut egui::Ui) {
        if is_sequential_block(model) {
            // If there are no combinatorial paths, then this acts like
            // a sequential block.
            let block_outline = egui::Rect::from_center_size(
                ui.min_rect().center(),
                egui::vec2(250.0, ui.available_height() / 2.0));
            ui.painter().rect(
                block_outline,
                egui::CornerRadius::ZERO,
                egui::Color32::WHITE,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
                egui::epaint::StrokeKind::Middle,
            );

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

            // Draw the clock triangles
            let mut clock_triangle_top: HashMap<String, egui::Pos2> = HashMap::new();
            let triangle_step_size = block_outline.width() / (clock_ports.len() + 1) as f32;
            for (clk_idx, clock_port) in clock_ports.iter().enumerate() {
                let triangle_base_center = block_outline.left_bottom() + egui::vec2(triangle_step_size * (clk_idx + 1) as f32, 0.0);
                let triangle_top = triangle_base_center + egui::vec2(0.0, -20.0);
                let triangle_left = triangle_base_center + egui::vec2(-10.0, 0.0);
                let triangle_right = triangle_base_center + egui::vec2(10.0, 0.0);
                ui.painter().line(vec![triangle_left, triangle_top, triangle_right, triangle_left], egui::Stroke::new(1.0, egui::Color32::BLACK));

                let arrow_steiner_point = triangle_base_center + egui::vec2(0.0, 50.0 * (clk_idx + 1) as f32);
                let arrow_start_point = egui::pos2(
                    block_outline.left() - 100.0,
                    arrow_steiner_point.y,
                );
                // TODO: Clocks may be inputs or outputs. They should be drawn accordingly.
                ui.painter().line(vec![arrow_start_point, arrow_steiner_point, triangle_base_center], egui::Stroke::new(5.0, egui::Color32::BLACK));
                ui.painter().text(arrow_start_point - egui::vec2(10.0, 0.0), egui::Align2::RIGHT_CENTER, &clock_port.name, egui::FontId::proportional(24.0), egui::Color32::BLACK);
                clock_triangle_top.insert(clock_port.name.clone(), triangle_top);
            }

            // Draw the inputs
            let input_step_size = block_outline.height() / (input_ports.len() + 2) as f32;
            for (input_idx, input_port) in input_ports.iter().enumerate() {
                let arrow_end_point = block_outline.left_top() + egui::vec2(0.0, input_step_size * (input_idx + 1) as f32);
                let arrow_start_point = arrow_end_point - egui::vec2(100.0, 0.0);
                ui.painter().arrow(arrow_start_point, arrow_end_point - arrow_start_point, egui::Stroke::new(5.0, egui::Color32::BLACK));
                ui.painter().text(arrow_start_point - egui::vec2(10.0, 0.0), egui::Align2::RIGHT_CENTER, &input_port.name, egui::FontId::proportional(24.0), egui::Color32::BLACK);
                // Draw the setup constraints.
                if self.show_setup_constraints && let Some(setup_clock_name) = &input_port.clock {
                    let setup_source_point = match clock_triangle_top.get(setup_clock_name) {
                        Some(p) => p,
                        None => {
                            ui.painter().debug_text(arrow_end_point, egui::Align2::LEFT_CENTER, egui::Color32::RED, format!("Cannot find clock: {}", setup_clock_name));
                            continue
                        }
                    };
                    let setup_sink_point = arrow_end_point;
                    let bezier_shape = QuadraticBezierShape::from_points_stroke(
                        [*setup_source_point, egui::pos2(setup_source_point.x, setup_sink_point.y), setup_sink_point],
                        false,
                        Color32::TRANSPARENT,
                        egui::Stroke::new(0.5, egui::Color32::RED),
                    );
                    ui.painter().add(bezier_shape);
                }
            }

            // Draw the outputs
            let output_step_size = block_outline.height() / (output_ports.len() + 2) as f32;
            for (output_idx, output_port) in output_ports.iter().enumerate() {
                let arrow_start_point = block_outline.right_top() + egui::vec2(0.0, output_step_size * (output_idx + 1) as f32);
                let arrow_end_point = arrow_start_point + egui::vec2(100.0, 0.0);
                ui.painter().arrow(arrow_start_point, arrow_end_point - arrow_start_point, egui::Stroke::new(5.0, egui::Color32::BLACK));
                ui.painter().text(arrow_end_point + egui::vec2(10.0, 0.0), egui::Align2::LEFT_CENTER, &output_port.name, egui::FontId::proportional(24.0), egui::Color32::BLACK);
                // Draw the hold constraints.
                if self.show_hold_constraints && let Some(hold_clock_name) = &output_port.clock {
                    let hold_source_point = match clock_triangle_top.get(hold_clock_name) {
                        Some(p) => p,
                        None => {
                            ui.painter().debug_text(arrow_start_point, egui::Align2::RIGHT_CENTER, egui::Color32::RED, format!("Cannot find clock: {}", hold_clock_name));
                            continue
                        },
                    };
                    let hold_sink_point = arrow_start_point;
                    let bezier_shape = QuadraticBezierShape::from_points_stroke(
                        [*hold_source_point, egui::pos2(hold_source_point.x, hold_sink_point.y), hold_sink_point],
                        false,
                        Color32::TRANSPARENT,
                        egui::Stroke::new(0.5, egui::Color32::RED),
                    );
                    ui.painter().add(bezier_shape);
                }
            }
        } else {
            ui.label("Non-sequential block drawing is WIP.");
        }
    }
}

fn is_sequential_block(model: &Model) -> bool {
    for port_group in [&model.input_ports, &model.output_ports] {
        for port in port_group {
            if !port.combinational_sink_ports.is_empty() {
                return false;
            }
        }
    }

    true
}