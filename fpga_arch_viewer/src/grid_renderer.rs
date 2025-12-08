use eframe::egui;
use crate::block_style::{DefaultBlockStyles, draw_block, darken_color};
use crate::grid::{DeviceGrid, GridCell};
use std::collections::HashMap;

pub fn render_grid(ui: &mut egui::Ui, grid: &DeviceGrid, block_styles: &DefaultBlockStyles, tile_colors: &HashMap<String, egui::Color32>) {
    // Cell size is based on the available space
    // TODO: may need to change later for routing
    let available_size = ui.available_size();
    let cell_size = (available_size.x.min(available_size.y) * 0.9) / grid.width.max(grid.height) as f32;
    let cell_size = cell_size.max(30.0).min(100.0);

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical(|ui| {
                for row in 0..grid.height {
                    ui.horizontal(|ui| {
                        for col in 0..grid.width {
                                if let Some(cell) = grid.get(row, col) {
                                    match cell {
                                        GridCell::Empty => {
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(cell_size, cell_size),
                                                egui::Sense::hover(),
                                            );

                                            ui.painter().rect_stroke(
                                                rect,
                                                0.0,
                                                egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                                            );
                                        }
                                        GridCell::Block(pb_type) => {
                                            // Get color from tile_colors map, fallback to default
                                            let color = tile_colors.get(pb_type)
                                                .copied()
                                                .unwrap_or(egui::Color32::from_rgb(0xD8, 0xE7, 0xFD));

                                            let (rect, response) = ui.allocate_exact_size(
                                                egui::vec2(cell_size, cell_size),
                                                egui::Sense::hover(),
                                            );

                                            if ui.is_rect_visible(rect) {
                                                let painter = ui.painter();
                                                let outline_color = darken_color(color, 0.5);

                                                // Draw filled square
                                                painter.rect_filled(rect, 0.0, color);

                                                // Draw outline
                                                painter.rect_stroke(
                                                    rect,
                                                    0.0,
                                                    egui::Stroke::new(2.0, outline_color),
                                                );

                                                // Draw tile name in center (uppercase)
                                                let tile_name_upper = pb_type.to_uppercase();
                                                painter.text(
                                                    rect.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    &tile_name_upper,
                                                    egui::FontId::proportional(cell_size * 0.2),
                                                    egui::Color32::BLACK,
                                                );
                                            }

                                            if response.hovered() {
                                                egui::show_tooltip_at_pointer(
                                                    ui.ctx(),
                                                    egui::Id::new(format!("grid_{}_{}", row, col)),
                                                    |ui| {
                                                        ui.label(format!("{} [{}, {}]", pb_type, row, col));
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        });
                    }
                });
            });
}
