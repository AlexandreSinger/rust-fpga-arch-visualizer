use eframe::egui;
use fpga_arch_parser::FPGAArch;

use crate::block_style::DefaultBlockStyles;
use crate::common_ui;
use crate::grid_view::GridView;
use crate::complex_block_view::ComplexBlockView;
use crate::settings;
use crate::summary_view::SummaryView;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Summary,
    Grid,
    ComplexBlock,
}

// NOTE: These act more like tabs, so while you are looking at settings,
//       the main page stays around in the background.
// TODO: We should make these actual tabs.
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Main,
    Settings,
}

pub struct ViewerContext {
    pub show_about: bool,
    pub current_page: Page,
    // Navigation state
    pub show_layer_list: bool,
    pub navigation_history: Vec<ViewMode>,
    pub skip_nav_history_update: bool,
    // Block styles
    pub block_styles: DefaultBlockStyles,
    // Currently loaded architecture file path
    pub loaded_file_path: Option<std::path::PathBuf>,
    // Cache the last window title we set
    pub window_title: String,
    // Theme setting
    pub dark_mode: bool,
    // Error window state
    pub show_error: bool,
    pub error_message: String,
}

pub struct FpgaViewer {
    // Parsed architecture
    pub architecture: Option<FPGAArch>,
    viewer_ctx: ViewerContext,

    summary_view: SummaryView,    
    grid_view: GridView,
    complex_block_view: ComplexBlockView,

    view_mode: ViewMode,
    next_view_mode: ViewMode,

}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            architecture: None,
            viewer_ctx: ViewerContext {
                show_about: false,
                current_page: Page::Main,
                show_layer_list: false,
                navigation_history: Vec::new(),
                skip_nav_history_update: false,
                block_styles: DefaultBlockStyles::new(),
                loaded_file_path: None,
                window_title: "FPGA Architecture Visualizer".to_string(),
                dark_mode: false,
                show_error: false,
                error_message: String::new(),
            },
            summary_view: SummaryView::default(),
            grid_view: GridView::default(),
            complex_block_view: ComplexBlockView::default(),
            view_mode: ViewMode::Summary,
            next_view_mode: ViewMode::Summary,
        }
    }

    fn loaded_arch_filename(&self) -> Option<String> {
        self.viewer_ctx.loaded_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
    }

    fn desired_window_title(&self) -> String {
        match self.loaded_arch_filename() {
            Some(name) if !name.is_empty() => format!("FPGA Architecture Visualizer - {name}"),
            _ => "FPGA Architecture Visualizer".to_string(),
        }
    }

    fn load_architecture_file(&mut self, file_path: std::path::PathBuf) {
        match fpga_arch_parser::parse(&file_path) {
            Ok(arch) => {
                // Update views with new architecture.
                self.grid_view.on_architecture_load(&arch);

                // Update viewer context.
                self.viewer_ctx.loaded_file_path = Some(file_path);
                self.architecture = Some(arch);
                self.next_view_mode = ViewMode::Summary;

                // Print success.
                if let Some(filename) = self.loaded_arch_filename() {
                    println!("Successfully loaded architecture file: {}", filename);
                }
            }
            Err(e) => {
                self.viewer_ctx.show_error = true;
                self.viewer_ctx.error_message = format!("Failed to parse architecture file.\n {:?}", e);
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

    fn navigate_back(&mut self) {
        if self.viewer_ctx.current_page == Page::Settings {
            self.viewer_ctx.current_page = Page::Main;
            return;
        }

        // Navigate back in view mode history
        if !self.viewer_ctx.navigation_history.is_empty() {
            if let Some(previous_mode) = self.viewer_ctx.navigation_history.pop() {
                self.next_view_mode = previous_mode;
                self.viewer_ctx.skip_nav_history_update = true;
            }
        }
    }

    fn toggle_layer_list(&mut self) {
        self.viewer_ctx.show_layer_list = !self.viewer_ctx.show_layer_list;
        self.viewer_ctx.current_page = Page::Main;
    }

    fn open_settings(&mut self) {
        self.viewer_ctx.current_page = Page::Settings;
    }

    fn render_navigation_buttons(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation_buttons")
            .resizable(false)
            .default_width(80.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    const BUTTON_SIZE: f32 = 50.0;

                    let list_button = ui.add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new(egui::RichText::new("☰").size(24.0))
                            .frame(true)
                            .rounding(BUTTON_SIZE / 2.0),
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

                    let settings_button = ui.add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new(egui::RichText::new("⚙").size(24.0))
                            .frame(true)
                            .rounding(BUTTON_SIZE / 2.0),
                    );
                    if settings_button.clicked() {
                        self.open_settings();
                    }
                    if settings_button.hovered() {
                        egui::show_tooltip_at_pointer(
                            ctx,
                            egui::Id::new("settings_tooltip"),
                            |ui| {
                                ui.label("Open settings");
                            },
                        );
                    }
                    ui.add_space(10.0);

                    let back_enabled = self.viewer_ctx.current_page == Page::Settings
                        || !self.viewer_ctx.navigation_history.is_empty();
                    let back_button = ui.add_enabled_ui(back_enabled, |ui| {
                        ui.add_sized(
                            [BUTTON_SIZE, BUTTON_SIZE],
                            egui::Button::new(egui::RichText::new("◀").size(24.0))
                                .frame(true)
                                .rounding(BUTTON_SIZE / 2.0),
                        )
                    });
                    if back_button.inner.clicked() {
                        self.navigate_back();
                    }
                    if back_button.inner.hovered() {
                        egui::show_tooltip_at_pointer(ctx, egui::Id::new("back_tooltip"), |ui| {
                            if self.viewer_ctx.current_page == Page::Settings {
                                ui.label("Back to main");
                            } else if self.view_mode == ViewMode::ComplexBlock {
                                ui.label("Back to grid view");
                            } else {
                                ui.label("Go back");
                            }
                        });
                    }
                });
            });
    }

    fn render_layer_list_panel(&self, ctx: &egui::Context) {
        if !self.viewer_ctx.show_layer_list {
            return;
        }

        // Layer list panel (toggleable via list button)
        // This panel will contain expandable layers and navigation to elements
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
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(
                        "! This feature is currently under development and will be implemented in a future update.",
                    ).color(egui::Color32::RED),
                );
            });
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
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
                    if ui.button("Summary View").clicked() {
                        self.next_view_mode = ViewMode::Summary;
                        ui.close_menu();
                    }
                    if ui.button("Grid View").clicked() {
                        self.next_view_mode = ViewMode::Grid;
                        ui.close_menu();
                    }
                    if ui.button("Complex Block View").clicked() {
                        self.next_view_mode = ViewMode::ComplexBlock;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.viewer_ctx.show_about = true;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(name) = self.loaded_arch_filename() {
                        ui.label(egui::RichText::new(name).strong());
                    } else {
                        ui.label(egui::RichText::new("No file loaded").weak());
                    }
                });
            });
        });
    }

    fn render_page(&mut self, ctx: &egui::Context) {
        match self.viewer_ctx.current_page {
            Page::Main => {
                self.render_main_page(ctx);
            },
            Page::Settings => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    settings::render_settings_page(ui, &self.viewer_ctx.block_styles, &mut self.viewer_ctx.dark_mode);
                });
            },
        }
    }

    fn render_main_page(&mut self, ctx: &egui::Context) {
        match &self.architecture {
            Some(arch) => match self.view_mode {
                ViewMode::Summary => self.summary_view.render(arch, &mut self.next_view_mode, ctx),
                ViewMode::Grid => self.grid_view.render(arch, &mut self.viewer_ctx, &mut self.complex_block_view.complex_block_view_state, &mut self.next_view_mode, ctx),
                ViewMode::ComplexBlock => self.complex_block_view.render(arch, &mut self.next_view_mode, self.viewer_ctx.dark_mode, ctx),
            },
            None => {
                // If no architecture is loaded, no view can be seen, so show a welcome message.
                egui::CentralPanel::default().show(ctx, |ui| {
                    common_ui::render_welcome_message(ui, &self.view_mode);
                });
            },
        }
    }

    fn render_error_window(&mut self, ctx: &egui::Context) {
        if !self.viewer_ctx.show_error {
            return;
        }

        egui::Window::new("Error")
            .collapsible(false)
            .resizable(true)
            .default_size([400.0, 200.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(&self.viewer_ctx.error_message).color(egui::Color32::RED));
                    ui.add_space(20.0);
                    if ui.button("Close").clicked() {
                        self.viewer_ctx.show_error = false;
                    }
                });
            });
    }

    fn render_about_window(&mut self, ctx: &egui::Context) {
        if !self.viewer_ctx.show_about {
            return;
        }

        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .default_size([300.0, 150.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("FPGA Architecture Visualizer");
                    ui.add_space(10.0);
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(10.0);
                    ui.label(
                        "A Rust-based visualizer for VTR FPGA architecture description files.",
                    );
                    ui.add_space(10.0);
                    ui.label("Copyright (c) 2025 AlexandreSinger");
                    ui.label("Licensed under MIT License (SPDX: MIT)");
                    ui.add_space(20.0);
                    if ui.button("Close").clicked() {
                        self.viewer_ctx.show_about = false;
                    }
                });
            });
    }
}

impl eframe::App for FpgaViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        if self.viewer_ctx.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Update window title
        let desired_title = self.desired_window_title();
        if desired_title != self.viewer_ctx.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(desired_title.clone()));
            self.viewer_ctx.window_title = desired_title;
        }

        self.viewer_ctx.block_styles.update_colors(self.viewer_ctx.dark_mode);

        // Render UI panels and windows
        self.render_menu_bar(ctx);
        self.render_navigation_buttons(ctx);
        self.render_layer_list_panel(ctx);

        // Render the page.
        self.render_page(ctx);

        // Error window
        self.render_error_window(ctx);

        // About window
        self.render_about_window(ctx);

        // Next state logic for the view mode.
        if self.view_mode != self.next_view_mode {
            // Push current mode to history before transitioning
            if !self.viewer_ctx.skip_nav_history_update {
                self.viewer_ctx.navigation_history.push(self.view_mode);
            }
            self.viewer_ctx.skip_nav_history_update = false;

            // Run code on the close of a view.
            match self.view_mode {
                ViewMode::ComplexBlock => self.complex_block_view.on_view_close(),
                _ => {},
            }

            // Run code on the open of a view.
            match self.next_view_mode {
                ViewMode::ComplexBlock => self.complex_block_view.on_view_open(&self.architecture),
                _ => {},
            }

            self.view_mode = self.next_view_mode;
        }
    }
}
