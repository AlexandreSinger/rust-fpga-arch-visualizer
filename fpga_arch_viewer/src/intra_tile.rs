//! Intra Tile Visualization
//!
//! Part of the FPGA Visualizer, this module renders the intra-tile view of an FPGA tile.

use eframe::egui;
use fpga_arch_parser::{FPGAArch, PBType, PBTypeClass, Port, Tile};
use std::collections::{HashMap, HashSet};

use crate::intra_block_drawing;
use crate::color_scheme;
use crate::intra_hierarchy_tree;

// ------------------------------------------------------------
// Constants
// ------------------------------------------------------------
const PADDING: f32 = 50.0;
const HEADER_HEIGHT: f32 = 35.0;
const MIN_BLOCK_SIZE: egui::Vec2 = egui::vec2(80.0, 120.0);
const MIN_PIN_SPACING: f32 = 25.0;

// ------------------------------------------------------------
// Intra Tile Drawing Entry Point
// ------------------------------------------------------------
#[derive(Default)]
pub struct IntraTileState {
    pub selected_modes: HashMap<String, usize>,
    pub highlighted_positions_this_frame: Vec<egui::Pos2>,
    pub highlighted_positions_next_frame: Vec<egui::Pos2>,
    pub hierarchy_tree_height: Option<f32>,
    pub expanded_blocks: HashSet<String>,
}

fn render_hierarchy_tree_panel(
    ui: &mut egui::Ui,
    arch: &FPGAArch,
    tile: &Tile,
    state: &mut IntraTileState,
    available_rect: egui::Rect,
    available_height: f32,
) -> Option<f32> {
    // Initialize hierarchy tree height
    let default_tree_height = (available_height * 0.3).min(400.0).max(100.0);
    let tree_height = state.hierarchy_tree_height.unwrap_or(default_tree_height);

    let min_tree_height = 50.0;
    let max_tree_height = available_height - 100.0;
    let tree_height = tree_height.clamp(min_tree_height, max_tree_height);

    // Allocate space for hierarchy tree
    let tree_rect = egui::Rect::from_min_size(
        available_rect.min,
        egui::vec2(available_rect.width(), tree_height),
    );
    ui.allocate_ui_at_rect(tree_rect, |ui| {
        ui.set_width(available_rect.width());
        egui::CollapsingHeader::new("Hierarchy Tree")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        intra_hierarchy_tree::render_hierarchy_tree(ui, arch, tile);
                    });
            });
    });

    // Draw resizable separator between hierarchy tree and visual layout
    let separator_y = tree_rect.max.y;
    let separator_rect = egui::Rect::from_min_size(
        egui::pos2(available_rect.min.x, separator_y - 2.0),
        egui::vec2(available_rect.width(), 4.0),
    );
    let separator_response = ui.allocate_ui_at_rect(separator_rect, |ui| {
        ui.allocate_response(separator_rect.size(), egui::Sense::drag())
    });

    ui.painter().rect_filled(
        separator_rect,
        0.0,
        ui.style().visuals.widgets.inactive.bg_fill,
    );
    ui.painter().line_segment(
        [separator_rect.left_top(), separator_rect.right_top()],
        ui.style().visuals.widgets.inactive.bg_stroke,
    );

    if separator_response.inner.dragged() {
        let delta_y = separator_response.inner.drag_delta().y;
        let new_height = (tree_height + delta_y).clamp(min_tree_height, max_tree_height);
        state.hierarchy_tree_height = Some(new_height);
    }

    // Update stored height if it's not set or if window was resized
    if state.hierarchy_tree_height.is_none() || separator_response.inner.drag_stopped() {
        state.hierarchy_tree_height = Some(tree_height);
    }

    Some(separator_rect.max.y)
}

fn render_visual_layout_canvas(
    ui: &mut egui::Ui,
    arch: &FPGAArch,
    tile: &Tile,
    state: &mut IntraTileState,
    sub_tile_index: usize,
    expand_all: bool,
    dark_mode: bool,
) {
    egui::ScrollArea::both()
        .id_source("intra_tile_canvas")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if sub_tile_index < tile.sub_tiles.len() {
                let sub_tile = &tile.sub_tiles[sub_tile_index];
                if let Some(site) = sub_tile.equivalent_sites.first() {
                    if let Some(root_pb) = arch
                        .complex_block_list
                        .iter()
                        .find(|pb| pb.name == site.pb_type)
                    {
                        // Draw pbtype here
                        let total_size = measure_pb_type(root_pb, state, &root_pb.name);
                        let (response, painter) = ui.allocate_painter(
                            total_size + egui::vec2(40.0, 40.0),
                            egui::Sense::drag(),
                        );
                        let start_pos = response.rect.min + egui::vec2(20.0, 20.0);

                        let _ = draw_pb_type(
                            &painter,
                            root_pb,
                            start_pos,
                            state,
                            &root_pb.name,
                            ui,
                            expand_all,
                            dark_mode,
                        );
                    } else {
                        ui.label("Root PBType not found");
                    }
                } else {
                    ui.label("No equivalent site found");
                }
            } else {
                ui.label("Invalid sub_tile index");
            }
        });
}

pub fn render_intra_tile_view(
    ui: &mut egui::Ui,
    arch: &FPGAArch,
    tile: &Tile,
    state: &mut IntraTileState,
    show_hierarchy_tree: bool,
    sub_tile_index: usize,
    expand_all: bool,
    dark_mode: bool,
) {
    state.highlighted_positions_this_frame =
        std::mem::take(&mut state.highlighted_positions_next_frame);
    ui.heading(format!("Tile: {}", tile.name));
    ui.separator();

    let available_rect = ui.available_rect_before_wrap();
    let available_height = available_rect.height();

    let separator_bottom = if show_hierarchy_tree {
        render_hierarchy_tree_panel(ui, arch, tile, state, available_rect, available_height)
    } else {
        None
    };

    // Allocate space for visual layout
    if let Some(separator_y) = separator_bottom {
        let layout_rect = egui::Rect::from_min_max(
            egui::pos2(available_rect.min.x, separator_y),
            available_rect.max,
        );
        ui.allocate_ui_at_rect(layout_rect, |ui| {
            ui.set_width(available_rect.width());
            ui.heading("Visual Layout");
            render_visual_layout_canvas(
                ui,
                arch,
                tile,
                state,
                sub_tile_index,
                expand_all,
                dark_mode,
            );
        });
    } else {
        ui.set_width(available_rect.width());
        ui.heading("Visual Layout");
        render_visual_layout_canvas(ui, arch, tile, state, sub_tile_index, expand_all, dark_mode);
    }
}

// ------------------------------------------------------------
// Expand Block Feature
// ------------------------------------------------------------
pub fn expand_all_blocks(state: &mut IntraTileState, pb_type: &PBType, instance_path: &str) {
    state.expanded_blocks.insert(instance_path.to_string());

    let mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);
    let children = get_children_for_mode(pb_type, mode_index);

    for child_pb in children {
        for i in 0..child_pb.num_pb {
            let instance_name = generate_child_instance_name(child_pb, i as usize);
            let child_path = format!("{}.{}", instance_path, instance_name);
            expand_all_blocks(state, child_pb, &child_path);
        }
    }
}

pub fn collapse_all_blocks(state: &mut IntraTileState) {
    state.expanded_blocks.clear();
}

// ------------------------------------------------------------
// PB Size Measurement
// ------------------------------------------------------------
enum LayoutDirection {
    Vertical,
    Horizontal,
}

fn get_layout_direction(pb_types: &[PBType]) -> LayoutDirection {
    if pb_types.len() > 1 {
        // Multiple different PB types (LUT + FF) -> Horizontal
        LayoutDirection::Horizontal
    } else {
        // Single PB type (Array of BLEs) -> Vertical
        LayoutDirection::Vertical
    }
}

/// Uses a character width approximation of 0.6 * font size.
fn estimate_text_width(font: &egui::FontId, text: &str) -> f32 {
    let char_width = font.size * 0.6;
    text.len() as f32 * char_width
}

/// Represents the type of port to count.
#[derive(Clone, Copy)]
enum PortType {
    Input,
    Output,
    Clock,
}

/// Counts pins of a specific type in a PBType.
fn count_pins(pb_type: &PBType, port_type: PortType) -> i32 {
    pb_type
        .ports
        .iter()
        .filter_map(|p| match (port_type, p) {
            (PortType::Input, Port::Input(ip)) => Some(ip.num_pins),
            (PortType::Output, Port::Output(op)) => Some(op.num_pins),
            (PortType::Clock, Port::Clock(cp)) => Some(cp.num_pins),
            _ => None,
        })
        .sum()
}

/// Calculates the header name width
fn calculate_header_name_width(pb_type: &PBType, has_children: bool) -> f32 {
    let font = egui::FontId::proportional(14.0);
    let name_width = estimate_text_width(&font, &pb_type.name);
    let header_name_width = if has_children {
        name_width + 25.0 // Expand indicator space
    } else {
        name_width + 5.0 // Just margin
    };
    if !pb_type.modes.is_empty() {
        header_name_width + 130.0 // Mode selector space
    } else {
        header_name_width
    }
}

/// Calculates the width needed for blif_model name.
fn calculate_blif_model_width(pb_type: &PBType) -> f32 {
    match pb_type.class {
        PBTypeClass::None => {
            if let Some(blif_model) = &pb_type.blif_model {
                let blif_font = egui::FontId::monospace(14.0);
                estimate_text_width(&blif_font, blif_model) + 20.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn generate_child_instance_name(child_pb: &PBType, index: usize) -> String {
    if child_pb.num_pb == 1 {
        child_pb.name.clone()
    } else {
        format!("{}[{}]", child_pb.name, index)
    }
}

fn get_children_for_mode(pb_type: &PBType, mode_index: usize) -> &[PBType] {
    if !pb_type.modes.is_empty() {
        if mode_index < pb_type.modes.len() {
            &pb_type.modes[mode_index].pb_types
        } else {
            &pb_type.modes[0].pb_types
        }
    } else {
        &pb_type.pb_types
    }
}

fn get_interconnects_for_mode(
    pb_type: &PBType,
    mode_index: usize,
) -> &[fpga_arch_parser::Interconnect] {
    if !pb_type.modes.is_empty() {
        if mode_index < pb_type.modes.len() {
            &pb_type.modes[mode_index].interconnects
        } else {
            &pb_type.modes[0].interconnects
        }
    } else {
        &pb_type.interconnects
    }
}

fn measure_pb_type(pb_type: &PBType, state: &IntraTileState, instance_path: &str) -> egui::Vec2 {
    let is_expanded = state.expanded_blocks.contains(instance_path);

    let mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);
    let children = get_children_for_mode(pb_type, mode_index);

    if !is_expanded && !children.is_empty() {
        let header_name_width_with_selector = calculate_header_name_width(pb_type, true);
        let blif_model_width = calculate_blif_model_width(pb_type);

        let min_width = MIN_BLOCK_SIZE
            .x
            .max(header_name_width_with_selector)
            .max(blif_model_width);
        return egui::vec2(min_width, HEADER_HEIGHT);
    }

    if children.is_empty() {
        let total_input_pins = count_pins(pb_type, PortType::Input);
        let total_output_pins = count_pins(pb_type, PortType::Output);
        let total_clock_pins = count_pins(pb_type, PortType::Clock);

        let max_side_pins = total_input_pins.max(total_output_pins) as f32;
        let min_height_for_pins = if max_side_pins > 0.0 {
            (max_side_pins + 1.0) * MIN_PIN_SPACING
        } else {
            0.0
        };

        let min_width_for_clock = if total_clock_pins > 0 {
            (total_clock_pins as f32 + 1.0) * MIN_PIN_SPACING
        } else {
            0.0
        };

        let header_name_width_with_selector = calculate_header_name_width(pb_type, false);
        let blif_model_width = calculate_blif_model_width(pb_type);

        let required_height = (HEADER_HEIGHT + min_height_for_pins).max(MIN_BLOCK_SIZE.y);
        let required_width = min_width_for_clock
            .max(MIN_BLOCK_SIZE.x)
            .max(header_name_width_with_selector)
            .max(blif_model_width);

        return egui::vec2(required_width, required_height);
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
                    let child_instance_name = generate_child_instance_name(child_pb, i as usize);
                    let child_path = format!("{}.{}", instance_path, child_instance_name);
                    let s = measure_pb_type(child_pb, state, &child_path);
                    max_instance_size = max_instance_size.max(s);
                }

                let total_instances_h = max_instance_size.y * num + PADDING * gaps;

                max_child_w = max_child_w.max(max_instance_size.x);
                current_h += total_instances_h + PADDING;
            }
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
                    let child_instance_name = generate_child_instance_name(child_pb, i as usize);
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

    let total_input_pins = count_pins(pb_type, PortType::Input);
    let total_output_pins = count_pins(pb_type, PortType::Output);
    let max_pins = total_input_pins.max(total_output_pins) as f32;
    let min_port_height = (max_pins + 1.0) * MIN_PIN_SPACING;

    let interconnect_width = if !pb_type.modes.is_empty() || !pb_type.interconnects.is_empty() {
        80.0
    } else {
        0.0
    };

    let header_name_width_with_selector =
        calculate_header_name_width(pb_type, !children.is_empty());
    let blif_model_width = calculate_blif_model_width(pb_type);

    let w = (total_w + PADDING * 2.0 + interconnect_width)
        .max(MIN_BLOCK_SIZE.x)
        .max(header_name_width_with_selector)
        .max(blif_model_width);
    let h = (HEADER_HEIGHT + PADDING + total_h + PADDING)
        .max(MIN_BLOCK_SIZE.y)
        .max(min_port_height);
    egui::vec2(w, h)
}

// ------------------------------------------------------------
// Drawing System
// ------------------------------------------------------------

/// Draws the expand indicator (▶) in the header.
fn draw_expand_indicator(painter: &egui::Painter, header_rect: egui::Rect, dark_mode: bool) {
    let indicator_x = header_rect.min.x + 8.0;
    let indicator_y = header_rect.center().y;
    painter.text(
        egui::pos2(indicator_x, indicator_y),
        egui::Align2::LEFT_CENTER,
        "▶",
        egui::FontId::proportional(12.0),
        color_scheme::theme_text_color(dark_mode),
    );
}

fn resolve_port_pos(
    port_ref: &str,
    current_pb_name: &str,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
) -> Option<egui::Pos2> {
    // clb.I -> I
    if let Some(stripped) = port_ref.strip_prefix(&format!("{}.", current_pb_name)) {
        if let Some(pos) = my_ports.get(stripped) {
            return Some(*pos);
        }
    }

    // I
    if let Some(pos) = my_ports.get(port_ref) {
        return Some(*pos);
    }

    // ble4[0].in[0]
    if let Some(pos) = children_ports.get(port_ref) {
        return Some(*pos);
    }

    // ble4.in -> ble4[0].in
    if let Some((instance, port)) = port_ref.split_once('.') {
        if !instance.contains('[') {
            let alt_key = format!("{}[0].{}", instance, port);
            if let Some(pos) = children_ports.get(&alt_key) {
                return Some(*pos);
            }
        } else if let Some(base_instance) = instance.strip_suffix("[0]") {
            // lut4[0].in -> lut4.in
            let alt_key = format!("{}.{}", base_instance, port);
            if let Some(pos) = children_ports.get(&alt_key) {
                return Some(*pos);
            }
        }
    }

    None
}

// "in" -> tries "in[0]", "in[1]", ... until "in[4]"
fn resolve_bus_list(
    port_list: &[String],
    current_pb_name: &str,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
) -> Vec<String> {
    let mut resolved = Vec::new();
    for port_ref in port_list {
        if resolve_port_pos(port_ref, current_pb_name, my_ports, children_ports).is_some() {
            resolved.push(port_ref.clone());
            continue;
        }

        let mut idx = 0;
        loop {
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

// "fle[3:0].out" -> ["fle[3].out", "fle[2].out", ...]
// "lut5[0:0].in[4:0]" -> ["lut5[0].in[4]", "lut5[0].in[3]", ..., "lut5[0].in[0]"]
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
                            break;
                        }
                    }
                }
                start_search = abs_close + 1;
            } else {
                break;
            }
        }

        if !expanded {
            i += 1;
        }
    }
    parts
}

fn draw_pb_type(
    painter: &egui::Painter,
    pb_type: &PBType,
    pos: egui::Pos2,
    state: &mut IntraTileState,
    instance_path: &str,
    ui: &mut egui::Ui,
    expand_all: bool,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let size = measure_pb_type(pb_type, state, instance_path);
    let rect = egui::Rect::from_min_size(pos, size);

    let mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);
    let children = get_children_for_mode(pb_type, mode_index);

    let has_children = !children.is_empty();
    let is_expanded = state.expanded_blocks.contains(instance_path);

    // Draw header with expand/collapse indicator
    let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT));

    // Make header clickable if it has children
    if has_children {
        let header_response = ui.allocate_ui_at_rect(header_rect, |ui| {
            ui.allocate_response(header_rect.size(), egui::Sense::click())
        });

        if header_response.inner.clicked() {
            if is_expanded {
                state.expanded_blocks.remove(instance_path);
            } else {
                state.expanded_blocks.insert(instance_path.to_string());
            }
        }
    }

    // If collapsed, only draw header and return empty port map
    if !is_expanded && has_children {
        // Draw just the header background
        painter.rect(
            header_rect,
            egui::Rounding::ZERO,
            color_scheme::theme_header_bg(dark_mode),
            egui::Stroke::NONE,
        );

        // Draw block name in header
        let name_x = if has_children {
            header_rect.min.x + 25.0
        } else {
            header_rect.min.x + 5.0
        };
        let font = egui::FontId::proportional(14.0);
        painter.text(
            egui::pos2(name_x, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &pb_type.name,
            font,
            color_scheme::theme_text_color(dark_mode),
        );

        // Draw expand/collapse indicator on top
        if has_children {
            draw_expand_indicator(painter, header_rect, dark_mode);
        }

        return HashMap::new();
    }

    // Determine specific visual style based on class
    let my_ports = match pb_type.class {
        PBTypeClass::Lut => {
            intra_block_drawing::draw_lut(painter, rect, pb_type, state, ui, dark_mode)
        }
        PBTypeClass::FlipFlop => {
            intra_block_drawing::draw_flip_flop(painter, rect, pb_type, state, ui, dark_mode)
        }
        PBTypeClass::Memory => {
            intra_block_drawing::draw_memory(painter, rect, pb_type, state, ui, dark_mode)
        }
        PBTypeClass::None => {
            if pb_type.blif_model.is_some() {
                intra_block_drawing::draw_blif_block(painter, rect, pb_type, state, ui, dark_mode)
            } else {
                intra_block_drawing::draw_generic_block(
                    painter, rect, pb_type, state, ui, dark_mode,
                )
            }
        }
    };

    // Draw collapse indicator
    if has_children && !is_expanded {
        draw_expand_indicator(painter, header_rect, dark_mode);
    }

    if pb_type.modes.len() > 1 {
        let mode_idx = *state.selected_modes.get(instance_path).unwrap_or(&0);
        let mode_name = &pb_type.modes[mode_idx].name;

        // Truncate mode name if it's too long
        let selector_width = (120.0_f32).min(rect.width() * 0.4);
        let display_name = if mode_name.len() > 15 {
            format!("{}...", &mode_name[..12])
        } else {
            mode_name.clone()
        };

        let selector_height = 18.0;
        let margin = 5.0;
        let selector_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(rect.width() - selector_width - margin, 2.0),
            egui::vec2(selector_width, selector_height),
        );

        let mut selected_mode = mode_idx;

        ui.put(selector_rect, |ui: &mut egui::Ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
            egui::ComboBox::from_id_source(format!("mode_sel_{}", instance_path))
                .width(selector_width)
                .selected_text(&display_name)
                .show_ui(ui, |ui| {
                    for (i, mode) in pb_type.modes.iter().enumerate() {
                        let item_text = if mode.name.len() > 40 {
                            format!("{}...", &mode.name[..37])
                        } else {
                            mode.name.clone()
                        };
                        ui.selectable_value(&mut selected_mode, i, &item_text);
                    }
                })
                .response
        });

        if selected_mode != mode_idx {
            state
                .selected_modes
                .insert(instance_path.to_string(), selected_mode);
            // If expand_all is enabled, re-expand when switching modes
            if expand_all {
                expand_all_blocks(state, pb_type, instance_path);
            }
        }
    }

    let mut children_ports: HashMap<String, egui::Pos2> = HashMap::new();

    if has_children && is_expanded {
        let direction = get_layout_direction(children);

        let start_x = rect.min.x + PADDING;
        let start_y = rect.min.y + HEADER_HEIGHT + PADDING;

        let mut cursor_x = start_x;
        let mut cursor_y = start_y;

        for child_pb in children {
            let mut max_col_width: f32 = 0.0;
            for i in 0..child_pb.num_pb {
                let instance_name = generate_child_instance_name(child_pb, i as usize);
                let child_path = format!("{}.{}", instance_path, instance_name);

                let child_single_size = measure_pb_type(child_pb, state, &child_path);
                max_col_width = max_col_width.max(child_single_size.x);

                let pos = egui::pos2(cursor_x, cursor_y);
                let child_map = draw_pb_type(
                    painter,
                    child_pb,
                    pos,
                    state,
                    &child_path,
                    ui,
                    expand_all,
                    dark_mode,
                );

                for (port_name, p) in child_map {
                    children_ports.insert(format!("{}.{}", instance_name, port_name), p);
                }

                cursor_y += child_single_size.y + PADDING;
            }

            match direction {
                LayoutDirection::Vertical => {}
                LayoutDirection::Horizontal => {
                    cursor_x += max_col_width + PADDING;
                    cursor_y = start_y;
                }
            }
        }
    }

    if has_children && is_expanded {
        let interconnects = get_interconnects_for_mode(pb_type, mode_index);

        for inter in interconnects {
            let (input_str, output_str, kind) = match inter {
                fpga_arch_parser::Interconnect::Direct(d) => (&d.input, &d.output, "direct"),
                fpga_arch_parser::Interconnect::Mux(m) => (&m.input, &m.output, "mux"),
                fpga_arch_parser::Interconnect::Complete(c) => (&c.input, &c.output, "complete"),
            };

            let raw_sources = expand_port_list(input_str);
            let raw_sinks = expand_port_list(output_str);
            let sources = resolve_bus_list(&raw_sources, &pb_type.name, &my_ports, &children_ports);
            let sinks = resolve_bus_list(&raw_sinks, &pb_type.name, &my_ports, &children_ports);

            if kind == "direct" || kind == "complete" {
                for (i, src) in sources.iter().enumerate() {
                    if i < sinks.len() {
                        let dst = &sinks[i];
                        draw_direct_connection(
                            painter,
                            src,
                            dst,
                            pb_type,
                            &my_ports,
                            &children_ports,
                            state,
                            ui,
                            rect,
                            dark_mode,
                        );
                    }
                }
            } else {
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
                    dark_mode,
                );
            }
        }
    }

    my_ports
}

//-----------------------------------------------------------
// Draw Wiring
//-----------------------------------------------------------
fn draw_wire_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
    parent_rect: egui::Rect,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    is_clock: bool,
) {
    let mut points = Vec::new();
    points.push(start);

    // Clock routing: simple patterns for clock connections
    if is_clock {
        if start.y == end.y {
            // Same Y level - direct horizontal routing
            points.push(end);
        } else if start.y < end.y {
            // Vertical routing for clock: A is above B
            if start.x > end.x {
                // A is on the right of B: down → left → down
                let mid_y = start.y + (end.y - start.y) / 2.0;
                points.push(egui::pos2(start.x, mid_y));
                points.push(egui::pos2(end.x, mid_y));
            } else {
                // A is on the left of B: down → right → down
                let mid_y = start.y + (end.y - start.y) / 2.0;
                points.push(egui::pos2(start.x, mid_y));
                points.push(egui::pos2(end.x, mid_y));
            }
            points.push(end);
        } else {
            if start.x > end.x {
                // A is on the right of B: up → left → up
                let mid_y = start.y + (end.y - start.y) / 2.0;
                points.push(egui::pos2(start.x, mid_y));
                points.push(egui::pos2(end.x, mid_y));
            } else {
                // A is on the left of B: up → right → up
                let mid_y = start.y + (end.y - start.y) / 2.0;
                points.push(egui::pos2(start.x, mid_y));
                points.push(egui::pos2(end.x, mid_y));
            }
            points.push(end);
        }
    } else if start.x < end.x {
        let dist = end.x - start.x;
        let is_parent_output = end.x >= parent_rect.max.x - 20.0;

        if dist > 100.0 && !is_parent_output {
            let channel_x_start = start.x + 10.0;
            let channel_x_end = end.x - 10.0;
            let route_y = parent_rect.min.y + 40.0;

            points.push(egui::pos2(channel_x_start, start.y));
            points.push(egui::pos2(channel_x_start, route_y));
            points.push(egui::pos2(channel_x_end, route_y));
            points.push(egui::pos2(channel_x_end, end.y));
        } else {
            let mid_x = start.x + dist / 2.0;
            points.push(egui::pos2(mid_x, start.y));
            points.push(egui::pos2(mid_x, end.y));
        }
    } else {
        let channel_out = start.x + 10.0;
        let channel_in = end.x - 10.0;
        let mid_y = start.y + (end.y - start.y) / 2.0;

        points.push(egui::pos2(channel_out, start.y));
        points.push(egui::pos2(channel_out, mid_y));
        points.push(egui::pos2(channel_in, mid_y));
        points.push(egui::pos2(channel_in, end.y));
    }
    points.push(end);

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
            state.highlighted_positions_next_frame.push(start);
            state.highlighted_positions_next_frame.push(end);
        }
    }

    painter.add(egui::Shape::line(points, stroke));
}

fn draw_direct_connection(
    painter: &egui::Painter,
    src: &str,
    dst: &str,
    current_pb: &PBType,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    parent_rect: egui::Rect,
    dark_mode: bool,
) {
    let src_pos = resolve_port_pos(src, &current_pb.name, my_ports, children_ports);
    let dst_pos = resolve_port_pos(dst, &current_pb.name, my_ports, children_ports);

    if let (Some(start), Some(end)) = (src_pos, dst_pos) {
        let is_highlighted = state
            .highlighted_positions_this_frame
            .iter()
            .any(|p| p.distance(start) < 1.0)
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(end) < 1.0);

        let stroke_color = if is_highlighted {
            color_scheme::HIGHLIGHT_COLOR
        } else {
            color_scheme::theme_interconnect_bg(dark_mode)
        };

        let stroke_width = if is_highlighted { 2.5 } else { 1.5 };
        let stroke = egui::Stroke::new(stroke_width, stroke_color);

        // Check if this is a clock connection by examining port names
        let is_clock = src.to_lowercase().contains("clk")
            || src.to_lowercase().contains("clock")
            || dst.to_lowercase().contains("clk")
            || dst.to_lowercase().contains("clock");
        draw_wire_segment(
            painter,
            start,
            end,
            stroke,
            parent_rect,
            state,
            ui,
            is_clock,
        );
    }
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
    dark_mode: bool,
) {
    let mut valid_sinks = Vec::new();
    for dst in sinks {
        if let Some(pos) = resolve_port_pos(dst, &current_pb.name, my_ports, children_ports) {
            valid_sinks.push((dst, pos));
        }
    }

    if valid_sinks.is_empty() {
        return;
    }

    let avg_y: f32 = valid_sinks.iter().map(|(_, p)| p.y).sum::<f32>() / valid_sinks.len() as f32;
    let min_x: f32 = valid_sinks
        .iter()
        .map(|(_, p)| p.x)
        .fold(f32::INFINITY, |a, b| a.min(b));

    let width = 35.0;

    let max_sink_y = valid_sinks
        .iter()
        .map(|(_, p)| p.y)
        .fold(f32::NEG_INFINITY, |a, b| a.max(b));
    let min_sink_y = valid_sinks
        .iter()
        .map(|(_, p)| p.y)
        .fold(f32::INFINITY, |a, b| a.min(b));

    let sink_spread = max_sink_y - min_sink_y;

    let input_spacing = 15.0;
    let input_spread = (sources.len() as f32 + 1.0) * input_spacing;

    let spread = sink_spread.max(input_spread);
    let height = (spread + 20.0).max(40.0).min(150.0);

    let block_center = egui::pos2(min_x - 50.0, avg_y);
    let rect = egui::Rect::from_center_size(block_center, egui::vec2(width, height));

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
        color_scheme::HIGHLIGHT_COLOR
    } else {
        color_scheme::theme_border_color(dark_mode)
    };
    let stroke = egui::Stroke::new(1.5, stroke_color);
    let fill_color = color_scheme::theme_block_bg(dark_mode);

    if kind == "mux" {
        // trapezoid
        let w = rect.width();
        let h = rect.height();
        let c = rect.center();
        let trap_points = vec![
            c + egui::vec2(-w / 2.0, -h / 2.0),
            c + egui::vec2(w / 2.0, -h / 4.0),
            c + egui::vec2(w / 2.0, h / 4.0),
            c + egui::vec2(-w / 2.0, h / 2.0),
        ];
        painter.add(egui::Shape::convex_polygon(trap_points, fill_color, stroke));
    } else {
        // rectangle with X
        painter.rect(rect, 2.0, fill_color, stroke);
        painter.line_segment([rect.min, rect.max], stroke);
        painter.line_segment(
            [
                egui::pos2(rect.min.x, rect.max.y),
                egui::pos2(rect.max.x, rect.min.y),
            ],
            stroke,
        );
    }

    let left_edge_x = rect.min.x;
    let mut resolved_sources = Vec::new();
    for src in sources {
        if let Some(src_pos) = resolve_port_pos(src, &current_pb.name, my_ports, children_ports) {
            resolved_sources.push((src, src_pos));
        }
    }
    // We sort by x coordinate, leftmost one will be at the top of the mux
    resolved_sources.sort_by(|a, b| {
        a.1.x
            .partial_cmp(&b.1.x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let input_step = rect.height() / (resolved_sources.len() as f32 + 1.0);

    for (i, (src_name, src_pos)) in resolved_sources.iter().enumerate() {
        let wire_highlighted = is_block_highlighted
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(*src_pos) < 1.0);
        let wire_color = if wire_highlighted {
            color_scheme::HIGHLIGHT_COLOR
        } else {
            color_scheme::theme_interconnect_bg(dark_mode)
        };
        let wire_stroke = egui::Stroke::new(1.5, wire_color);

        let input_y = rect.min.y + input_step * (i as f32 + 1.0);
        let target = egui::pos2(left_edge_x, input_y);

        // Check if this is a clock connection by examining source port name
        let is_clock =
            src_name.to_lowercase().contains("clk") || src_name.to_lowercase().contains("clock");
        draw_wire_segment(
            painter,
            *src_pos,
            target,
            wire_stroke,
            parent_rect,
            state,
            ui,
            is_clock,
        );

        if ui.rect_contains_pointer(rect) {
            state.highlighted_positions_next_frame.push(*src_pos);
        }
    }

    let right_edge_x = rect.max.x;
    for (dst_name, dst_pos) in valid_sinks {
        let wire_highlighted = is_block_highlighted
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(dst_pos) < 1.0);
        let wire_color = if wire_highlighted {
            color_scheme::HIGHLIGHT_COLOR
        } else {
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100)
        };
        let wire_stroke = egui::Stroke::new(1.5, wire_color);

        let start = egui::pos2(right_edge_x, block_center.y);
        // Check if this is a clock connection by examining sink port name
        let is_clock =
            dst_name.to_lowercase().contains("clk") || dst_name.to_lowercase().contains("clock");
        draw_wire_segment(
            painter,
            start,
            dst_pos,
            wire_stroke,
            parent_rect,
            state,
            ui,
            is_clock,
        );

        if ui.rect_contains_pointer(rect) {
            state.highlighted_positions_next_frame.push(dst_pos);
        }
    }
}
