use fpga_arch_parser::FPGAArch;

use crate::{
    common_ui,
    intra_tile::{self, IntraTileState},
    viewer::ViewMode,
};

pub struct ComplexBlockViewState {
    pub selected_complex_block_name: Option<String>,
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
                selected_complex_block_name: None,
                intra_tile_state: IntraTileState::default(),
                all_blocks_expanded: false,
                draw_intra_interconnects: true,
            },
        }
    }
}

impl ComplexBlockView {
    pub fn render(
        &mut self,
        arch: &FPGAArch,
        next_view_mode: &mut ViewMode,
        dark_mode: bool,
        ctx: &egui::Context,
    ) {
        self.render_side_panel(arch, ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_complex_block_view(arch, next_view_mode, dark_mode, ui);
        });
    }

    pub fn on_view_open(&mut self, arch: &Option<FPGAArch>) {
        if let Some(arch) = &arch {
            self.apply_expand_all_state(arch);
        }
    }

    pub fn on_view_close(&mut self) {
        self.complex_block_view_state.selected_complex_block_name = None;
    }

    fn render_complex_block_view(
        &mut self,
        arch: &FPGAArch,
        next_view_mode: &mut ViewMode,
        dark_mode: bool,
        ui: &mut egui::Ui,
    ) {
        if let Some(pb_type_name) = &self.complex_block_view_state.selected_complex_block_name {
            if let Some(root_pb) = arch
                .complex_block_list
                .iter()
                .find(|b| b.name == *pb_type_name)
            {
                intra_tile::render_intra_tile_view(
                    ui,
                    root_pb,
                    &mut self.complex_block_view_state.intra_tile_state,
                    self.complex_block_view_state.all_blocks_expanded,
                    self.complex_block_view_state.draw_intra_interconnects,
                    dark_mode,
                );
            } else if common_ui::render_centered_message(
                ui,
                "Complex block not found",
                &format!("Could not find complex block: {}", pb_type_name),
                Some("Go to Grid View"),
            ) {
                *next_view_mode = ViewMode::Grid;
            }
        } else if common_ui::render_centered_message(
            ui,
            "No complex block selected",
            "Please select a complex block from the dropdown or click on a tile in the grid view.",
            Some("Go to Grid View"),
        ) {
            *next_view_mode = ViewMode::Grid;
        }
    }

    fn render_side_panel(&mut self, arch: &FPGAArch, ctx: &egui::Context) {
        egui::SidePanel::right("complex_block_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let should_expand_all = render_intra_tile_controls_panel(
                            ui,
                            arch,
                            &mut self.complex_block_view_state.all_blocks_expanded,
                            &mut self.complex_block_view_state.draw_intra_interconnects,
                            &mut self.complex_block_view_state.selected_complex_block_name,
                        );
                        if should_expand_all {
                            self.apply_expand_all_state(arch);
                        }
                    });
            });
    }

    fn apply_expand_all_state(&mut self, arch: &FPGAArch) {
        if self.complex_block_view_state.all_blocks_expanded {
            if let Some(pb_type_name) = &self.complex_block_view_state.selected_complex_block_name
                && let Some(root_pb) = arch
                    .complex_block_list
                    .iter()
                    .find(|b| b.name == *pb_type_name)
            {
                intra_tile::expand_all_blocks(
                    &mut self.complex_block_view_state.intra_tile_state,
                    root_pb,
                    &root_pb.name,
                );
            }
        } else {
            intra_tile::collapse_all_blocks(&mut self.complex_block_view_state.intra_tile_state);
        }
    }
}

/// Renders the intra-tile view controls panel on the right side
fn render_intra_tile_controls_panel(
    ui: &mut egui::Ui,
    arch: &FPGAArch,
    all_blocks_expanded: &mut bool,
    draw_intra_interconnects: &mut bool,
    selected_complex_block_name: &mut Option<String>,
) -> bool {
    let mut expand_all = false;

    ui.heading("Complex Block View");
    ui.add_space(10.0);

    // Expand All toggle
    let mut expand_all_toggle_val = *all_blocks_expanded;
    if ui
        .checkbox(&mut expand_all_toggle_val, "Expand All")
        .changed()
    {
        *all_blocks_expanded = expand_all_toggle_val;
        expand_all = true;
    }

    // Interconnect toggle
    ui.checkbox(draw_intra_interconnects, "Draw Interconnects");

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Tile selector
    if !arch.complex_block_list.is_empty() {
        ui.label("Select Complex Block:");
        ui.add_space(5.0);

        let mut selected_complex_block_name_str = selected_complex_block_name
            .as_deref()
            .unwrap_or("")
            .to_string();

        egui::ComboBox::from_id_salt("complex_block_selector")
            .selected_text(if !selected_complex_block_name_str.is_empty() {
                selected_complex_block_name_str.as_str()
            } else {
                "Select a complex block"
            })
            .show_ui(ui, |ui| {
                for pb_type in &arch.complex_block_list {
                    ui.selectable_value(
                        &mut selected_complex_block_name_str,
                        pb_type.name.clone(),
                        &pb_type.name,
                    );
                }
            });

        // If tile selection changed, update state
        if selected_complex_block_name_str != selected_complex_block_name.as_deref().unwrap_or("") {
            *selected_complex_block_name = Some(selected_complex_block_name_str);
            expand_all = true;
        }
    } else {
        ui.label("No complex blocks available in architecture");
    }

    expand_all
}
