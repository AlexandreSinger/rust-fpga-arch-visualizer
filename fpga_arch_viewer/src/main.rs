//! FPGA Architecture Viewer
//!
//! A Rust-based visualizer for VTR FPGA architecture description files.

mod block_style;
mod color_scheme;
mod common_ui;
mod complex_block_view;
mod grid;
mod grid_renderer;
mod grid_view;
mod intra_block_drawing;
mod intra_hierarchy_tree;
mod intra_tile;
mod settings;
mod summary_view;
mod viewer;

fn main() -> Result<(), eframe::Error> {
    // Load the icon data.
    let icon_data =
        eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..]);
    // If the icon fails to load for any reason, use the default icon for the system.
    let icon_data = match icon_data {
        Ok(icon) => icon,
        Err(_) => egui::IconData::default(),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_icon(icon_data),
        ..Default::default()
    };

    eframe::run_native(
        "FPGA Architecture Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(viewer::FpgaViewer::new()))),
    )
}
