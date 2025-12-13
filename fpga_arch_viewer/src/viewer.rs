use eframe::egui;
use fpga_arch_parser::FPGAArch;

// Import IntraTileState from intra_tile module
use crate::block_style::DefaultBlockStyles;
use crate::grid::DeviceGrid;
use crate::grid::GridCell;
use crate::grid_renderer;
use crate::intra_tile::IntraTileState;
use crate::settings;
use std::collections::HashMap;

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
    // Cache the last window title we set.
    window_title: String,
    // Tile name to color mapping
    tile_colors: HashMap<String, egui::Color32>,
    // Parsed architecture (needed for intra-tile view)
    architecture: Option<FPGAArch>,
    // Selected tile for intra-tile view
    selected_tile_name: Option<String>,
    // Selected sub_tile for intra-tile view
    selected_sub_tile_index: usize,
    // Show hierarchy tree in intra-tile view
    show_hierarchy_tree: bool,
    intra_tile_state: IntraTileState,
    // Track if all blocks are expanded
    all_blocks_expanded: bool,
    // Theme setting
    dark_mode: bool,
    // Selected layout index (for switching between auto/fixed layouts)
    selected_layout_index: usize,
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
            window_title: "FPGA Architecture Visualizer".to_string(),
            tile_colors: HashMap::new(),
            architecture: None,
            selected_tile_name: None,
            selected_sub_tile_index: 0,
            show_hierarchy_tree: false,
            intra_tile_state: IntraTileState::default(),
            all_blocks_expanded: false,
            dark_mode: false,
            selected_layout_index: 0,
        }
    }

    fn loaded_arch_filename(&self) -> Option<String> {
        self.loaded_file_path
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
        if self.all_blocks_expanded {
            if let Some(arch) = &self.architecture {
                if let Some(tile_name) = &self.selected_tile_name {
                    if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                        if self.selected_sub_tile_index < tile.sub_tiles.len() {
                            let sub_tile = &tile.sub_tiles[self.selected_sub_tile_index];
                            if let Some(site) = sub_tile.equivalent_sites.first() {
                                if let Some(root_pb) = arch
                                    .complex_block_list
                                    .iter()
                                    .find(|pb| pb.name == site.pb_type)
                                {
                                    crate::intra_tile::expand_all_blocks(
                                        &mut self.intra_tile_state,
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
            crate::intra_tile::collapse_all_blocks(&mut self.intra_tile_state);
        }
    }

    fn load_architecture_file(&mut self, file_path: std::path::PathBuf) {
        match fpga_arch_parser::parse(&file_path) {
            Ok(parsed) => {
                self.architecture = Some(parsed);
                self.selected_layout_index = 0; // Default to first layout

                // Build tile color mapping and create grid
                if let Some(arch) = &self.architecture {
                    // Collect all tile names from all layouts
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

                    // Assign colors to tiles
                    self.tile_colors.clear();
                    let mut sorted_tiles: Vec<_> = tile_names.into_iter().collect();
                    sorted_tiles.sort(); // Sort for consistent ordering
                    let num_tiles = sorted_tiles.len();
                    for (i, tile_name) in sorted_tiles.iter().enumerate() {
                        let color = crate::block_style::get_tile_color(tile_name, i);
                        self.tile_colors.insert(tile_name.clone(), color);
                    }

                    // Create the grid based on first layout
                    self.rebuild_grid();
                    self.loaded_file_path = Some(file_path);
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

    // Rebuild the grid with new dimensions based on current architecture and selected layout
    fn rebuild_grid(&mut self) {
        if let Some(arch) = &self.architecture {
            if let Some(layout) = arch.layouts.get(self.selected_layout_index) {
                let grid = match layout {
                    fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                        // For AutoLayout, use user-specified dimensions
                        self.aspect_ratio = auto_layout.aspect_ratio;
                        DeviceGrid::from_auto_layout_with_dimensions(
                            arch,
                            self.grid_width,
                            self.grid_height,
                        )
                    }
                    fpga_arch_parser::Layout::FixedLayout(fixed_layout) => {
                        // For FixedLayout, use layout's fixed dimensions
                        self.grid_width = fixed_layout.width as usize;
                        self.grid_height = fixed_layout.height as usize;
                        DeviceGrid::from_fixed_layout(arch, self.selected_layout_index)
                    }
                };
                self.device_grid = Some(grid);
            }
        }
    }

    fn navigate_back(&mut self) {
        // setting page back to main
        if self.current_page == Page::Settings {
            self.current_page = Page::Main;
            return;
        }

        // If in intra-tile view, go back to inter-tile view
        if self.view_mode == ViewMode::IntraTile {
            self.view_mode = ViewMode::InterTile;
            self.selected_tile_name = None;
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

    fn render_welcome_message(&self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        ui.allocate_ui_at_rect(
            egui::Rect::from_center_size(available_rect.center(), egui::vec2(500.0, 200.0)),
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("FPGA Architecture Visualizer");
                    ui.add_space(20.0);
                    ui.label("No architecture file loaded.");
                    ui.add_space(10.0);
                    ui.label("Use File > Open Architecture File to load a VTR architecture file.");
                    ui.add_space(20.0);
                    ui.label(format!("Current mode: {:?}", self.view_mode));
                });
            },
        );
    }

    fn render_centered_message(
        &mut self,
        ui: &mut egui::Ui,
        heading: &str,
        message: &str,
        button_text: Option<&str>,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        ui.allocate_ui_at_rect(
            egui::Rect::from_center_size(available_rect.center(), egui::vec2(400.0, 150.0)),
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(heading);
                    ui.add_space(10.0);
                    ui.label(message);
                    if let Some(btn_text) = button_text {
                        ui.add_space(20.0);
                        if ui.button(btn_text).clicked() {
                            self.view_mode = ViewMode::InterTile;
                            self.selected_tile_name = None;
                        }
                    }
                });
            },
        );
    }

    fn update_grid_height_from_width(&mut self) {
        self.grid_height = (self.grid_width as f32 / self.aspect_ratio)
            .round()
            .max(1.0) as usize;
    }

    fn update_grid_width_from_height(&mut self) {
        self.grid_width = (self.grid_height as f32 * self.aspect_ratio)
            .round()
            .max(1.0) as usize;
    }

    fn get_layout_name(&self) -> String {
        if let Some(arch) = &self.architecture {
            if let Some(layout) = arch.layouts.get(self.selected_layout_index) {
                return match layout {
                    fpga_arch_parser::Layout::AutoLayout(_) => "Auto Layout".to_string(),
                    fpga_arch_parser::Layout::FixedLayout(fl) => format!("Fixed: {}", fl.name),
                };
            }
        }
        "No Layout".to_string()
    }
}

impl eframe::App for FpgaViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme based on dark_mode setting
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Keep the window title in sync with the loaded architecture file.
        let desired_title = self.desired_window_title();
        if desired_title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(desired_title.clone()));
            self.window_title = desired_title;
        }

        self.block_styles.update_colors(self.dark_mode);

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

                // Show loaded architecture filename on the far right.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(name) = self.loaded_arch_filename() {
                        ui.label(egui::RichText::new(name).strong());
                    } else {
                        ui.label(egui::RichText::new("No file loaded").weak());
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

                    // Settings button (⚙ icon)
                    // Opens the settings page for customizing block appearance
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

                    // Back button (◀ icon)
                    // Returns to previous layer, exits settings page, or goes back from intra-tile view
                    let back_enabled = self.current_page == Page::Settings
                        || self.view_mode == ViewMode::IntraTile
                        || !self.navigation_history.is_empty();
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
                            if self.current_page == Page::Settings {
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

        if self.current_page == Page::Main
            && self.view_mode == ViewMode::InterTile
            && self.device_grid.is_some()
        {
            egui::SidePanel::right("grid_controls")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Grid Settings");
                    ui.add_space(10.0);

                    // Layout selection dropdown
                    if let Some(arch) = &self.architecture {
                        if arch.layouts.len() > 1 {
                            ui.label("Layout:");
                            let mut layout_changed = false;
                            egui::ComboBox::from_id_source("layout_selector")
                                .selected_text(self.get_layout_name())
                                .show_ui(ui, |ui| {
                                    for (idx, layout) in arch.layouts.iter().enumerate() {
                                        let layout_name = match layout {
                                            fpga_arch_parser::Layout::AutoLayout(_) => {
                                                "Auto Layout".to_string()
                                            }
                                            fpga_arch_parser::Layout::FixedLayout(fl) => {
                                                format!("Fixed: {}", fl.name)
                                            }
                                        };
                                        if ui
                                            .selectable_value(
                                                &mut self.selected_layout_index,
                                                idx,
                                                layout_name,
                                            )
                                            .clicked()
                                        {
                                            layout_changed = true;
                                        }
                                    }
                                });

                            if layout_changed {
                                self.rebuild_grid();
                            }
                            ui.add_space(10.0);
                        }
                    }

                    // Check if current layout is fixed
                    let is_fixed_layout = if let Some(arch) = &self.architecture {
                        matches!(
                            arch.layouts.get(self.selected_layout_index),
                            Some(fpga_arch_parser::Layout::FixedLayout(_))
                        )
                    } else {
                        false
                    };

                    ui.label(if is_fixed_layout {
                        "Dimensions (Fixed by layout):"
                    } else {
                        "Adjust dimensions while maintaining aspect ratio:"
                    });
                    ui.add_space(10.0);

                    let mut grid_changed = false;

                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        let mut temp_width = self.grid_width as f64;
                        ui.add_enabled_ui(!is_fixed_layout, |ui| {
                            if ui
                                .add(
                                    egui::Slider::new(&mut temp_width, 1.0..=100.0)
                                        .step_by(1.0)
                                        .show_value(false),
                                )
                                .changed()
                            {
                                let new_width = temp_width.round() as usize;
                                if new_width != self.grid_width && new_width >= 1 {
                                    self.grid_width = new_width;
                                    self.update_grid_height_from_width();
                                    grid_changed = true;
                                }
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("       ");
                        let mut width_text = self.grid_width.to_string();
                        ui.add_enabled_ui(!is_fixed_layout, |ui| {
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut width_text).desired_width(60.0),
                                )
                                .changed()
                            {
                                if let Ok(new_width) = width_text.parse::<usize>() {
                                    if new_width >= 1
                                        && new_width <= 100
                                        && new_width != self.grid_width
                                    {
                                        self.grid_width = new_width;
                                        self.update_grid_height_from_width();
                                        grid_changed = true;
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        let mut temp_height = self.grid_height as f64;
                        ui.add_enabled_ui(!is_fixed_layout, |ui| {
                            if ui
                                .add(
                                    egui::Slider::new(&mut temp_height, 1.0..=100.0)
                                        .step_by(1.0)
                                        .show_value(false),
                                )
                                .changed()
                            {
                                let new_height = temp_height.round() as usize;
                                if new_height != self.grid_height && new_height >= 1 {
                                    self.grid_height = new_height;
                                    self.update_grid_width_from_height();
                                    grid_changed = true;
                                }
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("       ");
                        let mut height_text = self.grid_height.to_string();
                        ui.add_enabled_ui(!is_fixed_layout, |ui| {
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut height_text)
                                        .desired_width(60.0),
                                )
                                .changed()
                            {
                                if let Ok(new_height) = height_text.parse::<usize>() {
                                    if new_height >= 1
                                        && new_height <= 100
                                        && new_height != self.grid_height
                                    {
                                        self.grid_height = new_height;
                                        self.update_grid_width_from_height();
                                        grid_changed = true;
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.label(format!("Aspect Ratio: {:.2}", self.aspect_ratio));
                    ui.label(format!(
                        "Grid Size: {}x{}",
                        self.grid_width, self.grid_height
                    ));

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.heading("Tile Counts");
                    ui.add_space(10.0);

                    if let Some(grid) = &self.device_grid {
                        let mut tile_counts: std::collections::HashMap<String, usize> =
                            std::collections::HashMap::new();
                        for row in &grid.cells {
                            for cell in row {
                                if let GridCell::BlockAnchor { pb_type, .. } = cell {
                                    *tile_counts.entry(pb_type.clone()).or_insert(0) += 1;
                                }
                            }
                        }

                        let mut sorted_counts: Vec<_> = tile_counts.into_iter().collect();
                        sorted_counts.sort_by(|a, b| a.0.cmp(&b.0));

                        for (pb_type, count) in sorted_counts {
                            ui.label(format!("{}: {}", pb_type, count));
                        }
                    }

                    if grid_changed {
                        self.rebuild_grid();
                    }
                });
        }

        // Side panel for intra-tile view controls
        if self.current_page == Page::Main
            && self.view_mode == ViewMode::IntraTile
            && self.architecture.is_some()
        {
            egui::SidePanel::right("intra_tile_controls")
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.heading("Intra-Tile View");
                    ui.add_space(10.0);

                    // Hierarchy tree toggle
                    ui.checkbox(&mut self.show_hierarchy_tree, "Show Hierarchy Tree");

                    // Expand All toggle
                    let mut expand_all = self.all_blocks_expanded;
                    if ui.checkbox(&mut expand_all, "Expand All").changed() {
                        self.all_blocks_expanded = expand_all;
                        self.apply_expand_all_state();
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Tile selector - shows all available tiles from architecture
                    if let Some(arch) = &self.architecture {
                        if !arch.tiles.is_empty() {
                            ui.label("Select Tile:");
                            ui.add_space(5.0);

                            let mut selected_tile_name =
                                self.selected_tile_name.as_deref().unwrap_or("").to_string();

                            egui::ComboBox::from_id_source("tile_selector")
                                .selected_text(if !selected_tile_name.is_empty() {
                                    selected_tile_name.as_str()
                                } else {
                                    "Select a tile"
                                })
                                .show_ui(ui, |ui| {
                                    for tile in &arch.tiles {
                                        ui.selectable_value(
                                            &mut selected_tile_name,
                                            tile.name.clone(),
                                            &tile.name,
                                        );
                                    }
                                });

                            // If tile selection changed, update state
                            if selected_tile_name
                                != self.selected_tile_name.as_deref().unwrap_or("")
                            {
                                self.selected_tile_name = Some(selected_tile_name);
                                self.selected_sub_tile_index = 0;
                                self.apply_expand_all_state();
                            }
                        } else {
                            ui.label("No tiles available in architecture");
                        }
                    }
                });
        }

        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Main => {
                    match self.view_mode {
                        ViewMode::InterTile => {
                            if let Some(grid) = &self.device_grid {
                                // Check if a tile was clicked
                                if let Some(clicked_tile) = grid_renderer::render_grid(ui, grid, &self.block_styles, &self.tile_colors, self.dark_mode) {
                                    self.selected_tile_name = Some(clicked_tile);
                                    self.selected_sub_tile_index = 0;
                                    self.view_mode = ViewMode::IntraTile;
                                    self.apply_expand_all_state();
                                }
                            } else {
                                // No grid loaded, show welcome message
                                self.render_welcome_message(ui);
                            }
                        }
                        ViewMode::IntraTile => {
                            // Show intra-tile view
                            if self.architecture.is_none() {
                                // No architecture loaded - show welcome message
                                self.render_welcome_message(ui);
                            } else if let (Some(arch), Some(tile_name)) = (&self.architecture, &self.selected_tile_name) {
                                // Find the tile that matches the selected tile name
                                if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                                    // Ensure selected_sub_tile_index is valid
                                    let sub_tile_index = if self.selected_sub_tile_index < tile.sub_tiles.len() {
                                        self.selected_sub_tile_index
                                    } else {
                                        0
                                    };
                                    crate::intra_tile::render_intra_tile_view(
                                        ui,
                                        arch,
                                        tile,
                                        &mut self.intra_tile_state,
                                        self.show_hierarchy_tree,
                                        sub_tile_index,
                                        self.all_blocks_expanded,
                                        self.dark_mode,
                                    );
                                } else {
                                    self.render_centered_message(
                                        ui,
                                        "Tile not found",
                                        &format!("Could not find tile: {}", tile_name),
                                        Some("Back to Grid View"),
                                    );
                                }
                            } else {
                                self.render_centered_message(
                                    ui,
                                    "No tile selected",
                                    "Please select a tile from the dropdown or click on a tile in the grid view.",
                                    Some("Back to Grid View"),
                                );
                            }
                        }
                    }
                }
                Page::Settings => {
                    settings::render_settings_page(ui, &self.block_styles, &mut self.dark_mode);
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
                            self.show_about = false;
                        }
                    });
                });
        }
    }
}
