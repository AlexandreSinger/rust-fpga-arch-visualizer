use fpga_arch_parser::FPGAArch;

#[derive(Default)]
pub struct TileView {
    pub selected_tile_name: Option<String>,
}

impl TileView {
    pub fn render(&mut self, arch: &FPGAArch, ctx: &egui::Context) {
        egui::SidePanel::right("tile_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_side_panel(arch, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_tile(ui);
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

    fn render_tile(&mut self, ui: &mut egui::Ui) {
        match &self.selected_tile_name {
            Some(tile_name) => ui.label(format!("Selected tile: {tile_name}")),
            None => ui.label("No tile selected."),
        };
    }
}
