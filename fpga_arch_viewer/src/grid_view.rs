use egui;
use egui_extras;
use fpga_arch_parser::FPGAArch;
use std::collections::HashMap;

use crate::{grid::{DeviceGrid, GridCell}, grid_renderer, complex_block_view::ComplexBlockViewState, viewer::{ViewMode, ViewerContext}};

/// State for grid view
#[derive(Debug, Clone)]
pub struct GridState {
    pub grid_width: usize,
    pub grid_height: usize,
    pub aspect_ratio: f32,
    pub selected_layout_index: usize,
}

impl Default for GridState {
    fn default() -> Self {
        Self {
            grid_width: 10,
            grid_height: 10,
            aspect_ratio: 1.0,
            selected_layout_index: 0,
        }
    }
}

pub struct GridView {
    // Device grid and grid view state
    pub device_grid: Option<DeviceGrid>,
    pub grid_state: GridState,

    // Tile name to color mapping
    pub tile_colors: HashMap<String, egui::Color32>,
}

impl Default for GridView {
    fn default() -> Self {
        Self {
            grid_state: GridState::default(),
            device_grid: None,
            tile_colors: HashMap::new(),
        }
    }
}

impl GridView {
    pub fn on_architecture_load(&mut self, arch: &FPGAArch) {
        // Extract unique tile names from all layouts
        let mut tile_names = std::collections::HashSet::new();
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
        self.tile_colors.clear();
        let mut sorted_tiles: Vec<_> = tile_names.into_iter().collect();
        sorted_tiles.sort();
        for (i, tile_name) in sorted_tiles.iter().enumerate() {
            let color = crate::block_style::get_tile_color(tile_name, i);
            self.tile_colors.insert(tile_name.clone(), color);
        }

        // Reset layout selection and rebuild grid
        self.grid_state.selected_layout_index = 0;
        self.rebuild_grid(arch);
    }

    pub fn rebuild_grid(&mut self, arch: &FPGAArch) {
        if let Some(layout) = arch.layouts.get(self.grid_state.selected_layout_index) {
            let grid = match layout {
                fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                    self.grid_state.aspect_ratio = auto_layout.aspect_ratio;
                    DeviceGrid::from_auto_layout_with_dimensions(
                        arch,
                        self.grid_state.grid_width,
                        self.grid_state.grid_height,
                    )
                }
                fpga_arch_parser::Layout::FixedLayout(fixed_layout) => {
                    self.grid_state.grid_width = fixed_layout.width as usize;
                    self.grid_state.grid_height = fixed_layout.height as usize;
                    DeviceGrid::from_fixed_layout(arch, self.grid_state.selected_layout_index)
                }
            };
            self.device_grid = Some(grid);
        }
    }

    pub fn render(
        &mut self,
        arch: &FPGAArch,
        viewer_ctx: &mut ViewerContext,
        complex_block_view_state: &mut ComplexBlockViewState,
        next_view_mode: &mut ViewMode,
        ui: &mut egui::Ui
    ) {
        if let Some(grid) = &self.device_grid {
            if let Some(clicked_tile) = grid_renderer::render_grid(
                ui,
                grid,
                &viewer_ctx.block_styles,
                &self.tile_colors,
                viewer_ctx.dark_mode,
                arch,
            ) {
                complex_block_view_state.selected_tile_name = Some(clicked_tile);
                complex_block_view_state.selected_sub_tile_index = 0;
                *next_view_mode = ViewMode::ComplexBlock;
            }
        } else {
            // TODO: Render an error window
        }
    }

    pub fn render_side_panel(
        &mut self,
        arch: &FPGAArch,
        ctx: &egui::Context,
    ) {
        let grid_changed = render_grid_controls_panel(
            ctx,
            arch,
            &mut self.grid_state,
            self.device_grid.as_ref(),
            &self.tile_colors,
        );
        if grid_changed {
            self.rebuild_grid(arch);
        }
    }
}

/// Renders the grid controls panel on the right side
/// Returns true if grid dimensions changed
pub fn render_grid_controls_panel(
    ctx: &egui::Context,
    arch: &FPGAArch,
    state: &mut GridState,
    device_grid: Option<&DeviceGrid>,
    tile_colors: &HashMap<String, egui::Color32>,
) -> bool {
    let mut grid_changed = false;

    egui::SidePanel::right("grid_controls")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Grid Settings");
            ui.add_space(10.0);

            // Layout selection dropdown
            if arch.layouts.len() > 1 {
                ui.label("Layout:");
                let mut layout_changed = false;
                egui::ComboBox::from_id_source("layout_selector")
                    .selected_text(get_layout_name(arch, state.selected_layout_index))
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
                                .selectable_value(&mut state.selected_layout_index, idx, layout_name)
                                .clicked()
                            {
                                layout_changed = true;
                            }
                        }
                    });

                if layout_changed {
                    grid_changed = true;
                }
                ui.add_space(10.0);
            }

            // Check if current layout is fixed
            let is_fixed_layout = matches!(
                arch.layouts.get(state.selected_layout_index),
                Some(fpga_arch_parser::Layout::FixedLayout(_))
            );

            ui.label(if is_fixed_layout {
                "Dimensions (Fixed by layout):"
            } else {
                "Adjust dimensions while maintaining aspect ratio:"
            });
            ui.add_space(10.0);

            // Width slider and text input
            ui.horizontal(|ui| {
                ui.label("Width:");
                let mut temp_width = state.grid_width as f64;
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
                        if new_width != state.grid_width && new_width >= 1 {
                            state.grid_width = new_width;
                            update_grid_height_from_width(state);
                            grid_changed = true;
                        }
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.label("       ");
                let mut width_text = state.grid_width.to_string();
                ui.add_enabled_ui(!is_fixed_layout, |ui| {
                    if ui
                        .add(egui::TextEdit::singleline(&mut width_text).desired_width(60.0))
                        .changed()
                    {
                        if let Ok(new_width) = width_text.parse::<usize>() {
                            if new_width >= 1 && new_width <= 100 && new_width != state.grid_width {
                                state.grid_width = new_width;
                                update_grid_height_from_width(state);
                                grid_changed = true;
                            }
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Height slider and text input
            ui.horizontal(|ui| {
                ui.label("Height:");
                let mut temp_height = state.grid_height as f64;
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
                        if new_height != state.grid_height && new_height >= 1 {
                            state.grid_height = new_height;
                            update_grid_width_from_height(state);
                            grid_changed = true;
                        }
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.label("       ");
                let mut height_text = state.grid_height.to_string();
                ui.add_enabled_ui(!is_fixed_layout, |ui| {
                    if ui
                        .add(egui::TextEdit::singleline(&mut height_text).desired_width(60.0))
                        .changed()
                    {
                        if let Ok(new_height) = height_text.parse::<usize>() {
                            if new_height >= 1
                                && new_height <= 100
                                && new_height != state.grid_height
                            {
                                state.grid_height = new_height;
                                update_grid_width_from_height(state);
                                grid_changed = true;
                            }
                        }
                    }
                });
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(format!("Aspect Ratio: {:.2}", state.aspect_ratio));
            ui.label(format!("Grid Size: {}x{}", state.grid_width, state.grid_height));

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Tile counts table
            ui.heading("Tile Counts");
            ui.add_space(10.0);

            if let Some(grid) = device_grid {
                let mut tile_counts: std::collections::BTreeMap<String, usize> =
                    std::collections::BTreeMap::new();
                for row in &grid.cells {
                    for cell in row {
                        if let GridCell::BlockAnchor { pb_type, .. } = cell {
                            *tile_counts.entry(pb_type.clone()).or_insert(0) += 1;
                        }
                    }
                }

                let sorted_counts: Vec<_> = tile_counts.into_iter().collect();

                let table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .column(egui_extras::Column::auto().at_least(100.0))
                    .column(egui_extras::Column::auto().at_least(50.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Tile");
                        });
                        header.col(|ui| {
                            ui.strong("Count");
                        });
                    });

                table.body(|mut body| {
                    for (pb_type, count) in sorted_counts {
                        let color = tile_colors
                            .get(&pb_type)
                            .copied()
                            .unwrap_or(egui::Color32::TRANSPARENT);
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(rect, 0.0, color);
                                ui.label(pb_type.to_uppercase());
                            });
                            row.col(|ui| {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(rect, 0.0, color);
                                ui.label(count.to_string());
                            });
                        });
                    }
                });
            }
        });

    grid_changed
}

fn get_layout_name(arch: &FPGAArch, index: usize) -> String {
    if let Some(layout) = arch.layouts.get(index) {
        match layout {
            fpga_arch_parser::Layout::AutoLayout(_) => "Auto Layout".to_string(),
            fpga_arch_parser::Layout::FixedLayout(fl) => format!("Fixed: {}", fl.name),
        }
    } else {
        "No Layout".to_string()
    }
}

fn update_grid_height_from_width(state: &mut GridState) {
    state.grid_height = (state.grid_width as f32 / state.aspect_ratio)
        .round()
        .max(1.0) as usize;
}

fn update_grid_width_from_height(state: &mut GridState) {
    state.grid_width = (state.grid_height as f32 * state.aspect_ratio)
        .round()
        .max(1.0) as usize;
}