//! FPGA Architecture Viewer
//!
//! A Rust-based visualizer for VTR FPGA architecture description files.

use eframe::egui;

mod block_style;
mod grid;
mod grid_renderer;
mod intra_block_drawing;
mod intra_hierarchy_tree;
mod intra_tile;
mod intra_color_scheme;
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
        Box::new(|_cc| {
            // Theme is now controlled by the FpgaViewer state
            Box::new(FpgaViewer::new())
        }),
    )
}
