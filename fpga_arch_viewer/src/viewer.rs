use eframe::egui;
use fpga_arch_parser::FPGAArch;
use std::collections::HashMap;

use crate::block_style::DefaultBlockStyles;
use crate::common_ui;
use crate::grid::DeviceGrid;
use crate::grid_renderer;
use crate::inter_tile_view::{self, InterTileState};
use crate::intra_tile::{self, IntraTileState};
use crate::intra_tile_view;
use crate::settings;
use crate::summary_view;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Summary,
    InterTile,
    IntraTile,
}

#[derive(Debug, Clone, PartialEq)]
enum Page {
    Main,
    Settings,
}

pub struct ViewerContext {
    view_mode: ViewMode,
    show_about: bool,
    current_page: Page,
    // Navigation state
    show_layer_list: bool,
    navigation_history: Vec<String>,
    // Block styles
    block_styles: DefaultBlockStyles,
    // Device grid and inter-tile view state
    device_grid: Option<DeviceGrid>,
    inter_tile_state: InterTileState,
    // Currently loaded architecture file path
    loaded_file_path: Option<std::path::PathBuf>,
    // Cache the last window title we set
    window_title: String,
    // Tile name to color mapping
    tile_colors: HashMap<String, egui::Color32>,
    // Parsed architecture
    architecture: Option<FPGAArch>,
    // Intra-tile view state
    selected_tile_name: Option<String>,
    selected_sub_tile_index: usize,
    show_hierarchy_tree: bool,
    intra_tile_state: IntraTileState,
    all_blocks_expanded: bool,
    draw_intra_interconnects: bool,
    // Theme setting
    dark_mode: bool,
}

pub struct FpgaViewer {
    viewer_ctx: ViewerContext,
}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            viewer_ctx: ViewerContext {
                view_mode: ViewMode::Summary,
                show_about: false,
                current_page: Page::Main,
                show_layer_list: false,
                navigation_history: Vec::new(),
                block_styles: DefaultBlockStyles::new(),
                device_grid: None,
                inter_tile_state: InterTileState::default(),
                loaded_file_path: None,
                window_title: "FPGA Architecture Visualizer".to_string(),
                tile_colors: HashMap::new(),
                architecture: None,
                selected_tile_name: None,
                selected_sub_tile_index: 0,
                show_hierarchy_tree: false,
                intra_tile_state: IntraTileState::default(),
                all_blocks_expanded: false,
                draw_intra_interconnects: true,
                dark_mode: false,
            }
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
            if let Some(arch) = &viewer_ctx.architecture {
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
                self.viewer_ctx.architecture = Some(arch);
                self.viewer_ctx.inter_tile_state.selected_layout_index = 0;

                if let Some(arch) = &self.viewer_ctx.architecture {
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

                    self.viewer_ctx.tile_colors.clear();
                    let mut sorted_tiles: Vec<_> = tile_names.into_iter().collect();
                    sorted_tiles.sort();
                    let num_tiles = sorted_tiles.len();
                    for (i, tile_name) in sorted_tiles.iter().enumerate() {
                        let color = crate::block_style::get_tile_color(tile_name, i);
                        self.viewer_ctx.tile_colors.insert(tile_name.clone(), color);
                    }

                    self.rebuild_grid();
                    self.viewer_ctx.loaded_file_path = Some(file_path);
                    println!(
                        "Successfully loaded architecture file with {} tile types and {} layouts",
                        num_tiles, num_layouts
                    );
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

    fn rebuild_grid(&mut self) {
        let viewer_ctx = &mut self.viewer_ctx;
        if let Some(arch) = &viewer_ctx.architecture {
            if let Some(layout) = arch.layouts.get(viewer_ctx.inter_tile_state.selected_layout_index) {
                let grid = match layout {
                    fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                        viewer_ctx.inter_tile_state.aspect_ratio = auto_layout.aspect_ratio;
                        DeviceGrid::from_auto_layout_with_dimensions(
                            arch,
                            viewer_ctx.inter_tile_state.grid_width,
                            viewer_ctx.inter_tile_state.grid_height,
                        )
                    }
                    fpga_arch_parser::Layout::FixedLayout(fixed_layout) => {
                        viewer_ctx.inter_tile_state.grid_width = fixed_layout.width as usize;
                        viewer_ctx.inter_tile_state.grid_height = fixed_layout.height as usize;
                        DeviceGrid::from_fixed_layout(arch, viewer_ctx.inter_tile_state.selected_layout_index)
                    }
                };
                viewer_ctx.device_grid = Some(grid);
            }
        }
    }

    fn navigate_back(&mut self) {
        let viewer_ctx = &mut self.viewer_ctx;
        if viewer_ctx.current_page == Page::Settings {
            viewer_ctx.current_page = Page::Main;
            return;
        }

        if viewer_ctx.view_mode == ViewMode::IntraTile {
            viewer_ctx.view_mode = ViewMode::InterTile;
            viewer_ctx.selected_tile_name = None;
            return;
        }

        if viewer_ctx.view_mode == ViewMode::InterTile {
            viewer_ctx.view_mode = ViewMode::Summary;
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
                        || self.viewer_ctx.view_mode == ViewMode::IntraTile
                        || self.viewer_ctx.view_mode == ViewMode::InterTile
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
                            } else if self.viewer_ctx.view_mode == ViewMode::IntraTile {
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
                        self.viewer_ctx.view_mode = ViewMode::Summary;
                        ui.close_menu();
                    }
                    if ui.button("Inter-Tile View").clicked() {
                        self.viewer_ctx.view_mode = ViewMode::InterTile;
                        ui.close_menu();
                    }
                    if ui.button("Intra-Tile View").clicked() {
                        self.viewer_ctx.view_mode = ViewMode::IntraTile;
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
                Page::Main => match self.viewer_ctx.view_mode {
                    ViewMode::Summary => self.render_summary_view(ui),
                    ViewMode::InterTile => self.render_inter_tile_view(ui),
                    ViewMode::IntraTile => self.render_intra_tile_view(ui),
                },
                Page::Settings => {
                    settings::render_settings_page(ui, &self.viewer_ctx.block_styles, &mut self.viewer_ctx.dark_mode);
                }
            }
        });
    }

    fn render_summary_view(&mut self, ui: &mut egui::Ui) {
        if self.viewer_ctx.architecture.is_none() {
            common_ui::render_welcome_message(ui, &self.viewer_ctx.view_mode);
        } else if let Some(arch) = &self.viewer_ctx.architecture {
            if let Some(new_view_mode) = summary_view::render_summary_view(ui, arch) {
                self.viewer_ctx.view_mode = new_view_mode;
            }
        }
    }

    fn render_inter_tile_view(&mut self, ui: &mut egui::Ui) {
        if let (Some(grid), Some(arch)) = (&self.viewer_ctx.device_grid, &self.viewer_ctx.architecture) {
            if let Some(clicked_tile) = grid_renderer::render_grid(
                ui,
                grid,
                &self.viewer_ctx.block_styles,
                &self.viewer_ctx.tile_colors,
                self.viewer_ctx.dark_mode,
                arch,
            ) {
                self.viewer_ctx.selected_tile_name = Some(clicked_tile);
                self.viewer_ctx.selected_sub_tile_index = 0;
                self.viewer_ctx.view_mode = ViewMode::IntraTile;
                self.apply_expand_all_state();
            }
        } else {
            common_ui::render_welcome_message(ui, &self.viewer_ctx.view_mode);
        }
    }

    fn render_intra_tile_view(&mut self, ui: &mut egui::Ui) {
        if self.viewer_ctx.architecture.is_none() {
            common_ui::render_welcome_message(ui, &self.viewer_ctx.view_mode);
        } else if let (Some(arch), Some(tile_name)) = (&self.viewer_ctx.architecture, &self.viewer_ctx.selected_tile_name) {
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
                    self.viewer_ctx.view_mode = ViewMode::InterTile;
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
                self.viewer_ctx.view_mode = ViewMode::InterTile;
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

        // Inter-tile view controls
        if self.viewer_ctx.current_page == Page::Main
            && self.viewer_ctx.view_mode == ViewMode::InterTile
            && self.viewer_ctx.device_grid.is_some()
        {
            let grid_changed = inter_tile_view::render_grid_controls_panel(
                ctx,
                self.viewer_ctx.architecture.as_ref(),
                &mut self.viewer_ctx.inter_tile_state,
                self.viewer_ctx.device_grid.as_ref(),
                &self.viewer_ctx.tile_colors,
            );
            if grid_changed {
                self.rebuild_grid();
            }
        }

        // Intra-tile view controls
        if self.viewer_ctx.current_page == Page::Main
            && self.viewer_ctx.view_mode == ViewMode::IntraTile
            && self.viewer_ctx.architecture.is_some()
        {
            let should_expand_all = intra_tile_view::render_intra_tile_controls_panel(
                ctx,
                self.viewer_ctx.architecture.as_ref(),
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
    }
}
