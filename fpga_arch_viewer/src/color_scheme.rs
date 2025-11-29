//! Intra Tile Color Scheme
//!
//! This module contains all color definitions for the intra-tile visualization.

use eframe::egui;

// ----------------------------------------------------------------------------
// Intra Tile Colors
// ----------------------------------------------------------------------------

/// Text color that adapts to theme
pub fn theme_text_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    }
}

/// Header background color
pub fn theme_header_bg(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(60, 60, 60)
    } else {
        egui::Color32::from_rgb(200, 200, 200)
    }
}

/// Block background color
pub fn theme_block_bg(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(40, 40, 40)
    } else {
        egui::Color32::from_rgb(240, 240, 240)
    }
}

/// Border color for blocks
pub fn theme_border_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(120, 120, 120)
    } else {
        egui::Color32::from_rgb(100, 100, 100)
    }
}

/// Interconnect/wire background color (with transparency)
pub fn theme_interconnect_bg(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgba_unmultiplied(80, 80, 80, 150)
    } else {
        egui::Color32::from_rgba_unmultiplied(100, 100, 100, 150)
    }
}

// =======================
// Block-Specific Colors
// =======================
/// LUT colors
pub struct LutColors {
    pub bg: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
}

pub fn lut_colors(dark_mode: bool) -> LutColors {
    LutColors {
        bg: if dark_mode {
            egui::Color32::from_rgb(100, 100, 50)
        } else {
            egui::Color32::from_rgb(255, 250, 205)
        },
        border: if dark_mode {
            egui::Color32::from_rgb(150, 150, 0)
        } else {
            egui::Color32::from_rgb(180, 180, 0)
        },
        text: if dark_mode {
            egui::Color32::from_rgb(150, 150, 0)
        } else {
            egui::Color32::from_rgb(180, 180, 0)
        },
    }
}

/// Flip-Flop block colors
pub struct FlipFlopColors {
    pub bg: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
}

pub fn flip_flop_colors(dark_mode: bool) -> FlipFlopColors {
    FlipFlopColors {
        bg: if dark_mode {
            egui::Color32::from_rgb(50, 60, 100)
        } else {
            egui::Color32::from_rgb(220, 230, 255)
        },
        border: if dark_mode {
            egui::Color32::from_rgb(100, 100, 255)
        } else {
            egui::Color32::from_rgb(0, 0, 180)
        },
        text: if dark_mode {
            egui::Color32::from_rgb(100, 100, 255)
        } else {
            egui::Color32::from_rgb(0, 0, 180)
        },
    }
}

/// Memory block colors
pub struct MemoryColors {
    pub bg: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
    pub grid: egui::Color32,
}

pub fn memory_colors(dark_mode: bool) -> MemoryColors {
    let border = if dark_mode {
        egui::Color32::from_rgb(0, 150, 0)
    } else {
        egui::Color32::from_rgb(0, 100, 0)
    };
    MemoryColors {
        bg: if dark_mode {
            egui::Color32::from_rgb(50, 100, 50)
        } else {
            egui::Color32::from_rgb(200, 240, 200)
        },
        border,
        text: border,
        grid: border,
    }
}

/// BLIF block colors
pub struct BlifColors {
    pub bg: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
}

pub fn blif_colors(dark_mode: bool) -> BlifColors {
    BlifColors {
        bg: if dark_mode {
            egui::Color32::from_rgb(100, 50, 50)
        } else {
            egui::Color32::from_rgb(255, 220, 220)
        },
        border: if dark_mode {
            egui::Color32::from_rgb(200, 0, 0)
        } else {
            egui::Color32::from_rgb(180, 0, 0)
        },
        text: if dark_mode {
            egui::Color32::from_rgb(200, 0, 0)
        } else {
            egui::Color32::from_rgb(180, 0, 0)
        },
    }
}

// =======================
// Special Colors
// =======================

/// Highlight color for hovered/selected elements
pub const HIGHLIGHT_COLOR: egui::Color32 = egui::Color32::RED;

/// Input pin color
pub const PIN_COLOR: egui::Color32 = egui::Color32::BLACK;

/// Clock pin color
pub const CLOCK_PIN_COLOR: egui::Color32 = egui::Color32::RED;

// ----------------------------------------------------------------------------
// Grid Tile Colors (Inter-Tile View)
// ----------------------------------------------------------------------------

/// IO block
pub fn grid_io_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(80, 80, 80) // Dark gray
    } else {
        egui::Color32::from_rgb(0xF5, 0xF5, 0xF5) // #F5F5F5 - Light gray
    }
}

/// Logic Block
pub fn grid_lb_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(60, 70, 120) // Dark blue
    } else {
        egui::Color32::from_rgb(0xD8, 0xE7, 0xFD) // #D8E7FD - Light blue
    }
}

/// Switch Block
pub fn grid_sb_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(120, 90, 60) // Dark orange/brown
    } else {
        egui::Color32::from_rgb(0xFF, 0xE6, 0xCE) // #FFE6CE - Light orange
    }
}

/// Connection Block
pub fn grid_cb_color(dark_mode: bool) -> egui::Color32 {
    if dark_mode {
        egui::Color32::from_rgb(120, 110, 70) // Dark yellow/brown
    } else {
        egui::Color32::from_rgb(0xFF, 0xF3, 0xCC) // #FFF3CC - Light yellow
    }
}
