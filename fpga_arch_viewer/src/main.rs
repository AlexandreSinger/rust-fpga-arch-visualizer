//! FPGA Architecture Viewer
//!
//! A Rust-based visualizer for VTR FPGA architecture description files.

use eframe::egui;

mod viewer;

use viewer::FpgaViewer;

fn main() -> Result<(), eframe::Error> {
    let num = 10;
    println!("Hello, world! {num} plus one is {}!", fpga_arch_parser::add(num, 1));
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FPGA Architecture Visualizer",
        options,
        Box::new(|_cc| Box::new(FpgaViewer::new())),
    )
}
