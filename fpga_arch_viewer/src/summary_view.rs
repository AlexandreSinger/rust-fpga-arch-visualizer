use egui;
use crate::viewer::ViewMode;
use fpga_arch_parser::FPGAArch;

pub struct SummaryView {

}

impl Default for SummaryView {
    fn default() -> Self {
        Self {
        }
    }
}

impl SummaryView {

pub fn render(
    &mut self,
    arch: &FPGAArch,
    next_view_mode: &mut ViewMode,
    ui: &mut egui::Ui
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.heading("FPGA Architecture Summary");
            ui.add_space(10.0);

            // Device Info Section
            ui.group(|ui| {
                ui.heading("Device Information");
                ui.separator();

                ui.label(format!(
                    "Grid Logic Tile Area: {:.2}",
                    arch.device.area.grid_logic_tile_area
                ));
                ui.label(format!(
                    "Switch Block Type: {:?}",
                    arch.device.switch_block.sb_type
                ));
                if let Some(fs) = arch.device.switch_block.sb_fs {
                    ui.label(format!("Switch Block Fs: {}", fs));
                }
            });

            ui.add_space(10.0);

            // Tiles Section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("Tiles ({})", arch.tiles.len()));
                    if ui.button("View Tile Grid").clicked() {
                        *next_view_mode = ViewMode::Grid;
                    }
                });
                ui.separator();

                for (tile_idx, tile) in arch.tiles.iter().enumerate() {
                    ui.collapsing(format!("[{}] Tile: {}", tile_idx, &tile.name), |ui| {
                        ui.label(format!(
                            "Dimensions: {}x{} (Area: {})",
                            tile.width,
                            tile.height,
                            tile.area
                                .map(|a| format!("{:.2}", a))
                                .unwrap_or_else(|| "N/A".to_string())
                        ));
                        ui.label(format!("Sub-tiles: {}", tile.sub_tiles.len()));

                        for (idx, sub_tile) in tile.sub_tiles.iter().enumerate() {
                            ui.label(format!(
                                "  [{}] {} (capacity: {})",
                                idx, sub_tile.name, sub_tile.capacity
                            ));
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Layouts Section
            ui.group(|ui| {
                ui.heading(format!("Layouts ({})", arch.layouts.len()));
                ui.separator();

                for (idx, layout) in arch.layouts.iter().enumerate() {
                    let layout_name = match layout {
                        fpga_arch_parser::Layout::AutoLayout(_) => "Auto Layout",
                        fpga_arch_parser::Layout::FixedLayout(fixed_layout) => &fixed_layout.name,
                    };
                    ui.collapsing(format!("[{}] {}", idx, layout_name), |ui| {
                        match layout {
                            fpga_arch_parser::Layout::AutoLayout(auto_layout) => {
                                ui.label(format!(
                                    "Auto Layout - Aspect Ratio: {:.2}",
                                    auto_layout.aspect_ratio
                                ));
                            }
                            fpga_arch_parser::Layout::FixedLayout(fixed_layout) => {
                                ui.label(format!(
                                    "Fixed Layout: {} ({}x{})",
                                    fixed_layout.name, fixed_layout.width, fixed_layout.height
                                ));
                            }
                        }
                    });
                }
            });

            ui.add_space(10.0);

            // Switches Section
            ui.group(|ui| {
                ui.heading(format!("Switches ({})", arch.switch_list.len()));
                ui.separator();

                ui.collapsing("Switches", |ui| {
                    for switch in &arch.switch_list {
                        ui.label(format!(
                            "{}: {:?} (R: {}, C_in: {}, C_out: {})",
                            switch.name, switch.sw_type, switch.resistance, switch.c_in, switch.c_out
                        ));
                    }
                });
            });

            ui.add_space(10.0);

            // Segments Section
            ui.group(|ui| {
                ui.heading(format!("Segments ({})", arch.segment_list.len()));
                ui.separator();

                ui.collapsing("Segments", |ui| {
                    for (seg_idx, segment) in arch.segment_list.iter().enumerate() {
                        ui.collapsing(format!("[{}] L{}: {}", seg_idx, segment.length, &segment.name), |ui| {
                            ui.label(format!("Axis: {:?}", segment.axis));
                            ui.label(format!("Type: {:?}", segment.segment_type));
                            ui.label(format!("Length: {}", segment.length));
                            ui.label(format!("Frequency: {:.2}", segment.freq));
                            ui.label(format!(
                                "Metal: R={:.2}, C={:.2}",
                                segment.r_metal, segment.c_metal
                            ));
                        });
                    }
                });
            });

            ui.add_space(10.0);

            // Complex Blocks Section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("Complex Blocks ({})", arch.complex_block_list.len()));
                    if ui.button("View Complex Block Details").clicked() {
                        *next_view_mode = ViewMode::ComplexBlock;
                    }
                });
                ui.separator();

                ui.collapsing("Complex Blocks", |ui| {
                    for (pb_idx, pb_type) in arch.complex_block_list.iter().enumerate() {
                        ui.collapsing(format!("[{}] Complex Block: {}", pb_idx, &pb_type.name), |ui| {
                            ui.label(format!("Number of blocks: {}", pb_type.num_pb));
                            ui.label(format!("Modes: {}", pb_type.modes.len()));
                            ui.label(format!("Ports: {}", pb_type.ports.len()));
                        });
                    }
                });
            });

            ui.add_space(10.0);

            // Global Directs Section
            if !arch.direct_list.is_empty() {
                ui.group(|ui| {
                    ui.heading(format!("Global Directs ({})", arch.direct_list.len()));
                    ui.separator();

                    ui.collapsing("Global Directs", |ui| {
                        for direct in &arch.direct_list {
                            ui.label(format!(
                                "{}: {} -> {} (offset: {},{},{})",
                                direct.name,
                                direct.from_pin,
                                direct.to_pin,
                                direct.x_offset,
                                direct.y_offset,
                                direct.z_offset
                            ));
                        }
                    });
                });
            }
        });
}

}