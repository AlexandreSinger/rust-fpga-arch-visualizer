use egui;
use fpga_arch_parser::FPGAArch;

use crate::{common_ui, intra_tile::{self, IntraTileState}, viewer::ViewMode};

pub struct ComplexBlockViewState {
    pub selected_tile_name: Option<String>,
    pub selected_sub_tile_index: usize,
    pub show_hierarchy_tree: bool,
    pub intra_tile_state: IntraTileState,
    pub all_blocks_expanded: bool,
    pub draw_intra_interconnects: bool,
}

pub struct ComplexBlockView {
    pub complex_block_view_state: ComplexBlockViewState,
}

impl Default for ComplexBlockView {
    fn default() -> Self {
        Self {
            complex_block_view_state: ComplexBlockViewState {
                selected_tile_name: None,
                selected_sub_tile_index: 0,
                show_hierarchy_tree: false,
                intra_tile_state: IntraTileState::default(),
                all_blocks_expanded: false,
                draw_intra_interconnects: true,
            }
        }
    }
}

impl ComplexBlockView {

pub fn render(
    &mut self,
    arch: &FPGAArch,
    next_view_mode: &mut ViewMode,
    dark_mode: bool,
    ui: &mut egui::Ui,
) {
    if let Some(tile_name) = &self.complex_block_view_state.selected_tile_name {
        if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
            let sub_tile_index = if self.complex_block_view_state.selected_sub_tile_index < tile.sub_tiles.len() {
                self.complex_block_view_state.selected_sub_tile_index
            } else {
                0
            };
            intra_tile::render_intra_tile_view(
                ui,
                arch,
                tile,
                &mut self.complex_block_view_state.intra_tile_state,
                self.complex_block_view_state.show_hierarchy_tree,
                sub_tile_index,
                self.complex_block_view_state.all_blocks_expanded,
                self.complex_block_view_state.draw_intra_interconnects,
                dark_mode,
            );
        } else {
            if common_ui::render_centered_message(
                ui,
                "Tile not found",
                &format!("Could not find tile: {}", tile_name),
                Some("Back to Grid View"),
            ) {
                *next_view_mode = ViewMode::Grid;
            }
        }
    } else {
        if common_ui::render_centered_message(
            ui,
            "No tile selected",
            "Please select a tile from the dropdown or click on a tile in the grid view.",
            Some("Back to Grid View"),
        ) {
            *next_view_mode = ViewMode::Grid;
        }
    }
}

pub fn render_side_panel(
    &mut self,
    arch: &FPGAArch,
    ctx: &egui::Context,
) {
    let should_expand_all = render_intra_tile_controls_panel(
        ctx,
        arch,
        &mut self.complex_block_view_state.show_hierarchy_tree,
        &mut self.complex_block_view_state.all_blocks_expanded,
        &mut self.complex_block_view_state.draw_intra_interconnects,
        &mut self.complex_block_view_state.selected_tile_name,
        &mut self.complex_block_view_state.selected_sub_tile_index,
    );
    if should_expand_all {
        self.apply_expand_all_state(arch);
    }
}

pub fn on_view_open(
    &mut self,
    arch: &Option<FPGAArch>,
) {
    if let Some(arch) = &arch {
        self.apply_expand_all_state(arch);
    }
}

pub fn on_view_close(
    &mut self,
) {
    self.complex_block_view_state.selected_tile_name = None;
}

pub fn apply_expand_all_state(&mut self, arch: &FPGAArch) {
    if self.complex_block_view_state.all_blocks_expanded {
        if let Some(tile_name) = &self.complex_block_view_state.selected_tile_name {
            if let Some(tile) = arch.tiles.iter().find(|t| t.name == *tile_name) {
                if self.complex_block_view_state.selected_sub_tile_index < tile.sub_tiles.len() {
                    let sub_tile = &tile.sub_tiles[self.complex_block_view_state.selected_sub_tile_index];
                    if let Some(site) = sub_tile.equivalent_sites.first() {
                        if let Some(root_pb) = arch
                            .complex_block_list
                            .iter()
                            .find(|pb| pb.name == site.pb_type)
                        {
                            intra_tile::expand_all_blocks(
                                &mut self.complex_block_view_state.intra_tile_state,
                                root_pb,
                                &root_pb.name,
                            );
                        }
                    }
                }
            }
        }
    } else {
        intra_tile::collapse_all_blocks(&mut self.complex_block_view_state.intra_tile_state);
    }
}

}

/// Renders the intra-tile view controls panel on the right side
pub fn render_intra_tile_controls_panel(
    ctx: &egui::Context,
    arch: &FPGAArch,
    show_hierarchy_tree: &mut bool,
    all_blocks_expanded: &mut bool,
    draw_intra_interconnects: &mut bool,
    selected_tile_name: &mut Option<String>,
    selected_sub_tile_index: &mut usize,
) -> bool {
    let mut expand_all = false;

    egui::SidePanel::right("complex_block_controls")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Complex Block View");
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
        });

    expand_all
}