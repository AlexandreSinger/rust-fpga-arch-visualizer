use crate::block_style::darken_color;
use crate::grid::{DeviceGrid, GridCell};
use eframe::egui;
use fpga_arch_parser::FPGAArch;
use std::collections::HashMap;

pub struct GridRenderer {
    // Prerendered shapes that make up the grid.
    grid_shapes: Vec<egui::Shape>,
    // Prerendered shapes that make up the text on the grid.
    text_shapes: Vec<egui::Shape>,
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self {
            grid_shapes: Vec::new(),
            text_shapes: Vec::new(),
        }
    }
}

impl GridRenderer {
    pub fn prerender_grid(
        &mut self,
        grid: &DeviceGrid,
        tile_colors: &HashMap<String, egui::Color32>,
        zoom_factor: f32,
        ui: &egui::Ui,
    ) {
        self.grid_shapes.clear();
        self.text_shapes.clear();
        let cell_size = get_cell_size(grid, zoom_factor, ui);

        // Draw grid
        for row in 0..grid.height {
            for col in 0..grid.width {
                if let Some(cell) = grid.get(row, col) {
                    // Flip y-coordinate so (0,0) is at bottom-left
                    let cell_pos = egui::Pos2::new(
                        col as f32 * cell_size,
                        (grid.height - 1 - row) as f32 * cell_size,
                    );

                    match cell {
                        GridCell::Empty => {
                            // Draw empty cell outline
                            let rect = egui::Rect::from_min_size(
                                cell_pos,
                                egui::vec2(cell_size, cell_size),
                            );
                            self.grid_shapes.push(egui::Shape::rect_stroke(
                                rect,
                                egui::CornerRadius::ZERO,
                                egui::Stroke::new(0.5, egui::Color32::DARK_GRAY),
                                egui::epaint::StrokeKind::Inside,
                            ));
                        }
                        GridCell::BlockAnchor {
                            pb_type,
                            width,
                            height,
                        } => {
                            // Draw merged rectangle for multi-cell tile
                            let tile_width = *width as f32 * cell_size;
                            let tile_height = *height as f32 * cell_size;

                            let visual_top = egui::Pos2::new(
                                col as f32 * cell_size,
                                (grid.height - row - height) as f32 * cell_size,
                            );
                            let rect = egui::Rect::from_min_size(
                                visual_top,
                                egui::vec2(tile_width, tile_height),
                            );

                            let color = tile_colors
                                .get(pb_type)
                                .copied()
                                .unwrap_or(egui::Color32::from_rgb(0xD8, 0xE7, 0xFD));

                            let outline_color = darken_color(color, 0.5);

                            // Draw filled rectangle
                            self.grid_shapes.push(egui::Shape::rect_filled(
                                rect,
                                egui::CornerRadius::ZERO,
                                color,
                            ));

                            // Draw outline
                            // TODO: This can probably be combined with the filled rectangle.
                            self.grid_shapes.push(egui::Shape::rect_stroke(
                                rect,
                                egui::CornerRadius::ZERO,
                                egui::Stroke::new(1.0, outline_color),
                                egui::epaint::StrokeKind::Inside,
                            ));

                            // Only draw the text if the tile is large enough.
                            // TODO: Unify with the render code so we only generate when we need it.
                            if cell_size > 50.0 {
                                // Draw tile name in center (uppercase)
                                let tile_name_upper = pb_type.to_uppercase();
                                let font_size = (cell_size * 0.2).min(tile_height * 0.15);
                                ui.fonts(|fonts| {
                                    self.text_shapes.push(egui::Shape::text(
                                        fonts,
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        &tile_name_upper,
                                        egui::FontId::proportional(font_size),
                                        egui::Color32::BLACK,
                                    ));
                                });
                            }
                        }
                        GridCell::BlockOccupied { .. } => {
                            // Skip - this space is part of an anchor tile and already drawn
                        }
                    }
                }
            }
        }
    }

    pub fn render_grid(
        &mut self,
        ui: &mut egui::Ui,
        grid: &DeviceGrid,
        arch: &FPGAArch,
        zoom_factor: f32,
    ) -> Option<String> {
        // Cell size is based on the available space
        let cell_size = get_cell_size(grid, zoom_factor, ui);

        let mut clicked_tile: Option<String> = None;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let grid_size = egui::vec2(
                    grid.width as f32 * cell_size,
                    grid.height as f32 * cell_size,
                );
                let (response, painter) =
                    ui.allocate_painter(grid_size, egui::Sense::click().union(egui::Sense::hover()));

                let offset = response.rect.min;

                let mut shapes = self.grid_shapes.clone();
                for shape in &mut shapes {
                    shape.translate(offset.to_vec2());
                }

                // Paint all of the shapes.
                painter.extend(shapes);

                if cell_size > 50.0 {
                    let mut text_shapes = Vec::new();
                    for shape in &self.text_shapes {
                        if ui.is_rect_visible(shape.visual_bounding_rect().translate(offset.to_vec2())) {
                            let mut new_shape = shape.clone();
                            new_shape.translate(offset.to_vec2());
                            text_shapes.push(new_shape);
                        }
                    }

                    painter.extend(text_shapes);
                }

                // Check for which tile is currently being hovered over.
                if let Some(hover_pos) = response.hover_pos() {
                    let mut col = ((hover_pos.x - offset.x) / cell_size).floor() as usize;
                    let mut row = grid
                        .height
                        .saturating_sub(1)
                        .saturating_sub(((hover_pos.y - offset.y) / cell_size).floor() as usize);
                    if let Some(GridCell::BlockOccupied {
                        pb_type: _,
                        anchor_row,
                        anchor_col,
                    }) = grid.get(row, col)
                    {
                        col = *anchor_col;
                        row = *anchor_row;
                    }

                    if let Some(GridCell::BlockAnchor {
                        pb_type,
                        width,
                        height,
                    }) = grid.get(row, col)
                    {
                        // If a tile has been clicked, mark it as the clicked tile.
                        if response.clicked() {
                            clicked_tile = Some(pb_type.clone());
                        }

                        // On hover, show ui at the pointer.
                        response.on_hover_ui_at_pointer(|ui| {
                            ui.label(format!("{} [{}, {}]", pb_type, col, row));
                            ui.label(format!("Size: {}x{}", width, height));
                            if let Some(tile) = arch.tiles.iter().find(|t| t.name == *pb_type) {
                                ui.label(format!("Contains {} sub-tiles", tile.sub_tiles.len()));
                            }
                            ui.label("Click to view internal structure");
                        });
                    }
                }
            });

        clicked_tile
    }

}

pub fn get_cell_size(
    grid: &DeviceGrid,
    zoom_factor: f32,
    ui: &egui::Ui,
) -> f32 {
    let available_size = ui.available_size();
    let max_dim = grid.width.max(grid.height).max(1) as f32;
    let cell_size = (available_size.x.min(available_size.y) / max_dim) * zoom_factor;

    return cell_size;
}