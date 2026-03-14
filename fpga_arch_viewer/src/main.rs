//! FPGA Architecture Viewer
//!
//! A Rust-based visualizer for VTR FPGA architecture description files.

mod tile_rendering;
mod block_style;
mod color_scheme;
mod common_ui;
mod complex_block_view;
mod crr_sb_view;
mod grid;
mod grid_renderer;
mod grid_view;
mod intra_block_drawing;
mod intra_hierarchy_tree;
mod intra_tile;
mod samples;
mod settings;
mod summary_view;
mod tile_view;
mod viewer;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();

    // Load the icon data.
    let icon_data =
        eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..]);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_icon(icon_data.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "FPGA Architecture Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(viewer::FpgaViewer::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(viewer::FpgaViewer::new()))),
            )
            .await;

        // Remove the loading text and spinner:
        match start_result {
            Ok(_) => {
                if let Some(loading_text) = document.get_element_by_id("loading_text") {
                    loading_text.remove();
                }
            }
            Err(e) => {
                if let Some(loading_text) = document.get_element_by_id("loading_text") {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                }
                panic!("Failed to start eframe: {e:?}");
            }
        }
    });
}
