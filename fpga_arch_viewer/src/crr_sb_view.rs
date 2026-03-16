#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

use crr_sb_parser::{
    CRRSwitchBlockDeserialized, CRRSwitchDir, CRRSwitchSinkNodeInfo, CRRSwitchSourceNodeInfo, CRRSwitchSourcePin,
};
use fpga_arch_parser::{FPGAArch, Layout, Tile, TilePinMapper};

use crate::{color_scheme, crr_view::parse_sb_maps_yaml::{SBMapTemplate, SBMaps, parse_sb_maps_yaml_from_string}, grid::DeviceGrid, grid_view::get_layout_name, tile_rendering::tile_renderer::{TileRenderer, build_render_tile}};

pub struct CRRViewState {
    show_segment_connections: bool,
    show_switch_connections: bool,
    show_lb_pin_connections: bool,

    pub selected_layout_index: usize,

    sb_maps: SBMaps,
}

impl Default for CRRViewState {
    fn default() -> Self {
        // HACK! For now I am just going to hard-code the sb_maps file to make debugging easier.
        let sb_maps_str =
"
SB_MAPS:
    # ==================================================
    # Corners
    # ==================================================
    SB_0__0_: null
    SB_0__41_: null
    SB_41__0_: null
    SB_41__41_: null
    # ==================================================
    # IO Switchboxes
    # ==================================================
    SB_\\*__0_: sb_io.csv
    SB_0__\\*_: sb_io.csv
    SB_41__\\*_: sb_io.csv
    SB_\\*__41_: sb_io.csv
    # ==================================================
    # DSP Related Column 1
    # ==================================================
    SB_[6:41:8]__[1:41:4]_: sb_mult_36_0.csv
    SB_[6:41:8]__[2:41:4]_: sb_mult_36_1.csv
    SB_[6:41:8]__[3:41:4]_: sb_mult_36_2.csv
    SB_[6:41:8]__[4:41:4]_: sb_mult_36_3.csv
    # ==================================================
    # BRAM Related
    # ==================================================
    SB_[2:41:8]__[1:41:6]_: sb_memory_0.csv
    SB_[2:41:8]__[2:41:6]_: sb_memory_1.csv
    SB_[2:41:8]__[3:41:6]_: sb_memory_2.csv
    SB_[2:41:8]__[4:41:6]_: sb_memory_3.csv
    SB_[2:41:8]__[5:41:6]_: sb_memory_4.csv
    SB_[2:41:8]__[6:41:6]_: sb_memory_5.csv
    # ==================================================
    SB_\\*__\\*_: sb_main.csv
";
        let sb_maps = parse_sb_maps_yaml_from_string(sb_maps_str).expect("This should work.");
        Self {
            show_segment_connections: true,
            show_switch_connections: true,
            show_lb_pin_connections: false,
            selected_layout_index: 0,
            sb_maps,
        }
    }
}

struct CRRRenderTile {
    channel_wires: Vec<egui::Shape>,
    segment_connections: Vec<egui::Shape>,
    switch_connections: Vec<egui::Shape>,
    logic_block_renderer: TileRenderer,
    logic_block_connections: Vec<egui::Shape>,
}

pub struct CRRSBView {
    crr_sb_info: Option<crr_sb_parser::CRRSwitchBlockDeserialized>,
    crr_sb: Option<CRRSwitchBlock>,
    zoom_factor: f32,
    last_error: Option<String>,

    crr_view_state: CRRViewState,
}

impl Default for CRRSBView {
    fn default() -> Self {
        Self {
            crr_sb_info: None,
            crr_sb: None,
            zoom_factor: 1.0,
            last_error: None,
            crr_view_state: CRRViewState::default(),
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
        egui::SidePanel::right("crr_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_side_panel(arch, ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(arch, ui);
        });
    }

    fn render_side_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        ui.heading("CRR View");
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.checkbox(&mut self.crr_view_state.show_segment_connections, "Show Segment Connections");

        ui.add_space(10.0);

        ui.checkbox(&mut self.crr_view_state.show_switch_connections, "Show Switch Connections");

        ui.add_space(10.0);

        ui.checkbox(&mut self.crr_view_state.show_lb_pin_connections, "Show LB Pin Connections");

        // Layout selection dropdown
        ui.add_space(10.0);
        if arch.layouts.layout_list.len() > 1 {
            ui.label("Layout:");
            let mut layout_changed = false;
            egui::ComboBox::from_id_salt("layout_selector")
                .selected_text(get_layout_name(arch, self.crr_view_state.selected_layout_index))
                .show_ui(ui, |ui| {
                    for (idx, layout) in arch.layouts.layout_list.iter().enumerate() {
                        let layout_name = match &layout {
                            fpga_arch_parser::Layout::AutoLayout(_) => {
                                "Auto Layout".to_string()
                            }
                            fpga_arch_parser::Layout::FixedLayout(fl) => {
                                format!("Fixed: {}", fl.name)
                            }
                        };
                        if ui
                            .selectable_value(
                                &mut self.crr_view_state.selected_layout_index,
                                idx,
                                layout_name,
                            )
                            .clicked()
                        {
                            layout_changed = true;
                        }
                    }
                });
            if layout_changed {
                // grid_changed = true;
                // state.selected_die_id = 0;
            }
        }
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

                if let Some(Layout::FixedLayout(_)) = arch.layouts.layout_list.get(self.crr_view_state.selected_layout_index) {
                    let grid = DeviceGrid::from_fixed_layout(arch, self.crr_view_state.selected_layout_index);
                    self.render_crr_sb(&self.crr_view_state, crr_sb, crr_sb_info, &self.crr_view_state.sb_maps, &grid, arch, ui);
                }
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

    fn render_crr_sb(
        &self,
        crr_view_state: &CRRViewState,
        crr_sb: &CRRSwitchBlock,
        crr_sb_info: &CRRSwitchBlockDeserialized,
        sb_maps: &SBMaps,
        grid: &DeviceGrid,
        arch: &FPGAArch,
        ui: &mut egui::Ui,
    ) {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let grid_w = grid.width;
                let grid_h = grid.height;

                let max_chan_w = crr_sb.chan_x_width.max(crr_sb.chan_y_width);
                let smaller_available_size = ui.available_height().min(ui.available_width());
                let spacing_between_points =
                    ((smaller_available_size / max_chan_w as f32) / 2.0) * self.zoom_factor;
                let tile_size = egui::vec2(
                    max_chan_w as f32 * spacing_between_points * 2.0,
                    max_chan_w as f32 * spacing_between_points * 2.0,
                );
                let draw_area_size = tile_size * egui::Vec2::new(grid_w as f32, grid_h as f32);

                let (response, painter) = ui.allocate_painter(
                    draw_area_size,
                    egui::Sense::click().union(egui::Sense::hover()),
                );

                let chan_wire_stroke = spacing_between_points / 5.0;

                let tile_draw_area = egui::Rect::from_min_size(egui::Pos2::new(0.0, 0.0), tile_size);
                let tile = arch.tiles.iter().find(|&tile| {
                    tile.name == "clb"
                });
                let tile = match tile {
                    Some(t) => t,
                    None => {
                        panic!("Could not find clb tile. Hardcoded badness.");
                    }
                };
                let render_tile = CRRRenderTile::build_render_tile(tile, crr_sb, crr_sb_info, spacing_between_points, chan_wire_stroke, &tile_draw_area);

                let offset = response.rect.min;

                for i in 0..grid_w {
                    for j in 0..grid_h {
                        let tile_offset = offset + egui::Vec2::new(tile_size.x * i as f32, tile_size.y * j as f32);
                        Self::render_tile(&render_tile, tile_offset, crr_view_state, &painter);

                        let sb_template = sb_maps.get_sb_template(i, j);
                        let text = match &sb_template {
                            Some(SBMapTemplate::Null) => "NULL",
                            Some(SBMapTemplate::File { file_name }) => &file_name,
                            _ => "ERROR",
                        };

                        let font_size = tile_size.x / 10.0;

                        painter.text(
                            tile_offset + tile_size / 2.0,
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(font_size),
                            egui::Color32::RED,
                        );
                    }
                }
            });
    }

    fn render_tile(
        render_tile: &CRRRenderTile,
        offset: egui::Pos2,
        crr_view_state: &CRRViewState,
        painter: &egui::Painter,
    ) {
        let mut chan_shapes = render_tile.channel_wires.clone();
        for shape in &mut chan_shapes {
            shape.translate(offset.to_vec2());
        }
        painter.extend(chan_shapes);

        let mut lb_shapes = render_tile.logic_block_renderer.lb_shapes.clone();
        for shape in &mut lb_shapes {
            shape.translate(offset.to_vec2());
        }
        painter.extend(lb_shapes);

        let mut lb_pin_shapes = render_tile.logic_block_renderer.pin_shapes.clone();
        for shape in &mut lb_pin_shapes {
            shape.translate(offset.to_vec2());
        }
        painter.extend(lb_pin_shapes);

        if crr_view_state.show_segment_connections {
            let mut segment_connections = render_tile.segment_connections.clone();
            for shape in &mut segment_connections {
                shape.translate(offset.to_vec2());
            }
            painter.extend(segment_connections);
        }

        if crr_view_state.show_switch_connections {
            let mut switch_connections = render_tile.switch_connections.clone();
            for shape in &mut switch_connections {
                shape.translate(offset.to_vec2());
            }
            painter.extend(switch_connections);
        }

        if crr_view_state.show_lb_pin_connections {
            let mut logic_block_connections = render_tile.logic_block_connections.clone();
            for shape in &mut logic_block_connections {
                shape.translate(offset.to_vec2());
            }
            painter.extend(logic_block_connections);
        }
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

fn get_source_node_loc(
    source_node: &CRRSwitchSourceNodeInfo,
    spacing_between_points: f32,
    crr_sb: &CRRSwitchBlock,
    sb_size: &egui::Vec2,
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
    get_ptc_loc(ptc_num, spacing_between_points, source_node.dir, sb_size)
}

fn get_sink_node_loc(
    sink_node: &CRRSwitchSinkNodeInfo,
    spacing_between_points: f32,
    crr_sb: &CRRSwitchBlock,
    sb_size: &egui::Vec2,
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
    get_ptc_loc(ptc_num, spacing_between_points, sink_node.dir, sb_size)
}

fn get_ptc_loc(
    ptc_track_num: usize,
    spacing_between_points: f32,
    side: CRRSwitchDir,
    sb_size: &egui::Vec2,
) -> egui::Pos2 {
    egui::Pos2::new(
        match side {
            CRRSwitchDir::Left => 0.0,
            CRRSwitchDir::Right => sb_size.x,
            CRRSwitchDir::Top | CRRSwitchDir::Bottom => {
                (ptc_track_num as f32 * spacing_between_points) + (spacing_between_points / 2.0)
            }
            // FIXME: Update
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => 0.0,
        },
        match side {
            CRRSwitchDir::Top => 0.0,
            CRRSwitchDir::Bottom => sb_size.y,
            CRRSwitchDir::Left | CRRSwitchDir::Right => {
                (ptc_track_num as f32 * spacing_between_points) + (spacing_between_points / 2.0)
            }
            // FIXME: Update
            CRRSwitchDir::IPIN | CRRSwitchDir::OPIN => 0.0,
        },
    )
}

impl CRRRenderTile {
    pub fn build_render_tile(
        tile: &Tile,
        crr_sb: &CRRSwitchBlock,
        crr_sb_info: &CRRSwitchBlockDeserialized,
        spacing_between_points: f32,
        chan_wire_stroke: f32,
        tile_draw_area: &egui::Rect,
    ) -> CRRRenderTile {
        let sub_tile_size = tile_draw_area.size() / 2.0;
        let chan_x_rect = egui::Rect::from_min_size(tile_draw_area.min, sub_tile_size);
        let chan_y_rect = egui::Rect::from_min_size(tile_draw_area.min + sub_tile_size, sub_tile_size);
        let sb_rect = egui::Rect::from_min_size(
            tile_draw_area.min + egui::vec2(sub_tile_size.x, 0.0),
            sub_tile_size,
        );
        let lb_area_rect = egui::Rect::from_min_size(
            tile_draw_area.min + egui::vec2(0.0, sub_tile_size.y),
            sub_tile_size,
        );
        let mut channel_wires = Self::build_chan_x_shapes(
            crr_sb,
            spacing_between_points,
            chan_wire_stroke,
            &chan_x_rect,
            &sb_rect.size(),
        );
        channel_wires.append(&mut Self::build_chan_y_shapes(
            crr_sb,
            spacing_between_points,
            chan_wire_stroke,
            &chan_y_rect,
            &sb_rect.size(),
        ));
        let segment_connections = Self::build_segment_connection_shapes(
            crr_sb,
            spacing_between_points,
            chan_wire_stroke,
            &sb_rect,
        );
        let switch_connections = Self::build_switch_connection_shapes(crr_sb, crr_sb_info, spacing_between_points, &sb_rect);

        let lb_rect = egui::Rect::from_center_size(
            ((lb_area_rect.size() / 2.0) + lb_area_rect.min.to_vec2()).to_pos2(),
            lb_area_rect.size() / 1.25,
        );
        let lb_color = color_scheme::grid_lb_color(false);
        let logic_block_renderer = build_render_tile(&tile, &lb_rect, &lb_color);
        let logic_block_connections = Self::build_lb_connection_shapes(
            &tile.pin_mapper,
            crr_sb,
            crr_sb_info,
            spacing_between_points,
            &logic_block_renderer,
            &sb_rect,
        );

        CRRRenderTile { channel_wires, segment_connections, switch_connections, logic_block_renderer, logic_block_connections }

    }

    fn build_chan_x_shapes(
        crr_sb: &CRRSwitchBlock,
        spacing_between_points: f32,
        chan_wire_stroke: f32,
        chan_x_rect: &egui::Rect,
        sb_size: &egui::Vec2,
    ) -> Vec<egui::Shape> {
        let mut chan_x_shapes: Vec<egui::Shape> = Vec::with_capacity(crr_sb.chan_x_width);
        for chan_x_ptc in 0..crr_sb.chan_x_width {
            let left_conn_pt = get_ptc_loc(
                chan_x_ptc,
                spacing_between_points,
                CRRSwitchDir::Left,
                sb_size,
            );
            chan_x_shapes.push(egui::Shape::line_segment(
                [
                    left_conn_pt + chan_x_rect.left_top().to_vec2(),
                    left_conn_pt + chan_x_rect.right_top().to_vec2(),
                ],
                egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
            ));
        }

        chan_x_shapes
    }

    fn build_chan_y_shapes(
        crr_sb: &CRRSwitchBlock,
        spacing_between_points: f32,
        chan_wire_stroke: f32,
        chan_y_rect: &egui::Rect,
        sb_size: &egui::Vec2,
    ) -> Vec<egui::Shape> {
        let mut chan_y_shapes: Vec<egui::Shape> = Vec::with_capacity(crr_sb.chan_y_width);
        for chan_y_ptc in 0..crr_sb.chan_y_width {
            let left_conn_pt = get_ptc_loc(
                chan_y_ptc,
                spacing_between_points,
                CRRSwitchDir::Top,
                sb_size,
            );
            chan_y_shapes.push(egui::Shape::line_segment(
                [
                    left_conn_pt + chan_y_rect.left_top().to_vec2(),
                    left_conn_pt + chan_y_rect.left_bottom().to_vec2(),
                ],
                egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
            ));
        }

        chan_y_shapes
    }

    fn build_segment_connection_shapes(
        crr_sb: &CRRSwitchBlock,
        spacing_between_points: f32,
        chan_wire_stroke: f32,
        sb_rect: &egui::Rect,
    ) -> Vec<egui::Shape> {
        let mut segment_connection_shapes: Vec<egui::Shape> = Vec::new();
        for chan_x_lane in &crr_sb.chan_x_lanes {
            for i in 0..(chan_x_lane.segment_len - 1) {
                let left_source_ptc_num = chan_x_lane.starting_track_num + (i * 2);
                let right_source_ptc_num = left_source_ptc_num + 1;
                let right_sink_ptc_num = chan_x_lane.starting_track_num + ((i + 1) * 2);
                let left_sink_ptc_num = right_sink_ptc_num + 1;

                let left_source_loc = get_ptc_loc(
                    left_source_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Left,
                    &sb_rect.size(),
                );
                let left_sink_loc = get_ptc_loc(
                    left_sink_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Left,
                    &sb_rect.size(),
                );
                let right_source_loc = get_ptc_loc(
                    right_source_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Right,
                    &sb_rect.size(),
                );
                let right_sink_loc = get_ptc_loc(
                    right_sink_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Right,
                    &sb_rect.size(),
                );

                segment_connection_shapes.push(egui::Shape::line_segment(
                    [
                        left_source_loc + sb_rect.min.to_vec2(),
                        right_sink_loc + sb_rect.min.to_vec2(),
                    ],
                    egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
                ));
                segment_connection_shapes.push(egui::Shape::line_segment(
                    [
                        right_source_loc + sb_rect.min.to_vec2(),
                        left_sink_loc + sb_rect.min.to_vec2(),
                    ],
                    egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
                ));
            }
        }
        for chan_y_lane in &crr_sb.chan_y_lanes {
            for i in 0..(chan_y_lane.segment_len - 1) {
                let top_source_ptc_num = chan_y_lane.starting_track_num + (i * 2);
                let bottom_source_ptc_num = top_source_ptc_num + 1;
                let bottom_sink_ptc_num = chan_y_lane.starting_track_num + ((i + 1) * 2);
                let top_sink_ptc_num = bottom_sink_ptc_num + 1;

                let top_source_loc = get_ptc_loc(
                    top_source_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Top,
                    &sb_rect.size(),
                );
                let top_sink_loc = get_ptc_loc(
                    top_sink_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Top,
                    &sb_rect.size(),
                );
                let bottom_source_loc = get_ptc_loc(
                    bottom_source_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Bottom,
                    &sb_rect.size(),
                );
                let bottom_sink_loc = get_ptc_loc(
                    bottom_sink_ptc_num,
                    spacing_between_points,
                    CRRSwitchDir::Bottom,
                    &sb_rect.size(),
                );

                segment_connection_shapes.push(egui::Shape::line_segment(
                    [
                        top_source_loc + sb_rect.min.to_vec2(),
                        bottom_sink_loc + sb_rect.min.to_vec2(),
                    ],
                    egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
                ));
                segment_connection_shapes.push(egui::Shape::line_segment(
                    [
                        bottom_source_loc + sb_rect.min.to_vec2(),
                        top_sink_loc + sb_rect.min.to_vec2(),
                    ],
                    egui::Stroke::new(chan_wire_stroke, egui::Color32::BLACK),
                ));
            }
        }

        segment_connection_shapes
    }

    fn build_switch_connection_shapes(
        crr_sb: &CRRSwitchBlock,
        crr_sb_info: &CRRSwitchBlockDeserialized,
        spacing_between_points: f32,
        sb_rect: &egui::Rect,
    ) -> Vec<egui::Shape> {
        let mut switch_connection_shapes: Vec<egui::Shape> = Vec::new();
        for edge in &crr_sb_info.edges {
            let src_node = &crr_sb_info.source_nodes[edge.source_node_id];
            let sink_node = &crr_sb_info.sink_nodes[edge.sink_node_id];

            // Skip the IPIN/OPINs for now.
            if src_node.dir == CRRSwitchDir::OPIN || sink_node.dir == CRRSwitchDir::IPIN {
                continue;
            }

            let src_node_loc =
                get_source_node_loc(src_node, spacing_between_points, crr_sb, &sb_rect.size());
            let sink_node_loc =
                get_sink_node_loc(sink_node, spacing_between_points, crr_sb, &sb_rect.size());

            switch_connection_shapes.push(egui::Shape::line_segment(
                [
                    src_node_loc + sb_rect.min.to_vec2(),
                    sink_node_loc + sb_rect.min.to_vec2(),
                ],
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            ));
        }

        switch_connection_shapes
    }

    fn build_lb_connection_shapes(
        pin_mapper: &TilePinMapper,
        crr_sb: &CRRSwitchBlock,
        crr_sb_info: &CRRSwitchBlockDeserialized,
        spacing_between_points: f32,
        logic_block_renderer: &TileRenderer,
        sb_rect: &egui::Rect,
    ) -> Vec<egui::Shape> {
        let mut lb_connection_shapes: Vec<egui::Shape> = Vec::new();

        // Draw flylines from pins to their connections.
        for edge in &crr_sb_info.edges {
            let src_node = &crr_sb_info.source_nodes[edge.source_node_id];
            let sink_node = &crr_sb_info.sink_nodes[edge.sink_node_id];

            // Skip non IPIN/OPINs now.
            if src_node.dir != CRRSwitchDir::OPIN && sink_node.dir != CRRSwitchDir::IPIN {
                continue;
            }

            let src_node_locs = if src_node.dir == CRRSwitchDir::OPIN {
                // TODO: Clean this up.
                if let CRRSwitchSourcePin::Pin { pin_name } = &src_node.source_pin {
                    let pin_index = pin_mapper.parse_pin_name(pin_name);
                    let pin_index = match pin_index {
                        Ok(idx) => idx,
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    };
                    &logic_block_renderer.pin_locations[pin_index]
                } else {
                    continue;
                }
            } else {
                &vec![(get_source_node_loc(src_node, spacing_between_points, crr_sb, &sb_rect.size()) + sb_rect.min.to_vec2()).to_vec2(); 1]
            };

            let sink_node_locs = if sink_node.dir == CRRSwitchDir::IPIN {
                // TODO: Clean this up.
                if let Some(pin_name) = &sink_node.target_pin {
                    let pin_index = pin_mapper.parse_pin_name(pin_name);
                    let pin_index = match pin_index {
                        Ok(idx) => idx,
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    };
                    &logic_block_renderer.pin_locations[pin_index]
                } else {
                    continue;
                }
            } else {
                &vec![(get_sink_node_loc(sink_node, spacing_between_points, crr_sb, &sb_rect.size()) + sb_rect.min.to_vec2()).to_vec2(); 1]
            };

            for src_node_loc in src_node_locs {
                for sink_node_loc in sink_node_locs {
                    lb_connection_shapes.push(egui::Shape::line_segment(
                        [
                            src_node_loc.to_pos2(),
                            sink_node_loc.to_pos2(),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    ));
                }
            }
        }

        lb_connection_shapes
    }
}
