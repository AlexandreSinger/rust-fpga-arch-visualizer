use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    InterTile,
    IntraTile,
}

pub struct FpgaViewer {
    view_mode: ViewMode,
    show_about: bool,
}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::InterTile,
            show_about: false,
        }
    }
}

impl eframe::App for FpgaViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            // Top menu
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Architecture File...").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Inter-Tile View").clicked() {
                        self.view_mode = ViewMode::InterTile;
                        ui.close_menu();
                    }
                    if ui.button("Intra-Tile View").clicked() {
                        self.view_mode = ViewMode::IntraTile;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Side panel
        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();

                ui.label("No architecture loaded");

                ui.separator();

                ui.radio_value(&mut self.view_mode, ViewMode::InterTile, "Inter-Tile View");
                ui.radio_value(&mut self.view_mode, ViewMode::IntraTile, "Intra-Tile View");

                if self.view_mode == ViewMode::IntraTile {
                    ui.separator();
                    ui.label("Intra-Tile Controls:");
                }
                if self.view_mode == ViewMode::InterTile {
                    ui.separator();
                    ui.label("Inter-Tile Controls:");
                }
            });

        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.heading("FPGA Architecture Visualizer");
                ui.add_space(20.0);
                ui.label("No architecture file loaded.");
                ui.add_space(10.0);
                ui.label("Use File > Open Architecture File to load a VTR architecture file.");
                ui.add_space(20.0);
                ui.label(format!("Current mode: {:?}", self.view_mode));
            });
        });

        // About window
        if self.show_about {
            egui::Window::new("About")
                .collapsible(false)
                .resizable(false)
                .default_size([300.0, 150.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Nothing Implemented Yet");
                        ui.add_space(10.0);
                        ui.label("Version 0.0.0");
                        ui.add_space(20.0);
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }
    }
}
