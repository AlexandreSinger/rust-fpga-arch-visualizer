use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    InterTile,
    IntraTile,
}

#[derive(Debug, Clone, PartialEq)]
enum Page {
    Main,
    Settings,
}

pub struct FpgaViewer {
    view_mode: ViewMode,
    show_about: bool,
    current_page: Page,
    // Navigation state
    show_layer_list: bool,
    navigation_history: Vec<String>, // Will store layer/element navigation history
}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::InterTile,
            show_about: false,
            current_page: Page::Main,
            show_layer_list: false,
            navigation_history: Vec::new(),
        }
    }

    /// Handle navigation back action
    fn navigate_back(&mut self) {
        // First check if we're in settings page, go back to main
        if self.current_page == Page::Settings {
            self.current_page = Page::Main;
            return;
        }

        // Otherwise, handle layer navigation history
        if !self.navigation_history.is_empty() {
            self.navigation_history.pop();
            // TODO: Update current layer based on history
        }
    }

    /// Toggle layer list panel visibility
    fn toggle_layer_list(&mut self) {
        self.show_layer_list = !self.show_layer_list;
    }

    /// Navigate to settings page
    fn open_settings(&mut self) {
        self.current_page = Page::Settings;
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

        // Navigation buttons panel
        egui::SidePanel::left("navigation_buttons")
            .resizable(false)
            .default_width(80.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    const BUTTON_SIZE: f32 = 50.0;

                    // Layer list toggle button (☰ icon)
                    // Shows/hides the expandable layer navigation panel
                    let list_button = ui.add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new(
                            egui::RichText::new("☰")
                                .size(24.0)
                        )
                        .frame(true)
                        .rounding(BUTTON_SIZE / 2.0)
                    );
                    if list_button.clicked() {
                        self.toggle_layer_list();
                    }
                    if list_button.hovered() {
                        egui::show_tooltip_at_pointer(ctx, egui::Id::new("list_tooltip"), |ui| {
                            ui.label("Toggle layer list");
                        });
                    }
                    ui.add_space(10.0);

                    // Settings button (⚙ icon)
                    // Opens the settings page for customizing block appearance
                    let settings_button = ui.add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new(
                            egui::RichText::new("⚙")
                                .size(24.0)
                        )
                        .frame(true)
                        .rounding(BUTTON_SIZE / 2.0)
                    );
                    if settings_button.clicked() {
                        self.open_settings();
                    }
                    if settings_button.hovered() {
                        egui::show_tooltip_at_pointer(ctx, egui::Id::new("settings_tooltip"), |ui| {
                            ui.label("Open settings");
                        });
                    }
                    ui.add_space(10.0);

                    // Back button (◀ icon)
                    // Returns to previous layer or exits settings page
                    let back_enabled = self.current_page == Page::Settings || !self.navigation_history.is_empty();
                    let back_button = ui.add_enabled_ui(back_enabled, |ui| {
                        ui.add_sized(
                            [BUTTON_SIZE, BUTTON_SIZE],
                            egui::Button::new(
                                egui::RichText::new("◀")
                                    .size(24.0)
                            )
                            .frame(true)
                            .rounding(BUTTON_SIZE / 2.0)
                        )
                    });
                    if back_button.inner.clicked() {
                        self.navigate_back();
                    }
                    if back_button.inner.hovered() {
                        egui::show_tooltip_at_pointer(ctx, egui::Id::new("back_tooltip"), |ui| {
                            if self.current_page == Page::Settings {
                                ui.label("Back to main");
                            } else {
                                ui.label("Go back");
                            }
                        });
                    }
                });
            });

        // Layer list panel (toggleable via list button)
        // This panel will contain expandable layers and navigation to elements
        if self.show_layer_list {
            egui::SidePanel::left("layer_list")
                .resizable(true)
                .default_width(250.0)
                .min_width(200.0)
                .show(ctx, |ui| {
                    ui.heading("Layers");
                    ui.separator();

                    // TODO: Add expandable layer tree here
                    // Each layer will have:
                    // - Collapse/expand arrow (▼ when expanded, ▶ when collapsed)
                    // - Layer name
                    // - Nested elements when expanded
                    ui.label("No architecture loaded");
                    ui.add_space(10.0);
                    ui.label("Layer list will appear here once an architecture file is loaded.");
                });
        }

        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Main => {
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
                Page::Settings => {
                    ui.centered_and_justified(|ui| {
                        ui.heading("Settings");
                        ui.add_space(20.0);
                        ui.label("Settings page - Coming soon");
                        ui.add_space(10.0);
                        ui.label("This is where you can customize how each block looks.");
                    });
                }
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
