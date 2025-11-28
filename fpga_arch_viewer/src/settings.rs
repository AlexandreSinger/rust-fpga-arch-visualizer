use eframe::egui;
use crate::block_style::{DefaultBlockStyles, draw_block};

pub fn render_settings_page(ui: &mut egui::Ui, block_styles: &DefaultBlockStyles) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Settings");
        ui.add_space(20.0);

        // show default block styles
        ui.group(|ui| {
            ui.heading("Default Block Styles");
            ui.label("Inter-Tile Grid View");
            ui.add_space(15.0);

            // Display blocks in a grid layout
            let spacing = 150.0;
            let block_size = 80.0;

            ui.horizontal_wrapped(|ui| {
                // IO Block
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    draw_block(ui, &block_styles.io, block_size);
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(block_styles.io.full_name)
                            .size(12.0)
                    );
                });

                ui.add_space(spacing - block_size);

                // LB Block
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    draw_block(ui, &block_styles.lb, block_size);
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(block_styles.lb.full_name)
                            .size(12.0)
                    );
                });
            });

            ui.add_space(20.0);

            ui.horizontal_wrapped(|ui| {
                // SB Block
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    draw_block(ui, &block_styles.sb, block_size);
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(block_styles.sb.full_name)
                            .size(12.0)
                    );
                });

                ui.add_space(spacing - block_size);

                // CB Block
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    draw_block(ui, &block_styles.cb, block_size);
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(block_styles.cb.full_name)
                            .size(12.0)
                    );
                });
            });

            ui.add_space(20.0);

            ui.separator();
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Note: SB and CB blocks are half the size of IO and LB blocks")
                    .italics()
                    .size(11.0)
                    .color(egui::Color32::GRAY)
            );
        });

        ui.add_space(30.0);

        // Future customization section placeholder
        ui.group(|ui| {
            ui.heading("Customization");
            ui.add_space(10.0);
            ui.label("Custom block styling options will be available here.");
            ui.add_space(5.0);
            ui.label("You will be able to modify colors, shapes, and sizes of each block type.");
        });
    });
}