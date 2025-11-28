use eframe::egui;
use crate::block_style::DefaultBlockStyles;
use crate::grid::DeviceGrid;
use crate::grid_renderer;
use crate::settings;

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
    // Block styles
    block_styles: DefaultBlockStyles,
    // Device grid
    device_grid: Option<DeviceGrid>,
}

impl FpgaViewer {
    pub fn new() -> Self {
        // TODO: now only load one test file, Jack has code for opening arch file - replace that to here
        let device_grid = Self::load_default_architecture();

        Self {
            view_mode: ViewMode::InterTile,
            show_about: false,
            current_page: Page::Main,
            show_layer_list: false,
            navigation_history: Vec::new(),
            block_styles: DefaultBlockStyles::new(),
            device_grid,
        }
    }

    // TODO: only for testing, can delete this
    fn load_default_architecture() -> Option<DeviceGrid> {
        use std::path::PathBuf;

        let test_path = PathBuf::from("../fpga_arch_parser/tests/k4_N4_90nm.xml");

        if let Ok(parsed) = fpga_arch_parser::parse(&test_path) {
            if let Some(layout) = parsed.layouts.first() {
                match layout {
                    fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                        let grid = DeviceGrid::from_auto_layout(auto_layout, 10); // default size for auto layout - 10 for now
                        return Some(grid);
                    }
                    fpga_arch_parser::Layout::FixedLayout(_fixed_layout) => {
                        eprintln!("FixedLayout not yet supported");
                        return None;
                    }
                }
            }
        } else {
            eprintln!("Failed to load k4_N4_90nm.xml, using no grid");
        }

        None
    }

    fn navigate_back(&mut self) {
        // setting page back to main
        if self.current_page == Page::Settings {
            self.current_page = Page::Main;
            return;
        }

        if !self.navigation_history.is_empty() {
            self.navigation_history.pop();
            // TODO: Update current layer based on history
        }
    }

    fn toggle_layer_list(&mut self) {
        self.show_layer_list = !self.show_layer_list;
    }

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
                        // TODO: Jack's code here
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
                    if let Some(grid) = &self.device_grid {
                        grid_renderer::render_grid(ui, grid, &self.block_styles);
                    } else {
                        // No grid loaded, show welcome message
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
                }
                Page::Settings => {
                    settings::render_settings_page(ui, &self.block_styles);
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
