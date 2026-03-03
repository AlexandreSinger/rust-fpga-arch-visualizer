use fpga_arch_parser::FPGAArch;

use crate::{
    common_ui, complex_block_view::ComplexBlockViewState, intra_hierarchy_tree, viewer::ViewMode,
};

#[derive(Default)]
pub struct TileView {
    pub selected_tile_name: Option<String>,
}

impl TileView {
    pub fn render(
        &mut self,
        arch: &FPGAArch,
        complex_block_view_state: &mut ComplexBlockViewState,
        next_view_mode: &mut ViewMode,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::right("tile_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_side_panel(arch, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(arch, complex_block_view_state, next_view_mode, ui);
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
                self.selected_tile_name.as_deref().unwrap_or("").to_string();

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
            if selected_tile_name_str != self.selected_tile_name.as_deref().unwrap_or("") {
                self.selected_tile_name = Some(selected_tile_name_str);
            }
        } else {
            ui.label("No tiles available in architecture");
        }
    }

    fn render_central_panel(
        &mut self,
        arch: &FPGAArch,
        complex_block_view_state: &mut ComplexBlockViewState,
        next_view_mode: &mut ViewMode,
        ui: &mut egui::Ui,
    ) {
        match &self.selected_tile_name {
            Some(tile_name) => {
                if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                    self.render_tile(tile, complex_block_view_state, next_view_mode, arch, ui);
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
                    "Please select a tile from the dropdown or click on a tile in the grid view.",
                    Some("Back to Grid View"),
                ) {
                    *next_view_mode = ViewMode::Grid;
                }
            }
        };
    }

    fn render_tile(
        &mut self,
        tile: &fpga_arch_parser::Tile,
        complex_block_view_state: &mut ComplexBlockViewState,
        next_view_mode: &mut ViewMode,
        arch: &FPGAArch,
        ui: &mut egui::Ui,
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.heading(format!("Tile: {}", tile.name));
                ui.add_space(10.0);

                // Tile Info Section
                ui.group(|ui| {
                    ui.heading("Tile Information");
                    ui.separator();

                    ui.label(format!("Dimensions: {}x{}", tile.width, tile.height));
                    ui.label(format!(
                        "Area: {}",
                        tile.area
                            .map(|a| format!("{:.2}", a))
                            .unwrap_or_else(|| "N/A".to_string())
                    ));
                });

                ui.add_space(10.0);

                // PB Types Summary Section
                let mut pb_types = std::collections::HashSet::new();
                for sub_tile in &tile.sub_tiles {
                    for equiv_site in &sub_tile.equivalent_sites {
                        pb_types.insert(equiv_site.pb_type.clone());
                    }
                }

                if !pb_types.is_empty() {
                    ui.group(|ui| {
                        ui.heading(format!("Contained Complex Blocks ({})", pb_types.len()));
                        ui.separator();

                        let mut pb_types_vec: Vec<_> = pb_types.iter().collect();
                        pb_types_vec.sort();
                        for pb_type in pb_types_vec {
                            ui.horizontal(|ui| {
                                ui.label(pb_type);
                                if ui
                                    .button(format!("View {} Block Details", pb_type))
                                    .clicked()
                                {
                                    complex_block_view_state.selected_complex_block_name =
                                        Some(pb_type.clone());
                                    *next_view_mode = ViewMode::ComplexBlock;
                                }
                            });
                        }
                    });

                    ui.add_space(10.0);
                }

                // Ports Section
                if !tile.ports.is_empty() {
                    ui.group(|ui| {
                        ui.heading(format!("Ports ({})", tile.ports.len()));
                        ui.separator();

                        for port in &tile.ports {
                            let port_info = match port {
                                fpga_arch_parser::Port::Input(p) => {
                                    format!("{} (Input, {} pins)", p.name, p.num_pins)
                                }
                                fpga_arch_parser::Port::Output(p) => {
                                    format!("{} (Output, {} pins)", p.name, p.num_pins)
                                }
                                fpga_arch_parser::Port::Clock(p) => {
                                    format!("{} (Clock, {} pins)", p.name, p.num_pins)
                                }
                            };
                            ui.label(port_info);
                        }
                    });

                    ui.add_space(10.0);
                }

                // Sub-tiles Section
                ui.group(|ui| {
                    ui.heading(format!("Sub-tiles ({})", tile.sub_tiles.len()));
                    ui.separator();

                    for (idx, sub_tile) in tile.sub_tiles.iter().enumerate() {
                        ui.collapsing(format!("[{}] {}", idx, &sub_tile.name), |ui| {
                            ui.label(format!("Capacity: {}", sub_tile.capacity));
                            ui.label(format!("Ports: {}", sub_tile.ports.len()));

                            if !sub_tile.equivalent_sites.is_empty() {
                                ui.collapsing(
                                    format!(
                                        "Equivalent Sites ({})",
                                        sub_tile.equivalent_sites.len()
                                    ),
                                    |ui| {
                                        for (site_idx, equiv_site) in
                                            sub_tile.equivalent_sites.iter().enumerate()
                                        {
                                            ui.label(format!(
                                                "[{}] {} (Pin Mapping: {:?})",
                                                site_idx,
                                                equiv_site.pb_type,
                                                equiv_site.pin_mapping
                                            ));
                                        }
                                    },
                                );
                            } else {
                                ui.label("No equivalent sites");
                            }
                        });
                    }
                });

                ui.add_space(10.0);

                // Switch Block Locations Section
                if tile.switchblock_locations.is_some() {
                    ui.group(|ui| {
                        ui.heading("Switch Block Locations");
                        ui.separator();
                        ui.label("Switch block locations configured for this tile");
                    });
                }

                ui.add_space(10.0);

                // Hierarchy Tree Section
                ui.group(|ui| {
                    ui.heading("Hierarchy Tree");
                    ui.separator();
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            intra_hierarchy_tree::render_hierarchy_tree(ui, arch, tile);
                        });
                });
            });
    }
}
