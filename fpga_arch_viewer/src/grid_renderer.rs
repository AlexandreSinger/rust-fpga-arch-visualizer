use crate::block_style::{darken_color, DefaultBlockStyles};
use crate::grid::{DeviceGrid, GridCell};
use eframe::egui;
use std::collections::HashMap;

pub fn render_grid(
    ui: &mut egui::Ui,
    grid: &DeviceGrid,
    _block_styles: &DefaultBlockStyles,
    tile_colors: &HashMap<String, egui::Color32>,
    _dark_mode: bool,
) -> Option<String> {
    // Cell size is based on the available space
    let available_size = ui.available_size();
    let cell_size =
        (available_size.x.min(available_size.y) * 0.9) / grid.width.max(grid.height) as f32;
    let cell_size = cell_size.max(30.0).min(100.0);

    let mut clicked_tile: Option<String> = None;

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let grid_size = egui::vec2(grid.width as f32 * cell_size, grid.height as f32 * cell_size);
            let (response, painter) = ui.allocate_painter(
                grid_size,
                egui::Sense::click().union(egui::Sense::hover()),
            );

            let offset = response.rect.min;

            // Draw grid
            for row in 0..grid.height {
                for col in 0..grid.width {
                    if let Some(cell) = grid.get(row, col) {
                        let cell_pos = offset + egui::vec2(col as f32 * cell_size, row as f32 * cell_size);

                        match cell {
                            GridCell::Empty => {
                                // Draw empty cell outline
                                let rect = egui::Rect::from_min_size(cell_pos, egui::vec2(cell_size, cell_size));
                                painter.rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                                );
                            }
                            GridCell::BlockAnchor { pb_type, width, height } => {
                                // Draw merged rectangle for multi-cell tile
                                let tile_width = *width as f32 * cell_size;
                                let tile_height = *height as f32 * cell_size;
                                let rect = egui::Rect::from_min_size(cell_pos, egui::vec2(tile_width, tile_height));

                                let color = tile_colors
                                    .get(pb_type)
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(0xD8, 0xE7, 0xFD));

                                let outline_color = darken_color(color, 0.5);

                                // Draw filled rectangle
                                painter.rect_filled(rect, 0.0, color);

                                // Draw outline
                                painter.rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(2.0, outline_color),
                                );

                                // Draw tile name in center (uppercase)
                                let tile_name_upper = pb_type.to_uppercase();
                                let font_size = (cell_size * 0.2).min(tile_height * 0.15);
                                painter.text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &tile_name_upper,
                                    egui::FontId::proportional(font_size),
                                    egui::Color32::BLACK,
                                );

                                // Check if mouse is hovering over this tile
                                if let Some(hover_pos) = response.hover_pos() {
                                    if rect.contains(hover_pos) {
                                        egui::show_tooltip_at_pointer(
                                            ui.ctx(),
                                            egui::Id::new(format!("grid_{}_{}", row, col)),
                                            |ui| {
                                                ui.label(format!(
                                                    "{} [{}, {}]",
                                                    pb_type, row, col
                                                ));
                                                ui.label(format!("Size: {}x{}", width, height));
                                                ui.label("Click to view internal structure");
                                            },
                                        );
                                    }
                                }

                                // Check if mouse clicked on this tile
                                if response.clicked() {
                                    if let Some(click_pos) = response.interact_pointer_pos() {
                                        if rect.contains(click_pos) {
                                            clicked_tile = Some(pb_type.clone());
                                        }
                                    }
                                }
                            }
                            GridCell::BlockOccupied { .. } => {
                                // Skip - this space is part of an anchor tile and already drawn
                            }
                        }
                    }
                }
            }
        });

    clicked_tile
}
