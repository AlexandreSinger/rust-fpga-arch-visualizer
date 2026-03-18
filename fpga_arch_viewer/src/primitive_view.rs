use fpga_arch_parser::FPGAArch;



#[derive(Default)]
pub struct PrimitiveView {}

impl PrimitiveView {
    pub fn render(
        &mut self,
        arch: &FPGAArch,
        ctx: &egui::Context,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Primitive View");
        });
    }
}