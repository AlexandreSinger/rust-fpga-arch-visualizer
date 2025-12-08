use crate::color_scheme;
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

// Default color palette for tiles
pub fn get_default_color_palette() -> Vec<Color32> {
    vec![
        Color32::from_rgb(0xD8, 0xE7, 0xFD), // Light Blue
        Color32::from_rgb(0xF5, 0xF5, 0xF5), // Light Gray
        Color32::from_rgb(0xFF, 0xE6, 0xCE), // Light Orange
        Color32::from_rgb(0xFF, 0xF3, 0xCC), // Light Yellow
        Color32::from_rgb(0xE8, 0xF5, 0xE9), // Light Green
        Color32::from_rgb(0xF3, 0xE5, 0xF5), // Light Purple
        Color32::from_rgb(0xFF, 0xE0, 0xE0), // Light Pink
        Color32::from_rgb(0xE0, 0xF7, 0xFA), // Light Cyan
        Color32::from_rgb(0xFF, 0xF9, 0xC4), // Light Amber
        Color32::from_rgb(0xF0, 0xF4, 0xC3), // Light Lime
    ]
}

pub fn get_tile_color(tile_name: &str, tile_index: usize) -> Color32 {
    let palette = get_default_color_palette();
    palette[tile_index % palette.len()]
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
        Self::new_with_theme(false)
    }

    pub fn new_with_theme(dark_mode: bool) -> Self {
        Self {
            // IO - Input/Output Block
            io: BlockStyle::new(
                "IO",
                "Input/Output Block",
                BlockShape::Square,
                color_scheme::grid_io_color(dark_mode),
                1.0,
            ),

            // LB - Logic Block
            lb: BlockStyle::new(
                "LB",
                "Logic Block",
                BlockShape::Square,
                color_scheme::grid_lb_color(dark_mode),
                1.0,
            ),

            // SB - Switch Block
            sb: BlockStyle::new(
                "SB",
                "Switch Block",
                BlockShape::Square,
                color_scheme::grid_sb_color(dark_mode),
                0.5,
            ),

            // CB - Connection Block
            cb: BlockStyle::new(
                "CB",
                "Connection Block",
                BlockShape::Square,
                color_scheme::grid_cb_color(dark_mode),
                0.5,
            ),
        }
    }

    pub fn update_colors(&mut self, dark_mode: bool) {
        self.io.color = color_scheme::grid_io_color(dark_mode);
        self.lb.color = color_scheme::grid_lb_color(dark_mode);
        self.sb.color = color_scheme::grid_sb_color(dark_mode);
        self.cb.color = color_scheme::grid_cb_color(dark_mode);
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
    dark_mode: bool,
) -> egui::Response {
    let size = base_size * style.size_multiplier;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

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
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0, outline_color));
            } // Future shapes can be added here
        }

        // Draw text in center
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            style.short_name,
            egui::FontId::proportional(size * 0.3),
            color_scheme::theme_text_color(dark_mode),
        );
    }

    response
}
