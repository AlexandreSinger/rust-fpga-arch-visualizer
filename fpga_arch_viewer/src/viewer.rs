use eframe::egui;
use fpga_arch_parser::FPGAArch;
use std::path::PathBuf;

// Import IntraTileState from intra_tile module
use crate::intra_tile::IntraTileState;

#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    InterTile,
    IntraTile,
}

pub struct FpgaViewer {
    view_mode: ViewMode,
    show_about: bool,
    arch: Option<FPGAArch>,
    loaded_file: Option<PathBuf>,
    selected_tile_index: usize,
    intra_tile_state: IntraTileState,
}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::InterTile,
            show_about: false,
            arch: None,
            loaded_file: None,
            selected_tile_index: 0,
            intra_tile_state: IntraTileState::default(),
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
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("xml", &["xml"])
                            .pick_file()
                        {
                            match fpga_arch_parser::parse(&path) {
                                Ok(arch) => {
                                    self.arch = Some(arch);
                                    self.loaded_file = Some(path);
                                    self.selected_tile_index = 0;
                                }
                                Err(e) => {
                                    eprintln!("Error parsing file: {}", e);
                                }
                            }
                        }
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

                if let Some(path) = &self.loaded_file {
                    ui.label(format!(
                        "Loaded: {}",
                        path.file_name().unwrap().to_string_lossy()
                    ));
                } else {
                    ui.label("No architecture loaded");
                }

                ui.separator();

                ui.radio_value(&mut self.view_mode, ViewMode::InterTile, "Inter-Tile View");
                ui.radio_value(&mut self.view_mode, ViewMode::IntraTile, "Intra-Tile View");

                if self.view_mode == ViewMode::IntraTile {
                    ui.separator();
                    ui.label("Intra-Tile Controls:");

                    if let Some(arch) = &self.arch {
                        egui::ComboBox::from_label("Select Tile")
                            .selected_text(if self.selected_tile_index < arch.tiles.len() {
                                &arch.tiles[self.selected_tile_index].name
                            } else {
                                "None"
                            })
                            .show_ui(ui, |ui| {
                                for (i, tile) in arch.tiles.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.selected_tile_index,
                                        i,
                                        &tile.name,
                                    );
                                }
                            });
                    }
                }
                if self.view_mode == ViewMode::InterTile {
                    ui.separator();
                    ui.label("Inter-Tile Controls:");
                }
            });

        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(arch) = &self.arch {
                match self.view_mode {
                    ViewMode::InterTile => {
                        ui.centered_and_justified(|ui| {
                            ui.heading("Inter-Tile Visualization Placeholder");
                        });
                    }
                    ViewMode::IntraTile => {
                        if self.selected_tile_index < arch.tiles.len() {
                            let tile = &arch.tiles[self.selected_tile_index];
                            ui.heading(format!("Intra-Tile View: {}", tile.name));
                            crate::intra_tile::render_intra_tile_view(
                                ui,
                                arch,
                                tile,
                                &mut self.intra_tile_state,
                            );
                        } else {
                            ui.label("No tile selected.");
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.heading("FPGA Architecture Visualizer");
                    ui.add_space(20.0);
                    ui.label("No architecture file loaded.");
                    ui.add_space(10.0);
                    ui.label("Use File > Open Architecture File to load a VTR architecture file.");
                    ui.add_space(20.0);
                    ui.label(format!("Current mode: {:?}", self.view_mode));
                });
            }
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
