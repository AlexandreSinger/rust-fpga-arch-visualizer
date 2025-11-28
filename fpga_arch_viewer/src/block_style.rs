use eframe::egui::{self, Color32};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockShape {
    Square,
}

#[derive(Debug, Clone)]
pub struct BlockStyle {
    pub short_name: &'static str,
    pub full_name: &'static str,
    pub shape: BlockShape,
    pub color: Color32,
    // Relative size multiplier (1.0 = standard size , 0.5 = half size)
    // e.g. IO and LB would be standard size and SB and CB would be half size
    pub size_multiplier: f32,
}

impl BlockStyle {
    pub fn new(
        short_name: &'static str,
        full_name: &'static str,
        shape: BlockShape,
        color: Color32,
        size_multiplier: f32,
    ) -> Self {
        Self {
            short_name,
            full_name,
            shape,
            color,
            size_multiplier,
        }
    }
}

// Darken a block color for outline color
pub fn darken_color(color: Color32, factor: f32) -> Color32 {
    let factor = factor.clamp(0.0, 1.0);
    let multiplier = 1.0 - factor;

    Color32::from_rgb(
        (color.r() as f32 * multiplier) as u8,
        (color.g() as f32 * multiplier) as u8,
        (color.b() as f32 * multiplier) as u8,
    )
}

// Default block styles for the inter-tile grid view
pub struct DefaultBlockStyles {
    pub io: BlockStyle,
    pub lb: BlockStyle,
    pub sb: BlockStyle,
    pub cb: BlockStyle,
}

impl DefaultBlockStyles {
    pub fn new() -> Self {
        Self {
            // IO - Input/Output Block
            io: BlockStyle::new(
                "IO",
                "Input/Output Block",
                BlockShape::Square,
                Color32::from_rgb(0xF5, 0xF5, 0xF5), // #F5F5F5
                1.0,
            ),

            // LB - Logic Block
            lb: BlockStyle::new(
                "LB",
                "Logic Block",
                BlockShape::Square,
                Color32::from_rgb(0xD8, 0xE7, 0xFD), // #D8E7FD
                1.0,
            ),

            // SB - Switch Block
            sb: BlockStyle::new(
                "SB",
                "Switch Block",
                BlockShape::Square,
                Color32::from_rgb(0xFF, 0xE6, 0xCE), // #FFE6CE
                0.5,
            ),

            // CB - Connection Block
            cb: BlockStyle::new(
                "CB",
                "Connection Block",
                BlockShape::Square,
                Color32::from_rgb(0xFF, 0xF3, 0xCC), // #FFF3CC
                0.5,
            ),
        }
    }

    pub fn all_styles(&self) -> Vec<&BlockStyle> {
        vec![&self.io, &self.lb, &self.sb, &self.cb]
    }
}

impl Default for DefaultBlockStyles {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_block(
    ui: &mut egui::Ui,
    style: &BlockStyle,
    base_size: f32,
) -> egui::Response {
    let size = base_size * style.size_multiplier;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(size, size),
        egui::Sense::hover(),
    );

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let outline_color = darken_color(style.color, 0.5);

        match style.shape {
            BlockShape::Square => {
                // Draw filled square
                painter.rect_filled(
                    rect,
                    0.0, // No rounding for sharp corners
                    style.color,
                );

                // Draw outline
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(2.0, outline_color),
                );
            }
            // Future shapes can be added here
        }

        // Draw text in center
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            style.short_name,
            egui::FontId::proportional(size * 0.3),
            egui::Color32::BLACK,
        );
    }

    response
}
