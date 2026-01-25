//! Intra Tile Visualization
//!
//! Part of the FPGA Visualizer, this module renders the intra-tile view of an FPGA tile.

use eframe::egui;
use fpga_arch_parser::{FPGAArch, PBType, PBTypeClass, Port, Tile};
use std::collections::{HashMap, HashSet};

use crate::color_scheme;
use crate::intra_block_drawing;
use crate::intra_hierarchy_tree;

// ------------------------------------------------------------
// Constants
// ------------------------------------------------------------
const PADDING: f32 = 50.0;
const HEADER_HEIGHT: f32 = 35.0;
const MIN_BLOCK_SIZE: egui::Vec2 = egui::vec2(80.0, 120.0);
const MIN_PIN_SPACING: f32 = 25.0;
// Extra gutter between a parent PB and its children when mux blocks are present.
// This gives muxes space to sit without crowding child pins (notably in z1010 CLBs).
const MUX_GUTTER: f32 = 70.0;
// Only enable the mux gutter for "dense" mux regions; a single mux (e.g. k4_N4 mux1)
// shouldn't push children around and change local routing aesthetics.
const MUX_GUTTER_MIN_MUXES: usize = 4;

// ------------------------------------------------------------
// Intra Tile Drawing Entry Point
// ------------------------------------------------------------
pub struct IntraTileState {
    pub selected_modes: HashMap<String, usize>,
    pub highlighted_positions_this_frame: Vec<egui::Pos2>,
    pub highlighted_positions_next_frame: Vec<egui::Pos2>,
    pub hierarchy_tree_height: Option<f32>,
    pub expanded_blocks: HashSet<String>,
    pub pb_rects: HashMap<String, egui::Rect>,
    /// Zoom factor for the intra-tile canvas (1.0 = 100%).
    pub zoom: f32,
    // Cache for PBType measurements: (instance_path, is_expanded, mode_index) -> size
    measurement_cache: HashMap<(String, bool, usize), egui::Vec2>,
}

impl Default for IntraTileState {
    fn default() -> Self {
        Self {
            selected_modes: HashMap::new(),
            highlighted_positions_this_frame: Vec::new(),
            highlighted_positions_next_frame: Vec::new(),
            hierarchy_tree_height: None,
            expanded_blocks: HashSet::new(),
            pb_rects: HashMap::new(),
            zoom: 1.0,
            measurement_cache: HashMap::new(),
        }
    }
}

impl IntraTileState {
    pub(crate) fn zoom_clamped(&self) -> f32 {
        self.zoom.clamp(0.2, 4.0)
    }
}

fn apply_local_zoom_style(ui: &mut egui::Ui, zoom: f32) -> std::sync::Arc<egui::Style> {
    let old = ui.style().clone(); // Arc<Style>
    if (zoom - 1.0).abs() < f32::EPSILON {
        return old;
    }

    let mut style: egui::Style = (*old).clone();

    // Scale fonts used by widgets (ComboBox, labels, etc.) inside this UI scope.
    style.text_styles = style
        .text_styles
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                egui::FontId::new(v.size * zoom, v.family.clone()),
            )
        })
        .collect();

    // Scale a few key spacing values so the widget chrome scales too.
    style.spacing.item_spacing *= zoom;
    style.spacing.button_padding *= zoom;
    style.spacing.interact_size *= zoom;
    style.spacing.icon_width *= zoom;
    style.spacing.icon_width_inner *= zoom;
    style.spacing.icon_spacing *= zoom;

    ui.set_style(std::sync::Arc::new(style));
    old
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
    let default_tree_height = (available_height * 0.3).clamp(100.0, 400.0);
    let tree_height = state.hierarchy_tree_height.unwrap_or(default_tree_height);

    let min_tree_height = 50.0;
    let max_tree_height = available_height - 100.0;
    let tree_height = tree_height.clamp(min_tree_height, max_tree_height);

    // Allocate space for hierarchy tree
    let tree_rect = egui::Rect::from_min_size(
        available_rect.min,
        egui::vec2(available_rect.width(), tree_height),
    );
    let _tree_response = ui.scope_builder(egui::UiBuilder::new().max_rect(tree_rect), |ui| {
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
    let separator_response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(separator_rect), |ui| {
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
    draw_interconnects: bool,
    dark_mode: bool,
) {
    egui::ScrollArea::both()
        .id_salt("intra_tile_canvas")
        .auto_shrink([false, false])
        // Enable "click + drag" panning within the canvas area.
        // This remains confined to the ScrollArea viewport, so it won't overlap other UI panels.
        .show(ui, |ui| {
            // Canvas-local zoom controls (only active when pointer is over this viewport):
            // - Ctrl/Cmd + mouse wheel
            // - Trackpad pinch zoom
            let zoom_viewport = ui.clip_rect();
            let pointer_pos = ui.ctx().pointer_latest_pos();
            let pointer_over_canvas = pointer_pos
                .map(|p| zoom_viewport.contains(p))
                .unwrap_or(false);

            let (cmd_or_ctrl, scroll_y, pinch_zoom) = ui.input(|i| {
                let cmd_or_ctrl = i.modifiers.command || i.modifiers.ctrl;
                let scroll_y = i.raw_scroll_delta.y;
                let pinch_zoom = i.zoom_delta();
                (cmd_or_ctrl, scroll_y, pinch_zoom)
            });

            if pointer_over_canvas {
                let mut z = state.zoom_clamped();
                let mut changed = false;

                // Ctrl/Cmd + wheel: treat scroll as zoom steps.
                if cmd_or_ctrl && scroll_y.abs() > 0.0 {
                    let steps = (scroll_y / 200.0).clamp(-5.0, 5.0);
                    z = (z * 1.1_f32.powf(steps)).clamp(0.2, 4.0);
                    changed = true;
                }

                // Trackpad pinch zoom.
                if pinch_zoom != 1.0 {
                    z = (z * pinch_zoom).clamp(0.2, 4.0);
                    changed = true;
                }

                if changed {
                    state.zoom = z;
                }
            }

            if sub_tile_index < tile.sub_tiles.len() {
                let sub_tile = &tile.sub_tiles[sub_tile_index];
                if let Some(site) = sub_tile.equivalent_sites.first() {
                    if let Some(root_pb) = arch
                        .complex_block_list
                        .iter()
                        .find(|pb| pb.name == site.pb_type)
                    {
                        // Draw pbtype here
                        let zoom = state.zoom_clamped();
                        let total_size = measure_pb_type(root_pb, state, &root_pb.name);
                        let (response, painter) = ui.allocate_painter(
                            total_size + egui::vec2(40.0, 40.0) * zoom,
                            // Important: don't capture drags here, otherwise it prevents the
                            // ScrollArea from receiving drag-to-pan gestures.
                            egui::Sense::hover(),
                        );
                        let start_pos = response.rect.min + egui::vec2(20.0, 20.0) * zoom;

                        let _ = draw_pb_type(
                            &painter,
                            root_pb,
                            start_pos,
                            state,
                            &root_pb.name,
                            ui,
                            expand_all,
                            draw_interconnects,
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

fn render_visual_layout_controls(ui: &mut egui::Ui, state: &mut IntraTileState) {
    ui.horizontal(|ui| {
        ui.label("Zoom:");
        if ui.small_button("−").clicked() {
            state.zoom = (state.zoom_clamped() / 1.1).max(0.2);
        }
        ui.label(format!("{:.0}%", state.zoom_clamped() * 100.0));
        if ui.small_button("+").clicked() {
            state.zoom = (state.zoom_clamped() * 1.1).min(4.0);
        }
        if ui.small_button("Reset").clicked() {
            state.zoom = 1.0;
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
    draw_interconnects: bool,
    dark_mode: bool,
) {
    // Clear per-frame PB rects before drawing
    state.pb_rects.clear();
    // Clear measurement cache at start of each frame to ensure fresh calculations
    // when expanded_blocks or selected_modes change
    state.measurement_cache.clear();

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
        ui.scope_builder(egui::UiBuilder::new().max_rect(layout_rect), |ui| {
            ui.set_width(available_rect.width());
            ui.heading("Visual Layout");
            render_visual_layout_controls(ui, state);
            render_visual_layout_canvas(
                ui,
                arch,
                tile,
                state,
                sub_tile_index,
                expand_all,
                draw_interconnects,
                dark_mode,
            );
        });
    } else {
        ui.set_width(available_rect.width());
        ui.heading("Visual Layout");
        render_visual_layout_controls(ui, state);
        render_visual_layout_canvas(
            ui,
            arch,
            tile,
            state,
            sub_tile_index,
            expand_all,
            draw_interconnects,
            dark_mode,
        );
    }
}

// ------------------------------------------------------------
// Expand Block Feature
// ------------------------------------------------------------
pub fn expand_all_blocks(state: &mut IntraTileState, pb_type: &PBType, instance_path: &str) {
    state.expanded_blocks.insert(instance_path.to_string());

    let mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);
    let mode_index = validate_mode_index(pb_type, mode_index);
    // Update state with validated mode index if it was corrected
    if mode_index != *state.selected_modes.get(instance_path).unwrap_or(&0) {
        state
            .selected_modes
            .insert(instance_path.to_string(), mode_index);
    }
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
fn calculate_header_name_width(pb_type: &PBType, has_children: bool, zoom: f32) -> f32 {
    let font = egui::FontId::proportional(14.0 * zoom);
    let name_width = estimate_text_width(&font, &pb_type.name);
    let header_name_width = if has_children {
        name_width + 25.0 * zoom // Expand indicator space
    } else {
        name_width + 5.0 * zoom // Just margin
    };
    if !pb_type.modes.is_empty() {
        header_name_width + 130.0 * zoom // Mode selector space
    } else {
        header_name_width
    }
}

/// Calculates the width needed for blif_model name.
fn calculate_blif_model_width(pb_type: &PBType, zoom: f32) -> f32 {
    match pb_type.class {
        PBTypeClass::None => {
            if let Some(blif_model) = &pb_type.blif_model {
                let blif_font = egui::FontId::monospace(14.0 * zoom);
                estimate_text_width(&blif_font, blif_model) + 20.0 * zoom
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

/// Validates and corrects a mode index for a PBType, ensuring it's within bounds.
/// Returns a valid mode index (defaults to 0 if out of bounds).
fn validate_mode_index(pb_type: &PBType, mode_index: usize) -> usize {
    if pb_type.modes.is_empty() {
        0 // No modes, index doesn't matter
    } else if mode_index < pb_type.modes.len() {
        mode_index
    } else {
        0 // Out of bounds, default to first mode
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

fn measure_pb_type(
    pb_type: &PBType,
    state: &mut IntraTileState,
    instance_path: &str,
) -> egui::Vec2 {
    let zoom = state.zoom_clamped();
    let is_expanded = state.expanded_blocks.contains(instance_path);
    let mut mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);
    mode_index = validate_mode_index(pb_type, mode_index);
    // Update state with validated mode index if it was corrected
    if mode_index != *state.selected_modes.get(instance_path).unwrap_or(&0) {
        state
            .selected_modes
            .insert(instance_path.to_string(), mode_index);
    }

    // Check cache first
    let cache_key = (instance_path.to_string(), is_expanded, mode_index);
    if let Some(cached_size) = state.measurement_cache.get(&cache_key) {
        return *cached_size;
    }

    let children = get_children_for_mode(pb_type, mode_index);

    if !is_expanded && !children.is_empty() {
        let header_name_width_with_selector = calculate_header_name_width(pb_type, true, zoom);
        let blif_model_width = calculate_blif_model_width(pb_type, zoom);

        let min_width = (MIN_BLOCK_SIZE.x * zoom)
            .max(header_name_width_with_selector)
            .max(blif_model_width);
        return egui::vec2(min_width, HEADER_HEIGHT * zoom);
    }

    if children.is_empty() {
        let total_input_pins = count_pins(pb_type, PortType::Input);
        let total_output_pins = count_pins(pb_type, PortType::Output);
        let total_clock_pins = count_pins(pb_type, PortType::Clock);

        let max_side_pins = total_input_pins.max(total_output_pins) as f32;
        let min_height_for_pins = if max_side_pins > 0.0 {
            (max_side_pins + 1.0) * (MIN_PIN_SPACING * zoom)
        } else {
            0.0
        };

        let min_width_for_clock = if total_clock_pins > 0 {
            (total_clock_pins as f32 + 1.0) * (MIN_PIN_SPACING * zoom)
        } else {
            0.0
        };

        let header_name_width_with_selector = calculate_header_name_width(pb_type, false, zoom);
        let blif_model_width = calculate_blif_model_width(pb_type, zoom);

        let required_height =
            ((HEADER_HEIGHT * zoom) + min_height_for_pins).max(MIN_BLOCK_SIZE.y * zoom);
        let required_width = min_width_for_clock
            .max(MIN_BLOCK_SIZE.x * zoom)
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

                let total_instances_h = max_instance_size.y * num + (PADDING * zoom) * gaps;

                max_child_w = max_child_w.max(max_instance_size.x);
                current_h += total_instances_h + (PADDING * zoom);
            }
            if !children.is_empty() {
                current_h -= PADDING * zoom;
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

                let child_instances_h = max_instance_size.y * num + (PADDING * zoom) * gaps;
                let child_instances_w = max_instance_size.x;

                max_child_h = max_child_h.max(child_instances_h);
                current_w += child_instances_w + (PADDING * zoom);
            }

            if !children.is_empty() {
                current_w -= PADDING * zoom;
            }

            total_w = current_w;
            total_h = max_child_h;
        }
    }

    let total_input_pins = count_pins(pb_type, PortType::Input);
    let total_output_pins = count_pins(pb_type, PortType::Output);
    let max_pins = total_input_pins.max(total_output_pins) as f32;
    let min_port_height = (max_pins + 1.0) * (MIN_PIN_SPACING * zoom);

    let has_complete_interconnect = pb_type
        .interconnects
        .iter()
        .any(|i| matches!(i, fpga_arch_parser::Interconnect::Complete(_)));

    let is_clock_complete = pb_type
        .interconnects
        .iter()
        .any(|i| is_clock_complete_interconnect(i, pb_type, children));

    let mux_count = get_interconnects_for_mode(pb_type, mode_index)
        .iter()
        .filter(|i| matches!(i, fpga_arch_parser::Interconnect::Mux(_)))
        .count();
    let mux_gutter = if mux_count >= MUX_GUTTER_MIN_MUXES {
        MUX_GUTTER * zoom
    } else {
        0.0
    };

    let complete_spacing = if has_complete_interconnect && !is_clock_complete {
        180.0 * zoom
    } else {
        0.0
    };
    // Extra right-side padding only for clock completes; does not shift children.
    let clock_padding_right = if is_clock_complete { 140.0 * zoom } else { 0.0 };

    let interconnect_width = if !pb_type.modes.is_empty() || !pb_type.interconnects.is_empty() {
        if has_complete_interconnect {
            140.0 * zoom
        } else {
            80.0 * zoom
        }
    } else {
        0.0
    };

    let header_name_width_with_selector =
        calculate_header_name_width(pb_type, !children.is_empty(), zoom);
    let blif_model_width = calculate_blif_model_width(pb_type, zoom);

    let w = (total_w + (PADDING * zoom) * 2.0 + interconnect_width + clock_padding_right)
        .max(MIN_BLOCK_SIZE.x * zoom)
        .max(header_name_width_with_selector)
        .max(blif_model_width)
        .max(total_w + complete_spacing + mux_gutter + (PADDING * zoom) * 2.0)
        .max(total_w + mux_gutter + (PADDING * zoom) * 2.0);
    let h = ((HEADER_HEIGHT * zoom) + (PADDING * zoom) + total_h + (PADDING * zoom))
        .max(MIN_BLOCK_SIZE.y * zoom)
        .max(min_port_height);
    let size = egui::vec2(w, h);

    // Cache the result
    state.measurement_cache.insert(cache_key, size);
    size
}

// ------------------------------------------------------------
// Drawing System
// ------------------------------------------------------------

/// Draws the expand indicator (▶) in the header.
fn draw_expand_indicator(
    painter: &egui::Painter,
    header_rect: egui::Rect,
    zoom: f32,
    dark_mode: bool,
) {
    let indicator_x = header_rect.min.x + 8.0 * zoom;
    let indicator_y = header_rect.center().y;
    painter.text(
        egui::pos2(indicator_x, indicator_y),
        egui::Align2::LEFT_CENTER,
        "▶",
        egui::FontId::proportional(12.0 * zoom),
        color_scheme::theme_text_color(dark_mode),
    );
}

/// Checks if a port reference refers to a clock port.
/// Handles both current PBType ports and child instance ports.
/// Extracts the port name from references like "clk[0]", "clb.clk[0]", or "fle[0].clk[0]".
fn is_clock_port(port_ref: &str, current_pb: &PBType, children: &[PBType]) -> bool {
    use fpga_arch_parser::{Port, PortClass};

    // Extract port name and instance from various formats:
    // "clk[0]" -> (None, "clk")
    // "clb.clk[0]" -> (Some("clb"), "clk")
    // "fle[0].clk[0]" -> (Some("fle[0]"), "clk")
    let (instance_prefix, port_name) = if let Some(dot_idx) = port_ref.rfind('.') {
        // Has instance prefix
        let instance_part = &port_ref[..dot_idx];
        let port_part = &port_ref[dot_idx + 1..];
        let port_name = if let Some(bracket_idx) = port_part.find('[') {
            &port_part[..bracket_idx]
        } else {
            port_part
        };
        (Some(instance_part), port_name)
    } else if let Some(bracket_idx) = port_ref.find('[') {
        // Just port name with index
        (None, &port_ref[..bracket_idx])
    } else {
        (None, port_ref)
    };

    // If there's an instance prefix, check child PBTypes
    if let Some(instance) = instance_prefix {
        // Extract child name (e.g., "fle" from "fle[0]")
        let child_name = if let Some(bracket_idx) = instance.find('[') {
            &instance[..bracket_idx]
        } else {
            instance
        };

        // Find matching child PBType
        for child_pb in children {
            if child_pb.name == child_name {
                // Check ports in this child PBType
                for port in &child_pb.ports {
                    match port {
                        Port::Clock(c) => {
                            if c.name == port_name {
                                return true;
                            }
                        }
                        Port::Input(p) => {
                            if p.name == port_name {
                                // Check port_class first, then fall back to name matching for backward compatibility
                                if matches!(p.port_class, PortClass::Clock) {
                                    return true;
                                }
                                // Fallback: check if port name contains "clk" or "clock"
                                let name_lower = p.name.to_lowercase();
                                if name_lower.contains("clk") || name_lower.contains("clock") {
                                    return true;
                                }
                            }
                        }
                        Port::Output(p) => {
                            if p.name == port_name {
                                // Check port_class first, then fall back to name matching for backward compatibility
                                if matches!(p.port_class, PortClass::Clock) {
                                    return true;
                                }
                                // Fallback: check if port name contains "clk" or "clock"
                                let name_lower = p.name.to_lowercase();
                                if name_lower.contains("clk") || name_lower.contains("clock") {
                                    return true;
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    } else {
        // Check current PBType ports
        for port in &current_pb.ports {
            match port {
                Port::Clock(c) => {
                    if c.name == port_name {
                        return true;
                    }
                }
                Port::Input(p) => {
                    if p.name == port_name {
                        // Check port_class first, then fall back to name matching for backward compatibility
                        if matches!(p.port_class, PortClass::Clock) {
                            return true;
                        }
                        // Fallback: check if port name contains "clk" or "clock"
                        let name_lower = p.name.to_lowercase();
                        if name_lower.contains("clk") || name_lower.contains("clock") {
                            return true;
                        }
                    }
                }
                Port::Output(p) => {
                    if p.name == port_name {
                        // Check port_class first, then fall back to name matching for backward compatibility
                        if matches!(p.port_class, PortClass::Clock) {
                            return true;
                        }
                        // Fallback: check if port name contains "clk" or "clock"
                        let name_lower = p.name.to_lowercase();
                        if name_lower.contains("clk") || name_lower.contains("clock") {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Final fallback: if port name itself contains "clk" or "clock", treat as clock
    // This handles cases where the port might not be found in the PBType but the name suggests it's a clock
    let port_name_lower = port_name.to_lowercase();
    if port_name_lower.contains("clk") || port_name_lower.contains("clock") {
        return true;
    }

    false
}

/// Checks if a Complete interconnect is a clock interconnect by verifying
/// that all ports in both input and output lists are clock ports.
fn is_clock_complete_interconnect(
    interconnect: &fpga_arch_parser::Interconnect,
    current_pb: &PBType,
    children: &[PBType],
) -> bool {
    if let fpga_arch_parser::Interconnect::Complete(c) = interconnect {
        // Expand port lists to get individual port references
        let raw_inputs = expand_port_list(&c.input);
        let raw_outputs = expand_port_list(&c.output);

        // Check if all input ports are clock ports
        let all_inputs_clock = raw_inputs
            .iter()
            .all(|port_ref| is_clock_port(port_ref, current_pb, children));

        // Check if all output ports are clock ports
        let all_outputs_clock = raw_outputs
            .iter()
            .all(|port_ref| is_clock_port(port_ref, current_pb, children));

        all_inputs_clock && all_outputs_clock
    } else {
        false
    }
}

fn resolve_port_pos(
    port_ref: &str,
    current_pb_name: &str,
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
) -> Option<egui::Pos2> {
    // clb.I -> I
    if let Some(stripped) = port_ref.strip_prefix(&format!("{}.", current_pb_name))
        && let Some(pos) = my_ports.get(stripped)
    {
        return Some(*pos);
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
        const MAX_BUS_ITERATIONS: usize = 1024; // Prevent infinite loops
        loop {
            if idx >= MAX_BUS_ITERATIONS {
                // Log warning for debugging (could use eprintln! or a logging framework)
                eprintln!(
                    "Warning: Bus resolution hit max iterations ({}) for port: {}",
                    MAX_BUS_ITERATIONS, port_ref
                );
                break;
            }

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

                    if let Some((msb_str, lsb_str)) = content.split_once(':')
                        && let (Ok(msb), Ok(lsb)) = (msb_str.parse::<i32>(), lsb_str.parse::<i32>())
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
    draw_interconnects: bool,
    dark_mode: bool,
) -> HashMap<String, egui::Pos2> {
    let zoom = state.zoom_clamped();
    let size = measure_pb_type(pb_type, state, instance_path);
    let rect = egui::Rect::from_min_size(pos, size);

    // Record this PB's rect for downstream placement (e.g., interconnect boxes)
    state.pb_rects.insert(instance_path.to_string(), rect);
    let mut mode_index = *state.selected_modes.get(instance_path).unwrap_or(&0);
    mode_index = validate_mode_index(pb_type, mode_index);
    // Update state with validated mode index if it was corrected
    if mode_index != *state.selected_modes.get(instance_path).unwrap_or(&0) {
        state
            .selected_modes
            .insert(instance_path.to_string(), mode_index);
    }
    let children = get_children_for_mode(pb_type, mode_index);

    let has_children = !children.is_empty();
    let has_complete_interconnect = pb_type
        .interconnects
        .iter()
        .any(|i| matches!(i, fpga_arch_parser::Interconnect::Complete(_)));
    let _is_clock_complete = pb_type
        .interconnects
        .iter()
        .any(|i| is_clock_complete_interconnect(i, pb_type, children));
    let is_expanded = state.expanded_blocks.contains(instance_path);

    // Draw header with expand/collapse indicator
    let header_rect =
        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT * zoom));

    // Make header clickable if it has children
    if has_children {
        let header_response = ui
            .scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
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
            egui::CornerRadius::ZERO,
            color_scheme::theme_header_bg(dark_mode),
            egui::Stroke::NONE,
            egui::epaint::StrokeKind::Inside,
        );

        // Draw block name in header
        let name_x = if has_children {
            header_rect.min.x + 25.0 * zoom
        } else {
            header_rect.min.x + 5.0 * zoom
        };
        let font = egui::FontId::proportional(14.0 * zoom);
        painter.text(
            egui::pos2(name_x, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &pb_type.name,
            font,
            color_scheme::theme_text_color(dark_mode),
        );

        // Draw expand/collapse indicator on top
        if has_children {
            draw_expand_indicator(painter, header_rect, zoom, dark_mode);
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

    if pb_type.modes.len() > 1 {
        let mut mode_idx = *state.selected_modes.get(instance_path).unwrap_or(&0);
        mode_idx = validate_mode_index(pb_type, mode_idx);
        // Update state with validated mode index if it was corrected
        if mode_idx != *state.selected_modes.get(instance_path).unwrap_or(&0) {
            state
                .selected_modes
                .insert(instance_path.to_string(), mode_idx);
        }
        let mode_name = &pb_type.modes[mode_idx].name;

        // Truncate mode name if it's too long
        let selector_width = (120.0_f32 * zoom).min(rect.width() * 0.4);
        let display_name = if mode_name.len() > 15 {
            format!("{}...", &mode_name[..12])
        } else {
            mode_name.clone()
        };

        let selector_height = 18.0 * zoom;
        let margin = 5.0 * zoom;
        let selector_rect = egui::Rect::from_min_size(
            rect.min + egui::vec2(rect.width() - selector_width - margin, 2.0 * zoom),
            egui::vec2(selector_width, selector_height),
        );

        let mut selected_mode = mode_idx;

        ui.put(selector_rect, |ui: &mut egui::Ui| {
            let old_style = apply_local_zoom_style(ui, zoom);
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
            let response = egui::ComboBox::from_id_salt(format!("mode_sel_{}", instance_path))
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
                .response;
            ui.set_style(old_style);
            response
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

        // If this block has a non-clock complete interconnect, push children
        // rightward to create an intentional gutter. Clock completes keep
        // children aligned; their space is handled on the right at measure time.
        let complete_spacing = if has_complete_interconnect {
            180.0 * zoom
        } else {
            0.0
        };

        let mux_count = get_interconnects_for_mode(pb_type, mode_index)
            .iter()
            .filter(|i| matches!(i, fpga_arch_parser::Interconnect::Mux(_)))
            .count();
        let mux_gutter = if mux_count >= MUX_GUTTER_MIN_MUXES {
            MUX_GUTTER * zoom
        } else {
            0.0
        };

        let start_x = rect.min.x + (PADDING * zoom) + mux_gutter + complete_spacing;
        let start_y = rect.min.y + (HEADER_HEIGHT * zoom) + (PADDING * zoom);

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
                    draw_interconnects,
                    dark_mode,
                );

                for (port_name, p) in child_map {
                    children_ports.insert(format!("{}.{}", instance_name, port_name), p);
                }

                cursor_y += child_single_size.y + (PADDING * zoom);
            }

            match direction {
                LayoutDirection::Vertical => {}
                LayoutDirection::Horizontal => {
                    cursor_x += max_col_width + (PADDING * zoom);
                    cursor_y = start_y;
                }
            }
        }
    }

    if draw_interconnects && has_children && is_expanded {
        let interconnects = get_interconnects_for_mode(pb_type, mode_index);

        for inter in interconnects {
            match inter {
                fpga_arch_parser::Interconnect::Direct(d) => {
                    let raw_sources = expand_port_list(&d.input);
                    let raw_sinks = expand_port_list(&d.output);
                    let mut sources =
                        resolve_bus_list(&raw_sources, &pb_type.name, &my_ports, &children_ports);
                    let mut sinks =
                        resolve_bus_list(&raw_sinks, &pb_type.name, &my_ports, &children_ports);

                    // Align buses by numeric index when both sides are indexed, so
                    // fle[0] maps to O[0], fle[1] to O[1], etc. We try to parse an
                    // outer index (before the dot) first, then an inner bit index.
                    // If either side lacks indices or lengths differ, we keep order.
                    let parse_indices = |s: &str| {
                        // outer like "fle[3].out[0]"
                        let outer = s.find('[').and_then(|start| {
                            s[start + 1..]
                                .find(']')
                                .and_then(|e| s[start + 1..start + 1 + e].parse::<i32>().ok())
                        });
                        // inner like ".out[0]"
                        let inner = s.rfind('[').and_then(|start| {
                            s[start + 1..]
                                .find(']')
                                .and_then(|e| s[start + 1..start + 1 + e].parse::<i32>().ok())
                        });
                        (outer, inner)
                    };

                    let src_all_idx = sources.iter().all(|s| parse_indices(s).0.is_some());
                    let dst_all_idx = sinks.iter().all(|s| parse_indices(s).0.is_some());

                    if src_all_idx && dst_all_idx && sources.len() == sinks.len() {
                        let mut src_with = sources
                            .into_iter()
                            .filter_map(|s| {
                                let (o, i) = parse_indices(&s);
                                Some(((o?, i.unwrap_or(0)), s))
                            })
                            .collect::<Vec<_>>();
                        let mut dst_with = sinks
                            .into_iter()
                            .filter_map(|d| {
                                let (o, i) = parse_indices(&d);
                                Some(((o?, i.unwrap_or(0)), d))
                            })
                            .collect::<Vec<_>>();
                        src_with.sort_by_key(|(k, _)| *k);
                        dst_with.sort_by_key(|(k, _)| *k);
                        sources = src_with.into_iter().map(|(_, s)| s).collect();
                        sinks = dst_with.into_iter().map(|(_, d)| d).collect();
                    }

                    for (i, src) in sources.iter().enumerate() {
                        if i < sinks.len() {
                            let dst = &sinks[i];
                            draw_direct_connection(
                                painter,
                                src,
                                dst,
                                pb_type,
                                children,
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
                fpga_arch_parser::Interconnect::Mux(m) => {
                    let raw_sources = expand_port_list(&m.input);
                    let raw_sinks = expand_port_list(&m.output);
                    let sources =
                        resolve_bus_list(&raw_sources, &pb_type.name, &my_ports, &children_ports);
                    let sinks =
                        resolve_bus_list(&raw_sinks, &pb_type.name, &my_ports, &children_ports);

                    draw_interconnect_block(
                        painter,
                        "mux",
                        &sources,
                        &sinks,
                        pb_type,
                        children,
                        &my_ports,
                        &children_ports,
                        state,
                        ui,
                        rect,
                        dark_mode,
                    );
                }
                fpga_arch_parser::Interconnect::Complete(c) => {
                    let raw_sources = expand_port_list(&c.input);
                    let raw_sinks = expand_port_list(&c.output);
                    let sources =
                        resolve_bus_list(&raw_sources, &pb_type.name, &my_ports, &children_ports);
                    let sinks =
                        resolve_bus_list(&raw_sinks, &pb_type.name, &my_ports, &children_ports);

                    draw_complete_interconnect(
                        painter,
                        &sources,
                        &sinks,
                        pb_type,
                        children,
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
    let zoom = state.zoom_clamped();
    let mut points = Vec::new();
    points.push(start);

    // Clock routing: simple patterns for clock connections
    if is_clock {
        if start.y == end.y {
            // Same Y level - direct horizontal routing
            points.push(end);
        } else {
            // Vertical routing for clock: route through midpoint
            let mid_y = start.y + (end.y - start.y) / 2.0;
            points.push(egui::pos2(start.x, mid_y));
            points.push(egui::pos2(end.x, mid_y));
            points.push(end);
        }
    } else if start.x < end.x {
        let dist = end.x - start.x;
        let is_parent_output = end.x >= parent_rect.max.x - 20.0 * zoom;

        // Scale these thresholds/offsets with zoom so detours remain "above"
        // the same relative region when zooming.
        if dist > 100.0 * zoom && !is_parent_output {
            let channel_x_start = start.x + 10.0 * zoom;
            let channel_x_end = end.x - 10.0 * zoom;
            let route_y = parent_rect.min.y + 40.0 * zoom;

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
        let channel_out = start.x + 10.0 * zoom;
        let channel_in = end.x - 10.0 * zoom;
        let mid_y = start.y + (end.y - start.y) / 2.0;

        points.push(egui::pos2(channel_out, start.y));
        points.push(egui::pos2(channel_out, mid_y));
        points.push(egui::pos2(channel_in, mid_y));
        points.push(egui::pos2(channel_in, end.y));
    }
    // Only push end if it wasn't already pushed (clock routing already includes end)
    if !is_clock {
        points.push(end);
    }

    if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
        let mut hovered = false;
        for i in 0..points.len() - 1 {
            let p1 = points[i];
            let p2 = points[i + 1];
            let pad = 5.0 * zoom;
            let min_x = p1.x.min(p2.x) - pad;
            let max_x = p1.x.max(p2.x) + pad;
            let min_y = p1.y.min(p2.y) - pad;
            let max_y = p1.y.max(p2.y) + pad;

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

fn pb_name_from_port(port: &str) -> Option<&str> {
    port.split('.').next()
}

fn find_pb_rect<'a>(state: &'a IntraTileState, pb_name: &str) -> Option<&'a egui::Rect> {
    state
        .pb_rects
        .iter()
        .find(|(k, _)| k.ends_with(pb_name))
        .map(|(_, r)| r)
}

fn clock_channel_y_between(
    state: &IntraTileState,
    src_pb: &str,
    dst_pb: &str,
    default: f32,
) -> f32 {
    let src_rect = find_pb_rect(state, src_pb);
    let dst_rect = find_pb_rect(state, dst_pb);

    match (src_rect, dst_rect) {
        (Some(sr), Some(dr)) => {
            // Prefer horizontal routing through overlapping vertical band if available.
            let overlap_top = sr.min.y.max(dr.min.y);
            let overlap_bot = sr.max.y.min(dr.max.y);
            if overlap_bot > overlap_top {
                (overlap_top + overlap_bot) * 0.5
            } else {
                // No overlap; route midway between the two rects vertically.
                (sr.max.y + dr.min.y) * 0.5
            }
        }
        _ => default,
    }
}

fn draw_clock_wire_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
    parent_rect: egui::Rect,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    channel_y: f32,
) {
    let _ = parent_rect;
    let points = vec![
        start,
        egui::pos2(start.x, channel_y),
        egui::pos2(end.x, channel_y),
        end,
    ];

    // Minimal hover highlight reuse from draw_wire_segment
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
    children: &[PBType],
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

        // Check if this is a clock connection using port class instead of string matching
        let is_clock =
            is_clock_port(src, current_pb, children) || is_clock_port(dst, current_pb, children);

        // For direct clock links (e.g., ff clk -> ble clk), keep a simple route
        // using the generic wire segment to avoid long detours.
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

fn draw_complete_interconnect(
    painter: &egui::Painter,
    sources: &[String],
    sinks: &[String],
    current_pb: &PBType,
    children: &[PBType],
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    parent_rect: egui::Rect,
    dark_mode: bool,
) {
    let zoom = state.zoom_clamped();
    // Resolve sources and group by prefix (e.g., "clb", "fle")
    let mut source_groups: Vec<Vec<(String, egui::Pos2)>> = Vec::new();
    let mut current_group: Option<(String, Vec<(String, egui::Pos2)>)> = None;

    for src in sources {
        if let Some(pos) = resolve_port_pos(src, &current_pb.name, my_ports, children_ports) {
            // Extract prefix: everything before first '.' or '['
            let prefix = src.split(['.', '[']).next().unwrap_or(src).to_string();

            match current_group.as_mut() {
                Some((last_prefix, group)) if *last_prefix == prefix => {
                    // Same prefix, add to current group
                    group.push((src.clone(), pos));
                }
                _ => {
                    // New prefix, start a new group
                    if let Some((_, group)) = current_group.take() {
                        source_groups.push(group);
                    }
                    current_group = Some((prefix.clone(), vec![(src.clone(), pos)]));
                }
            }
        }
    }
    // Push the last group
    if let Some((_, group)) = current_group {
        source_groups.push(group);
    }

    let mut resolved_sinks = Vec::new();
    for dst in sinks {
        if let Some(pos) = resolve_port_pos(dst, &current_pb.name, my_ports, children_ports) {
            resolved_sinks.push((dst, pos));
        }
    }

    if source_groups.is_empty() || resolved_sinks.is_empty() {
        return;
    }

    let resolved_sources: Vec<_> = source_groups[0].clone();

    if resolved_sources.is_empty() || resolved_sinks.is_empty() {
        return;
    }

    for group in &mut source_groups {
        group.sort_by(|a, b| {
            a.1.y
                .partial_cmp(&b.1.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    resolved_sinks.sort_by(|a, b| {
        a.1.y
            .partial_cmp(&b.1.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let all_y: Vec<f32> = resolved_sources
        .iter()
        .map(|(_, p)| p.y)
        .chain(resolved_sinks.iter().map(|(_, p)| p.y))
        .collect();
    let avg_y = all_y.iter().sum::<f32>() / all_y.len() as f32;

    let source_max_x = resolved_sources
        .iter()
        .map(|(_, p)| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let sink_min_x = resolved_sinks
        .iter()
        .map(|(_, p)| p.x)
        .fold(f32::INFINITY, f32::min);
    let sink_max_x = resolved_sinks
        .iter()
        .map(|(_, p)| p.x)
        .fold(f32::NEG_INFINITY, f32::max);

    let rows = resolved_sources.len().max(resolved_sinks.len());
    let row_spacing = 18.0 * zoom;
    let available_height = (parent_rect.height() - 20.0 * zoom).max(60.0 * zoom);
    let height = ((rows as f32 + 1.0) * row_spacing)
        .max(60.0 * zoom)
        .min(available_height);
    let width = 55.0 * zoom;

    // Special-case clock crossbars: place them in the gap between the source
    // cluster (clb clk pins) and the sink cluster (fle clk pins), based on
    // their bounding edges rather than a fixed offset.
    let is_clock_block = resolved_sources
        .iter()
        .all(|(s, _)| is_clock_port(s, current_pb, children))
        && resolved_sinks
            .iter()
            .all(|(s, _)| is_clock_port(s, current_pb, children));

    let block_center_x = if is_clock_block {
        // Try to derive the sink PB bounds from recorded rects.
        let sink_pb_names: Vec<String> = resolved_sinks
            .iter()
            .filter_map(|(s, _)| s.rsplit('.').nth(1).map(|p| p.to_string()))
            .collect();

        let mut sink_rect_max = f32::NEG_INFINITY;
        for (path, rect) in state.pb_rects.iter() {
            if sink_pb_names.iter().any(|n| path.ends_with(n)) {
                sink_rect_max = sink_rect_max.max(rect.max.x);
            }
        }

        // Fallback to pin-based span if we couldn't find rects.
        if !sink_rect_max.is_finite() {
            sink_rect_max = sink_max_x;
        }

        ((sink_rect_max + parent_rect.max.x) * 0.5)
            .max(parent_rect.min.x + width * 0.5 + 6.0 * zoom)
            .min(parent_rect.max.x - width * 0.5 - 6.0 * zoom)
    } else {
        // Position the block based on the actual gap between sources and sinks.
        let gap = sink_min_x - source_max_x;
        let gap_is_usable = gap.is_finite() && gap > width + 20.0 * zoom;

        if gap_is_usable {
            // Leave 1/3 of the free gap on each side of the crossbar
            let side_margin = ((gap - width).max(0.0)) / 3.0;
            let left_x = source_max_x + side_margin;
            let right_x = sink_min_x - side_margin;
            ((left_x + right_x) * 0.5).clamp(
                parent_rect.min.x + width * 0.5 + 4.0 * zoom,
                parent_rect.max.x - width * 0.5 - 4.0 * zoom,
            )
        } else {
            // Gap too small: bias near sinks but keep a buffer from the parent edge
            (sink_min_x - 20.0 * zoom - width * 0.5)
                .max(parent_rect.min.x + width * 0.5 + 10.0 * zoom)
                .min(parent_rect.max.x - width * 0.5 - 10.0 * zoom)
        }
    };

    let rect =
        egui::Rect::from_center_size(egui::pos2(block_center_x, avg_y), egui::vec2(width, height));

    let block_hovered = ui.rect_contains_pointer(rect);
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
    let fill_color = color_scheme::theme_block_bg(dark_mode);
    let stroke = egui::Stroke::new(1.5 * zoom, stroke_color);

    painter.rect(
        rect,
        egui::CornerRadius::ZERO,
        fill_color,
        stroke,
        egui::epaint::StrokeKind::Inside,
    );
    // Draw a large X across the block instead of text.
    let x_stroke = egui::Stroke::new(2.0 * zoom, stroke_color);
    painter.line_segment([rect.min, rect.max], x_stroke);
    painter.line_segment(
        [
            egui::pos2(rect.min.x, rect.max.y),
            egui::pos2(rect.max.x, rect.min.y),
        ],
        x_stroke,
    );

    for group in &mut source_groups {
        let clb_step = if group.is_empty() {
            rect.height()
        } else {
            (rect.height() / (group.len() as f32 + 1.0)).max(20.0 * zoom)
        };

        for (i, (src_name, src_pos)) in group.iter().enumerate() {
            let y = rect.min.y + clb_step * (i as f32 + 1.0);
            let target = egui::pos2(rect.min.x, y);

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
            let wire_stroke = egui::Stroke::new(1.5 * zoom, wire_color);

            let is_clock = is_clock_port(src_name, current_pb, children);
            if is_clock {
                // Approach from the left with an extra turn: left offset, then up to target y, then into block.
                let offset_x = rect.min.x - 10.0 * zoom;
                let path = vec![
                    *src_pos,
                    egui::pos2(offset_x, src_pos.y),
                    egui::pos2(offset_x, target.y),
                    target,
                ];
                painter.add(egui::Shape::line(path, wire_stroke));
            } else {
                // Special-case only when the source sits to the right of the interconnect:
                // route via a top rail (using fle top minus padding) to avoid cutting across the block.
                if src_pos.x > rect.min.x {
                    // Route: up to top rail, left, down, then right horizontally to interconnect edge
                    let mut fle_top = f32::INFINITY;
                    for (path, r) in state.pb_rects.iter() {
                        if path.contains(".fle[") || path.ends_with("fle") {
                            fle_top = fle_top.min(r.min.y);
                        }
                    }
                    let padding = 10.0 * zoom;
                    let mut top_y = fle_top - padding;
                    if !top_y.is_finite() {
                        top_y = rect.min.y - padding; // fallback
                    }
                    top_y = top_y.max(parent_rect.min.y + 2.0 * zoom);

                    let offset_x = rect.min.x - 10.0 * zoom;
                    let path = vec![
                        *src_pos,
                        egui::pos2(src_pos.x, top_y),
                        egui::pos2(offset_x, top_y),
                        egui::pos2(offset_x, target.y),
                        target,
                    ];
                    painter.add(egui::Shape::line(path, wire_stroke));
                } else {
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
                }
            }

            if block_hovered {
                state.highlighted_positions_next_frame.push(*src_pos);
            }
        }
    }
    let sink_step = rect.height() / (resolved_sinks.len() as f32 + 1.0);
    for (i, (dst_name, dst_pos)) in resolved_sinks.iter().enumerate() {
        let y = rect.min.y + sink_step * (i as f32 + 1.0);
        let start = egui::pos2(rect.max.x, y);

        let wire_highlighted = is_block_highlighted
            || state
                .highlighted_positions_this_frame
                .iter()
                .any(|p| p.distance(*dst_pos) < 1.0);
        let wire_color = if wire_highlighted {
            color_scheme::HIGHLIGHT_COLOR
        } else {
            color_scheme::theme_interconnect_bg(dark_mode)
        };
        let wire_stroke = egui::Stroke::new(1.5 * zoom, wire_color);

        let is_clock = is_clock_port(dst_name, current_pb, children);
        if is_clock {
            let channel_y = dst_pos.y + 5.0 * zoom;
            // Add a right-hand offset before heading toward the sink.
            let offset_x = rect.max.x + 10.0 * zoom;
            let path = vec![
                start,
                egui::pos2(offset_x, start.y),
                egui::pos2(offset_x, channel_y),
                egui::pos2(dst_pos.x, channel_y),
                *dst_pos,
            ];
            painter.add(egui::Shape::line(path, wire_stroke));
        } else {
            draw_wire_segment(
                painter,
                start,
                *dst_pos,
                wire_stroke,
                parent_rect,
                state,
                ui,
                is_clock,
            );
        }

        if block_hovered {
            state.highlighted_positions_next_frame.push(*dst_pos);
        }
    }
}

fn draw_interconnect_block(
    painter: &egui::Painter,
    kind: &str,
    sources: &[String],
    sinks: &[String],
    current_pb: &PBType,
    children: &[PBType],
    my_ports: &HashMap<String, egui::Pos2>,
    children_ports: &HashMap<String, egui::Pos2>,
    state: &mut IntraTileState,
    ui: &mut egui::Ui,
    parent_rect: egui::Rect,
    dark_mode: bool,
) {
    let zoom = state.zoom_clamped();

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

    let width = 35.0 * zoom;

    let max_sink_y = valid_sinks
        .iter()
        .map(|(_, p)| p.y)
        .fold(f32::NEG_INFINITY, |a, b| a.max(b));
    let min_sink_y = valid_sinks
        .iter()
        .map(|(_, p)| p.y)
        .fold(f32::INFINITY, |a, b| a.min(b));

    let sink_spread = max_sink_y - min_sink_y;

    let input_spacing = 15.0 * zoom;
    let input_spread = (sources.len() as f32 + 1.0) * input_spacing;

    let spread = sink_spread.max(input_spread);
    let height = (spread + 20.0 * zoom).max(40.0 * zoom).min(150.0 * zoom);

    // Place the interconnect block slightly left of its sink pins, but clamp it
    // within the parent PB rect so it never gets clipped by the canvas/painter.
    let desired_center_x = min_x - 50.0 * zoom;
    let center_x = desired_center_x.clamp(
        parent_rect.min.x + width * 0.5 + 4.0 * zoom,
        parent_rect.max.x - width * 0.5 - 4.0 * zoom,
    );
    let block_center = egui::pos2(center_x, avg_y);
    let rect = egui::Rect::from_center_size(block_center, egui::vec2(width, height));

    // For sources that sit to the right of this interconnect block (very common in z1010),
    // route them via a top rail to avoid long horizontal segments cutting across many blocks.
    let top_rail_y = {
        let mut min_child_y = f32::INFINITY;
        for (_, r) in state.pb_rects.iter() {
            // Only consider children that are inside this parent PB rect.
            if parent_rect.contains(r.center()) {
                min_child_y = min_child_y.min(r.min.y);
            }
        }
        let padding = 10.0 * zoom;
        let mut y = if min_child_y.is_finite() {
            min_child_y - padding
        } else {
            // Fallback: just above the interconnect block.
            rect.min.y - padding
        };
        y = y.max(parent_rect.min.y + 2.0 * zoom);
        y
    };

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
    let stroke = egui::Stroke::new(1.5 * zoom, stroke_color);
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
        painter.rect(
            rect,
            egui::CornerRadius::ZERO,
            fill_color,
            stroke,
            egui::epaint::StrokeKind::Inside,
        );
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
        let wire_stroke = egui::Stroke::new(1.5 * zoom, wire_color);

        let input_y = rect.min.y + input_step * (i as f32 + 1.0);
        let target = egui::pos2(left_edge_x, input_y);

        // Check if this is a clock connection by examining source port name
        let is_clock = is_clock_port(src_name, current_pb, children);
        if is_clock {
            let default_mid = src_pos.y + (target.y - src_pos.y) * 0.5;
            let channel_y = match (pb_name_from_port(src_name), Some(current_pb.name.as_str())) {
                (Some(spb), Some(dpb)) => clock_channel_y_between(state, spb, dpb, default_mid),
                _ => default_mid,
            };
            draw_clock_wire_segment(
                painter,
                *src_pos,
                target,
                wire_stroke,
                parent_rect,
                state,
                ui,
                channel_y,
            );
        } else {
            // If the source is to the right of the mux/X block, avoid a mid-y horizontal
            // run across the whole cluster by routing via a top rail.
            if src_pos.x > rect.min.x {
                let offset_x = rect.min.x - 10.0 * zoom;
                let path = vec![
                    *src_pos,
                    egui::pos2(src_pos.x, top_rail_y),
                    egui::pos2(offset_x, top_rail_y),
                    egui::pos2(offset_x, target.y),
                    target,
                ];
                painter.add(egui::Shape::line(path, wire_stroke));
            } else {
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
            }
        }

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
        let wire_stroke = egui::Stroke::new(1.5 * zoom, wire_color);

        let start = egui::pos2(right_edge_x, block_center.y);
        // Check if this is a clock connection by examining sink port name
        let is_clock = is_clock_port(dst_name, current_pb, children);
        if is_clock {
            let default_mid = start.y + (dst_pos.y - start.y) * 0.5;
            let channel_y = match (pb_name_from_port(dst_name), Some(current_pb.name.as_str())) {
                (Some(dpb), Some(spb)) => clock_channel_y_between(state, spb, dpb, default_mid),
                _ => default_mid,
            };
            draw_clock_wire_segment(
                painter,
                start,
                dst_pos,
                wire_stroke,
                parent_rect,
                state,
                ui,
                channel_y,
            );
        } else {
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
        }

        if ui.rect_contains_pointer(rect) {
            state.highlighted_positions_next_frame.push(dst_pos);
        }
    }
}
