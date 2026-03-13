#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
use std::ops::RangeInclusive;

use crr_sb_parser::{
    CRRSwitchBlockDeserialized, CRRSwitchDir, CRRSwitchSinkNodeInfo, CRRSwitchSourceNodeInfo, CRRSwitchSourcePin,
};
use fpga_arch_parser::{FPGAArch, PinLoc, PinSide, Port, SubTile, SubTilePinLocations};

use crate::{block_style, color_scheme};

pub struct CRRSBView {
    crr_sb_info: Option<crr_sb_parser::CRRSwitchBlockDeserialized>,
    crr_sb: Option<CRRSwitchBlock>,
    zoom_factor: f32,
    last_error: Option<String>,
}

impl Default for CRRSBView {
    fn default() -> Self {
        Self {
            crr_sb_info: None,
            crr_sb: None,
            zoom_factor: 1.0,
            last_error: None,
        }
    }
}

impl CRRSBView {
    #[cfg(not(target_arch = "wasm32"))]
    fn load_crr_csv_file(&mut self, file_path: std::path::PathBuf) {
        self.crr_sb_info = match crr_sb_parser::parse_csv_file(&file_path) {
            Ok(crr_sb_info) => {
                self.last_error = None;
                Some(crr_sb_info)
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                None
            }
        };

        if let Some(crr_sb_info) = &self.crr_sb_info {
            self.crr_sb = match get_crr_switch_block(crr_sb_info) {
                Ok(crr_sb) => {
                    self.last_error = None;
                    Some(crr_sb)
                }
                Err(e) => {
                    self.last_error = Some(e.to_string());
                    None
                }
            }
        }
    }

    pub fn render(&mut self, arch: &FPGAArch, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(arch, ui);
        });
    }

    fn render_central_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        if let Some(crr_sb_info) = &self.crr_sb_info {
            if let Some(crr_sb) = &self.crr_sb {
                // Handle zoom input (Cmd + scroll wheel or pinch gesture)
                let input = ui.input(|i| {
                    let scroll_delta = i.raw_scroll_delta.y;
                    let zoom_modifier = i.modifiers.command;
                    (scroll_delta, zoom_modifier)
                });
                let (scroll_delta, zoom_modifier) = input;
                if zoom_modifier && scroll_delta != 0.0 {
                    if scroll_delta > 0.0 {
                        self.zoom_factor *= 1.1;
                    } else {
                        self.zoom_factor /= 1.1;
                    }
                }
                // Check for pinch gesture (trackpad zoom on macOS)
                let zoom_delta = ui.input(|i| i.zoom_delta());
                if zoom_delta != 1.0 {
                    self.zoom_factor *= zoom_delta;
                }

                self.render_crr_sb(crr_sb, crr_sb_info, arch, ui);
            } else if let Some(error_msg) = &self.last_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error_msg));
            } else {
                ui.label("Error occurred while interpreting the CRR SB.");
            }
        } else {
            ui.label("The CRR View is currently under development.");
            if let Some(error_msg) = &self.last_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error_msg));
            }
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Select CSV file to view").clicked() {
                // TODO: Make this cleaner by combining with view.
                // TODO: Add WASM support.
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("CRR CSV Files", &["csv"])
                    .set_title("Open CRR CSV File")
                    .pick_file()
                {
                    self.load_crr_csv_file(path);
                }
            }

            #[cfg(target_arch = "wasm32")]
            ui.label("CRR CSV loading is not available in the web build yet.");
        }
    }

    fn get_source_node_loc(
        source_node: &CRRSwitchSourceNodeInfo,
        spacing_between_points: f32,
        crr_sb: &CRRSwitchBlock,
    ) -> egui::Pos2 {
        // TODO: Catch for underflow!
        let lane_num = source_node.lane_num - 1;
        let tap_num = match source_node.source_pin {
            CRRSwitchSourcePin::Tap { tap_num } => tap_num,
            _ => 1,
        };
        let ptc_offset = (tap_num - 1) * 2;
        let ptc_num = match source_node.dir {
            // TODO: Verify that the lane is allowed.
            CRRSwitchDir::Left => crr_sb.chan_x_lanes[lane_num].starting_track_num + ptc_offset,
            CRRSwitchDir::Right => {
                crr_sb.chan_x_lanes[lane_num].starting_track_num + ptc_offset + 1
            }
            CRRSwitchDir::Top => crr_sb.chan_y_lanes[lane_num].starting_track_num + ptc_offset,
            CRRSwitchDir::Bottom => {
                crr_sb.chan_y_lanes[lane_num].starting_track_num + ptc_offset + 1
            }
            // FIXME: Update.
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => {0}
        };
        let chan_w = match source_node.dir {
            CRRSwitchDir::Left | CRRSwitchDir::Right => crr_sb.chan_x_width,
            CRRSwitchDir::Top | CRRSwitchDir::Bottom => crr_sb.chan_y_width,
            // FIXME: Update.
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => crr_sb.chan_x_width,
        };
        Self::get_ptc_loc(ptc_num, spacing_between_points, source_node.dir, chan_w)
    }

    fn get_sink_node_loc(
        sink_node: &CRRSwitchSinkNodeInfo,
        spacing_between_points: f32,
        crr_sb: &CRRSwitchBlock,
    ) -> egui::Pos2 {
        // TODO: Catch for underflow!
        let lane_num = sink_node.lane_num - 1;
        let ptc_offset = 0;
        let ptc_num = match sink_node.dir {
            // TODO: Verify that the lane is allowed.
            CRRSwitchDir::Left => crr_sb.chan_x_lanes[lane_num].starting_track_num + ptc_offset + 1,
            CRRSwitchDir::Right => crr_sb.chan_x_lanes[lane_num].starting_track_num + ptc_offset,
            CRRSwitchDir::Top => crr_sb.chan_y_lanes[lane_num].starting_track_num + ptc_offset + 1,
            CRRSwitchDir::Bottom => crr_sb.chan_y_lanes[lane_num].starting_track_num + ptc_offset,
            // FIXME: Handle this correctly.
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => 0,
        };
        let chan_w = match sink_node.dir {
            CRRSwitchDir::Left | CRRSwitchDir::Right => crr_sb.chan_x_width,
            CRRSwitchDir::Top | CRRSwitchDir::Bottom => crr_sb.chan_y_width,
            // FIXME: Update.
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => crr_sb.chan_x_width,
        };
        Self::get_ptc_loc(ptc_num, spacing_between_points, sink_node.dir, chan_w)
    }

    fn get_ptc_loc(
        ptc_track_num: usize,
        spacing_between_points: f32,
        side: CRRSwitchDir,
        chan_w: usize,
    ) -> egui::Pos2 {
        egui::Pos2::new(
            match side {
                CRRSwitchDir::Left => 0.0,
                CRRSwitchDir::Right => chan_w as f32 * spacing_between_points,
                CRRSwitchDir::Top | CRRSwitchDir::Bottom => {
                    (ptc_track_num as f32 * spacing_between_points) + (spacing_between_points / 2.0)
                }
                // FIXME: Update
                CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => 0.0,
            },
            match side {
                CRRSwitchDir::Top => 0.0,
                CRRSwitchDir::Bottom => chan_w as f32 * spacing_between_points,
                CRRSwitchDir::Left | CRRSwitchDir::Right => {
                    (ptc_track_num as f32 * spacing_between_points) + (spacing_between_points / 2.0)
                }
                // FIXME: Update
                CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => 0.0,
            },
        )
    }

    fn render_crr_sb(
        &self,
        crr_sb: &CRRSwitchBlock,
        crr_sb_info: &CRRSwitchBlockDeserialized,
        arch: &FPGAArch,
        ui: &mut egui::Ui,
    ) {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let max_chan_w = crr_sb.chan_x_width.max(crr_sb.chan_y_width);
                let smaller_available_size = ui.available_height().min(ui.available_width());
                let spacing_between_points =
                    ((smaller_available_size / max_chan_w as f32) / 2.0) * self.zoom_factor;
                let draw_area_size = egui::vec2(
                    (max_chan_w + 1) as f32 * spacing_between_points * 2.0,
                    (max_chan_w + 1) as f32 * spacing_between_points * 2.0,
                );

                let (response, painter) = ui.allocate_painter(
                    draw_area_size,
                    egui::Sense::click().union(egui::Sense::hover()),
                );

                let chan_x_draw_offset = response.rect.min;
                let chan_y_draw_offset = response.rect.min + (draw_area_size / 2.0);
                let sb_draw_offset = response.rect.min + egui::vec2(draw_area_size.x / 2.0, 0.0);
                let lb_draw_offset = response.rect.min + egui::vec2(0.0, draw_area_size.y / 2.0);

                let chan_wire_stroke = spacing_between_points / 5.0;

                // Draw the chan_x channels
                for chan_x_ptc in 0..crr_sb.chan_x_width {
                    let left_conn_pt = Self::get_ptc_loc(
                        chan_x_ptc,
                        spacing_between_points,
                        CRRSwitchDir::Left,
                        crr_sb.chan_x_width,
                    );
                    painter.line_segment(
                        [
                            left_conn_pt + chan_x_draw_offset.to_vec2(),
                            left_conn_pt + sb_draw_offset.to_vec2(),
                        ],
                        egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
                    );
                }

                // Draw the chan_y channels
                for chan_y_ptc in 0..crr_sb.chan_y_width {
                    let left_conn_pt = Self::get_ptc_loc(
                        chan_y_ptc,
                        spacing_between_points,
                        CRRSwitchDir::Bottom,
                        crr_sb.chan_y_width,
                    );
                    painter.line_segment(
                        [
                            left_conn_pt + chan_y_draw_offset.to_vec2(),
                            left_conn_pt + sb_draw_offset.to_vec2(),
                        ],
                        egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
                    );
                }

                // Draw the connection points to the channels
                let all_sides = [
                    CRRSwitchDir::Left,
                    CRRSwitchDir::Right,
                    CRRSwitchDir::Top,
                    CRRSwitchDir::Bottom,
                ];
                for side in all_sides {
                    let chan_w = match side {
                        CRRSwitchDir::Left | CRRSwitchDir::Right => crr_sb.chan_x_width,
                        CRRSwitchDir::Top | CRRSwitchDir::Bottom => crr_sb.chan_y_width,
                        // FIXME: Update.
                        CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => crr_sb.chan_x_width,
                    };
                    for chan_ptc in 0..chan_w {
                        let conn_pt =
                            Self::get_ptc_loc(chan_ptc, spacing_between_points, side, chan_w);
                        painter.circle(
                            conn_pt + sb_draw_offset.to_vec2(),
                            2.5,
                            egui::Color32::BLACK,
                            egui::Stroke::new(0.5, egui::Color32::BLACK),
                        );
                    }
                }

                // Draw the segment connections (i.e. the hardened connections along the segment).
                for chan_x_lane in &crr_sb.chan_x_lanes {
                    for i in 0..(chan_x_lane.segment_len - 1) {
                        let left_source_ptc_num = chan_x_lane.starting_track_num + (i * 2);
                        let right_source_ptc_num = left_source_ptc_num + 1;
                        let right_sink_ptc_num = chan_x_lane.starting_track_num + ((i + 1) * 2);
                        let left_sink_ptc_num = right_sink_ptc_num + 1;

                        let left_source_loc = Self::get_ptc_loc(
                            left_source_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Left,
                            crr_sb.chan_x_width,
                        );
                        let left_sink_loc = Self::get_ptc_loc(
                            left_sink_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Left,
                            crr_sb.chan_x_width,
                        );
                        let right_source_loc = Self::get_ptc_loc(
                            right_source_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Right,
                            crr_sb.chan_x_width,
                        );
                        let right_sink_loc = Self::get_ptc_loc(
                            right_sink_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Right,
                            crr_sb.chan_x_width,
                        );

                        painter.line_segment(
                            [
                                left_source_loc + sb_draw_offset.to_vec2(),
                                right_sink_loc + sb_draw_offset.to_vec2(),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        );
                        painter.line_segment(
                            [
                                right_source_loc + sb_draw_offset.to_vec2(),
                                left_sink_loc + sb_draw_offset.to_vec2(),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        );
                    }
                }
                for chan_y_lane in &crr_sb.chan_y_lanes {
                    for i in 0..(chan_y_lane.segment_len - 1) {
                        let top_source_ptc_num = chan_y_lane.starting_track_num + (i * 2);
                        let bottom_source_ptc_num = top_source_ptc_num + 1;
                        let bottom_sink_ptc_num = chan_y_lane.starting_track_num + ((i + 1) * 2);
                        let top_sink_ptc_num = bottom_sink_ptc_num + 1;

                        let top_source_loc = Self::get_ptc_loc(
                            top_source_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Top,
                            crr_sb.chan_y_width,
                        );
                        let top_sink_loc = Self::get_ptc_loc(
                            top_sink_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Top,
                            crr_sb.chan_y_width,
                        );
                        let bottom_source_loc = Self::get_ptc_loc(
                            bottom_source_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Bottom,
                            crr_sb.chan_y_width,
                        );
                        let bottom_sink_loc = Self::get_ptc_loc(
                            bottom_sink_ptc_num,
                            spacing_between_points,
                            CRRSwitchDir::Bottom,
                            crr_sb.chan_y_width,
                        );

                        painter.line_segment(
                            [
                                top_source_loc + sb_draw_offset.to_vec2(),
                                bottom_sink_loc + sb_draw_offset.to_vec2(),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        );
                        painter.line_segment(
                            [
                                bottom_source_loc + sb_draw_offset.to_vec2(),
                                top_sink_loc + sb_draw_offset.to_vec2(),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        );
                    }
                }

                // Draw all edges between points (i.e. switch edges).
                // TODO: Add button to disable showing these.
                for edge in &crr_sb_info.edges {
                    let src_node = &crr_sb_info.source_nodes[edge.source_node_id];
                    let sink_node = &crr_sb_info.sink_nodes[edge.sink_node_id];

                    // Skip the IPIN/OPINs for now.
                    if src_node.dir == CRRSwitchDir::OPIN || sink_node.dir == CRRSwitchDir::IPIN {
                        continue;
                    }

                    let src_node_loc =
                        Self::get_source_node_loc(src_node, spacing_between_points, crr_sb);
                    let sink_node_loc =
                        Self::get_sink_node_loc(sink_node, spacing_between_points, crr_sb);

                    painter.line_segment(
                        [
                            src_node_loc + sb_draw_offset.to_vec2(),
                            sink_node_loc + sb_draw_offset.to_vec2(),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    );
                }

                // Draw the logic block. For now, its just a rectangle.
                let lb_rect = egui::Rect::from_center_size(
                    ((draw_area_size / 4.0) + lb_draw_offset.to_vec2()).to_pos2(),
                    (draw_area_size / 2.0) / 1.25,
                );
                let lb_color = color_scheme::grid_lb_color(false);
                // Draw filled square
                painter.rect_filled(
                    lb_rect, 0.0, // No rounding for sharp corners
                    lb_color,
                );
                // Draw outline
                painter.rect_stroke(
                    lb_rect,
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(2.0, block_style::darken_color(lb_color, 0.5)),
                    egui::epaint::StrokeKind::Inside,
                );

                // Draw pins
                let clb_pin_mapper = TilePinMapper::new("clb", arch).expect("What?");
                for (pin_index, pin_loc) in clb_pin_mapper.pin_locations.iter().enumerate() {
                    let pin_pos = (*pin_loc * lb_rect.size()) + lb_rect.left_top().to_vec2();
                    painter.circle(
                        pin_pos.to_pos2(),
                        2.5 * self.zoom_factor,
                        egui::Color32::BLACK,
                        egui::Stroke::new(0.5, egui::Color32::BLACK),
                    );
                    let hit_rect = egui::Rect::from_center_size(pin_pos.to_pos2(), egui::Vec2::new(5.0 * self.zoom_factor, 5.0 * self.zoom_factor));
                    let response = ui.put(hit_rect, egui::Label::new(""));
                    let pin_name = &clb_pin_mapper.pin_name_lookup[pin_index];
                    response.on_hover_ui(|ui| {
                        ui.label(pin_name);
                    });
                }
            });
    }
}

struct CRRSwitchBlock {
    chan_x_lanes: Vec<CRRSwitchBlockLane>,
    chan_y_lanes: Vec<CRRSwitchBlockLane>,
    chan_x_width: usize,
    chan_y_width: usize,
}

struct CRRSwitchBlockLane {
    starting_track_num: usize,
    segment_len: usize,
}

#[cfg(not(target_arch = "wasm32"))]
fn get_segment_len(segment_type: &str) -> Result<usize, &'static str> {
    // This method is currently a bit hacky. The correct way to do this is to get
    // this information from the architecture file; however, this interface supports
    // case-insensitive segment types which is very strange.
    // According to Amin, all segments will follow this naming convention in the tileable
    // architecture.
    let segment_type = segment_type.to_lowercase();
    if !segment_type.starts_with('l') {
        return Err("Unsupported segment type should start with l.");
    }
    match segment_type[1..].parse::<usize>() {
        Ok(0) => Err("Unsupported segment type should have a positive length."),
        Ok(v) => Ok(v),
        Err(_e) => Err("Unsupported segment type."),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_crr_switch_block(
    crr_sb_info: &CRRSwitchBlockDeserialized,
) -> Result<CRRSwitchBlock, &'static str> {
    // To get the lanes, we go through the sink nodes.
    let mut left_lane_num_to_segment = HashMap::new();
    let mut right_lane_num_to_segment = HashMap::new();
    let mut top_lane_num_to_segment = HashMap::new();
    let mut bottom_lane_num_to_segment = HashMap::new();
    for sink_node in &crr_sb_info.sink_nodes {
        if sink_node.dir == CRRSwitchDir::IPIN {
            continue;
        }
        if sink_node.lane_num == 0 {
            return Err("Invalid lane num of 0 found.");
        }
        let lane_num = sink_node.lane_num - 1;
        match sink_node.dir {
            CRRSwitchDir::Left => {
                left_lane_num_to_segment.insert(lane_num, &sink_node.segment_type);
            }
            CRRSwitchDir::Right => {
                right_lane_num_to_segment.insert(lane_num, &sink_node.segment_type);
            }
            CRRSwitchDir::Top => {
                top_lane_num_to_segment.insert(lane_num, &sink_node.segment_type);
            }
            CRRSwitchDir::Bottom => {
                bottom_lane_num_to_segment.insert(lane_num, &sink_node.segment_type);
            }
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => {},
        }
    }

    let num_chan_x_lanes = left_lane_num_to_segment.len();
    if right_lane_num_to_segment.len() != num_chan_x_lanes {
        return Err("Left target node lanes do not match right target node lanes.");
    }
    let num_chan_y_lanes = top_lane_num_to_segment.len();
    if bottom_lane_num_to_segment.len() != num_chan_y_lanes {
        return Err("Top target node lanes do not match bottom target node lanes.");
    }
    let mut chan_x_lanes: Vec<CRRSwitchBlockLane> = Vec::with_capacity(num_chan_x_lanes);
    let mut chan_y_lanes: Vec<CRRSwitchBlockLane> = Vec::with_capacity(num_chan_y_lanes);
    let mut curr_chan_x_track_num: usize = 0;
    for lane_num in 0..num_chan_x_lanes {
        if !left_lane_num_to_segment.contains_key(&lane_num)
            || !right_lane_num_to_segment.contains_key(&lane_num)
        {
            return Err("Lane missing in target nodes.");
        }
        if left_lane_num_to_segment[&lane_num] != right_lane_num_to_segment[&lane_num] {
            return Err("Left and right target lanes do not target the same segment.");
        }
        let segment_len = get_segment_len(left_lane_num_to_segment[&lane_num])?;
        chan_x_lanes.push(CRRSwitchBlockLane {
            starting_track_num: curr_chan_x_track_num,
            segment_len,
        });
        curr_chan_x_track_num += segment_len * 2;
    }
    let mut curr_chan_y_track_num: usize = 0;
    for lane_num in 0..num_chan_y_lanes {
        if !top_lane_num_to_segment.contains_key(&lane_num)
            || !bottom_lane_num_to_segment.contains_key(&lane_num)
        {
            return Err("Lane missing in target nodes.");
        }
        if top_lane_num_to_segment[&lane_num] != bottom_lane_num_to_segment[&lane_num] {
            return Err("Left and right target lanes do not target the same segment.");
        }
        let segment_len = get_segment_len(top_lane_num_to_segment[&lane_num])?;
        chan_y_lanes.push(CRRSwitchBlockLane {
            starting_track_num: curr_chan_y_track_num,
            segment_len,
        });
        curr_chan_y_track_num += segment_len * 2;
    }

    // Validate the source nodes to ensure that they are consistent with the sink nodes.
    // This prevents crashes in the code later.
    for source_node in &crr_sb_info.source_nodes {
        if source_node.dir == CRRSwitchDir::OPIN {
            continue;
        }
        if source_node.lane_num == 0 {
            return Err("Found a source node with lane num 0.");
        }
        let source_node_lane_id = source_node.lane_num - 1;
        let source_node_lane = match source_node.dir {
            CRRSwitchDir::Left | CRRSwitchDir::Right => {
                if source_node_lane_id >= chan_x_lanes.len() {
                    return Err("Found a source node with an invalid lane.");
                }
                &chan_x_lanes[source_node_lane_id]
            }
            CRRSwitchDir::Top | CRRSwitchDir::Bottom => {
                if source_node_lane_id >= chan_y_lanes.len() {
                    return Err("Found a source node with an invalid lane.");
                }
                &chan_y_lanes[source_node_lane_id]
            }
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => {panic!("TODO: Handle this.")},
        };
        if source_node_lane.segment_len != get_segment_len(&source_node.segment_type)? {
            return Err("Found a source node in a lane with the wrong segment type.");
        }
        let tap_num = match source_node.source_pin {
            CRRSwitchSourcePin::Tap { tap_num } => tap_num,
            _ => 1,
        };
        if tap_num == 0 || tap_num > source_node_lane.segment_len {
            return Err("Found a source node with an invalid tap number.");
        }
    }

    Ok(CRRSwitchBlock {
        chan_x_lanes,
        chan_y_lanes,
        chan_x_width: curr_chan_x_track_num,
        chan_y_width: curr_chan_y_track_num,
    })
}

// FIXME: Everything below belongs in the architecture parser in my opinion.
//      -- The pin locations would need to be moved out since they do not make sense in this context.
type TilePinIndexMap = HashMap<String, Vec<HashMap<String, Vec<usize>>>>;

struct TilePinMapper {
    tile_name: String,
    pin_locations: Vec<egui::Vec2>,
    num_pins_in_tile: usize,
    // [sub_tile_name][sub_tile_cap_index][port_bus_name][port_index] -> pin_index
    pin_index_lookup: TilePinIndexMap,

    pin_name_lookup: Vec<String>,
}

impl TilePinMapper {
    pub fn new(tile_name: &str, arch: &FPGAArch) -> Result<TilePinMapper, String> {
        let tile = arch.tiles.iter().find(|&tile| {
            tile.name == tile_name
        });
        let tile = match tile {
            Some(t) => t,
            None => {
                return Err("Could not find a tile with the given name.".to_string());
            }
        };

        let mut num_pins_in_tile: usize = 0;
        let mut pin_index_lookup: TilePinIndexMap = HashMap::new();
        let mut pin_name_lookup: Vec<String> = Vec::new();
        for sub_tile in &tile.sub_tiles {
            let mut sub_tile_pin_lookup = Vec::new();
            for sub_tile_cap_index in 0..sub_tile.capacity {
                let mut port_name_pin_lookup = HashMap::new();
                for port in &sub_tile.ports {
                    let (port_name, num_pins) = match port {
                        Port::Input(input_port) => (&input_port.name, input_port.num_pins),
                        Port::Output(output_port) => (&output_port.name, output_port.num_pins),
                        Port::Clock(clock_port) => (&clock_port.name, clock_port.num_pins),
                    };
                    let num_pins = num_pins as usize;
                    let mut pin_indices = Vec::new();
                    for pin_index in num_pins_in_tile..num_pins_in_tile+num_pins {
                        pin_indices.push(pin_index);
                        pin_name_lookup.push(format!("{}[{}].{}[{}]", sub_tile.name, sub_tile_cap_index, port_name, pin_index - num_pins_in_tile));
                    }
                    num_pins_in_tile += num_pins;
                    // TODO: Check for dupes.
                    port_name_pin_lookup.insert(port_name.clone(), pin_indices);
                }
                sub_tile_pin_lookup.push(port_name_pin_lookup);
            }
            // TODO: Check for dupes.
            pin_index_lookup.insert(sub_tile.name.clone(), sub_tile_pin_lookup);
        }

        let mut top_pins: Vec<usize> = Vec::new();
        let mut bottom_pins: Vec<usize> = Vec::new();
        let mut left_pins: Vec<usize> = Vec::new();
        let mut right_pins: Vec<usize> = Vec::new();
        for sub_tile in &tile.sub_tiles {
            match &sub_tile.pin_locations {
                SubTilePinLocations::Custom(custom_pin_locations) => {
                    for loc in &custom_pin_locations.pin_locations {
                        // TODO: Handle xoffset and yoffset
                        let mut pins = get_pins_in_pin_loc(loc, sub_tile, &pin_index_lookup)?;
                        match loc.side {
                            PinSide::Top => top_pins.append(&mut pins),
                            PinSide::Bottom => bottom_pins.append(&mut pins),
                            PinSide::Left => left_pins.append(&mut pins),
                            PinSide::Right => right_pins.append(&mut pins),
                        }
                    }
                },
                _ => {},
            }
        }

        let mut pin_locations: Vec<egui::Vec2> = vec![egui::Vec2::new(0.0, 0.0); num_pins_in_tile];
        for (i, pin_index) in top_pins.iter().enumerate() {
            pin_locations[*pin_index] = egui::Vec2::new((i as f32) / (top_pins.len() as f32), 0.0);
        }
        for (i, pin_index) in bottom_pins.iter().enumerate() {
            pin_locations[*pin_index] = egui::Vec2::new((i as f32) / (bottom_pins.len() as f32), 1.0);
        }
        for (i, pin_index) in left_pins.iter().enumerate() {
            pin_locations[*pin_index] = egui::Vec2::new(0.0, (i as f32) / (left_pins.len() as f32));
        }
        for (i, pin_index) in right_pins.iter().enumerate() {
            pin_locations[*pin_index] = egui::Vec2::new(1.0, (i as f32) / (right_pins.len() as f32));
        }

        Ok(TilePinMapper {
            tile_name: tile_name.to_string(),
            pin_locations,
            num_pins_in_tile,
            pin_index_lookup,
            pin_name_lookup,
        })
    }
}

fn get_pins_in_pin_loc(loc: &PinLoc, sub_tile: &SubTile, pin_index_lookup: &TilePinIndexMap) -> Result<Vec<usize>, String> {
    let mut pins: Vec<usize> = Vec::new();
    for pin_string in &loc.pin_strings {
        let split_pin_string: Vec<&str> = pin_string.split(".").collect();
        // Expect there to only be 2.
        // <sub_tile_name>([{bus}])?.<sub_tile_port>([{bus}])?
        if split_pin_string.len() != 2 {
            return Err("Invalid pin string, expected to be of the form '<sub_tile_name>.<sub_tile_port>'.".to_string());
        }
        let sub_tile_portion = split_pin_string[0];
        let port_portion = split_pin_string[1];

        // Parse the sub-tile portion.
        let (sub_tile_name, sub_tile_bus_slice) = split_bus_name(sub_tile_portion)?;
        if sub_tile_name != sub_tile.name {
            return Err("Invalid pin string, does not start with the correct sub-tile.".to_string());
        }
        let sub_tile_bus = match sub_tile_bus_slice {
            Some(bus_slice) => parse_bus(bus_slice)?,
            None => 0..=(sub_tile.capacity-1),
        };

        // Parse port portion.
        let (port_name, port_bus_slice) = split_bus_name(port_portion)?;
        // TODO: We can make this lookup much faster by having a lookup between [sub-tile][port] -> num_pins
        let port = sub_tile.ports.iter().find(|&port| {
            let other_port_name = match &port {
                Port::Input(input_port) => &input_port.name,
                Port::Output(output_port) => &output_port.name,
                Port::Clock(clock_port) => &clock_port.name,
            };
            other_port_name == port_name
        });
        let port = match port {
            Some(p) => p,
            None => { return Err("Cannot find port in pin string".to_string()) }
        };
        let num_port_pins = match &port {
            Port::Input(input_port) => input_port.num_pins,
            Port::Output(output_port) => output_port.num_pins,
            Port::Clock(clock_port) => clock_port.num_pins,
        };
        let port_bus = match port_bus_slice {
            Some(bus_slice) => parse_bus(bus_slice)?,
            None => 0..=(num_port_pins-1),
        };

        // Get the pins.
        for sub_tile_cap_index in sub_tile_bus {
            if sub_tile_cap_index < 0 || sub_tile_cap_index >= sub_tile.capacity {
                return Err("Invalid sub tile index.".to_string());
            }
            let sub_tile_pin_index_lookup = &pin_index_lookup[sub_tile_name][sub_tile_cap_index as usize][port_name];
            for bit in port_bus.clone() {
                if bit < 0 || bit >= num_port_pins {
                    return Err("Invalid port bit position.".to_string());
                }
                pins.push(sub_tile_pin_index_lookup[bit as usize]);
            }
        }
    }

    Ok(pins)
}

fn split_bus_name(s: &str) -> Result<(&str, Option<&str>), String> {
    if let Some(idx) = s.find('[') {
        let (name, slice) = s.split_at(idx);

        if !slice.ends_with(']') {
            return Err(format!("Invalid bus slice: {}", s));
        }

        Ok((name, Some(slice)))
    } else {
        Ok((s, None))
    }
}

fn parse_bus(bus: &str) -> Result<RangeInclusive<i32>, String> {
    if !bus.starts_with('[') || !bus.ends_with(']') {
        return Err(format!("Invalid bus format: {}", bus));
    }

    let inner = &bus[1..bus.len() - 1];

    if let Some((a, b)) = inner.split_once(':') {
        let msb: i32 = a.trim().parse().map_err(|_| "Invalid number")?;
        let lsb: i32 = b.trim().parse().map_err(|_| "Invalid number")?;
        Ok(msb.min(lsb)..=msb.max(lsb))
    } else {
        let bit: i32 = inner.trim().parse().map_err(|_| "Invalid number")?;
        Ok(bit..=bit)
    }
}

// fn get_full_tile_pin_name(pin_string: &str, sub_tile: &SubTile)
