#[derive(Default)]
pub struct TileView {}

impl TileView {
    pub fn render(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_tile(ui);
        });
    }

    fn render_tile(&mut self, ui: &mut egui::Ui) {
        ui.heading("Tile View");
    }
}
