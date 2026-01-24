use egui;
use crate::viewer::ViewMode;

/// Renders a welcome message when no architecture is loaded
pub fn render_welcome_message(
    ui: &mut egui::Ui,
    view_mode: &ViewMode,
) {
    let available_rect = ui.available_rect_before_wrap();
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(available_rect.center(), egui::vec2(500.0, 200.0))),
        |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("FPGA Architecture Visualizer");
                ui.add_space(20.0);
                ui.label("No architecture file loaded.");
                ui.add_space(10.0);
                ui.label("Use File > Open Architecture File to load a VTR architecture file.");
                ui.add_space(20.0);
                ui.label(format!("Current mode: {:?}", view_mode));
            });
        },
    );
}

/// Renders a centered message with optional action button
pub fn render_centered_message(
    ui: &mut egui::Ui,
    heading: &str,
    message: &str,
    button_text: Option<&str>,
) -> bool {
    let available_rect = ui.available_rect_before_wrap();
    let mut button_clicked = false;
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(available_rect.center(), egui::vec2(400.0, 150.0))),
        |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(heading);
                ui.add_space(10.0);
                ui.label(message);
                if let Some(btn_text) = button_text {
                    ui.add_space(20.0);
                    if ui.button(btn_text).clicked() {
                        button_clicked = true;
                    }
                }
            });
        },
    );
    button_clicked
}