use fpga_arch_parser::FPGAArch;

use crate::{common_ui, viewer::ViewMode};

#[derive(Default)]
pub struct TileView {
    pub selected_tile_name: Option<String>,
}

impl TileView {
    pub fn render(&mut self, arch: &FPGAArch, next_view_mode: &mut ViewMode, ctx: &egui::Context) {
        egui::SidePanel::right("tile_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_side_panel(arch, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(arch, next_view_mode, ui);
        });
    }

    fn render_side_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        ui.heading("Tile View");

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Tile selector.
        if !arch.tiles.is_empty() {
            ui.label("Select Tile:");
            ui.add_space(5.0);

            let mut selected_tile_name_str =
                self.selected_tile_name.clone().unwrap_or("".to_string());

            egui::ComboBox::from_id_salt("tile_selector_combobox")
                .selected_text(if !selected_tile_name_str.is_empty() {
                    selected_tile_name_str.as_str()
                } else {
                    "Select a tile"
                })
                .show_ui(ui, |ui| {
                    for tile in &arch.tiles {
                        ui.selectable_value(
                            &mut selected_tile_name_str,
                            tile.name.clone(),
                            &tile.name,
                        );
                    }
                });

            // If tile selection changed, update state
            if selected_tile_name_str != self.selected_tile_name.clone().unwrap_or("".to_string()) {
                self.selected_tile_name = Some(selected_tile_name_str);
            }
        } else {
            ui.label("No tiles available in architecture");
        }
    }

    fn render_central_panel(
        &mut self,
        arch: &FPGAArch,
        next_view_mode: &mut ViewMode,
        ui: &mut egui::Ui,
    ) {
        match &self.selected_tile_name {
            Some(tile_name) => {
                if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                    self.render_tile(tile, ui);
                } else if common_ui::render_centered_message(
                    ui,
                    "Tile not found",
                    &format!("Could not find tile: {}", tile_name),
                    Some("Back to Grid View"),
                ) {
                    *next_view_mode = ViewMode::Grid;
                }
            }
            None => {
                if common_ui::render_centered_message(
                    ui,
                    "No tile selected",
                    "Please select a tile from the dropdown or clock on a tile in the grid view.",
                    Some("Back to Grid View"),
                ) {
                    *next_view_mode = ViewMode::Grid;
                }
            }
        };
    }

    fn render_tile(&mut self, tile: &fpga_arch_parser::Tile, ui: &mut egui::Ui) {
        ui.label(format!("Selected tile: {}", tile.name));
    }
}
