use crate::block_style::{DefaultBlockStyles, draw_block};
use crate::grid::{DeviceGrid, GridCell};
use eframe::egui;

pub fn render_grid(
    ui: &mut egui::Ui,
    grid: &DeviceGrid,
    block_styles: &DefaultBlockStyles,
) -> Option<String> {
    // Cell size is based on the available space
    // TODO: may need to change later for routing
    let available_size = ui.available_size();
    let cell_size =
        (available_size.x.min(available_size.y) * 0.9) / grid.width.max(grid.height) as f32;
    let cell_size = cell_size.max(30.0).min(100.0);

    let mut clicked_tile: Option<String> = None;

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
                                        let block_style = match pb_type.as_str() {
                                            "io" => &block_styles.io,
                                            "clb" => &block_styles.lb,
                                            _ => &block_styles.lb,
                                        };

                                        let response = draw_block(ui, block_style, cell_size);

                                        if response.hovered() {
                                            egui::show_tooltip_at_pointer(
                                                ui.ctx(),
                                                egui::Id::new(format!("grid_{}_{}", row, col)),
                                                |ui| {
                                                    ui.label(format!(
                                                        "{} [{}, {}]",
                                                        pb_type, row, col
                                                    ));
                                                    ui.label("Click to view internal structure");
                                                },
                                            );
                                        }

                                        if response.clicked() {
                                            clicked_tile = Some(pb_type.clone());
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            });
        });

    clicked_tile
}
