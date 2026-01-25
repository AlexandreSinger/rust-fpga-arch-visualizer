use eframe::egui;
use fpga_arch_parser::{FPGAArch, FPGAArchParseError};
use std::io::{BufRead, BufReader};

use crate::block_style::DefaultBlockStyles;
use crate::common_ui;
use crate::grid_view::GridView;
use crate::complex_block_view::ComplexBlockView;
use crate::settings;
use crate::summary_view::SummaryView;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Summary,
    Grid,
    ComplexBlock,
}

// NOTE: These act more like tabs, so while you are looking at settings,
//       the main page stays around in the background.
// TODO: We should make these actual tabs.
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Main,
    Settings,
}

pub struct ViewerContext {
    pub show_about: bool,
    pub current_page: Page,
    // Navigation state
    pub navigation_history: Vec<ViewMode>,
    pub skip_nav_history_update: bool,
    // Block styles
    pub block_styles: DefaultBlockStyles,
    // Currently loaded architecture file path
    pub loaded_file_path: Option<std::path::PathBuf>,
    // Cache the last window title we set
    pub window_title: String,
    // Theme setting
    pub dark_mode: bool,
    // Error window state
    pub show_error: bool,
    pub error_title: String,
    pub error_message: String,
}

pub struct FpgaViewer {
    // Parsed architecture
    pub architecture: Option<FPGAArch>,
    viewer_ctx: ViewerContext,

    summary_view: SummaryView,    
    grid_view: GridView,
    complex_block_view: ComplexBlockView,

    view_mode: ViewMode,
    next_view_mode: ViewMode,

}

fn get_file_line(file_path: &std::path::Path, line_num: u64) -> Option<String> {
    let file = std::fs::File::open(file_path).ok()?;
    let reader = BufReader::new(file);
    let target = line_num.saturating_sub(1) as usize;
    reader.lines().nth(target).and_then(Result::ok)
}

fn format_context_line(line: &str, column: u64) -> String {
    let column = column as usize;
    let mut result = format!("  {}\n", line);
    let offset = column.saturating_sub(1).min(line.len());
    result.push_str(&format!("  {}{}", " ".repeat(offset), "^"));
    result
}

fn format_parse_error(error: &FPGAArchParseError, file_path: Option<&std::path::Path>) -> String {
    match error {
        FPGAArchParseError::ArchFileOpenError(msg) => {
            format!("Failed to open architecture file:\n{}", msg)
        }
        FPGAArchParseError::MissingRequiredTag(tag) => {
            format!("Missing required XML tag: {}", tag)
        }
        FPGAArchParseError::MissingRequiredAttribute(attr, pos) => {
            let mut msg = format!(
                "Missing required attribute '{}' at line {}, column {}",
                attr, pos.row + 1, pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::InvalidTag(tag, pos) => {
            let mut msg = format!(
                "Invalid or unexpected tag '{}' at line {}, column {}",
                tag, pos.row + 1, pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::XMLParseError(msg_text, pos) => {
            let mut msg = format!(
                "XML parsing error at line {}, column {}:\n{}",
                pos.row + 1, pos.column + 1, msg_text
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::UnknownAttribute(attr, pos) => {
            let mut msg = format!(
                "Unknown attribute '{}' at line {}, column {}",
                attr, pos.row + 1, pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::DuplicateTag(tag, pos) => {
            let mut msg = format!(
                "Duplicate tag '{}' at line {}, column {}",
                tag, pos.row + 1, pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::DuplicateAttribute(attr, pos) => {
            let mut msg = format!(
                "Duplicate attribute '{}' at line {}, column {}",
                attr, pos.row + 1, pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::UnexpectedEndTag(tag, pos) => {
            let mut msg = format!(
                "Unexpected end tag '</{}>' at line {}, column {}",
                tag, pos.row + 1, pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::AttributeParseError(msg_text, pos) => {
            let mut msg = format!(
                "Failed to parse attribute at line {}, column {}:\n{}",
                pos.row + 1, pos.column + 1, msg_text
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1) {
                    msg.push_str("\n\n");
                    msg.push_str(&format_context_line(&line, pos.column + 1));
                }
            msg
        }
        FPGAArchParseError::UnexpectedEndOfDocument(msg) => {
            format!("Unexpected end of document:\n{}", msg)
        }
    }
}

impl FpgaViewer {
    pub fn new() -> Self {
        Self {
            architecture: None,
            viewer_ctx: ViewerContext {
                show_about: false,
                current_page: Page::Main,
                navigation_history: Vec::new(),
                skip_nav_history_update: false,
                block_styles: DefaultBlockStyles::new(),
                loaded_file_path: None,
                window_title: "FPGA Architecture Visualizer".to_string(),
                dark_mode: false,
                show_error: false,
                error_title: String::new(),
                error_message: String::new(),
            },
            summary_view: SummaryView::default(),
            grid_view: GridView::default(),
            complex_block_view: ComplexBlockView::default(),
            view_mode: ViewMode::Summary,
            next_view_mode: ViewMode::Summary,
        }
    }

    fn loaded_arch_filename(&self) -> Option<String> {
        self.viewer_ctx.loaded_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
    }

    fn desired_window_title(&self) -> String {
        match self.loaded_arch_filename() {
            Some(name) if !name.is_empty() => format!("FPGA Architecture Visualizer - {name}"),
            _ => "FPGA Architecture Visualizer".to_string(),
        }
    }

    fn load_architecture_file(&mut self, file_path: std::path::PathBuf) {
        match fpga_arch_parser::parse(&file_path) {
            Ok(arch) => {
                // Update views with new architecture.
                self.grid_view.on_architecture_load(&arch);

                // Update viewer context.
                self.architecture = Some(arch);
                self.viewer_ctx.show_error = false;
                self.viewer_ctx.error_title.clear();
                self.viewer_ctx.error_message.clear();

                // Print success.
                println!("Successfully loaded architecture file: {:?}", file_path);
            }
            Err(e) => {
                self.architecture = None;
                self.viewer_ctx.show_error = true;
                self.viewer_ctx.error_title = "Parse Error".to_owned();
                self.viewer_ctx.error_message = format!("Error loading architecture:\n{:?}\n\n{}", file_path, format_parse_error(&e, Some(&file_path)));
            }
        }

        // Since this is a tool for debugging architectures, we should remember
        // the path of the loaded file even if it fails so it can be fixed.
        self.viewer_ctx.loaded_file_path = Some(file_path);
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("XML Architecture Files", &["xml"])
            .set_title("Open FPGA Architecture File")
            .pick_file()
        {
            self.load_architecture_file(path);
        }
    }

    fn navigate_back(&mut self) {
        if self.viewer_ctx.current_page == Page::Settings {
            self.viewer_ctx.current_page = Page::Main;
            return;
        }

        // Navigate back in view mode history
        if let Some(previous_mode) = self.viewer_ctx.navigation_history.pop() {
            self.next_view_mode = previous_mode;
            self.viewer_ctx.skip_nav_history_update = true;
        }
    }

    fn open_settings(&mut self) {
        self.viewer_ctx.current_page = Page::Settings;
    }

    fn render_navigation_buttons(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation_buttons")
            .resizable(false)
            .default_width(80.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    const BUTTON_SIZE: f32 = 50.0;

                    let open_button = ui.add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new(egui::RichText::new("📁").size(30.0))
                            .frame(true)
                            .corner_radius(BUTTON_SIZE / 2.0),
                    );
                    if open_button.clicked() {
                        self.open_file_dialog();
                    }
                    if open_button.hovered() {
                        open_button.on_hover_text("Open architecture file");
                    }
                    ui.add_space(10.0);

                    let reload_enabled = self.viewer_ctx.loaded_file_path.is_some();
                    let reload_button = ui.add_enabled_ui(reload_enabled, |ui| {
                        ui.add_sized(
                            [BUTTON_SIZE, BUTTON_SIZE],
                            egui::Button::new(egui::RichText::new("🔄").size(24.0))
                                .frame(true)
                                .corner_radius(BUTTON_SIZE / 2.0),
                        )
                    });
                    if reload_button.inner.clicked()
                        && let Some(path) = self.viewer_ctx.loaded_file_path.clone() {
                            self.load_architecture_file(path);
                        }
                    if reload_button.inner.hovered() {
                        reload_button.inner.on_hover_text("Reload architecture file");
                    }
                    ui.add_space(10.0);

                    let settings_button = ui.add_sized(
                        [BUTTON_SIZE, BUTTON_SIZE],
                        egui::Button::new(egui::RichText::new("⚙").size(24.0))
                            .frame(true)
                            .corner_radius(BUTTON_SIZE / 2.0),
                    );
                    if settings_button.clicked() {
                        self.open_settings();
                    }
                    if settings_button.hovered() {
                        settings_button.on_hover_text("Open settings");
                    }
                    ui.add_space(10.0);

                    let back_enabled = self.viewer_ctx.current_page == Page::Settings
                        || !self.viewer_ctx.navigation_history.is_empty();
                    let back_button = ui.add_enabled_ui(back_enabled, |ui| {
                        ui.add_sized(
                            [BUTTON_SIZE, BUTTON_SIZE],
                            egui::Button::new(egui::RichText::new("◀").size(24.0))
                                .frame(true)
                                .corner_radius(BUTTON_SIZE / 2.0),
                        )
                    });
                    if back_button.inner.clicked() {
                        self.navigate_back();
                    }
                    if back_button.inner.hovered() {
                        if self.viewer_ctx.current_page == Page::Settings {
                            back_button.inner.on_hover_text("Back to main");
                        } else if self.view_mode == ViewMode::ComplexBlock {
                            back_button.inner.on_hover_text("Back to grid view");
                        } else {
                            back_button.inner.on_hover_text("Go back");
                        }
                    }
                });
            });
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Architecture File...").clicked() {
                        self.open_file_dialog();
                        ui.close();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Summary View").clicked() {
                        self.next_view_mode = ViewMode::Summary;
                        ui.close();
                    }
                    if ui.button("Grid View").clicked() {
                        self.next_view_mode = ViewMode::Grid;
                        ui.close();
                    }
                    if ui.button("Complex Block View").clicked() {
                        self.next_view_mode = ViewMode::ComplexBlock;
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.viewer_ctx.show_about = true;
                        ui.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(name) = self.loaded_arch_filename() {
                        ui.label(egui::RichText::new(name).strong());
                    } else {
                        ui.label(egui::RichText::new("No file loaded").weak());
                    }
                });
            });
        });
    }

    fn render_page(&mut self, ctx: &egui::Context) {
        match self.viewer_ctx.current_page {
            Page::Main => {
                self.render_main_page(ctx);
            },
            Page::Settings => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    settings::render_settings_page(ui, &self.viewer_ctx.block_styles, &mut self.viewer_ctx.dark_mode);
                });
            },
        }
    }

    fn render_main_page(&mut self, ctx: &egui::Context) {
        match &self.architecture {
            Some(arch) => match self.view_mode {
                ViewMode::Summary => self.summary_view.render(arch, &mut self.next_view_mode, ctx),
                ViewMode::Grid => self.grid_view.render(arch, &mut self.viewer_ctx, &mut self.complex_block_view.complex_block_view_state, &mut self.next_view_mode, ctx),
                ViewMode::ComplexBlock => self.complex_block_view.render(arch, &mut self.next_view_mode, self.viewer_ctx.dark_mode, ctx),
            },
            None => {
                // If no architecture is loaded, no view can be seen, so show a welcome message.
                egui::CentralPanel::default().show(ctx, |ui| {
                    common_ui::render_welcome_message(ui, &self.view_mode);
                });
            },
        }
    }

    fn render_error_window(&mut self, ctx: &egui::Context) {
        if !self.viewer_ctx.show_error {
            return;
        }

        egui::Window::new(&self.viewer_ctx.error_title)
            .collapsible(false)
            .resizable(true)
            .default_size([300.0, 150.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&self.viewer_ctx.error_message)
                            .color(egui::Color32::LIGHT_RED)
                            .monospace()
                    );
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("Close").clicked() {
                            self.viewer_ctx.show_error = false;
                        }
                    });
                });
            });
    }

    fn render_about_window(&mut self, ctx: &egui::Context) {
        if !self.viewer_ctx.show_about {
            return;
        }

        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .default_size([300.0, 150.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("FPGA Architecture Visualizer");
                    ui.add_space(10.0);
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(10.0);
                    ui.label(
                        "A Rust-based visualizer for VTR FPGA architecture description files.",
                    );
                    ui.add_space(10.0);
                    ui.label("Copyright (c) 2025 AlexandreSinger");
                    ui.label("Licensed under MIT License (SPDX: MIT)");
                    ui.add_space(20.0);
                    if ui.button("Close").clicked() {
                        self.viewer_ctx.show_about = false;
                    }
                });
            });
    }
}

impl eframe::App for FpgaViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        if self.viewer_ctx.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Update window title
        let desired_title = self.desired_window_title();
        if desired_title != self.viewer_ctx.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(desired_title.clone()));
            self.viewer_ctx.window_title = desired_title;
        }

        self.viewer_ctx.block_styles.update_colors(self.viewer_ctx.dark_mode);

        // Render UI panels and windows
        self.render_menu_bar(ctx);
        self.render_navigation_buttons(ctx);

        // Render the page.
        self.render_page(ctx);

        // Error window
        self.render_error_window(ctx);

        // About window
        self.render_about_window(ctx);

        // Next state logic for the view mode.
        if self.view_mode != self.next_view_mode {
            // Push current mode to history before transitioning
            if !self.viewer_ctx.skip_nav_history_update {
                self.viewer_ctx.navigation_history.push(self.view_mode);
            }
            self.viewer_ctx.skip_nav_history_update = false;

            // Run code on the close of a view.
            if self.view_mode == ViewMode::ComplexBlock { self.complex_block_view.on_view_close() }

            // Run code on the open of a view.
            if self.next_view_mode == ViewMode::ComplexBlock { self.complex_block_view.on_view_open(&self.architecture) }

            self.view_mode = self.next_view_mode;
        }
    }
}
