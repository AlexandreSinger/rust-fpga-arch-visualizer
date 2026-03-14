//! Tile Renderer Object
//! 
//! Used for rendering a tile, described in the architecture description file.
//! 
//! NOTE: This only draws the logical block. It does not include the switch block
//!       or the channel wires.

use crate::{block_style, crr_sb_view::TilePinMapper};

pub struct TileRenderer {
    /// The shapes that make up the logical block.
    pub lb_shapes: Vec<egui::Shape>,

    /// The shapes that make up the pins on the tile.
    pub pin_shapes: Vec<egui::Shape>,

    /// The locations of each of the pins.
    ///     [pin_index] -> position
    pub pin_locations: Vec<egui::Vec2>,
}

pub fn build_render_tile(
    tile_bounding_box: &egui::Rect,
    color: &egui::Color32,
    pin_mapper: &TilePinMapper,
) -> TileRenderer {
    // Build the logic block. For now its just a rectangle the size of the tile.
    let mut lb_shapes: Vec<egui::Shape> = Vec::new();
    // Draw filled square.
    lb_shapes.push(egui::Shape::rect_filled(
        *tile_bounding_box,
        0.0, // No rounding for sharp corners
        *color,
    ));
    // Draw outline.
    lb_shapes.push(egui::Shape::rect_stroke(
        *tile_bounding_box,
        egui::CornerRadius::ZERO, 
        egui::Stroke::new(2.0,block_style::darken_color(*color, 0.5)),
        egui::epaint::StrokeKind::Inside,
    ));

    // Get the locations of the pins.
    // TODO: We should pull the pin locations out of the pin mapper struct.
    let mut pin_locations: Vec<egui::Vec2> = vec![tile_bounding_box.min.to_vec2(); pin_mapper.num_pins_in_tile];
    let mut pin_shapes: Vec<egui::Shape> = Vec::with_capacity(pin_mapper.num_pins_in_tile);
    for (pin_index, pin_loc) in pin_mapper.pin_locations.iter().enumerate() {
        let pin_pos = (*pin_loc * tile_bounding_box.size()) + tile_bounding_box.min.to_vec2();
        pin_locations[pin_index] = pin_pos;
        pin_shapes.push(egui::Shape::circle_filled(
            pin_pos.to_pos2(),
            2.5,    // FIXME: This should be relative to the bounding box.
            egui::Color32::BLACK,
        ));
        // TODO: Revive this.
        //      - The struct should return the hit-boxes for each of the pins.
        //  - We can have this object return all of the hit boxes and text.
        // let hit_rect = egui::Rect::from_center_size(pin_pos.to_pos2(), egui::Vec2::new(5.0 * self.zoom_factor, 5.0 * self.zoom_factor));
        // let response = ui.put(hit_rect, egui::Label::new(""));
        // let pin_name = &clb_pin_mapper.pin_name_lookup[pin_index];
        // response.on_hover_ui(|ui| {
        //     ui.label(pin_name);
        // });
    }

    TileRenderer { lb_shapes, pin_shapes, pin_locations }
}