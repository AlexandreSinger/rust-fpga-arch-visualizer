//! FPGA Architecture Viewer
//!
//! A Rust-based visualizer for VTR FPGA architecture description files.

use eframe::egui;

mod intra_block_drawing;
mod block_style;
mod grid;
mod grid_renderer;
mod intra_hierarchy_tree;
mod intra_tile;
mod settings;
mod viewer;

use viewer::FpgaViewer;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FPGA Architecture Visualizer",
        options,
        Box::new(|cc| {
            // Force light mode
            // TODO: Support dark mode later, add option to select or auto
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Box::new(FpgaViewer::new())
        }),
    )
}
