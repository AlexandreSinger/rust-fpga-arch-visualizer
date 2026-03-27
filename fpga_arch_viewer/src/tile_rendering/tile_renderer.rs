//! Tile Renderer Object
//!
//! Used for rendering a tile, described in the architecture description file.
//!
//! NOTE: This only draws the logical block. It does not include the switch block
//!       or the channel wires.

use fpga_arch_parser::{PinSide, Tile};

use crate::block_style;

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
        egui::Stroke::new(2.0, block_style::darken_color(*color, 0.5)),
        egui::epaint::StrokeKind::Inside,
    ));
    // Draw lines to distinguish the grid-tile lines.
    for i in 1..tile.width {
        let x_offset =
            (tile_bounding_box.width() / (tile.width as f32)) * i as f32 + tile_bounding_box.left();
        lb_shapes.push(egui::Shape::line_segment(
            [
                egui::pos2(x_offset, tile_bounding_box.top()),
                egui::pos2(x_offset, tile_bounding_box.bottom()),
            ],
            egui::Stroke::new(1.0, block_style::darken_color(*color, 0.5)),
        ));
    }
    for j in 1..tile.height {
        let y_offset = (tile_bounding_box.height() / (tile.height as f32)) * j as f32
            + tile_bounding_box.top();
        lb_shapes.push(egui::Shape::line_segment(
            [
                egui::pos2(tile_bounding_box.left(), y_offset),
                egui::pos2(tile_bounding_box.right(), y_offset),
            ],
            egui::Stroke::new(1.0, block_style::darken_color(*color, 0.5)),
        ));
    }

    // Get the locations of the pins.
    let mut pin_locations: Vec<Vec<egui::Vec2>> =
        vec![Vec::new(); tile.pin_mapper.num_pins_in_tile];
    let mut pin_shapes: Vec<egui::Shape> = Vec::with_capacity(tile.pin_mapper.num_pins_in_tile);
    // Group pins by (side, xoffset, yoffset) so they are distributed within
    // the edge of the specific grid cell they belong to, not the full tile side.
    // Key: (side as u8, xoffset, yoffset) where side encoding: 0=Left, 1=Right, 2=Bottom, 3=Top
    let mut grouped: std::collections::HashMap<(u8, usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (pin_index, pin_locs) in tile.pin_mapper.pin_locs.iter().enumerate() {
        for pin_loc in pin_locs {
            let side_key: u8 = match pin_loc.side {
                PinSide::Left => 0,
                PinSide::Right => 1,
                PinSide::Bottom => 2,
                PinSide::Top => 3,
            };
            grouped
                .entry((side_key, pin_loc.xoffset, pin_loc.yoffset))
                .or_default()
                .push(pin_index);
        }
    }

    let max_pins_per_cell_side = grouped.values().map(|v| v.len()).max().unwrap_or(1);
    let cell_min_length = (tile_bounding_box.width() / tile.width as f32)
        .min(tile_bounding_box.height() / tile.height as f32);
    let max_pin_radius = cell_min_length / 50.0;
    let pin_radius = (cell_min_length / (max_pins_per_cell_side as f32 * 3.0)).min(max_pin_radius);

    // Internal pins on shared cell boundaries are nudged inward by 2 pin radii so that
    // pins on opposite sides of the same internal edge don't overlap.
    let cell_width_norm = 1.0 / tile.width as f32;
    let cell_height_norm = 1.0 / tile.height as f32;
    let internal_nudge_x = 2.0 * pin_radius / tile_bounding_box.width();
    let internal_nudge_y = 2.0 * pin_radius / tile_bounding_box.height();
    for (&(side_key, xoffset, yoffset), pins) in &grouped {
        let x_start = xoffset as f32 * cell_width_norm;
        let x_end = (xoffset + 1) as f32 * cell_width_norm;
        // Arch yoffset=0 is the bottom row; screen y increases downward, so flip.
        let y_start = (tile.height as usize - 1 - yoffset) as f32 * cell_height_norm;
        let y_end = (tile.height as usize - yoffset) as f32 * cell_height_norm;

        // A side is internal when its edge is shared with an adjacent cell inside the tile.
        let is_internal = match side_key {
            0 => xoffset > 0,                        // Left
            1 => xoffset < tile.width as usize - 1,  // Right
            2 => yoffset > 0,                        // Bottom
            _ => yoffset < tile.height as usize - 1, // Top
        };
        let nudge_x = if is_internal { internal_nudge_x } else { 0.0 };
        let nudge_y = if is_internal { internal_nudge_y } else { 0.0 };

        let n = pins.len();
        for (i, &pin_index) in pins.iter().enumerate() {
            let t = (i + 1) as f32 / (n + 1) as f32;
            let pos = match side_key {
                0 => egui::Vec2::new(x_start + nudge_x, y_start + t * (y_end - y_start)), // Left
                1 => egui::Vec2::new(x_end - nudge_x, y_start + t * (y_end - y_start)),   // Right
                2 => egui::Vec2::new(x_start + t * (x_end - x_start), y_end - nudge_y),   // Bottom
                _ => egui::Vec2::new(x_start + t * (x_end - x_start), y_start + nudge_y), // Top
            };
            pin_locations[pin_index].push(pos);
        }
    }

    // A bit of a hack. All numbers above are normalized to the size of the tile. Fix it here.
    for pin_location in &mut pin_locations {
        for pin_pos in pin_location {
            *pin_pos = *pin_pos * tile_bounding_box.size() + tile_bounding_box.min.to_vec2();
        }
    }

    for pin_location in &pin_locations {
        for pin_pos in pin_location {
            pin_shapes.push(egui::Shape::circle_filled(
                pin_pos.to_pos2(),
                pin_radius,
                egui::Color32::BLACK,
            ));
        }
    }

    TileRenderer {
        lb_shapes,
        pin_shapes,
        pin_locations,
        pin_radius,
    }
}
