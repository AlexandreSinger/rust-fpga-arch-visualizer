use eframe::egui;
use fpga_arch_parser::FPGAArch;
use std::collections::HashMap;

use crate::block_style::DefaultBlockStyles;
use crate::common_ui;
use crate::grid_view::{self, GridView};
use crate::intra_tile::{self, IntraTileState};
use crate::intra_tile_view;
use crate::settings;
use crate::summary_view;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Summary,
    Grid,
    IntraTile,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Main,
    Settings,
}

pub struct ViewerContext {
    pub next_view_mode: ViewMode,
    pub show_about: bool,
    pub current_page: Page,
    // Navigation state
    pub show_layer_list: bool,
    pub navigation_history: Vec<String>,
    // Block styles
    pub block_styles: DefaultBlockStyles,
    // Currently loaded architecture file path
    pub loaded_file_path: Option<std::path::PathBuf>,
    // Cache the last window title we set
    pub window_title: String,
    // Tile name to color mapping
    pub tile_colors: HashMap<String, egui::Color32>,
    // Intra-tile view state
    pub selected_tile_name: Option<String>,
    pub selected_sub_tile_index: usize,
    pub show_hierarchy_tree: bool,
    pub intra_tile_state: IntraTileState,
    pub all_blocks_expanded: bool,
    pub draw_intra_interconnects: bool,
    // Theme setting
    pub dark_mode: bool,
}

pub struct FpgaViewer {
    // Parsed architecture
    pub architecture: Option<FPGAArch>,
    viewer_ctx: ViewerContext,

    grid_view: GridView,

    view_mode: ViewMode,
}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            architecture: None,
            viewer_ctx: ViewerContext {
                next_view_mode: ViewMode::Summary,
                show_about: false,
                current_page: Page::Main,
                show_layer_list: false,
                navigation_history: Vec::new(),
                block_styles: DefaultBlockStyles::new(),
                loaded_file_path: None,
                window_title: "FPGA Architecture Visualizer".to_string(),
                tile_colors: HashMap::new(),
                selected_tile_name: None,
                selected_sub_tile_index: 0,
                show_hierarchy_tree: false,
                intra_tile_state: IntraTileState::default(),
                all_blocks_expanded: false,
                draw_intra_interconnects: true,
                dark_mode: false,
            },
            grid_view: GridView::default(),
            view_mode: ViewMode::Summary,
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

    fn apply_expand_all_state(&mut self) {
        let viewer_ctx = &mut self.viewer_ctx;
        if viewer_ctx.all_blocks_expanded {
            if let Some(arch) = &self.architecture {
                if let Some(tile_name) = &viewer_ctx.selected_tile_name {
                    if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                        if viewer_ctx.selected_sub_tile_index < tile.sub_tiles.len() {
                            let sub_tile = &tile.sub_tiles[viewer_ctx.selected_sub_tile_index];
                            if let Some(site) = sub_tile.equivalent_sites.first() {
                                if let Some(root_pb) = arch
                                    .complex_block_list
                                    .iter()
                                    .find(|pb| pb.name == site.pb_type)
                                {
                                    intra_tile::expand_all_blocks(
                                        &mut viewer_ctx.intra_tile_state,
                                        root_pb,
                                        &root_pb.name,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        } else {
            intra_tile::collapse_all_blocks(&mut viewer_ctx.intra_tile_state);
        }
    }

    fn load_architecture_file(&mut self, file_path: std::path::PathBuf) {
        match fpga_arch_parser::parse(&file_path) {
            Ok(arch) => {
                // Extract unique tile names from all layouts
                let mut tile_names = std::collections::HashSet::new();
                let num_layouts = arch.layouts.len();
                for layout in &arch.layouts {
                    let grid_locations = match layout {
                        fpga_arch_parser::Layout::AutoLayout(al) => &al.grid_locations,
                        fpga_arch_parser::Layout::FixedLayout(fl) => &fl.grid_locations,
                    };

                    for location in grid_locations {
                        let pb_type = match location {
                            fpga_arch_parser::GridLocation::Fill(f) => &f.pb_type,
                            fpga_arch_parser::GridLocation::Perimeter(p) => &p.pb_type,
                            fpga_arch_parser::GridLocation::Corners(c) => &c.pb_type,
                            fpga_arch_parser::GridLocation::Single(s) => &s.pb_type,
                            fpga_arch_parser::GridLocation::Col(c) => &c.pb_type,
                            fpga_arch_parser::GridLocation::Row(r) => &r.pb_type,
                            fpga_arch_parser::GridLocation::Region(r) => &r.pb_type,
                        };
                        if pb_type != "EMPTY" {
                            tile_names.insert(pb_type.clone());
                        }
                    }
                }

                // Assign colors to tile types
                self.viewer_ctx.tile_colors.clear();
                let mut sorted_tiles: Vec<_> = tile_names.into_iter().collect();
                sorted_tiles.sort();
                let num_tiles = sorted_tiles.len();
                for (i, tile_name) in sorted_tiles.iter().enumerate() {
                    let color = crate::block_style::get_tile_color(tile_name, i);
                    self.viewer_ctx.tile_colors.insert(tile_name.clone(), color);
                }

                // Update grid view with new architecture
                self.grid_view.on_architecture_load(&arch);

                // Update viewer context
                self.viewer_ctx.loaded_file_path = Some(file_path);
                self.architecture = Some(arch);
                self.viewer_ctx.next_view_mode = ViewMode::Summary;
                println!(
                    "Successfully loaded architecture file with {} tile types and {} layouts",
                    num_tiles, num_layouts
                );
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

    fn navigate_back(&mut self) {
        let viewer_ctx = &mut self.viewer_ctx;
        if viewer_ctx.current_page == Page::Settings {
            viewer_ctx.current_page = Page::Main;
            return;
        }

        if self.view_mode == ViewMode::IntraTile {
            viewer_ctx.next_view_mode = ViewMode::Grid;
            viewer_ctx.selected_tile_name = None;
            return;
        }

        if self.view_mode == ViewMode::Grid {
            viewer_ctx.next_view_mode = ViewMode::Summary;
            return;
        }

        if !viewer_ctx.navigation_history.is_empty() {
            viewer_ctx.navigation_history.pop();
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
                        || self.view_mode == ViewMode::IntraTile
                        || self.view_mode == ViewMode::Grid
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
                            } else if self.view_mode == ViewMode::IntraTile {
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
                        self.viewer_ctx.next_view_mode = ViewMode::Summary;
                        ui.close_menu();
                    }
                    if ui.button("Grid View").clicked() {
                        self.viewer_ctx.next_view_mode = ViewMode::Grid;
                        ui.close_menu();
                    }
                    if ui.button("Intra-Tile View").clicked() {
                        self.viewer_ctx.next_view_mode = ViewMode::IntraTile;
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

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.viewer_ctx.current_page {
                Page::Main => match &self.architecture {
                    None => common_ui::render_welcome_message(ui, &self.view_mode),
                    Some(arch) => match self.view_mode {
                        ViewMode::Summary => self.render_summary_view(ui),
                        ViewMode::Grid => self.grid_view.render(arch, &mut self.viewer_ctx, ui),
                        ViewMode::IntraTile => self.render_intra_tile_view(ui),
                    }
                },
                Page::Settings => {
                    settings::render_settings_page(ui, &self.viewer_ctx.block_styles, &mut self.viewer_ctx.dark_mode);
                },
            }
        });
    }

    fn render_summary_view(&mut self, ui: &mut egui::Ui) {
        if self.architecture.is_none() {
            common_ui::render_welcome_message(ui, &self.view_mode);
        } else if let Some(arch) = &self.architecture {
            if let Some(new_view_mode) = summary_view::render_summary_view(ui, arch) {
                self.viewer_ctx.next_view_mode = new_view_mode;
            }
        }
    }

    fn render_intra_tile_view(&mut self, ui: &mut egui::Ui) {
        if self.architecture.is_none() {
            common_ui::render_welcome_message(ui, &self.view_mode);
        } else if let (Some(arch), Some(tile_name)) = (&self.architecture, &self.viewer_ctx.selected_tile_name) {
            if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                let sub_tile_index = if self.viewer_ctx.selected_sub_tile_index < tile.sub_tiles.len() {
                    self.viewer_ctx.selected_sub_tile_index
                } else {
                    0
                };
                intra_tile::render_intra_tile_view(
                    ui,
                    arch,
                    tile,
                    &mut self.viewer_ctx.intra_tile_state,
                    self.viewer_ctx.show_hierarchy_tree,
                    sub_tile_index,
                    self.viewer_ctx.all_blocks_expanded,
                    self.viewer_ctx.draw_intra_interconnects,
                    self.viewer_ctx.dark_mode,
                );
            } else {
                if common_ui::render_centered_message(
                    ui,
                    "Tile not found",
                    &format!("Could not find tile: {}", tile_name),
                    Some("Back to Grid View"),
                ) {
                    self.viewer_ctx.next_view_mode = ViewMode::Grid;
                    self.viewer_ctx.selected_tile_name = None;
                }
            }
        } else {
            if common_ui::render_centered_message(
                ui,
                "No tile selected",
                "Please select a tile from the dropdown or click on a tile in the grid view.",
                Some("Back to Grid View"),
            ) {
                self.viewer_ctx.next_view_mode = ViewMode::Grid;
                self.viewer_ctx.selected_tile_name = None;
            }
        }
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
                    ui.label("All rights reserved?");
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

        // Grid view controls
        if self.viewer_ctx.current_page == Page::Main
            && self.view_mode == ViewMode::Grid
            && self.grid_view.device_grid.is_some()
        {
            let grid_changed = grid_view::render_grid_controls_panel(
                ctx,
                self.architecture.as_ref(),
                &mut self.grid_view.grid_state,
                self.grid_view.device_grid.as_ref(),
                &self.viewer_ctx.tile_colors,
            );
            if grid_changed {
                self.grid_view.rebuild_grid(self.architecture.as_ref().unwrap());
            }
        }

        // Intra-tile view controls
        if self.viewer_ctx.current_page == Page::Main
            && self.view_mode == ViewMode::IntraTile
            && self.architecture.is_some()
        {
            let should_expand_all = intra_tile_view::render_intra_tile_controls_panel(
                ctx,
                self.architecture.as_ref(),
                &mut self.viewer_ctx.show_hierarchy_tree,
                &mut self.viewer_ctx.all_blocks_expanded,
                &mut self.viewer_ctx.draw_intra_interconnects,
                &mut self.viewer_ctx.selected_tile_name,
                &mut self.viewer_ctx.selected_sub_tile_index,
            );
            if should_expand_all {
                self.apply_expand_all_state();
            }
        }

        // Central panel content
        self.render_central_panel(ctx);

        // About window
        self.render_about_window(ctx);

        // Next state logic for the view mode.
        if self.view_mode != self.viewer_ctx.next_view_mode {
            // Run some code on the transision from one state to the next.
            if self.viewer_ctx.next_view_mode == ViewMode::IntraTile {
                self.apply_expand_all_state();
            }
            self.view_mode = self.viewer_ctx.next_view_mode.clone();
        }
    }
}
