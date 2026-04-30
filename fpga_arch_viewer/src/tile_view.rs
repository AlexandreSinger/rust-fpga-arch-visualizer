use egui::ScrollArea;
use fpga_arch_parser::FPGAArch;

use std::collections::HashMap;

use crate::{
    color_scheme, common_ui, complex_block_view::ComplexBlockViewState, intra_hierarchy_tree,
    tile_rendering::tile_renderer::build_render_tile, viewer::ViewMode,
};

pub struct TileView {
    pub selected_tile_name: Option<String>,
    pub tile_zoom: f32,
}

impl Default for TileView {
    fn default() -> Self {
        Self {
            selected_tile_name: None,
            tile_zoom: 1.0,
        }
    }
}

impl TileView {
    pub fn render(
        &mut self,
        arch: &FPGAArch,
        complex_block_view_state: &mut ComplexBlockViewState,
        next_view_mode: &mut ViewMode,
        tile_colors: &HashMap<String, egui::Color32>,
        dark_mode: bool,
        ctx: &egui::Context,
    ) {
        egui::SidePanel::right("tile_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_side_panel(arch, ui);
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(
                arch,
                complex_block_view_state,
                next_view_mode,
                tile_colors,
                dark_mode,
                ui,
            );
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
        tile_colors: &HashMap<String, egui::Color32>,
        dark_mode: bool,
        ui: &mut egui::Ui,
    ) {
        match &self.selected_tile_name {
            Some(tile_name) => {
                if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                    self.render_tile(
                        tile,
                        complex_block_view_state,
                        next_view_mode,
                        tile_colors,
                        dark_mode,
                        arch,
                        ui,
                    );
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
        tile_colors: &HashMap<String, egui::Color32>,
        dark_mode: bool,
        arch: &FPGAArch,
        ui: &mut egui::Ui,
    ) {
        // Fixed tile visualization panel at the top
        ui.heading(format!("Tile: {}", tile.name));
        ui.add_space(5.0);

        // Handle zoom input (Cmd + scroll wheel or pinch gesture)
        let input = ui.input(|i| {
            let scroll_delta = i.raw_scroll_delta.y;
            let zoom_modifier = i.modifiers.command;
            (scroll_delta, zoom_modifier)
        });
        let (scroll_delta, zoom_modifier) = input;
        if zoom_modifier && scroll_delta != 0.0 {
            if scroll_delta > 0.0 {
                self.tile_zoom *= 1.1;
            } else {
                self.tile_zoom /= 1.1;
            }
        }
        // Check for pinch gesture (trackpad zoom on macOS)
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            self.tile_zoom *= zoom_delta;
        }

        // Fixed region for tile shape and pins visualization
        ui.allocate_ui(egui::vec2(ui.available_width(), 300.0), |ui| {
            ScrollArea::both()
                .id_salt("tile_visualization_scroll")
                .show(ui, |ui| {
                    let tile_size =
                        egui::vec2(250.0 * tile.width as f32, 250.0 * tile.height as f32)
                            * self.tile_zoom;
                    let painter_size = egui::vec2(
                        (tile_size.x * 1.05).max(ui.available_width()),
                        (tile_size.y * 1.05).max(ui.available_height()),
                    );
                    let (response, painter) = ui.allocate_painter(
                        painter_size,
                        egui::Sense::click().union(egui::Sense::hover()),
                    );
                    let tile_bounding_box =
                        egui::Rect::from_center_size(response.rect.center(), tile_size);
                    let color = tile_colors
                        .get(&tile.name)
                        .copied()
                        .unwrap_or(color_scheme::grid_lb_color(dark_mode));
                    let tile_renderer = build_render_tile(tile, &tile_bounding_box, &color, dark_mode);
                    painter.extend(tile_renderer.lb_shapes);
                    painter.extend(tile_renderer.pin_shapes);

                    // When hovering over a pin, print the name of the pin.
                    for (pin_index, pin_locations) in tile_renderer.pin_locations.iter().enumerate()
                    {
                        let pin_name = &tile.pin_mapper.pin_name_lookup[pin_index];
                        for pin_location in pin_locations {
                            let hit_rect = egui::Rect::from_center_size(
                                pin_location.to_pos2(),
                                egui::Vec2::ONE * tile_renderer.pin_radius * 3.0,
                            );
                            let pin_hit_response = ui.put(hit_rect, egui::Label::new(""));
                            pin_hit_response.on_hover_ui(|ui| {
                                ui.label(pin_name);
                            });
                        }
                    }
                });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Scrollable details section below the visualization
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
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
                ui.allocate_ui(egui::vec2(ui.available_width(), 450.0), |ui| {
                    ui.group(|ui| {
                        ui.heading("Hierarchy Tree");
                        ui.separator();
                        egui::ScrollArea::both()
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                intra_hierarchy_tree::render_hierarchy_tree(ui, arch, tile);
                            });
                    });
                });
            });
    }
}
