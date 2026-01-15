use egui;
use fpga_arch_parser::FPGAArch;

/// Renders the intra-tile view controls panel on the right side
pub fn render_intra_tile_controls_panel(
    ctx: &egui::Context,
    arch: Option<&FPGAArch>,
    show_hierarchy_tree: &mut bool,
    all_blocks_expanded: &mut bool,
    draw_intra_interconnects: &mut bool,
    selected_tile_name: &mut Option<String>,
    selected_sub_tile_index: &mut usize,
) -> bool {
    if arch.is_none() {
        return false;
    }

    let mut expand_all = false;

    egui::SidePanel::right("intra_tile_controls")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Intra-Tile View");
            ui.add_space(10.0);

            // Hierarchy tree toggle
            ui.checkbox(show_hierarchy_tree, "Show Hierarchy Tree");

            // Expand All toggle
            let mut expand_all_toggle_val = *all_blocks_expanded;
            if ui.checkbox(&mut expand_all_toggle_val, "Expand All").changed() {
                *all_blocks_expanded = expand_all_toggle_val;
                expand_all = true;
            }

            // Interconnect toggle
            ui.checkbox(draw_intra_interconnects, "Draw Interconnects");

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Tile selector
            if let Some(arch) = arch {
                if !arch.tiles.is_empty() {
                    ui.label("Select Tile:");
                    ui.add_space(5.0);

                    let mut selected_tile_name_str =
                        selected_tile_name.as_deref().unwrap_or("").to_string();

                    egui::ComboBox::from_id_source("tile_selector")
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
                    if selected_tile_name_str
                        != selected_tile_name.as_deref().unwrap_or("")
                    {
                        *selected_tile_name = Some(selected_tile_name_str);
                        *selected_sub_tile_index = 0;
                        expand_all = true;
                    }
                } else {
                    ui.label("No tiles available in architecture");
                }
            }
        });

    expand_all
}