//! Tile Renderer Object
//! 
//! Used for rendering a tile, described in the architecture description file.
//! 
//! NOTE: This only draws the logical block. It does not include the switch block
//!       or the channel wires.

use fpga_arch_parser::{PinSide, Tile, TilePinMapper};

use crate::{block_style};

pub struct TileRenderer {
    /// The shapes that make up the logical block.
    pub lb_shapes: Vec<egui::Shape>,

    /// The shapes that make up the pins on the tile.
    pub pin_shapes: Vec<egui::Shape>,

    /// The locations of each of the pins. Pins may exist in multiple locations.
    /// This occurs when a pin can exit different sides of the tile. Electrically,
    /// there is a short between these locations.
    ///     [pin_index] -> Vec<position>
    pub pin_locations: Vec<Vec<egui::Vec2>>,

    /// The radius of drawn pins.
    pub pin_radius: f32,
}

pub fn build_render_tile(
    tile: &Tile,
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
    // Draw lines to distinguish the grid-tile lines.
    for i in 1..tile.width {
        let x_offset = (tile_bounding_box.width() / (tile.width as f32)) * i as f32 + tile_bounding_box.left();
        lb_shapes.push(egui::Shape::line_segment(
            [
                egui::pos2(x_offset, tile_bounding_box.top()),
                egui::pos2(x_offset, tile_bounding_box.bottom()),
            ],
            egui::Stroke::new(1.0,block_style::darken_color(*color, 0.5)),
        ));
    }
    for j in 1..tile.height {
        let y_offset = (tile_bounding_box.height() / (tile.height as f32)) * j as f32 + tile_bounding_box.top();
        lb_shapes.push(egui::Shape::line_segment(
            [
                egui::pos2(tile_bounding_box.left(), y_offset),
                egui::pos2(tile_bounding_box.right(), y_offset),
            ],
            egui::Stroke::new(1.0,block_style::darken_color(*color, 0.5)),
        ));
    }

    // Get the locations of the pins.
    let mut pin_locations: Vec<Vec<egui::Vec2>> = vec![Vec::new(); pin_mapper.num_pins_in_tile];
    let mut pin_shapes: Vec<egui::Shape> = Vec::with_capacity(pin_mapper.num_pins_in_tile);
    // TODO: To handle offsets, we need to have TileW x TileH of these.
    let mut top_pins: Vec<usize> = Vec::new();
    let mut bottom_pins: Vec<usize> = Vec::new();
    let mut left_pins: Vec<usize> = Vec::new();
    let mut right_pins: Vec<usize> = Vec::new();
    for (pin_index, pin_locs) in pin_mapper.pin_locs.iter().enumerate() {
        for pin_loc in pin_locs {
            // TODO: Handle xoffset and yoffset
            match pin_loc.side {
                PinSide::Top => top_pins.push(pin_index),
                PinSide::Bottom => bottom_pins.push(pin_index),
                PinSide::Left => left_pins.push(pin_index),
                PinSide::Right => right_pins.push(pin_index),
            };
        }
    }
    for (i, pin_index) in top_pins.iter().enumerate() {
        pin_locations[*pin_index].push(egui::Vec2::new(((i + 1) as f32) / ((top_pins.len() + 1) as f32), 0.0));
    }
    for (i, pin_index) in bottom_pins.iter().enumerate() {
        pin_locations[*pin_index].push(egui::Vec2::new(((i + 1) as f32) / ((bottom_pins.len() + 1) as f32), 1.0));
    }
    for (i, pin_index) in left_pins.iter().enumerate() {
        pin_locations[*pin_index].push(egui::Vec2::new(0.0, ((i + 1) as f32) / ((left_pins.len() + 1) as f32)));
    }
    for (i, pin_index) in right_pins.iter().enumerate() {
        pin_locations[*pin_index].push(egui::Vec2::new(1.0, ((i + 1) as f32) / ((right_pins.len() + 1) as f32)));
    }

    // A bit of a hack. All numbers above are normalized to the size of the tile. Fix it here.
    for pin_location in &mut pin_locations {
        for pin_pos in pin_location {
            *pin_pos = *pin_pos * tile_bounding_box.size() + tile_bounding_box.min.to_vec2();
        }
    }

    let max_pins_per_side = top_pins.len().max(bottom_pins.len()).max(left_pins.len()).max(right_pins.len());
    let min_tile_length = tile_bounding_box.width().min(tile_bounding_box.height());
    let max_pin_radius = min_tile_length / 50.0;
    let pin_radius = (min_tile_length / (max_pins_per_side as f32 * 3.0)).min(max_pin_radius);

    for pin_location in &pin_locations {
        for pin_pos in pin_location {
            pin_shapes.push(egui::Shape::circle_filled(
                pin_pos.to_pos2(),
                pin_radius,
                egui::Color32::BLACK,
            ));
        }
    }

    TileRenderer { lb_shapes, pin_shapes, pin_locations, pin_radius }
}