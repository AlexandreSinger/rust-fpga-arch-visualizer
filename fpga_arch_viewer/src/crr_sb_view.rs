use crr_sb_parser::{
    CRRSwitchBlockDeserialized, CRRSwitchDir, CRRSwitchSinkNodeInfo, CRRSwitchSourceNodeInfo,
};

pub struct CRRSBView {
    crr_sb_info: Option<crr_sb_parser::CRRSwitchBlockDeserialized>,
    zoom_factor: f32,
}

impl Default for CRRSBView {
    fn default() -> Self {
        Self {
            crr_sb_info: None,
            zoom_factor: 1.0,
        }
    }
}

impl CRRSBView {
    fn load_crr_csv_file(&mut self, file_path: std::path::PathBuf) {
        self.crr_sb_info = match crr_sb_parser::parse_csv_file(&file_path) {
            Ok(crr_sb_info) => Some(crr_sb_info),
            Err(e) => {
                println!("{e}");
                None
            }
        };
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(ui);
        });
    }

    fn render_central_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(crr_sb_info) = &self.crr_sb_info {
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

            self.render_crr_sb(crr_sb_info, ui);
        } else if ui.button("Select CSV file to view").clicked() {
            // TODO: Make this cleaner by combining with view.
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CRR CSV Files", &["csv"])
                .set_title("Open CRR CSV File")
                .pick_file()
            {
                self.load_crr_csv_file(path);
            }
        }
    }

    fn get_point_loc(
        side: &CRRSwitchDir,
        lane_num: usize,
        tap_num: usize,
        is_source: bool,
        num_points_per_side: usize,
        spacing_between_points: f32,
    ) -> egui::Pos2 {
        let offset = match is_source {
            true => match side {
                CRRSwitchDir::Left | CRRSwitchDir::Top => 0,
                CRRSwitchDir::Right | CRRSwitchDir::Bottom => 1,
            },
            false => match side {
                CRRSwitchDir::Left | CRRSwitchDir::Top => 1,
                CRRSwitchDir::Right | CRRSwitchDir::Bottom => 0,
            },
        };
        let starting_ptc_track_num = (lane_num - 1) * 4 * 2;
        let ptc_track_num = starting_ptc_track_num + (2 * (tap_num - 1)) + offset;
        egui::Pos2::new(
            match side {
                CRRSwitchDir::Left => 0.0,
                CRRSwitchDir::Right => num_points_per_side as f32 * spacing_between_points,
                CRRSwitchDir::Top | CRRSwitchDir::Bottom => {
                    (ptc_track_num as f32 * spacing_between_points) + (spacing_between_points / 2.0)
                }
            },
            match side {
                CRRSwitchDir::Top => 0.0,
                CRRSwitchDir::Bottom => num_points_per_side as f32 * spacing_between_points,
                CRRSwitchDir::Left | CRRSwitchDir::Right => {
                    (ptc_track_num as f32 * spacing_between_points) + (spacing_between_points / 2.0)
                }
            },
        )
    }

    fn get_source_node_loc(
        source_node: &CRRSwitchSourceNodeInfo,
        num_points_per_side: usize,
        spacing_between_points: f32,
    ) -> egui::Pos2 {
        Self::get_point_loc(
            &source_node.dir,
            source_node.lane_num,
            source_node.tap_num,
            true,
            num_points_per_side,
            spacing_between_points,
        )
    }

    fn get_sink_node_loc(
        sink_node: &CRRSwitchSinkNodeInfo,
        num_points_per_side: usize,
        spacing_between_points: f32,
        tap_num: usize,
    ) -> egui::Pos2 {
        Self::get_point_loc(
            &sink_node.dir,
            sink_node.lane_num,
            tap_num,
            false,
            num_points_per_side,
            spacing_between_points,
        )
    }

    fn render_crr_sb(&self, crr_sb_info: &CRRSwitchBlockDeserialized, ui: &mut egui::Ui) {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let num_points_per_side = crr_sb_info.sink_nodes.len() * 2;
                let spacing_between_points =
                    (ui.available_height() / num_points_per_side as f32) * self.zoom_factor;
                let switch_block_size = egui::vec2(
                    (num_points_per_side + 1) as f32 * spacing_between_points,
                    (num_points_per_side + 1) as f32 * spacing_between_points,
                );

                let (response, painter) = ui.allocate_painter(
                    switch_block_size,
                    egui::Sense::click().union(egui::Sense::hover()),
                );

                let offset = response.rect.min;

                for sink_node in &crr_sb_info.sink_nodes {
                    for i in 1..5 {
                        let conn_pt = Self::get_sink_node_loc(
                            sink_node,
                            num_points_per_side,
                            spacing_between_points,
                            i,
                        );
                        painter.circle(
                            conn_pt + offset.to_vec2(),
                            2.5,
                            egui::Color32::BLACK,
                            egui::Stroke::new(0.5, egui::Color32::BLACK),
                        );
                    }
                }
                for source_node in &crr_sb_info.source_nodes {
                    let conn_pt = Self::get_source_node_loc(
                        source_node,
                        num_points_per_side,
                        spacing_between_points,
                    );
                    painter.circle(
                        conn_pt + offset.to_vec2(),
                        2.5,
                        egui::Color32::BLACK,
                        egui::Stroke::new(0.5, egui::Color32::BLACK),
                    );
                }

                for edge in &crr_sb_info.edges {
                    let src_node = &crr_sb_info.source_nodes[edge.source_node_id];
                    let sink_node = &crr_sb_info.sink_nodes[edge.sink_node_id];

                    let src_node_loc = Self::get_source_node_loc(
                        src_node,
                        num_points_per_side,
                        spacing_between_points,
                    );
                    let sink_node_loc = Self::get_sink_node_loc(
                        sink_node,
                        num_points_per_side,
                        spacing_between_points,
                        1,
                    );

                    painter.line_segment(
                        [
                            src_node_loc + offset.to_vec2(),
                            sink_node_loc + offset.to_vec2(),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    );
                }
            });
    }
}
