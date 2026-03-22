use std::collections::HashMap;

use egui::{Color32, epaint::QuadraticBezierShape};
use fpga_arch_parser::{
    DelayInfo, DelayType, FPGAArch, Model, ModelPort, PBType, TimingConstraintType,
};

// --- Visual style constants ---

const BLOCK_FILL: Color32 = Color32::from_rgb(228, 238, 255);
const BLOCK_STROKE_COLOR: Color32 = Color32::from_rgb(55, 85, 150);
const BLOCK_STROKE_WIDTH: f32 = 2.0;

const FF_FILL: Color32 = Color32::from_rgb(255, 248, 215);
const FF_STROKE_COLOR: Color32 = Color32::from_rgb(110, 85, 20);
const FF_STROKE_WIDTH: f32 = 1.5;

const CLOCK_COLOR: Color32 = Color32::from_rgb(120, 50, 175);
const CLOCK_FILL: Color32 = Color32::from_rgb(210, 185, 245);
const INPUT_COLOR: Color32 = Color32::from_rgb(25, 105, 190);
const OUTPUT_COLOR: Color32 = Color32::from_rgb(195, 80, 15);

const SETUP_COLOR: Color32 = Color32::from_rgb(210, 40, 40);
const CLOCK_TO_Q_COLOR: Color32 = Color32::from_rgb(35, 80, 210);
const COMB_PATH_COLOR: Color32 = Color32::from_rgb(25, 155, 60);

/// Color used for missing delay annotations and the constraint arcs they affect.
const MISSING_COLOR: Color32 = Color32::from_rgb(220, 30, 30);
/// Subtle color for present delay value labels (shown in tooltips only).
const DELAY_LABEL_COLOR: Color32 = Color32::from_rgb(60, 60, 60);

const CONSTRAINT_STROKE_WIDTH: f32 = 1.5;
const SIGNAL_STROKE_WIDTH: f32 = 3.0;
const PORT_LABEL_FONT_SIZE: f32 = 24.0;
const MODEL_NAME_FONT_SIZE: f32 = 32.0;
/// Font size for the "⚠ MISSING" annotation labels (at zoom = 1).
const DELAY_LABEL_FONT_SIZE: f32 = 11.0;

// --- Layout constants ---

/// Vertical spacing between data ports in sequential blocks.
const PORT_STEP: f32 = 50.0;
/// Height of a flip-flop symbol.
const FF_HEIGHT: f32 = 75.0;
/// Width of a flip-flop symbol.
const FF_WIDTH: f32 = 50.0;
/// Distance from the FF top edge to the D/Q port connection point.
const FF_PORT_OFFSET: f32 = 20.0;
/// Extra vertical padding added to the port step when FFs are present in a combinational block.
const FF_PORT_STEP_PADDING: f32 = 20.0;
/// Vertical spacing between clock signal rows.
const CLOCK_STEP: f32 = 50.0;
/// Length of a port arrow outside the block boundary.
const ARROW_LENGTH: f32 = 100.0;
/// Gap between an arrow tip/tail and the adjacent port label.
const ARROW_GAP: f32 = 10.0;
/// In non-sequential blocks, arrows start/end this far inside the block boundary.
const NON_SEQ_ARROW_INNER: f32 = 50.0;
const NON_SEQ_ARROW_LENGTH: f32 = ARROW_LENGTH + NON_SEQ_ARROW_INNER;
/// Extra horizontal padding between a port label and the block edge.
const LABEL_MARGIN: f32 = 20.0;
/// Vertical margin above and below the block on the canvas.
const V_MARGIN: f32 = 50.0;
/// Vertical offset from the block top edge to the baseline of the name label.
const BLOCK_NAME_V_OFFSET: f32 = 8.0;

// --- Zoom constants ---

const ZOOM_STEP: f32 = 1.25;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 4.0;
/// Leave this fraction of margin around the block when computing the fit zoom.
const ZOOM_FIT_MARGIN: f32 = 0.9;

// --- Data types ---

/// Ports of a model grouped by role, used to pass port data between rendering functions.
struct PortGroups<'a> {
    input_ports: &'a [&'a ModelPort],
    output_ports: &'a [&'a ModelPort],
    input_clock_ports: &'a [&'a ModelPort],
    output_clock_ports: &'a [&'a ModelPort],
}

/// A pb_type found in the complex block hierarchy that references a model,
/// together with its ancestry path (root-to-leaf list of pb_type names).
pub struct PBTypeMatch<'a> {
    pub path: Vec<String>,
    pub pb_type: &'a PBType,
}

impl<'a> PBTypeMatch<'a> {
    /// Returns the path formatted as "a > b > c".
    pub fn path_display(&self) -> String {
        self.path.join(" > ")
    }
}

// --- Delay annotation helpers ---

/// State of a single timing annotation for one arc or wire.
enum DelayAnnotation {
    /// No pb_type is selected; show nothing.
    NotActive,
    /// Pb_type is selected and all expected annotations were found. String is the tooltip content.
    Present(String),
    /// One or more expected annotations are absent. String is the detail shown in the tooltip,
    /// listing both present values and which ones are missing.
    Missing(String),
}

/// Looks up delay and timing-constraint annotations from a selected `PBType`.
struct DelayLookup<'a> {
    pb_type: &'a PBType,
}

impl<'a> DelayLookup<'a> {
    fn new(pb_type: &'a PBType) -> Self {
        Self { pb_type }
    }

    /// Strip the "pb_type_name." prefix from port references like `"adder.a"` → `"a"`.
    fn strip_prefix(port_ref: &str) -> &str {
        port_ref
            .split_once('.')
            .map(|(_, p)| p)
            .unwrap_or(port_ref)
    }

    /// Look up all combinational delays from `in_port` to `out_port` and format them as a
    /// human-readable string for use in a tooltip.
    ///
    /// Returns `Some(text)` if at least one matching entry was found, `None` otherwise.
    /// For matrices, every per-pin value is listed; if all values are equal a compact
    /// summary is shown instead.  Matrices with more than 16 entries are summarised to
    /// keep the tooltip manageable.
    fn comb_delay_text(&self, in_port: &str, out_port: &str) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        for delay in &self.pb_type.delays {
            match delay {
                DelayInfo::Constant {
                    min,
                    max,
                    in_port: ip,
                    out_port: op,
                } => {
                    if Self::strip_prefix(ip) == in_port && Self::strip_prefix(op) == out_port {
                        parts.push(format!("D = {}", format_delay_range(*min, *max)));
                    }
                }
                DelayInfo::Matrix {
                    matrix,
                    in_port: ip,
                    out_port: op,
                    delay_type,
                } => {
                    if Self::strip_prefix(ip) != in_port || Self::strip_prefix(op) != out_port {
                        continue;
                    }
                    let flat: Vec<f32> = matrix.iter().flatten().copied().collect();
                    if flat.is_empty() {
                        continue;
                    }
                    // Count only non-empty rows: the parser produces blank rows for
                    // leading/trailing newlines in the XML text content.
                    let n_in = matrix.iter().filter(|r| !r.is_empty()).count();
                    let n_out = matrix
                        .iter()
                        .find(|r| !r.is_empty())
                        .map(Vec::len)
                        .unwrap_or(0);
                    let type_str = match delay_type {
                        DelayType::Max => "max",
                        DelayType::Min => "min",
                    };

                    // Summarise if all entries are equal or the matrix is large.
                    let all_equal = flat.windows(2).all(|w| w[0] == w[1]);
                    if all_equal {
                        parts.push(format!(
                            "D ({type_str}, {n_in}×{n_out}, all equal) = {}",
                            format_delay(flat[0])
                        ));
                    } else if flat.len() > 16 {
                        let min = flat.iter().cloned().fold(f32::MAX, f32::min);
                        let max = flat.iter().cloned().fold(f32::MIN, f32::max);
                        parts.push(format!(
                            "D ({type_str}, {n_in}×{n_out} matrix)\n  min = {}  max = {}",
                            format_delay(min),
                            format_delay(max)
                        ));
                    } else {
                        let mut lines =
                            vec![format!("D ({type_str}, {n_in}×{n_out} matrix):")];
                        for (i, row) in matrix.iter().filter(|r| !r.is_empty()).enumerate() {
                            for (j, val) in row.iter().enumerate() {
                                lines.push(format!("  [{i}→{j}]  {}", format_delay(*val)));
                            }
                        }
                        parts.push(lines.join("\n"));
                    }
                }
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    /// Look up `T_setup` for `(port, clock)`. Returns `Some(value_s)` if found.
    fn t_setup(&self, port: &str, clock: &str) -> Option<f32> {
        self.pb_type
            .timing_constraints
            .iter()
            .find(|tc| {
                matches!(tc.constraint_type, TimingConstraintType::Setup)
                    && Self::strip_prefix(&tc.port) == port
                    && tc.clock == clock
            })
            .map(|tc| tc.max_value)
    }

    /// Look up `T_hold` for `(port, clock)`. Returns `Some(value_s)` if found.
    fn t_hold(&self, port: &str, clock: &str) -> Option<f32> {
        self.pb_type
            .timing_constraints
            .iter()
            .find(|tc| {
                matches!(tc.constraint_type, TimingConstraintType::Hold)
                    && Self::strip_prefix(&tc.port) == port
                    && tc.clock == clock
            })
            .map(|tc| tc.max_value)
    }

    /// Look up `T_clock_to_Q` for `(port, clock)`. Returns `Some((min_s, max_s))` if found.
    fn t_clock_to_q(&self, port: &str, clock: &str) -> Option<(f32, f32)> {
        self.pb_type
            .timing_constraints
            .iter()
            .find(|tc| {
                matches!(tc.constraint_type, TimingConstraintType::ClockToQ)
                    && Self::strip_prefix(&tc.port) == port
                    && tc.clock == clock
            })
            .map(|tc| (tc.min_value, tc.max_value))
    }
}

/// Builds a combined setup + hold `DelayAnnotation` for the clock→D arc.
///
/// Both `T_setup` and `T_hold` are reported. The arc turns red and shows "⚠ MISSING" if
/// either value is absent from the selected pb_type.
fn build_setup_hold_annotation(
    delays: Option<&DelayLookup<'_>>,
    port: &str,
    clock: &str,
) -> DelayAnnotation {
    let Some(d) = delays else {
        return DelayAnnotation::NotActive;
    };
    let tsu = d.t_setup(port, clock);
    let th = d.t_hold(port, clock);

    let tsu_str = tsu
        .map(|v| format!("Tsu = {}", format_delay(v)))
        .unwrap_or_else(|| "Tsu = ⚠ MISSING".to_string());
    let th_str = th
        .map(|v| format!("Th  = {}", format_delay(v)))
        .unwrap_or_else(|| "Th  = ⚠ MISSING".to_string());
    let detail = format!("{tsu_str}\n{th_str}");

    if tsu.is_some() && th.is_some() {
        DelayAnnotation::Present(detail)
    } else {
        DelayAnnotation::Missing(detail)
    }
}

/// Format a delay value in seconds as a human-readable string (ps or ns).
fn format_delay(v: f32) -> String {
    let ps = v * 1e12;
    if ps.abs() >= 1000.0 {
        format!("{:.3} ns", ps / 1000.0)
    } else {
        format!("{:.2} ps", ps)
    }
}

/// Format a (min, max) delay range. Shows a single value when min ≈ max.
fn format_delay_range(min: f32, max: f32) -> String {
    if (max - min).abs() <= 1e-15 * max.abs().max(1.0) {
        format_delay(max)
    } else {
        format!("{} – {}", format_delay(min), format_delay(max))
    }
}

/// Returns the effective stroke color for a timing arc, turning it bright red when missing.
fn annotation_stroke_color(normal: Color32, ann: &DelayAnnotation) -> Color32 {
    match ann {
        DelayAnnotation::Missing(_) => MISSING_COLOR,
        _ => normal,
    }
}

/// Midpoint of a quadratic Bézier at t = 0.5.
fn bezier_midpoint(p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2) -> egui::Pos2 {
    egui::pos2(
        0.25 * p0.x + 0.5 * p1.x + 0.25 * p2.x,
        0.25 * p0.y + 0.5 * p1.y + 0.25 * p2.y,
    )
}

/// Draws a timing annotation near `midpoint`:
/// - `NotActive`: nothing drawn.
/// - `Present(label)`: invisible hover zone; tooltip shows the delay value on hover.
/// - `Missing`: always-visible bold **"⚠ MISSING"** in red; tooltip explains on hover.
///
/// `id` must be unique per element per frame.
fn draw_timing_label(
    ui: &mut egui::Ui,
    midpoint: egui::Pos2,
    ann: &DelayAnnotation,
    zoom: f32,
    id: egui::Id,
) {
    let font_size = DELAY_LABEL_FONT_SIZE * zoom;
    match ann {
        DelayAnnotation::NotActive => {}
        DelayAnnotation::Present(label) => {
            // Show on hover only — hover is checked by proximity to midpoint.
            if ui
                .ctx()
                .pointer_hover_pos()
                .map(|p| p.distance(midpoint) < 30.0 * zoom)
                .unwrap_or(false)
            {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    id,
                    egui::PopupAnchor::Pointer,
                )
                .show(|ui| {
                    ui.label(
                        egui::RichText::new(label.as_str())
                            .color(DELAY_LABEL_COLOR)
                            .monospace(),
                    );
                });
            }
        }
        DelayAnnotation::Missing(detail) => {
            // Always-visible warning badge.
            ui.painter().text(
                midpoint,
                egui::Align2::CENTER_CENTER,
                "⚠ MISSING",
                egui::FontId::proportional(font_size),
                MISSING_COLOR,
            );
            // Tooltip with full detail on hover.
            if ui
                .ctx()
                .pointer_hover_pos()
                .map(|p| p.distance(midpoint) < 40.0 * zoom)
                .unwrap_or(false)
            {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    id,
                    egui::PopupAnchor::Pointer,
                )
                .show(|ui| {
                    ui.label(
                        egui::RichText::new(detail.as_str())
                            .color(MISSING_COLOR)
                            .monospace(),
                    );
                });
            }
        }
    }
}

pub struct PrimitiveView {
    pub selected_model_name: Option<String>,
    /// Index into the list of pb_types that use the selected model.
    pub selected_pb_type_idx: Option<usize>,
    show_setup_constraints: bool,
    show_clock_to_q: bool,
    show_combinational_paths: bool,
    /// Current zoom level. `None` means "auto-fit": use `fit_zoom` each frame.
    zoom: Option<f32>,
    /// Most recently computed fit-to-view zoom, updated each frame by `render_model`.
    fit_zoom: f32,
    /// Tracks which model was rendered last frame; used to detect external model changes.
    last_rendered_model_name: Option<String>,
}

impl Default for PrimitiveView {
    fn default() -> Self {
        Self {
            selected_model_name: None,
            selected_pb_type_idx: None,
            show_setup_constraints: true,
            show_clock_to_q: true,
            show_combinational_paths: true,
            zoom: None,
            fit_zoom: 1.0,
            last_rendered_model_name: None,
        }
    }
}

impl PrimitiveView {
    /// Returns the zoom level that should be applied this frame.
    fn effective_zoom(&self) -> f32 {
        self.zoom.unwrap_or(self.fit_zoom)
    }

    pub fn render(&mut self, arch: &FPGAArch, ctx: &egui::Context) {
        egui::SidePanel::right("primitive_view_controls")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.render_side_panel(arch, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_central_panel(arch, ui);
        });
    }

    fn render_side_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        ui.heading("Primitive View");

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if !arch.models.is_empty() {
            ui.label("Select Model:");
            ui.add_space(5.0);

            let mut selected_model_name_str = self.selected_model_name.as_deref().unwrap_or("");

            egui::ComboBox::from_id_salt("model_selector_combobox")
                .selected_text(if !selected_model_name_str.is_empty() {
                    selected_model_name_str
                } else {
                    "Select a model"
                })
                .show_ui(ui, |ui| {
                    for model in &arch.models {
                        ui.selectable_value(&mut selected_model_name_str, &model.name, &model.name);
                    }
                });

            if selected_model_name_str != self.selected_model_name.as_deref().unwrap_or("") {
                self.selected_model_name = Some(selected_model_name_str.to_string());
                self.selected_pb_type_idx = None;
            }
        } else {
            ui.label("No models available in architecture");
        }

        if let Some(model_name) = &self.selected_model_name.clone() {
            let matches = find_pb_types_for_model(arch, model_name);
            if !matches.is_empty() {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label("Used by pb_types:");
                ui.add_space(5.0);
                egui::ScrollArea::vertical()
                    .id_salt("pb_type_list_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (idx, m) in matches.iter().enumerate() {
                            let selected = self.selected_pb_type_idx == Some(idx);
                            if ui.selectable_label(selected, m.path_display()).clicked() {
                                self.selected_pb_type_idx =
                                    if selected { None } else { Some(idx) };
                            }
                        }
                    });
            }
        }

        ui.add_space(10.0);

        // Zoom controls
        ui.separator();
        ui.add_space(10.0);
        ui.label("Zoom:");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let zoom = self.effective_zoom();
            if ui.button("−").clicked() {
                self.zoom = Some((zoom / ZOOM_STEP).clamp(ZOOM_MIN, ZOOM_MAX));
            }
            ui.label(format!("{:.0}%", zoom * 100.0));
            if ui.button("+").clicked() {
                self.zoom = Some((zoom * ZOOM_STEP).clamp(ZOOM_MIN, ZOOM_MAX));
            }
            if ui.button("Fit").clicked() {
                self.zoom = None;
            }
        });
        ui.add_space(10.0);

        ui.separator();
        ui.add_space(10.0);
        ui.label("Legend:");
        ui.add_space(6.0);

        ui.label("Signals:");
        ui.add_space(4.0);
        legend_entry(ui, "Input", INPUT_COLOR);
        ui.add_space(4.0);
        legend_entry(ui, "Output", OUTPUT_COLOR);
        ui.add_space(4.0);
        legend_entry(ui, "Clock", CLOCK_COLOR);
        ui.add_space(10.0);

        ui.label("Timing Constraints:");
        ui.add_space(4.0);
        constraint_checkbox(
            ui,
            &mut self.show_setup_constraints,
            "Setup & Hold Constraints",
            SETUP_COLOR,
        );
        ui.add_space(4.0);
        constraint_checkbox(
            ui,
            &mut self.show_clock_to_q,
            "Clock to Q",
            CLOCK_TO_Q_COLOR,
        );
        ui.add_space(4.0);
        constraint_checkbox(
            ui,
            &mut self.show_combinational_paths,
            "Combinational Paths",
            COMB_PATH_COLOR,
        );
    }

    fn render_central_panel(&mut self, arch: &FPGAArch, ui: &mut egui::Ui) {
        if let Some(selected_model_name) = &self.selected_model_name
            && let Some(model) = arch.models.iter().find(|m| m.name == *selected_model_name)
        {
            // Build a delay lookup for the selected pb_type (if any).
            let matches = find_pb_types_for_model(arch, selected_model_name);
            let selected_pb_type = self
                .selected_pb_type_idx
                .and_then(|i| matches.get(i))
                .map(|m| m.pb_type);
            let delay_lookup = selected_pb_type.map(DelayLookup::new);

            // Handle zoom via Cmd+scroll and pinch gestures when the pointer is over the canvas.
            // This must happen before the ScrollArea is created so we can consume scroll
            // events that should zoom rather than scroll.
            if ui.rect_contains_pointer(ui.max_rect()) {
                let zoom_delta = ui.input_mut(|i| {
                    // Pinch-to-zoom on trackpad. egui also folds Cmd+scroll into
                    // zoom_delta(), so just consume the raw scroll events when Cmd
                    // is held to prevent the ScrollArea from panning at the same time.
                    if i.modifiers.command && i.smooth_scroll_delta.y != 0.0 {
                        i.smooth_scroll_delta = egui::Vec2::ZERO;
                        i.raw_scroll_delta = egui::Vec2::ZERO;
                    }
                    i.zoom_delta()
                });
                if (zoom_delta - 1.0).abs() > f32::EPSILON {
                    self.zoom =
                        Some((self.effective_zoom() * zoom_delta).clamp(ZOOM_MIN, ZOOM_MAX));
                }
            }

            let available_size = ui.available_size();
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.render_model(model, delay_lookup.as_ref(), ui, available_size);
                });
        }
    }

    fn render_model(
        &mut self,
        model: &Model,
        delays: Option<&DelayLookup<'_>>,
        ui: &mut egui::Ui,
        available_size: egui::Vec2,
    ) {
        // Reset zoom to auto-fit whenever the displayed model changes (e.g. from the combobox
        // or from an external navigation action like the summary view buttons).
        if self.last_rendered_model_name.as_deref() != Some(&model.name) {
            self.zoom = None;
            self.last_rendered_model_name = Some(model.name.clone());
        }

        // Classify ports: separate data ports from clocks, and input clocks from output clocks.
        let mut input_ports: Vec<&ModelPort> = Vec::new();
        let mut output_ports: Vec<&ModelPort> = Vec::new();
        let mut input_clock_ports: Vec<&ModelPort> = Vec::new();
        let mut output_clock_ports: Vec<&ModelPort> = Vec::new();
        for port in &model.input_ports {
            if port.is_clock {
                input_clock_ports.push(port);
            } else {
                input_ports.push(port);
            }
        }
        for port in &model.output_ports {
            if port.is_clock {
                output_clock_ports.push(port);
            } else {
                output_ports.push(port);
            }
        }

        let port_groups = PortGroups {
            input_ports: &input_ports,
            output_ports: &output_ports,
            input_clock_ports: &input_clock_ports,
            output_clock_ports: &output_clock_ports,
        };

        let max_ports = input_ports.len().max(output_ports.len());
        let clock_extra_height =
            (input_clock_ports.len() + output_clock_ports.len()) as f32 * CLOCK_STEP;

        // Measure label widths to compute exact horizontal padding (at zoom = 1).
        // Each side needs: ARROW_LENGTH (outside block) + ARROW_GAP + LABEL_MARGIN + label width.
        let font_id = egui::FontId::proportional(PORT_LABEL_FONT_SIZE);
        let measure = |name: &str| -> f32 {
            ui.fonts(|f| {
                f.layout_no_wrap(name.to_string(), font_id.clone(), Color32::WHITE)
                    .size()
                    .x
            })
        };
        let max_left_label_width = input_ports
            .iter()
            .chain(input_clock_ports.iter())
            .map(|p| measure(&p.name))
            .fold(0.0_f32, f32::max);
        let max_right_label_width = output_ports
            .iter()
            .chain(output_clock_ports.iter())
            .map(|p| measure(&p.name))
            .fold(0.0_f32, f32::max);
        let left_padding = ARROW_LENGTH + ARROW_GAP + LABEL_MARGIN + max_left_label_width;
        let right_padding = ARROW_LENGTH + ARROW_GAP + LABEL_MARGIN + max_right_label_width;

        let is_sequential = is_sequential_block(model);
        let block_width = if is_sequential { 250.0_f32 } else { 500.0_f32 };
        let block_height = if is_sequential {
            (max_ports + 2) as f32 * PORT_STEP
        } else {
            let has_flops = input_ports
                .iter()
                .chain(output_ports.iter())
                .any(|p| p.clock.is_some());
            let port_step = if has_flops {
                FF_HEIGHT + FF_PORT_STEP_PADDING
            } else {
                PORT_STEP
            };
            (max_ports + 2) as f32 * port_step
        };

        // Compute the natural (zoom = 1) canvas dimensions and update the fit zoom.
        let natural_width = left_padding + block_width + right_padding;
        let natural_height = block_height + clock_extra_height + 2.0 * V_MARGIN;
        self.fit_zoom = (ZOOM_FIT_MARGIN
            * (available_size.x / natural_width).min(available_size.y / natural_height))
        .clamp(ZOOM_MIN, ZOOM_MAX);

        let zoom = self.effective_zoom();

        let block_outline = allocate_block_canvas(
            ui,
            block_width,
            block_height,
            left_padding,
            right_padding,
            clock_extra_height,
            available_size.x,
            zoom,
        );
        draw_block_outline(model, block_outline, zoom, ui);

        if is_sequential {
            self.render_sequential_block(block_outline, &port_groups, delays, zoom, ui);
        } else {
            self.render_combinational_block(block_outline, &port_groups, delays, zoom, ui);
        }
    }

    fn render_sequential_block(
        &self,
        block_outline: egui::Rect,
        ports: &PortGroups<'_>,
        delays: Option<&DelayLookup<'_>>,
        zoom: f32,
        ui: &mut egui::Ui,
    ) {
        // Draw clock triangles along the bottom edge and their wires.
        // Input clocks: wire from the left. Output clocks: wire to the right.
        let all_clocks: Vec<&ModelPort> = ports
            .input_clock_ports
            .iter()
            .chain(ports.output_clock_ports.iter())
            .copied()
            .collect();
        let mut clock_triangle_top: HashMap<String, egui::Pos2> = HashMap::new();
        let triangle_step = block_outline.width() / (all_clocks.len() + 1) as f32;
        for (idx, port) in all_clocks.iter().enumerate() {
            let is_output = ports.output_clock_ports.iter().any(|p| p.name == port.name);
            let base =
                block_outline.left_bottom() + egui::vec2(triangle_step * (idx + 1) as f32, 0.0);
            let top = base + egui::vec2(0.0, -20.0 * zoom);
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    base + egui::vec2(-10.0 * zoom, 0.0),
                    top,
                    base + egui::vec2(10.0 * zoom, 0.0),
                ],
                CLOCK_FILL,
                egui::Stroke::new(FF_STROKE_WIDTH * zoom, CLOCK_COLOR),
            ));

            let steiner = base + egui::vec2(0.0, CLOCK_STEP * zoom * (idx + 1) as f32);
            if is_output {
                let end = egui::pos2(block_outline.right() + ARROW_LENGTH * zoom, steiner.y);
                ui.painter().line(
                    vec![base, steiner, end],
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, CLOCK_COLOR),
                );
                ui.painter().text(
                    end + egui::vec2(ARROW_GAP * zoom, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &port.name,
                    egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                    CLOCK_COLOR,
                );
            } else {
                let start = egui::pos2(block_outline.left() - ARROW_LENGTH * zoom, steiner.y);
                ui.painter().line(
                    vec![start, steiner, base],
                    egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, CLOCK_COLOR),
                );
                ui.painter().text(
                    start - egui::vec2(ARROW_GAP * zoom, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &port.name,
                    egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                    CLOCK_COLOR,
                );
            }
            clock_triangle_top.insert(port.name.clone(), top);
        }

        // Draw input ports with arrows and optional setup constraint curves.
        let input_step = block_outline.height() / (ports.input_ports.len() + 2) as f32;
        for (idx, port) in ports.input_ports.iter().enumerate() {
            let tip = block_outline.left_top() + egui::vec2(0.0, input_step * (idx + 1) as f32);
            let start = tip - egui::vec2(ARROW_LENGTH * zoom, 0.0);
            ui.painter().arrow(
                start,
                tip - start,
                egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, INPUT_COLOR),
            );
            ui.painter().text(
                start - egui::vec2(ARROW_GAP * zoom, 0.0),
                egui::Align2::RIGHT_CENTER,
                &port.name,
                egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                INPUT_COLOR,
            );
            if self.show_setup_constraints
                && let Some(clock_name) = &port.clock
            {
                let clock_top = match clock_triangle_top.get(clock_name) {
                    Some(p) => p,
                    None => {
                        ui.painter().debug_text(
                            tip,
                            egui::Align2::LEFT_CENTER,
                            Color32::RED,
                            format!("Cannot find clock: {clock_name}"),
                        );
                        continue;
                    }
                };
                let control = egui::pos2(clock_top.x, tip.y);
                let setup_ann =
                    build_setup_hold_annotation(delays, &port.name, clock_name);
                let arc_color = annotation_stroke_color(SETUP_COLOR, &setup_ann);
                ui.painter().add(QuadraticBezierShape::from_points_stroke(
                    [*clock_top, control, tip],
                    false,
                    Color32::TRANSPARENT,
                    egui::Stroke::new(CONSTRAINT_STROKE_WIDTH * zoom, arc_color),
                ));
                let midpoint = bezier_midpoint(*clock_top, control, tip);
                draw_timing_label(
                    ui,
                    midpoint,
                    &setup_ann,
                    zoom,
                    egui::Id::new(("seq_setup", port.name.as_str(), clock_name.as_str())),
                );
            }
        }

        // Draw output ports with arrows and optional clock-to-Q arcs.
        let output_step = block_outline.height() / (ports.output_ports.len() + 2) as f32;
        for (idx, port) in ports.output_ports.iter().enumerate() {
            let start =
                block_outline.right_top() + egui::vec2(0.0, output_step * (idx + 1) as f32);
            let tip = start + egui::vec2(ARROW_LENGTH * zoom, 0.0);
            ui.painter().arrow(
                start,
                tip - start,
                egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, OUTPUT_COLOR),
            );
            ui.painter().text(
                tip + egui::vec2(ARROW_GAP * zoom, 0.0),
                egui::Align2::LEFT_CENTER,
                &port.name,
                egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                OUTPUT_COLOR,
            );
            if self.show_clock_to_q
                && let Some(clock_name) = &port.clock
            {
                let clock_top = match clock_triangle_top.get(clock_name) {
                    Some(p) => p,
                    None => {
                        ui.painter().debug_text(
                            start,
                            egui::Align2::RIGHT_CENTER,
                            Color32::RED,
                            format!("Cannot find clock: {clock_name}"),
                        );
                        continue;
                    }
                };
                let control = egui::pos2(clock_top.x, start.y);
                let ctq_ann = match delays {
                    None => DelayAnnotation::NotActive,
                    Some(d) => match d.t_clock_to_q(&port.name, clock_name) {
                        Some((min, max)) => DelayAnnotation::Present(format!(
                            "Tcq = {}",
                            format_delay_range(min, max)
                        )),
                        None => DelayAnnotation::Missing(
                            "Tcq = ⚠ MISSING".to_string(),
                        ),
                    },
                };
                let arc_color = annotation_stroke_color(CLOCK_TO_Q_COLOR, &ctq_ann);
                ui.painter().add(QuadraticBezierShape::from_points_stroke(
                    [*clock_top, control, start],
                    false,
                    Color32::TRANSPARENT,
                    egui::Stroke::new(CONSTRAINT_STROKE_WIDTH * zoom, arc_color),
                ));
                let midpoint = bezier_midpoint(*clock_top, control, start);
                draw_timing_label(
                    ui,
                    midpoint,
                    &ctq_ann,
                    zoom,
                    egui::Id::new(("seq_ctq", port.name.as_str(), clock_name.as_str())),
                );
            }
        }
    }

    fn render_combinational_block(
        &self,
        block_outline: egui::Rect,
        ports: &PortGroups<'_>,
        delays: Option<&DelayLookup<'_>>,
        zoom: f32,
        ui: &mut egui::Ui,
    ) {
        // Draw clock labels and record their drive points.
        // Input clocks are on the left; output clocks are on the right.
        let mut clock_drive_point: HashMap<String, egui::Pos2> = HashMap::new();
        for (idx, port) in ports.input_clock_ports.iter().enumerate() {
            let point = egui::pos2(
                block_outline.left() - ARROW_LENGTH * zoom,
                block_outline.bottom() + CLOCK_STEP * zoom * (idx + 1) as f32,
            );
            ui.painter().text(
                point - egui::vec2(ARROW_GAP * zoom, 0.0),
                egui::Align2::RIGHT_CENTER,
                &port.name,
                egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                CLOCK_COLOR,
            );
            clock_drive_point.insert(port.name.clone(), point);
        }
        for (idx, port) in ports.output_clock_ports.iter().enumerate() {
            let row = ports.input_clock_ports.len() + idx + 1;
            let point = egui::pos2(
                block_outline.right() + ARROW_LENGTH * zoom,
                block_outline.bottom() + CLOCK_STEP * zoom * row as f32,
            );
            ui.painter().text(
                point + egui::vec2(ARROW_GAP * zoom, 0.0),
                egui::Align2::LEFT_CENTER,
                &port.name,
                egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                CLOCK_COLOR,
            );
            clock_drive_point.insert(port.name.clone(), point);
        }

        // Draw output ports. Record signal start points for combinational path drawing.
        let mut output_signal_start: HashMap<String, egui::Pos2> = HashMap::new();
        let output_step = block_outline.height() / (ports.output_ports.len() + 2) as f32;
        for (idx, port) in ports.output_ports.iter().enumerate() {
            let arrow_start = block_outline.right_top()
                + egui::vec2(-NON_SEQ_ARROW_INNER * zoom, output_step * (idx + 1) as f32);
            let arrow_tip = arrow_start + egui::vec2(NON_SEQ_ARROW_LENGTH * zoom, 0.0);
            ui.painter().arrow(
                arrow_start,
                arrow_tip - arrow_start,
                egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, OUTPUT_COLOR),
            );
            ui.painter().text(
                arrow_tip + egui::vec2(ARROW_GAP * zoom, 0.0),
                egui::Align2::LEFT_CENTER,
                &port.name,
                egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                OUTPUT_COLOR,
            );

            let mut signal_start = arrow_start;
            if let Some(clock_name) = &port.clock {
                let ff_outline = egui::Rect::from_min_size(
                    egui::pos2(
                        signal_start.x - FF_WIDTH * zoom,
                        signal_start.y - FF_PORT_OFFSET * zoom,
                    ),
                    egui::vec2(FF_WIDTH * zoom, FF_HEIGHT * zoom),
                );
                // Output FF: the clock-to-Q delay (clock → Q output) is annotated on the
                // clock-to-Q arc inside the FF symbol.
                let ctq_ann = match delays {
                    None => DelayAnnotation::NotActive,
                    Some(d) => match d.t_clock_to_q(&port.name, clock_name) {
                        Some((min, max)) => DelayAnnotation::Present(format!(
                            "Tcq = {}",
                            format_delay_range(min, max)
                        )),
                        None => DelayAnnotation::Missing(
                            "Tcq = ⚠ MISSING".to_string(),
                        ),
                    },
                };
                draw_flip_flop(
                    &ff_outline,
                    &DelayAnnotation::NotActive,
                    &ctq_ann,
                    self.show_setup_constraints,
                    self.show_clock_to_q,
                    zoom,
                    ui,
                );
                signal_start -= egui::vec2(FF_WIDTH * zoom, 0.0);
                draw_ff_clock_path(&ff_outline, clock_name, &clock_drive_point, zoom, ui);
            }
            output_signal_start.insert(port.name.clone(), signal_start);
        }

        // Draw input ports. Connect to output signal start points for combinational paths.
        let input_step = block_outline.height() / (ports.input_ports.len() + 2) as f32;
        for (idx, port) in ports.input_ports.iter().enumerate() {
            let arrow_tip = block_outline.left_top()
                + egui::vec2(NON_SEQ_ARROW_INNER * zoom, input_step * (idx + 1) as f32);
            let arrow_start = arrow_tip - egui::vec2(NON_SEQ_ARROW_LENGTH * zoom, 0.0);
            ui.painter().arrow(
                arrow_start,
                arrow_tip - arrow_start,
                egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, INPUT_COLOR),
            );
            ui.painter().text(
                arrow_start - egui::vec2(ARROW_GAP * zoom, 0.0),
                egui::Align2::RIGHT_CENTER,
                &port.name,
                egui::FontId::proportional(PORT_LABEL_FONT_SIZE * zoom),
                INPUT_COLOR,
            );

            let mut signal_start = arrow_tip;
            if let Some(clock_name) = &port.clock {
                let ff_outline = egui::Rect::from_min_size(
                    egui::pos2(signal_start.x, signal_start.y - FF_PORT_OFFSET * zoom),
                    egui::vec2(FF_WIDTH * zoom, FF_HEIGHT * zoom),
                );
                // Input FF: the setup time (D → clock) is annotated on the setup arc inside
                // the FF symbol.
                let setup_ann =
                    build_setup_hold_annotation(delays, &port.name, clock_name);
                draw_flip_flop(
                    &ff_outline,
                    &setup_ann,
                    &DelayAnnotation::NotActive,
                    self.show_setup_constraints,
                    self.show_clock_to_q,
                    zoom,
                    ui,
                );
                signal_start += egui::vec2(FF_WIDTH * zoom, 0.0);
                draw_ff_clock_path(&ff_outline, clock_name, &clock_drive_point, zoom, ui);
            }

            if self.show_combinational_paths {
                for sink_name in &port.combinational_sink_ports {
                    match output_signal_start.get(sink_name) {
                        Some(sink) => {
                            let comb_ann = match delays {
                                None => DelayAnnotation::NotActive,
                                Some(d) => match d.comb_delay_text(&port.name, sink_name) {
                                    Some(text) => DelayAnnotation::Present(text),
                                    None => DelayAnnotation::Missing(
                                        "D = ⚠ MISSING".to_string(),
                                    ),
                                },
                            };
                            let wire_color =
                                annotation_stroke_color(COMB_PATH_COLOR, &comb_ann);
                            ui.painter().line_segment(
                                [signal_start, *sink],
                                egui::Stroke::new(CONSTRAINT_STROKE_WIDTH * zoom, wire_color),
                            );
                            let midpoint = egui::pos2(
                                (signal_start.x + sink.x) / 2.0,
                                (signal_start.y + sink.y) / 2.0,
                            );
                            draw_timing_label(
                                ui,
                                midpoint,
                                &comb_ann,
                                zoom,
                                egui::Id::new((
                                    "comb",
                                    port.name.as_str(),
                                    sink_name.as_str(),
                                )),
                            );
                        }
                        None => {
                            ui.painter().debug_text(
                                arrow_tip,
                                egui::Align2::RIGHT_CENTER,
                                Color32::RED,
                                format!("Cannot find port: {sink_name}"),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Allocates a scaled canvas and returns the block outline rect centered within it.
fn allocate_block_canvas(
    ui: &mut egui::Ui,
    block_width: f32,
    block_height: f32,
    left_padding: f32,
    right_padding: f32,
    clock_extra_height: f32,
    available_width: f32,
    zoom: f32,
) -> egui::Rect {
    let content_width = (left_padding + block_width + right_padding) * zoom;
    let canvas_size = egui::vec2(
        content_width.max(available_width),
        (block_height + clock_extra_height + 2.0 * V_MARGIN) * zoom,
    );
    let (canvas_rect, _) = ui.allocate_exact_size(canvas_size, egui::Sense::empty());
    let extra_x = (canvas_rect.width() - content_width).max(0.0);
    let block_center = egui::pos2(
        canvas_rect.left() + extra_x / 2.0 + left_padding * zoom + block_width * zoom / 2.0,
        canvas_rect.top() + V_MARGIN * zoom + block_height * zoom / 2.0,
    );
    egui::Rect::from_center_size(
        block_center,
        egui::vec2(block_width * zoom, block_height * zoom),
    )
}

/// Draws the block rectangle and its name label.
fn draw_block_outline(model: &Model, block_outline: egui::Rect, zoom: f32, ui: &mut egui::Ui) {
    ui.painter().rect(
        block_outline,
        egui::CornerRadius::same((8.0 * zoom) as u8),
        BLOCK_FILL,
        egui::Stroke::new(BLOCK_STROKE_WIDTH * zoom, BLOCK_STROKE_COLOR),
        egui::epaint::StrokeKind::Middle,
    );
    ui.painter().text(
        block_outline.center_top() + egui::vec2(0.0, BLOCK_NAME_V_OFFSET * zoom),
        egui::Align2::CENTER_TOP,
        &model.name,
        egui::FontId::proportional(MODEL_NAME_FONT_SIZE * zoom),
        BLOCK_STROKE_COLOR,
    );
}

/// Draws the wired clock path from a clock drive point to the clock input of a flip-flop.
fn draw_ff_clock_path(
    ff_outline: &egui::Rect,
    clock_name: &str,
    clock_drive_points: &HashMap<String, egui::Pos2>,
    zoom: f32,
    ui: &mut egui::Ui,
) {
    match clock_drive_points.get(clock_name) {
        Some(drive_point) => {
            let steiner = egui::pos2(ff_outline.center().x, drive_point.y);
            ui.painter().line(
                vec![*drive_point, steiner, ff_outline.center_bottom()],
                egui::Stroke::new(SIGNAL_STROKE_WIDTH * zoom, CLOCK_COLOR),
            );
        }
        None => {
            ui.painter().debug_text(
                ff_outline.center_bottom(),
                egui::Align2::CENTER_CENTER,
                Color32::RED,
                format!("Unable to find clock: {clock_name}"),
            );
        }
    }
}

/// Draws a flip-flop symbol at `ff_outline`.
///
/// `setup_annotation` annotates the clock-to-D setup arc (relevant for input-side FFs).
/// `ctq_annotation` annotates the clock-to-Q propagation arc (relevant for output-side FFs).
fn draw_flip_flop(
    ff_outline: &egui::Rect,
    setup_annotation: &DelayAnnotation,
    ctq_annotation: &DelayAnnotation,
    show_setup_constraints: bool,
    show_clock_to_q: bool,
    zoom: f32,
    ui: &mut egui::Ui,
) {
    ui.painter().rect(
        *ff_outline,
        egui::CornerRadius::same((4.0 * zoom) as u8),
        FF_FILL,
        egui::Stroke::new(FF_STROKE_WIDTH * zoom, FF_STROKE_COLOR),
        egui::epaint::StrokeKind::Middle,
    );

    // Draw the clock triangle at the bottom center of the FF.
    let triangle_base_length = ff_outline.width() * 2.0 / 5.0;
    let triangle_t = ff_outline.center_bottom() - egui::vec2(0.0, triangle_base_length);
    let triangle_bl = ff_outline.center_bottom() - egui::vec2(triangle_base_length / 2.0, 0.0);
    let triangle_br = ff_outline.center_bottom() + egui::vec2(triangle_base_length / 2.0, 0.0);
    ui.painter().add(egui::Shape::convex_polygon(
        vec![triangle_bl, triangle_t, triangle_br],
        CLOCK_FILL,
        egui::Stroke::new(FF_STROKE_WIDTH * zoom, CLOCK_COLOR),
    ));

    // Draw setup (clock→D) and clock-to-Q (clock→Q) arcs inside the FF symbol.
    let d_port = ff_outline.left_top() + egui::vec2(0.0, FF_PORT_OFFSET * zoom);
    let q_port = ff_outline.right_top() + egui::vec2(0.0, FF_PORT_OFFSET * zoom);
    let control = egui::pos2(triangle_t.x, d_port.y);

    // Use the FF outline's top-left corner (as bit-pattern) to build unique IDs across
    // multiple FF instances on the same canvas.
    let ff_id_x = ff_outline.min.x.to_bits();
    let ff_id_y = ff_outline.min.y.to_bits();

    if show_setup_constraints {
        let arc_color = annotation_stroke_color(SETUP_COLOR, setup_annotation);
        ui.painter().add(QuadraticBezierShape::from_points_stroke(
            [triangle_t, control, d_port],
            false,
            Color32::TRANSPARENT,
            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH * zoom, arc_color),
        ));
        let midpoint = bezier_midpoint(triangle_t, control, d_port);
        draw_timing_label(
            ui,
            midpoint,
            setup_annotation,
            zoom,
            egui::Id::new(("ff_setup", ff_id_x, ff_id_y)),
        );
    }
    if show_clock_to_q {
        let arc_color = annotation_stroke_color(CLOCK_TO_Q_COLOR, ctq_annotation);
        ui.painter().add(QuadraticBezierShape::from_points_stroke(
            [triangle_t, control, q_port],
            false,
            Color32::TRANSPARENT,
            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH * zoom, arc_color),
        ));
        let midpoint = bezier_midpoint(triangle_t, control, q_port);
        draw_timing_label(
            ui,
            midpoint,
            ctq_annotation,
            zoom,
            egui::Id::new(("ff_ctq", ff_id_x, ff_id_y)),
        );
    }
}

fn legend_entry(ui: &mut egui::Ui, label: &str, color: Color32) {
    ui.horizontal(|ui| {
        // Indent to align with constraint checkboxes (checkbox width ~20px + spacing)
        ui.add_space(24.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 14.0), egui::Sense::empty());
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            egui::Stroke::new(SIGNAL_STROKE_WIDTH, color),
        );
        ui.add_space(4.0);
        ui.label(label);
    });
}

fn constraint_checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.checkbox(value, "");
        let dimmed = if *value {
            color
        } else {
            color.gamma_multiply(0.35)
        };
        let (rect, swatch_response) =
            ui.allocate_exact_size(egui::vec2(28.0, 14.0), egui::Sense::click());
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            egui::Stroke::new(CONSTRAINT_STROKE_WIDTH, dimmed),
        );
        ui.add_space(4.0);
        let text_color = if *value {
            ui.visuals().text_color()
        } else {
            ui.visuals().weak_text_color()
        };
        let label_response = ui.add(
            egui::Label::new(egui::RichText::new(label).color(text_color))
                .sense(egui::Sense::click()),
        );
        if swatch_response.clicked() || label_response.clicked() {
            *value = !*value;
        }
    });
}

/// Recursively collects all `PBType` leaf nodes in `pb_type`'s subtree whose
/// `blif_model` references `model_name` (i.e. equals `.subckt <model_name>`).
fn collect_pb_types_for_model<'a>(
    pb_type: &'a PBType,
    model_name: &str,
    path: &mut Vec<String>,
    results: &mut Vec<PBTypeMatch<'a>>,
) {
    path.push(pb_type.name.clone());
    // Built-in models (.input, .output, .latch, .names) are referenced by their name
    // directly as the blif_model value.  Custom models are referenced as ".subckt <name>".
    let blif_model_matches = if model_name.starts_with('.') {
        pb_type.blif_model.as_deref() == Some(model_name)
    } else {
        pb_type.blif_model.as_deref() == Some(&format!(".subckt {model_name}"))
    };
    if blif_model_matches {
        results.push(PBTypeMatch {
            path: path.clone(),
            pb_type,
        });
    }
    for child in &pb_type.pb_types {
        collect_pb_types_for_model(child, model_name, path, results);
    }
    for mode in &pb_type.modes {
        for child in &mode.pb_types {
            collect_pb_types_for_model(child, model_name, path, results);
        }
    }
    path.pop();
}

/// Returns all `PBType`s in the complex block hierarchy that reference `model_name`.
fn find_pb_types_for_model<'a>(arch: &'a FPGAArch, model_name: &str) -> Vec<PBTypeMatch<'a>> {
    let mut results = Vec::new();
    let mut path = Vec::new();
    for root in &arch.complex_block_list {
        collect_pb_types_for_model(root, model_name, &mut path, &mut results);
    }
    results
}

fn is_sequential_block(model: &Model) -> bool {
    // A sequential block has no combinational timing paths between input and output ports.
    model
        .input_ports
        .iter()
        .chain(model.output_ports.iter())
        .all(|p| p.combinational_sink_ports.is_empty())
}
