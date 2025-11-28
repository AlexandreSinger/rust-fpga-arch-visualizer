use eframe::egui;
use fpga_arch_parser::{FPGAArch, PBType, PBTypeClass, Port, Tile};
use std::collections::HashMap;

#[derive(Default)]
pub struct IntraTileState {
    selected_modes: HashMap<String, usize>,
    highlighted_positions_this_frame: Vec<egui::Pos2>,
    highlighted_positions_next_frame: Vec<egui::Pos2>,
}

pub fn render_intra_tile_view(
    ui: &mut egui::Ui,
    arch: &FPGAArch,
    tile: &Tile,
    state: &mut IntraTileState,
) {
    // Cycle frame highlights
    state.highlighted_positions_this_frame =
        std::mem::take(&mut state.highlighted_positions_next_frame);
    // Split view: Top/Left is tree, Bottom/Right is visual canvas
    // We'll use a collapsing header for the tree view to save space
    ui.heading(format!("Tile: {}", tile.name));
    ui.separator();

    egui::CollapsingHeader::new("Hierarchy Tree").show(ui, |ui| {
        render_hierarchy_tree(ui, arch, tile);
    });

    ui.separator();
    ui.heading("Visual Layout");

    egui::ScrollArea::both()
        .id_source("intra_tile_canvas")
        .show(ui, |ui| {
            // Determine the root PBType to visualize
            // For now, visualize the first sub-tile's first equivalent site
            if let Some(sub_tile) = tile.sub_tiles.first() {
                if let Some(site) = sub_tile.equivalent_sites.first() {
                    if let Some(root_pb) = arch
                        .complex_block_list
                        .iter()
                        .find(|pb| pb.name == site.pb_type)
                    {
                        // Calculate total size to set min_size for ScrollArea
                        let total_size = measure_pb_type(root_pb, state, &root_pb.name);

                        // Allocate a large canvas with the size we calculated
                        // Using total_size for allocation ensures the scroll area knows how big the content is
                        let (response, painter) = ui.allocate_painter(
                            total_size + egui::vec2(40.0, 40.0),
                            egui::Sense::drag(),
                        );

                        // Determine layout
                        let start_pos = response.rect.min + egui::vec2(20.0, 20.0);

                        let _ =
                            draw_pb_type(&painter, root_pb, start_pos, state, &root_pb.name, ui);
                    } else {
                        ui.label("Root PBType not found");
                    }
                }
            }
        });
}

fn render_hierarchy_tree(ui: &mut egui::Ui, arch: &FPGAArch, tile: &Tile) {
    for sub_tile in &tile.sub_tiles {
        ui.collapsing(format!("SubTile: {}", sub_tile.name), |ui| {
            ui.label(format!("Capacity: {}", sub_tile.capacity));
            for site in &sub_tile.equivalent_sites {
                ui.label(format!("Site PB Type: {}", site.pb_type));
                // Find the PBType definition
                if let Some(pb_type) = arch
                    .complex_block_list
                    .iter()
                    .find(|pb| pb.name == site.pb_type)
                {
                    ui.push_id(format!("pb_{}", pb_type.name), |ui| {
                        render_pb_type_tree_node(ui, pb_type);
                    });
                } else {
                    ui.colored_label(egui::Color32::RED, "PBType not found!");
                }
            }
        });
    }
}

fn render_pb_type_tree_node(ui: &mut egui::Ui, pb_type: &PBType) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("PB Type: {}", pb_type.name)).strong());
            ui.label(format!("(Num: {})", pb_type.num_pb));
            if let Some(class) = &pb_type.blif_model {
                ui.label(format!("Model: {}", class));
            }
            match pb_type.class {
                PBTypeClass::Lut => {
                    ui.label(egui::RichText::new("[LUT]").color(egui::Color32::YELLOW));
                }
                PBTypeClass::FlipFlop => {
                    ui.label(egui::RichText::new("[FF]").color(egui::Color32::LIGHT_BLUE));
                }
                PBTypeClass::Memory => {
                    ui.label(egui::RichText::new("[MEM]").color(egui::Color32::GREEN));
                }
                PBTypeClass::None => {}
            }
        });

        ui.indent("ports", |ui| {
            for port in &pb_type.ports {
                // Display ports
                let (direction, name, num_pins) = match port {
                    Port::Input(p) => ("In", &p.name, p.num_pins),
                    Port::Output(p) => ("Out", &p.name, p.num_pins),
                    Port::Clock(p) => ("Clock", &p.name, p.num_pins),
                };
                ui.label(format!("{} Port: {} [{}]", direction, name, num_pins));
            }
        });

        if !pb_type.modes.is_empty() {
            ui.indent("modes", |ui| {
                for mode in &pb_type.modes {
                    ui.collapsing(format!("Mode: {}", mode.name), |ui| {
                        for child_pb in &mode.pb_types {
                            render_pb_type_tree_node(ui, child_pb);
                        }
                        if !mode.interconnects.is_empty() {
                            ui.collapsing("Interconnects", |ui| {
                                for inter in &mode.interconnects {
                                    let (kind, name, input, output, pack_pattern) = match inter {
                                        fpga_arch_parser::Interconnect::Direct(d) => (
                                            "Direct",
                                            &d.name,
                                            &d.input,
                                            &d.output,
                                            &d.pack_pattern,
                                        ),
                                        fpga_arch_parser::Interconnect::Mux(m) => {
                                            ("Mux", &m.name, &m.input, &m.output, &m.pack_pattern)
                                        }
                                        fpga_arch_parser::Interconnect::Complete(c) => (
                                            "Complete",
                                            &c.name,
                                            &c.input,
                                            &c.output,
                                            &c.pack_pattern,
                                        ),
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{}: {} ({} -> {})",
                                            kind, name, input, output
                                        ));
                                        if let Some(pp) = pack_pattern {
                                            ui.label(format!("[Pack: {}]", pp.name));
                                        }
                                    });
                                }
                            });
                        }
                    });
                }
            });
        }
    });
}

// --- Visualization Logic ---

const PADDING: f32 = 50.0;
const HEADER_HEIGHT: f32 = 35.0;
const MIN_BLOCK_SIZE: egui::Vec2 = egui::vec2(80.0, 120.0);
const PORT_LENGTH: f32 = 15.0;
const MIN_PIN_SPACING: f32 = 25.0;
const PIN_SQUARE_SIZE: f32 = 6.0;

enum LayoutDirection {
    Vertical,
    Horizontal,
}

fn get_layout_direction(children: &[PBType]) -> LayoutDirection {
    if children.len() > 1 {
        // Multiple different child types (e.g. LUT + FF) -> Horizontal flow
        LayoutDirection::Horizontal
    } else {
        // Single child type repeated (e.g. Array of BLEs) -> Vertical stack
        LayoutDirection::Vertical
    }
}

fn measure_pb_type(pb_type: &PBType, state: &IntraTileState, instance_path: &str) -> egui::Vec2 {
    let mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);

    let children = if !pb_type.modes.is_empty() {
        if mode_index < pb_type.modes.len() {
            &pb_type.modes[mode_index].pb_types
        } else {
            &pb_type.modes[0].pb_types
        }
    } else {
        &pb_type.pb_types
    };

    if children.is_empty() {
        // Leaf node
        return MIN_BLOCK_SIZE;
    }

    let direction = get_layout_direction(children);

    let total_w: f32;
    let total_h: f32;

    match direction {
        LayoutDirection::Vertical => {
            let mut max_child_w: f32 = 0.0;
            let mut current_h: f32 = 0.0;

            for child_pb in children {
                let num = child_pb.num_pb as f32;
                let gaps = std::cmp::max(child_pb.num_pb - 1, 0) as f32;

                let mut max_instance_size = egui::vec2(0.0, 0.0);

                for i in 0..child_pb.num_pb {
                    let child_instance_name = if child_pb.num_pb == 1 {
                        child_pb.name.clone()
                    } else {
                        format!("{}[{}]", child_pb.name, i)
                    };
                    let child_path = format!("{}.{}", instance_path, child_instance_name);
                    let s = measure_pb_type(child_pb, state, &child_path);
                    max_instance_size = max_instance_size.max(s);
                }

                let total_instances_h = max_instance_size.y * num + PADDING * gaps;

                max_child_w = max_child_w.max(max_instance_size.x);
                current_h += total_instances_h + PADDING;
            }
            // Remove last padding if added
            if !children.is_empty() {
                current_h -= PADDING;
            }

            total_w = max_child_w;
            total_h = current_h;
        }
        LayoutDirection::Horizontal => {
            let mut max_child_h: f32 = 0.0;
            let mut current_w: f32 = 0.0;

            for child_pb in children {
                let num = child_pb.num_pb as f32;
                let gaps = std::cmp::max(child_pb.num_pb - 1, 0) as f32;

                let mut max_instance_size = egui::vec2(0.0, 0.0);
                for i in 0..child_pb.num_pb {
                    let child_instance_name = if child_pb.num_pb == 1 {
                        child_pb.name.clone()
                    } else {
                        format!("{}[{}]", child_pb.name, i)
                    };
                    let child_path = format!("{}.{}", instance_path, child_instance_name);
                    let s = measure_pb_type(child_pb, state, &child_path);
                    max_instance_size = max_instance_size.max(s);
                }

                let child_instances_h = max_instance_size.y * num + PADDING * gaps;
                let child_instances_w = max_instance_size.x;

                max_child_h = max_child_h.max(child_instances_h);
                current_w += child_instances_w + PADDING;
            }

            if !children.is_empty() {
                current_w -= PADDING;
            }

            total_w = current_w;
            total_h = max_child_h;
        }
    }

    // Ensure minimum size
    let total_input_pins: i32 = pb_type
        .ports
        .iter()
        .filter_map(|p| match p {
            Port::Input(ip) => Some(ip.num_pins),
            _ => None,
        })
        .sum();
    let total_output_pins: i32 = pb_type
        .ports
        .iter()
        .filter_map(|p| match p {
            Port::Output(op) => Some(op.num_pins),
            _ => None,
        })
        .sum();
    let max_pins = total_input_pins.max(total_output_pins) as f32;
    let min_port_height = (max_pins + 1.0) * MIN_PIN_SPACING;

    // Reserve space for interconnects (Muxes) on the right if they exist
    let interconnect_width = if !pb_type.modes.is_empty() || !pb_type.interconnects.is_empty() {
        80.0 // Extra space for Mux/Crossbar column
    } else {
        0.0
    };

    let w = (total_w + PADDING * 2.0 + interconnect_width).max(MIN_BLOCK_SIZE.x);
    // Height: Header + Top Padding + Children Height + Bottom Padding
    let h = (HEADER_HEIGHT + PADDING + total_h + PADDING)
        .max(MIN_BLOCK_SIZE.y)
        .max(min_port_height);
    egui::vec2(w, h)
}

fn resolve_bus_list(
    port_list: &[String],
    current_pb_name: &str,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
) -> Vec<String> {
    let mut resolved = Vec::new();
    for port_ref in port_list {
        // 1. Try resolving exact match first
        if resolve_port_pos(port_ref, current_pb_name, my_ports, children_ports).is_some() {
            resolved.push(port_ref.clone());
            continue;
        }

        // 2. If exact match fails, try expanding as bus [0]..[N]
        let mut idx = 0;
        loop {
            // We need to insert [idx] at the end of the port name part
            let candidate = if let Some((prefix, suffix)) = port_ref.rsplit_once('.') {
                format!("{}.{}[{}]", prefix, suffix, idx)
            } else {
                format!("{}[{}]", port_ref, idx)
            };

            if resolve_port_pos(&candidate, current_pb_name, my_ports, children_ports).is_some() {
                resolved.push(candidate);
                idx += 1;
            } else {
                break;
            }
        }
    }
    resolved
}

fn draw_pb_type(
    painter: &egui::Painter,
    pb_type: &PBType,
    pos: egui::Pos2,
    state: &mut IntraTileState,
    instance_path: &str,
    ui: &mut egui::Ui,
) -> HashMap<String, egui::Pos2> {
    let size = measure_pb_type(pb_type, state, instance_path);
    let rect = egui::Rect::from_min_size(pos, size);

    // Determine specific visual style based on class
    let my_ports = match pb_type.class {
        PBTypeClass::Lut => draw_lut(painter, rect, pb_type, state, ui),
        PBTypeClass::FlipFlop => draw_flip_flop(painter, rect, pb_type, state, ui),
        PBTypeClass::Memory => draw_memory(painter, rect, pb_type, state, ui),
        PBTypeClass::None => draw_generic_block(painter, rect, pb_type, state, ui),
    };

    // Draw Mode Selector if multiple modes exist
    if pb_type.modes.len() > 1 {
        // Position selector in title bar right side
        let mode_idx = *state.selected_modes.get(instance_path).unwrap_or(&0);
        let mode_name = &pb_type.modes[mode_idx].name;

        // We need to place a UI widget on top of the canvas.
        // We can use ui.put() with a specific rect.
        let selector_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(rect.width() - 100.0, 2.0),
            egui::vec2(90.0, 18.0),
        );

        // Create a child ui for the combo box to clip it or position it
        // Actually, purely using `ui.put` with a ComboBox might be tricky if the painter is separate.
        // But we are inside a `ui` scope (ScrollArea).
        // We just need to ensure coordinates align.
        // Note: `rect` is in painter coordinates (screen space).

        let mut selected_mode = mode_idx;

        ui.put(selector_rect, |ui: &mut egui::Ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
            egui::ComboBox::from_id_source(format!("mode_sel_{}", instance_path))
                .selected_text(mode_name)
                .show_ui(ui, |ui| {
                    for (i, mode) in pb_type.modes.iter().enumerate() {
                        ui.selectable_value(&mut selected_mode, i, &mode.name);
                    }
                })
                .response
        });

        if selected_mode != mode_idx {
            state
                .selected_modes
                .insert(instance_path.to_string(), selected_mode);
        }
    }

    let mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);

    // Draw children if any (recurse)
    let children = if !pb_type.modes.is_empty() {
        if mode_index < pb_type.modes.len() {
            &pb_type.modes[mode_index].pb_types
        } else {
            &pb_type.modes[0].pb_types
        }
    } else {
        &pb_type.pb_types
    };

    let mut children_ports: HashMap<String, egui::Pos2> = HashMap::new();

    if !children.is_empty() {
        let direction = get_layout_direction(children);

        let start_x = rect.min.x + PADDING;
        let start_y = rect.min.y + HEADER_HEIGHT + PADDING;

        let mut cursor_x = start_x;
        let mut cursor_y = start_y;

        for child_pb in children {
            // Pre-calculate max dimensions for this child type row/column to align
            // (Simplified alignment logic)

            for i in 0..child_pb.num_pb {
                // Register child ports with prefix
                let instance_name = if child_pb.num_pb == 1 {
                    child_pb.name.clone()
                } else {
                    format!("{}[{}]", child_pb.name, i)
                };
                let child_path = format!("{}.{}", instance_path, instance_name);

                let child_single_size = measure_pb_type(child_pb, state, &child_path);

                let pos = egui::pos2(cursor_x, cursor_y);
                let child_map = draw_pb_type(painter, child_pb, pos, state, &child_path, ui);

                for (port_name, p) in child_map {
                    children_ports.insert(format!("{}.{}", instance_name, port_name), p);
                }

                cursor_y += child_single_size.y + PADDING;
            }

            let mut max_col_width: f32 = 0.0;
            let mut max_col_height: f32 = 0.0; // For Horizontal layout vertical sizing

            for i in 0..child_pb.num_pb {
                let instance_name = if child_pb.num_pb == 1 {
                    child_pb.name.clone()
                } else {
                    format!("{}[{}]", child_pb.name, i)
                };
                let child_path = format!("{}.{}", instance_path, instance_name);
                let s = measure_pb_type(child_pb, state, &child_path);
                max_col_width = max_col_width.max(s.x);
                max_col_height = max_col_height.max(s.y); // Not really used for Vertical stack height accumulation
            }

            match direction {
                LayoutDirection::Vertical => {
                    // Vertical Layout implies children are stacked.
                    // Wait, my `get_layout_direction` logic:
                    // Single child type -> Vertical stack.
                    // We just iterated `for child_pb in children` (which has len 1).
                    // And inside `for i in 0..num_pb` we stacked them.
                    // So we are done.
                }
                LayoutDirection::Horizontal => {
                    // Heterogeneous children.
                    // We placed `child_pb` instances vertically (if num_pb > 1).
                    // Now we move cursor X for the NEXT `child_pb` type.
                    cursor_x += max_col_width + PADDING;
                    cursor_y = start_y;
                }
            }
        }
    }

    // Draw Interconnects
    let interconnects = if !pb_type.modes.is_empty() {
        // Use selected mode!
        if mode_index < pb_type.modes.len() {
            &pb_type.modes[mode_index].interconnects
        } else {
            &pb_type.modes[0].interconnects
        }
    } else {
        &pb_type.interconnects
    };

    for inter in interconnects {
        let (input_str, output_str, kind) = match inter {
            fpga_arch_parser::Interconnect::Direct(d) => (&d.input, &d.output, "direct"),
            fpga_arch_parser::Interconnect::Mux(m) => (&m.input, &m.output, "mux"),
            fpga_arch_parser::Interconnect::Complete(c) => (&c.input, &c.output, "complete"),
        };

        // Simple parsing: split by space, but we need to handle ranges like fle[3:0].out
        let raw_sources = expand_port_list(input_str);
        let raw_sinks = expand_port_list(output_str);

        // Resolve implicit buses to individual pins
        let sources = resolve_bus_list(&raw_sources, &pb_type.name, &my_ports, &children_ports);
        let sinks = resolve_bus_list(&raw_sinks, &pb_type.name, &my_ports, &children_ports);

        if kind == "direct" {
            // Direct: 1-to-1 connection
            for (i, src) in sources.iter().enumerate() {
                if i < sinks.len() {
                    let dst = &sinks[i];
                    draw_connection(
                        painter,
                        src,
                        dst,
                        pb_type,
                        &my_ports,
                        &children_ports,
                        state,
                        ui,
                        rect,
                    );
                }
            }
        } else {
            // Complete or Mux: Draw an explicit block
            draw_interconnect_block(
                painter,
                kind,
                &sources,
                &sinks,
                pb_type,
                &my_ports,
                &children_ports,
                state,
                ui,
                rect,
            );
        }
    }

    my_ports
}

fn draw_interconnect_block(
    painter: &egui::Painter,
    kind: &str,
    sources: &[String],
    sinks: &[String],
    current_pb: &PBType,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    parent_rect: egui::Rect,
) {
    // 1. Resolve positions
    let mut valid_sinks = Vec::new();
    for dst in sinks {
        if let Some(pos) = resolve_port_pos(dst, &current_pb.name, my_ports, children_ports) {
            valid_sinks.push((dst, pos));
        }
    }

    if valid_sinks.is_empty() {
        return;
    }

    // 2. Calculate Block Position
    // Place it to the left of the average sink position
    let avg_y: f32 = valid_sinks.iter().map(|(_, p)| p.y).sum::<f32>() / valid_sinks.len() as f32;
    let min_x: f32 = valid_sinks
        .iter()
        .map(|(_, p)| p.x)
        .fold(f32::INFINITY, |a, b| a.min(b));

    // Mux dimensions
    let width = 35.0;

    // Calculate height based on BOTH sinks spread and number of sources (inputs)
    // because we want to separate inputs vertically.
    let max_sink_y = valid_sinks
        .iter()
        .map(|(_, p)| p.y)
        .fold(f32::NEG_INFINITY, |a, b| a.max(b));
    let min_sink_y = valid_sinks
        .iter()
        .map(|(_, p)| p.y)
        .fold(f32::INFINITY, |a, b| a.min(b));

    let sink_spread = max_sink_y - min_sink_y;

    // Height needed for inputs
    let input_spacing = 15.0;
    let input_spread = (sources.len() as f32 + 1.0) * input_spacing;

    let spread = sink_spread.max(input_spread);
    let height = (spread + 20.0).max(40.0).min(150.0); // Clamp height

    let block_center = egui::pos2(min_x - 50.0, avg_y); // Shift left more for wider block
    let rect = egui::Rect::from_center_size(block_center, egui::vec2(width, height));

    // ... highlight logic ...
    // 3. Highlight Check
    // If mouse is over the block, highlight everything
    let mut block_hovered = false;
    if ui.rect_contains_pointer(rect) {
        block_hovered = true;
    }

    let is_block_highlighted = block_hovered
        || state
            .highlighted_positions_this_frame
            .iter()
            .any(|p| rect.contains(*p));

    let stroke_color = if is_block_highlighted {
        egui::Color32::RED
    } else {
        egui::Color32::from_rgb(100, 100, 100)
    };
    let stroke = egui::Stroke::new(1.5, stroke_color);
    let fill_color = egui::Color32::from_rgb(230, 230, 230);

    // 4. Draw Block Shape
    if kind == "mux" {
        // Trapezoid
        // Wider on left (input), narrower on right (output)
        let w = rect.width();
        let h = rect.height();
        let c = rect.center();
        let trap_points = vec![
            c + egui::vec2(-w / 2.0, -h / 2.0), // Top-Left
            c + egui::vec2(w / 2.0, -h / 4.0),  // Top-Right (narrower)
            c + egui::vec2(w / 2.0, h / 4.0),   // Bottom-Right
            c + egui::vec2(-w / 2.0, h / 2.0),  // Bottom-Left
        ];
        painter.add(egui::Shape::convex_polygon(trap_points, fill_color, stroke));
    } else {
        // Complete (Rectangle)
        painter.rect(rect, 2.0, fill_color, stroke);
        // Add a "X" or crossbar visual?
        painter.line_segment([rect.min, rect.max], stroke);
        painter.line_segment(
            [
                egui::pos2(rect.min.x, rect.max.y),
                egui::pos2(rect.max.x, rect.min.y),
            ],
            stroke,
        );
    }

    // 5. Route Connections
    // Sources -> Block (Left Edge)
    // Distribute inputs vertically along the left edge
    let left_edge_x = rect.min.x;

    // Resolve all sources first to sort them
    let mut resolved_sources = Vec::new();
    for src in sources {
        if let Some(src_pos) = resolve_port_pos(src, &current_pb.name, my_ports, children_ports) {
            resolved_sources.push((src, src_pos));
        }
    }

    // Sort sources by X position (ascending)
    // This puts left-most sources (LUTs) at the top (index 0)
    // and right-most sources (FFs) at the bottom (higher index)
    resolved_sources.sort_by(|a, b| {
        a.1.x
            .partial_cmp(&b.1.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let input_step = rect.height() / (resolved_sources.len() as f32 + 1.0);

    for (i, (_src_name, src_pos)) in resolved_sources.iter().enumerate() {
        // Highlight check
        let wire_highlighted = is_block_highlighted
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(*src_pos) < 1.0);
        let wire_color = if wire_highlighted {
            egui::Color32::RED
        } else {
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100)
        };
        let wire_stroke = egui::Stroke::new(1.5, wire_color);

        // Calculate specific input position on the block
        let input_y = rect.min.y + input_step * (i as f32 + 1.0);
        let target = egui::pos2(left_edge_x, input_y);

        // Draw H-V-H to target
        draw_wire_segment(
            painter,
            *src_pos,
            target,
            wire_stroke,
            parent_rect,
            state,
            ui,
        );

        // Hover logic: if src is hovered, highlight this
        if ui.rect_contains_pointer(rect) {
            state.highlighted_positions_next_frame.push(*src_pos);
        }
    }

    // Block (Right Edge) -> Sinks
    let right_edge_x = rect.max.x;
    for (_, dst_pos) in valid_sinks {
        let wire_highlighted = is_block_highlighted
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(dst_pos) < 1.0);
        let wire_color = if wire_highlighted {
            egui::Color32::RED
        } else {
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100)
        };
        let wire_stroke = egui::Stroke::new(1.5, wire_color);

        let start = egui::pos2(right_edge_x, block_center.y);
        draw_wire_segment(painter, start, dst_pos, wire_stroke, parent_rect, state, ui);

        if ui.rect_contains_pointer(rect) {
            state.highlighted_positions_next_frame.push(dst_pos);
        }
    }
}

fn draw_wire_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
    parent_rect: egui::Rect,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) {
    let mut points = Vec::new();
    points.push(start);

    if start.x < end.x {
        let dist = end.x - start.x;

        // Check if destination is the Parent Output (Right Edge)
        // If so, we are routing through the reserved interconnect channel, which is empty.
        // So we can use simple midpoint routing instead of going up.
        let is_parent_output = end.x >= parent_rect.max.x - 20.0;

        // If distance is large (likely crossing a block), route "over" it
        // UNLESS we are targeting the parent output
        if dist > 100.0 && !is_parent_output {
            // Go Right -> Up -> Right -> Down -> Right
            let channel_x_start = start.x + 10.0;
            let channel_x_end = end.x - 10.0;

            // Route above the blocks using the parent container's top area
            // Use top padding area: rect.min.y + HEADER_HEIGHT + small_gap
            // HEADER_HEIGHT is 35.0. Let's use +40.0 to be safe below header line.
            let route_y = parent_rect.min.y + 40.0;

            points.push(egui::pos2(channel_x_start, start.y));
            points.push(egui::pos2(channel_x_start, route_y));
            points.push(egui::pos2(channel_x_end, route_y));
            points.push(egui::pos2(channel_x_end, end.y));
        } else {
            // Adjacent blocks: Route via midpoint
            let mid_x = start.x + dist / 2.0;
            points.push(egui::pos2(mid_x, start.y));
            points.push(egui::pos2(mid_x, end.y));
        }
    } else {
        // Feedback (Right to Left)
        let channel_out = start.x + 10.0;
        let channel_in = end.x - 10.0;
        let mid_y = start.y + (end.y - start.y) / 2.0;

        points.push(egui::pos2(channel_out, start.y));
        points.push(egui::pos2(channel_out, mid_y));
        points.push(egui::pos2(channel_in, mid_y));
        points.push(egui::pos2(channel_in, end.y));
    }
    points.push(end);

    // Check hover on segments
    if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
        let mut hovered = false;
        for i in 0..points.len() - 1 {
            let p1 = points[i];
            let p2 = points[i + 1];
            let min_x = p1.x.min(p2.x) - 5.0;
            let max_x = p1.x.max(p2.x) + 5.0;
            let min_y = p1.y.min(p2.y) - 5.0;
            let max_y = p1.y.max(p2.y) + 5.0;

            if pointer_pos.x >= min_x
                && pointer_pos.x <= max_x
                && pointer_pos.y >= min_y
                && pointer_pos.y <= max_y
            {
                hovered = true;
                break;
            }
        }

        if hovered {
            // Highlight the entire connection (start and end ports)
            state.highlighted_positions_next_frame.push(start);
            state.highlighted_positions_next_frame.push(end);
        }
    }

    painter.add(egui::Shape::line(points, stroke));
}

fn draw_connection(
    painter: &egui::Painter,
    src: &str,
    dst: &str,
    current_pb: &PBType,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    parent_rect: egui::Rect,
) {
    let src_pos = resolve_port_pos(src, &current_pb.name, my_ports, children_ports);
    let dst_pos = resolve_port_pos(dst, &current_pb.name, my_ports, children_ports);

    if let (Some(start), Some(end)) = (src_pos, dst_pos) {
        // Check if src or dst is highlighted
        // Distance tolerance for float equality is handled by `distance < 1.0` check
        let is_highlighted = state
            .highlighted_positions_this_frame
            .iter()
            .any(|p| p.distance(start) < 1.0)
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(end) < 1.0);

        let stroke_color = if is_highlighted {
            egui::Color32::RED
        } else {
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 150)
        };

        let stroke_width = if is_highlighted { 2.5 } else { 1.5 };
        let stroke = egui::Stroke::new(stroke_width, stroke_color);

        draw_wire_segment(painter, start, end, stroke, parent_rect, state, ui);
    }
}

fn expand_port_list(port_list_str: &str) -> Vec<String> {
    let mut parts: Vec<String> = port_list_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let mut i = 0;
    while i < parts.len() {
        let part = parts[i].clone();
        let mut expanded = false;
        let mut start_search = 0;

        while let Some(open_rel) = part[start_search..].find('[') {
            let abs_open = start_search + open_rel;
            if let Some(close_rel) = part[abs_open..].find(']') {
                let abs_close = abs_open + close_rel;
                let content = &part[abs_open + 1..abs_close];

                if content.contains(':') {
                    // Found a range to expand
                    let prefix = &part[..abs_open];
                    let suffix = &part[abs_close + 1..];

                    if let Some((msb_str, lsb_str)) = content.split_once(':') {
                        if let (Ok(msb), Ok(lsb)) = (msb_str.parse::<i32>(), lsb_str.parse::<i32>())
                        {
                            let step = if msb >= lsb { -1 } else { 1 };
                            let mut current = msb;
                            let mut new_items = Vec::new();
                            loop {
                                new_items.push(format!("{}[{}]{}", prefix, current, suffix));
                                if current == lsb {
                                    break;
                                }
                                current += step;
                            }

                            parts.splice(i..i + 1, new_items);
                            expanded = true;
                            break; // Restart processing on the expanded items
                        }
                    }
                }
                // If not a range or failed to parse, continue searching this string
                start_search = abs_close + 1;
            } else {
                // No closing bracket
                break;
            }
        }

        if !expanded {
            i += 1;
        }
    }
    parts
}

fn resolve_port_pos(
    port_ref: &str,
    current_pb_name: &str,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
) -> Option<egui::Pos2> {
    // Case 1: "current_block_name.port" -> look in my_ports with "port"
    if let Some(stripped) = port_ref.strip_prefix(&format!("{}.", current_pb_name)) {
        // Remove potential index if current block is indexed in string (which shouldn't happen inside definition, but just in case)
        // Actually, inside definition, "clb.I" means port I of clb.
        // But we stored it as "I".
        if let Some(pos) = my_ports.get(stripped) {
            return Some(*pos);
        }
        // VTR XML sometimes uses index for single instance? e.g. "clb[0].I"
        // Not standard within the block definition, usually.
    }

    // Case 2: Direct port name (e.g. "I") -> look in my_ports
    if let Some(pos) = my_ports.get(port_ref) {
        return Some(*pos);
    }

    // Case 3: Child port (e.g. "ble4[0].in") -> look in children_ports
    // We need to handle ranges or simple lookups.
    // For now, exact match.
    // TODO: Expand "ble4.in" to "ble4[0].in" if num_pb=1?
    // Our children_ports keys are canonical "name.port" or "name[i].port".

    // Try direct lookup
    if let Some(pos) = children_ports.get(port_ref) {
        return Some(*pos);
    }

    // Try adding [0] if missing index?
    // Parsing logic is complex without robust VTR name parser.
    // Let's try simple "name.port" -> "name[0].port" heuristic
    if let Some((instance, port)) = port_ref.split_once('.') {
        if !instance.contains('[') {
            let alt_key = format!("{}[0].{}", instance, port);
            if let Some(pos) = children_ports.get(&alt_key) {
                return Some(*pos);
            }
        } else if let Some(base_instance) = instance.strip_suffix("[0]") {
            // Try removing [0] if present (e.g. lut4[0].in -> lut4.in)
            // This handles cases where user specifies index 0 but visualizer uses unindexed name for single instances
            let alt_key = format!("{}.{}", base_instance, port);
            if let Some(pos) = children_ports.get(&alt_key) {
                return Some(*pos);
            }
        }
    }

    None
}

fn draw_generic_block(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(240, 240, 240), // Light Gray background
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 100, 100)),
    );

    // Title bar
    let title_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT));
    painter.rect(
        title_rect,
        egui::Rounding::ZERO,
        egui::Color32::from_rgb(200, 200, 200),
        egui::Stroke::NONE,
    );

    painter.text(
        rect.min + egui::vec2(5.0, 5.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(14.0),
        egui::Color32::BLACK,
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

fn draw_lut(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) -> HashMap<String, egui::Pos2> {
    // LUT Shape: Trapezoid-ish or just distinct color
    // Let's do a rectangle with a diagonal line to suggest internal logic table
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(255, 250, 205), // Lemon Chiffon (Light Yellow)
        egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 180, 0)),
    );

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "LUT",
        egui::FontId::monospace(16.0),
        egui::Color32::from_rgb(180, 180, 0),
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0),
        egui::Color32::BLACK,
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

fn draw_flip_flop(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(220, 230, 255), // Light Blue
        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 0, 180)),
    );

    // Draw clock triangle on bottom or side
    let triangle_size = 8.0;
    let bottom_center = rect.center_bottom();

    // Check for clock port to be accurate? simplified for now
    painter.add(egui::Shape::convex_polygon(
        vec![
            bottom_center + egui::vec2(-triangle_size, 0.0),
            bottom_center + egui::vec2(triangle_size, 0.0),
            bottom_center + egui::vec2(0.0, -triangle_size), // Pointing up into the block
        ],
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(1.5, egui::Color32::BLACK),
    ));

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "FF",
        egui::FontId::monospace(16.0),
        egui::Color32::from_rgb(0, 0, 180),
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0),
        egui::Color32::BLACK,
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

fn draw_memory(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) -> HashMap<String, egui::Pos2> {
    painter.rect(
        rect,
        0.0,
        egui::Color32::from_rgb(200, 240, 200), // Light Green
        egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 100, 0)),
    );

    // Draw grid lines to suggest memory array
    let grid_spacing = 10.0;
    let mut y = rect.min.y + 20.0;
    while y < rect.max.y - 10.0 {
        painter.line_segment(
            [
                egui::pos2(rect.min.x + 10.0, y),
                egui::pos2(rect.max.x - 10.0, y),
            ],
            egui::Stroke::new(0.5, egui::Color32::from_rgb(0, 100, 0)),
        );
        y += grid_spacing;
    }

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "RAM",
        egui::FontId::monospace(16.0),
        egui::Color32::from_rgb(0, 100, 0),
    );

    painter.text(
        rect.min + egui::vec2(5.0, 2.0),
        egui::Align2::LEFT_TOP,
        &pb_type.name,
        egui::FontId::proportional(10.0),
        egui::Color32::BLACK,
    );

    let mut port_map = HashMap::new();
    draw_ports(painter, rect, pb_type, &mut port_map, state, ui);
    port_map
}

fn draw_ports(
    painter: &egui::Painter,
    rect: egui::Rect,
    pb_type: &PBType,
    port_map: &mut HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
) {
    // Structure to represent individual pins (expanded from ports)
    struct PinInfo<'a> {
        name: &'a str,
        index: i32,
    }

    // Expand all ports into individual pins
    let mut input_pins: Vec<PinInfo> = Vec::new();
    let mut output_pins: Vec<PinInfo> = Vec::new();
    let mut clock_pins: Vec<PinInfo> = Vec::new();

    for port in &pb_type.ports {
        match port {
            Port::Input(p) => {
                for i in 0..p.num_pins {
                    input_pins.push(PinInfo {
                        name: &p.name,
                        index: i,
                    });
                }
            }
            Port::Output(p) => {
                for i in 0..p.num_pins {
                    output_pins.push(PinInfo {
                        name: &p.name,
                        index: i,
                    });
                }
            }
            Port::Clock(p) => {
                for i in 0..p.num_pins {
                    clock_pins.push(PinInfo {
                        name: &p.name,
                        index: i,
                    });
                }
            }
        }
    }

    // Draw input pins on the left side
    if !input_pins.is_empty() {
        let total_pins = input_pins.len() as f32;
        // Use minimum spacing, but allow more if block is tall enough
        let min_required_height = (total_pins + 1.0) * MIN_PIN_SPACING;
        let spacing = if rect.height() >= min_required_height {
            rect.height() / (total_pins + 1.0)
        } else {
            MIN_PIN_SPACING
        };
        // Center the pins vertically
        let total_pin_height = spacing * (total_pins - 1.0);
        let start_y = rect.min.y + (rect.height() - total_pin_height) / 2.0;
        let x_pos = rect.min.x;

        for (i, pin) in input_pins.iter().enumerate() {
            let y_pos = start_y + spacing * i as f32;
            let start = egui::pos2(x_pos, y_pos);
            let end = egui::pos2(x_pos - PORT_LENGTH, y_pos);
            let port_pos = end;

            // Check if this pin is highlighted
            let is_highlighted = state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(port_pos) < 1.0);

            let stroke_color = if is_highlighted {
                egui::Color32::RED
            } else {
                egui::Color32::BLACK
            };
            let stroke_width = if is_highlighted { 2.5 } else { 1.0 };
            let stroke = egui::Stroke::new(stroke_width, stroke_color);

            // Draw the line
            painter.line_segment([start, end], stroke);

            // Register pin in port_map with indexed name (e.g., "in[0]", "in[1]")
            let pin_name = format!("{}[{}]", pin.name, pin.index);
            port_map.insert(pin_name.clone(), port_pos);

            // Draw small solid square at the border (where wire meets block)
            let square_size = PIN_SQUARE_SIZE;
            let square_rect =
                egui::Rect::from_center_size(start, egui::vec2(square_size, square_size));
            painter.rect_filled(square_rect, 0.0, stroke_color);

            // Hover detection and tooltip
            let hit_rect = square_rect.expand(3.0); // Slightly larger for easier clicking
            let response = ui.put(hit_rect, egui::Label::new(""));
            if response.hovered() {
                state.highlighted_positions_next_frame.push(port_pos);
            }
            response.on_hover_ui(|ui| {
                ui.label(&pin_name);
            });
        }
    }

    // Draw output pins on the right side
    if !output_pins.is_empty() {
        let total_pins = output_pins.len() as f32;
        // Use minimum spacing, but allow more if block is tall enough
        let min_required_height = (total_pins + 1.0) * MIN_PIN_SPACING;
        let spacing = if rect.height() >= min_required_height {
            rect.height() / (total_pins + 1.0)
        } else {
            MIN_PIN_SPACING
        };
        // Center the pins vertically
        let total_pin_height = spacing * (total_pins - 1.0);
        let start_y = rect.min.y + (rect.height() - total_pin_height) / 2.0;
        let x_pos = rect.max.x;

        for (i, pin) in output_pins.iter().enumerate() {
            let y_pos = start_y + spacing * i as f32;
            let start = egui::pos2(x_pos, y_pos);
            let end = egui::pos2(x_pos + PORT_LENGTH, y_pos);
            let port_pos = end;

            // Check if this pin is highlighted
            let is_highlighted = state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(port_pos) < 1.0);

            let stroke_color = if is_highlighted {
                egui::Color32::RED
            } else {
                egui::Color32::BLACK
            };
            let stroke_width = if is_highlighted { 2.5 } else { 1.0 };
            let stroke = egui::Stroke::new(stroke_width, stroke_color);

            // Draw the line
            painter.line_segment([start, end], stroke);

            // Register pin in port_map with indexed name
            let pin_name = format!("{}[{}]", pin.name, pin.index);
            port_map.insert(pin_name.clone(), port_pos);

            // Draw small solid square at the border (where wire meets block)
            let square_size = PIN_SQUARE_SIZE;
            let square_rect =
                egui::Rect::from_center_size(start, egui::vec2(square_size, square_size));
            painter.rect_filled(square_rect, 0.0, stroke_color);

            // Hover detection and tooltip
            let hit_rect = square_rect.expand(3.0); // Slightly larger for easier clicking
            let response = ui.put(hit_rect, egui::Label::new(""));
            if response.hovered() {
                state.highlighted_positions_next_frame.push(port_pos);
            }
            response.on_hover_ui(|ui| {
                ui.label(&pin_name);
            });
        }
    }

    // Draw clock pins on the top
    if !clock_pins.is_empty() {
        let total_pins = clock_pins.len() as f32;
        // Use minimum spacing, but allow more if block is wide enough
        let min_required_width = (total_pins + 1.0) * MIN_PIN_SPACING;
        let spacing = if rect.width() >= min_required_width {
            rect.width() / (total_pins + 1.0)
        } else {
            MIN_PIN_SPACING
        };
        // Center the pins horizontally
        let total_pin_width = spacing * (total_pins - 1.0);
        let start_x = rect.min.x + (rect.width() - total_pin_width) / 2.0;

        for (i, pin) in clock_pins.iter().enumerate() {
            let x_pos = start_x + spacing * i as f32;
            let start = egui::pos2(x_pos, rect.min.y);
            let end = egui::pos2(x_pos, rect.min.y - PORT_LENGTH);
            let port_pos = end;

            // Check if this pin is highlighted
            let is_highlighted = state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(port_pos) < 1.0);

            let stroke_color = if is_highlighted {
                egui::Color32::RED
            } else {
                egui::Color32::RED // Clock ports are typically red
            };
            let stroke_width = if is_highlighted { 2.5 } else { 1.0 };
            let stroke = egui::Stroke::new(stroke_width, stroke_color);

            // Draw the line
            painter.line_segment([start, end], stroke);

            // Register pin in port_map with indexed name
            let pin_name = format!("{}[{}]", pin.name, pin.index);
            port_map.insert(pin_name.clone(), port_pos);

            // Draw small solid square at the border (where wire meets block)
            let square_size = PIN_SQUARE_SIZE;
            let square_rect =
                egui::Rect::from_center_size(start, egui::vec2(square_size, square_size));
            painter.rect_filled(square_rect, 0.0, stroke_color);

            // Hover detection and tooltip
            let hit_rect = square_rect.expand(3.0); // Slightly larger for easier clicking
            let response = ui.put(hit_rect, egui::Label::new(""));
            if response.hovered() {
                state.highlighted_positions_next_frame.push(port_pos);
            }
            response.on_hover_ui(|ui| {
                ui.label(&pin_name);
            });
        }
    }
}
