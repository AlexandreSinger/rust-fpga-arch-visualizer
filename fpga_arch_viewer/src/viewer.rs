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
    // Grid dimensions (for AutoLayout)
    grid_width: usize,
    grid_height: usize,
    aspect_ratio: f32,
    // Currently loaded architecture file path
    loaded_file_path: Option<std::path::PathBuf>,
}

impl FpgaViewer {
    pub fn new() -> Self {
        // Start with no grid - user will load architecture file via File menu
        Self {
            view_mode: ViewMode::InterTile,
            show_about: false,
            current_page: Page::Main,
            show_layer_list: false,
            navigation_history: Vec::new(),
            block_styles: DefaultBlockStyles::new(),
            device_grid: None,
            grid_width: 10,
            grid_height: 10,
            aspect_ratio: 1.0,
            loaded_file_path: None,
        }
    }

    fn load_architecture_file(&mut self, file_path: std::path::PathBuf) {
        match fpga_arch_parser::parse(&file_path) {
            Ok(parsed) => {
                if let Some(layout) = parsed.layouts.first() {
                    match layout {
                        fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                            let default_size = 10;
                            let grid = DeviceGrid::from_auto_layout(auto_layout, default_size);
                            self.grid_width = grid.width;
                            self.grid_height = grid.height;
                            self.aspect_ratio = auto_layout.aspect_ratio;
                            self.device_grid = Some(grid);
                            self.loaded_file_path = Some(file_path);
                            println!("Successfully loaded architecture file");
                        }
                        fpga_arch_parser::Layout::FixedLayout(_fixed_layout) => {
                            eprintln!("FixedLayout not yet supported");
                        }
                    }
                } else {
                    eprintln!("No layouts found in architecture file");
                }
            }
            Err(e) => {
                eprintln!("Failed to parse architecture file: {:?}", e);
            }
        }
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("XML Architecture Files", &["xml"])
            .set_title("Open FPGA Architecture File")
            .pick_file()
        {
            self.load_architecture_file(path);
        }
    }

    // Rebuild the grid with new dimensions based on current architecture
    fn rebuild_grid(&mut self) {
        if let Some(file_path) = &self.loaded_file_path.clone() {
            if let Ok(parsed) = fpga_arch_parser::parse(file_path) {
                if let Some(layout) = parsed.layouts.first() {
                    match layout {
                        fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                            // Calculate default_size based on which dimension user is controlling
                            // Use the larger dimension as the default_size
                            let default_size = self.grid_width.max(self.grid_height);
                            let grid = DeviceGrid::from_auto_layout(auto_layout, default_size);
                            self.device_grid = Some(grid);
                        }
                        _ => {}
                    }
                }
            }
        }
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
        self.current_page = Page::Main;
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
                        self.open_file_dialog();
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

        // Side panel for grid size controls
        if self.current_page == Page::Main && self.device_grid.is_some() {
            egui::SidePanel::right("grid_controls")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Grid Settings");
                    ui.add_space(10.0);

                    ui.label("Adjust dimensions while maintaining aspect ratio:");
                    ui.add_space(10.0);

                    let mut grid_changed = false;

                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        let mut temp_width = self.grid_width as f64;
                        if ui.add(
                            egui::Slider::new(&mut temp_width, 1.0..=100.0)
                                .step_by(1.0)
                                .show_value(false)
                        ).changed() {
                            let new_width = temp_width.round() as usize;
                            if new_width != self.grid_width && new_width >= 1 {
                                self.grid_width = new_width;
                                self.grid_height = (self.grid_width as f32 / self.aspect_ratio).round().max(1.0) as usize;
                                grid_changed = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("       ");
                        let mut width_text = self.grid_width.to_string();
                        if ui.add(
                            egui::TextEdit::singleline(&mut width_text)
                                .desired_width(60.0)
                        ).changed() {
                            if let Ok(new_width) = width_text.parse::<usize>() {
                                if new_width >= 1 && new_width <= 100 && new_width != self.grid_width {
                                    self.grid_width = new_width;
                                    self.grid_height = (self.grid_width as f32 / self.aspect_ratio).round().max(1.0) as usize;
                                    grid_changed = true;
                                }
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        let mut temp_height = self.grid_height as f64;
                        if ui.add(
                            egui::Slider::new(&mut temp_height, 1.0..=100.0)
                                .step_by(1.0)
                                .show_value(false)
                        ).changed() {
                            let new_height = temp_height.round() as usize;
                            if new_height != self.grid_height && new_height >= 1 {
                                self.grid_height = new_height;
                                self.grid_width = (self.grid_height as f32 * self.aspect_ratio).round().max(1.0) as usize;
                                grid_changed = true;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("       ");
                        let mut height_text = self.grid_height.to_string();
                        if ui.add(
                            egui::TextEdit::singleline(&mut height_text)
                                .desired_width(60.0)
                        ).changed() {
                            if let Ok(new_height) = height_text.parse::<usize>() {
                                if new_height >= 1 && new_height <= 100 && new_height != self.grid_height {
                                    self.grid_height = new_height;
                                    self.grid_width = (self.grid_height as f32 * self.aspect_ratio).round().max(1.0) as usize;
                                    grid_changed = true;
                                }
                            }
                        }
                    });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(format!("Aspect Ratio: {:.2}", self.aspect_ratio));
                    ui.label(format!("Grid Size: {}x{}", self.grid_width, self.grid_height));

                    if grid_changed {
                        self.rebuild_grid();
                    }
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
