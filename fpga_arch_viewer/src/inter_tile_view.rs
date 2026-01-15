use egui;
use egui_extras;
use fpga_arch_parser::FPGAArch;
use std::collections::HashMap;

use crate::grid::{DeviceGrid, GridCell};

/// State for inter-tile (grid) view
#[derive(Debug, Clone)]
pub struct InterTileState {
    pub grid_width: usize,
    pub grid_height: usize,
    pub aspect_ratio: f32,
    pub selected_layout_index: usize,
}

impl Default for InterTileState {
    fn default() -> Self {
        Self {
            grid_width: 10,
            grid_height: 10,
            aspect_ratio: 1.0,
            selected_layout_index: 0,
        }
    }
}

/// Renders the grid controls panel on the right side
/// Returns true if grid dimensions changed
pub fn render_grid_controls_panel(
    ctx: &egui::Context,
    arch: Option<&FPGAArch>,
    state: &mut InterTileState,
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
            if let Some(arch) = arch {
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
            }

            // Check if current layout is fixed
            let is_fixed_layout = if let Some(arch) = arch {
                matches!(
                    arch.layouts.get(state.selected_layout_index),
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

fn update_grid_height_from_width(state: &mut InterTileState) {
    state.grid_height = (state.grid_width as f32 / state.aspect_ratio)
        .round()
        .max(1.0) as usize;
}

fn update_grid_width_from_height(state: &mut InterTileState) {
    state.grid_width = (state.grid_height as f32 * state.aspect_ratio)
        .round()
        .max(1.0) as usize;
}