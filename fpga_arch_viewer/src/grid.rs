use fpga_arch_parser::{AutoLayout, GridLocation, FPGAArch};
use std::collections::HashMap;

// A single cell in the FPGA grid
#[derive(Debug, Clone, PartialEq)]
pub enum GridCell {
    Empty,
    // Anchor cell: the top-left cell of a tile (stores tile name, width, height)
    BlockAnchor {
        pb_type: String,
        width: usize,
        height: usize
    },
    // Occupied cell: part of a multi-cell tile, points to the anchor's coordinates
    BlockOccupied {
        pb_type: String,
        anchor_row: usize,
        anchor_col: usize
    },
}

// FPGA device grid
#[derive(Debug, Clone)]
pub struct DeviceGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GridCell>>,
    // Map from tile name to (width, height)
    tile_sizes: HashMap<String, (usize, usize)>,
}

impl DeviceGrid {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![GridCell::Empty; width]; height];
        Self {
            width,
            height,
            cells,
            tile_sizes: HashMap::new(),
        }
    }

    fn build_tile_size_map(arch: &FPGAArch) -> HashMap<String, (usize, usize)> {
        let mut tile_sizes = HashMap::new();
        for tile in &arch.tiles {
            tile_sizes.insert(
                tile.name.clone(),
                (tile.width as usize, tile.height as usize)
            );
        }
        tile_sizes
    }

    #[allow(dead_code)]
    pub fn from_auto_layout(arch: &FPGAArch, default_size: usize) -> Self {
        let auto_layout = match arch.layouts.first() {
            Some(fpga_arch_parser::Layout::AutoLayout(al)) => al,
            _ => panic!("Expected AutoLayout"),
        };

        let tile_sizes = Self::build_tile_size_map(arch);
        let (width, height) = Self::calculate_dimensions(auto_layout, default_size);
        Self::from_auto_layout_impl(auto_layout, width, height, tile_sizes)
    }

    pub fn from_auto_layout_with_dimensions(arch: &FPGAArch, width: usize, height: usize) -> Self {
        let auto_layout = match arch.layouts.first() {
            Some(fpga_arch_parser::Layout::AutoLayout(al)) => al,
            _ => panic!("Expected AutoLayout"),
        };

        let tile_sizes = Self::build_tile_size_map(arch);
        Self::from_auto_layout_impl(auto_layout, width, height, tile_sizes)
    }

    pub fn from_fixed_layout(arch: &FPGAArch, layout_index: usize) -> Self {
        let fixed_layout = match arch.layouts.get(layout_index) {
            Some(fpga_arch_parser::Layout::FixedLayout(fl)) => fl,
            _ => panic!("Expected FixedLayout at index {}", layout_index),
        };

        let tile_sizes = Self::build_tile_size_map(arch);
        let width = fixed_layout.width as usize;
        let height = fixed_layout.height as usize;

        let mut grid = Self {
            width,
            height,
            cells: vec![vec![GridCell::Empty; width]; height],
            tile_sizes,
        };

        // Sort locations by priority
        let mut location_indices: Vec<_> = fixed_layout.grid_locations
            .iter()
            .enumerate()
            .map(|(i, loc)| {
                let priority = match loc {
                    GridLocation::Fill(f) => f.priority,
                    GridLocation::Perimeter(p) => p.priority,
                    GridLocation::Corners(c) => c.priority,
                    GridLocation::Single(s) => s.priority,
                    GridLocation::Col(c) => c.priority,
                    GridLocation::Row(r) => r.priority,
                    GridLocation::Region(r) => r.priority,
                };
                (priority, i)
            })
            .collect();

        location_indices.sort_by_key(|(priority, _)| *priority);

        for (_, idx) in location_indices {
            grid.apply_grid_location(&fixed_layout.grid_locations[idx]);
        }

        grid
    }

    #[allow(dead_code)]
    fn calculate_dimensions(auto_layout: &AutoLayout, default_size: usize) -> (usize, usize) {
        let aspect_ratio = auto_layout.aspect_ratio;

        if aspect_ratio >= 1.0 {
            let width = default_size;
            let height = (default_size as f32 / aspect_ratio).round() as usize;
            (width, height.max(1))
        } else {
            let height = default_size;
            let width = (default_size as f32 * aspect_ratio).round() as usize;
            (width.max(1), height)
        }
    }

    fn from_auto_layout_impl(
        auto_layout: &AutoLayout,
        width: usize,
        height: usize,
        tile_sizes: HashMap<String, (usize, usize)>
    ) -> Self {
        let mut grid = Self {
            width,
            height,
            cells: vec![vec![GridCell::Empty; width]; height],
            tile_sizes,
        };

        // Sort locations by priority
        let mut location_indices: Vec<_> = auto_layout.grid_locations
            .iter()
            .enumerate()
            .map(|(i, loc)| {
                let priority = match loc {
                    GridLocation::Fill(f) => f.priority,
                    GridLocation::Perimeter(p) => p.priority,
                    GridLocation::Corners(c) => c.priority,
                    GridLocation::Single(s) => s.priority,
                    GridLocation::Col(c) => c.priority,
                    GridLocation::Row(r) => r.priority,
                    GridLocation::Region(r) => r.priority,
                };
                (priority, i)
            })
            .collect();

        location_indices.sort_by_key(|(priority, _)| *priority);

        for (_, idx) in location_indices {
            grid.apply_grid_location(&auto_layout.grid_locations[idx]);
        }

        grid
    }

    fn get_tile_size(&self, pb_type: &str) -> (usize, usize) {
        self.tile_sizes.get(pb_type).copied().unwrap_or((1, 1))
    }

    /// Place a multi-cell tile at the given position
    /// Returns true if successfully placed, false if there wasn't enough space
    fn place_tile(&mut self, row: usize, col: usize, pb_type: &str) -> bool {
        if pb_type == "EMPTY" {
            self.cells[row][col] = GridCell::Empty;
            return true;
        }

        let (tile_width, tile_height) = self.get_tile_size(pb_type);

        // Check if there's enough space
        if row + tile_height > self.height || col + tile_width > self.width {
            return false;
        }

        // First, find all tiles that will be intersected and clear them entirely
        let mut tiles_to_clear = Vec::new();
        for dy in 0..tile_height {
            for dx in 0..tile_width {
                let check_row = row + dy;
                let check_col = col + dx;
                if check_row < self.height && check_col < self.width {
                    match &self.cells[check_row][check_col] {
                        GridCell::BlockAnchor { .. } => {
                            // Found an anchor that will be overwritten
                            tiles_to_clear.push((check_row, check_col));
                        }
                        GridCell::BlockOccupied { anchor_row, anchor_col, .. } => {
                            // Found an occupied cell - need to clear its anchor
                            tiles_to_clear.push((*anchor_row, *anchor_col));
                        }
                        GridCell::Empty => {}
                    }
                }
            }
        }

        // Clear all intersected tiles completely
        for (anchor_row, anchor_col) in tiles_to_clear {
            if let GridCell::BlockAnchor { width, height, .. } = &self.cells[anchor_row][anchor_col] {
                let old_width = *width;
                let old_height = *height;
                // Clear the entire old tile
                for dy in 0..old_height {
                    for dx in 0..old_width {
                        let clear_row = anchor_row + dy;
                        let clear_col = anchor_col + dx;
                        if clear_row < self.height && clear_col < self.width {
                            self.cells[clear_row][clear_col] = GridCell::Empty;
                        }
                    }
                }
            }
        }

        // Place the anchor cell
        self.cells[row][col] = GridCell::BlockAnchor {
            pb_type: pb_type.to_string(),
            width: tile_width,
            height: tile_height,
        };

        // Place occupied cells
        for dy in 0..tile_height {
            for dx in 0..tile_width {
                if dx == 0 && dy == 0 {
                    continue; // Skip anchor cell
                }
                if row + dy < self.height && col + dx < self.width {
                    self.cells[row + dy][col + dx] = GridCell::BlockOccupied {
                        pb_type: pb_type.to_string(),
                        anchor_row: row,
                        anchor_col: col,
                    };
                }
            }
        }

        true
    }

    fn apply_grid_location(&mut self, location: &GridLocation) {
        match location {
            GridLocation::Fill(fill) => {
                let (tile_width, tile_height) = self.get_tile_size(&fill.pb_type);
                let mut row = 0;
                while row < self.height {
                    let mut col = 0;
                    while col < self.width {
                        if self.place_tile(row, col, &fill.pb_type) {
                            col += tile_width;
                        } else {
                            col += 1;
                        }
                    }
                    row += tile_height;
                }
            }
            GridLocation::Perimeter(perimeter) => {
                let (tile_width, tile_height) = self.get_tile_size(&perimeter.pb_type);
                // Top edge
                let mut col = 0;
                while col < self.width {
                    if self.place_tile(0, col, &perimeter.pb_type) {
                        col += tile_width;
                    } else {
                        col += 1;
                    }
                }

                // Bottom edge
                if self.height > 1 {
                    let mut col = 0;
                    while col < self.width {
                        if self.place_tile(self.height - 1, col, &perimeter.pb_type) {
                            col += tile_width;
                        } else {
                            col += 1;
                        }
                    }
                }

                // Left edge
                let mut row = 0;
                while row < self.height {
                    if self.place_tile(row, 0, &perimeter.pb_type) {
                        row += tile_height;
                    } else {
                        row += 1;
                    }
                }

                // Right edge
                if self.width > 1 {
                    let mut row = 0;
                    while row < self.height {
                        if self.place_tile(row, self.width - 1, &perimeter.pb_type) {
                            row += tile_height;
                        } else {
                            row += 1;
                        }
                    }
                }
            }
            GridLocation::Corners(corners) => {
                let corners_positions = [
                    (0, 0),
                    (0, self.width.saturating_sub(1)),
                    (self.height.saturating_sub(1), 0),
                    (self.height.saturating_sub(1), self.width.saturating_sub(1)),
                ];

                for (row, col) in corners_positions {
                    if row < self.height && col < self.width {
                        self.place_tile(row, col, &corners.pb_type);
                    }
                }
            }
            GridLocation::Single(single) => {
                let (tile_width, tile_height) = self.get_tile_size(&single.pb_type);
                if let (Some(x), Some(y)) = (
                    self.eval_expr(&single.x_expr, tile_width, tile_height),
                    self.eval_expr(&single.y_expr, tile_width, tile_height),
                ) {
                    if y < self.height && x < self.width {
                        self.place_tile(y, x, &single.pb_type);
                    }
                }
            }
            GridLocation::Col(col_loc) => {
                let (tile_width, tile_height) = self.get_tile_size(&col_loc.pb_type);
                if let (Some(start_x), Some(start_y)) = (
                    self.eval_expr(&col_loc.start_x_expr, tile_width, tile_height),
                    self.eval_expr(&col_loc.start_y_expr, tile_width, tile_height),
                ) {
                    let incr_y = self.eval_expr(&col_loc.incr_y_expr, tile_width, tile_height)
                        .unwrap_or(self.height);
                    let repeat_x = col_loc.repeat_x_expr.as_ref()
                        .and_then(|expr| self.eval_expr(expr, tile_width, tile_height))
                        .unwrap_or(self.width);

                    for x in (start_x..self.width).step_by(repeat_x) {
                        for y in (start_y..self.height).step_by(incr_y) {
                            self.place_tile(y, x, &col_loc.pb_type);
                        }
                    }
                }
            }
            GridLocation::Row(row_loc) => {
                let (tile_width, tile_height) = self.get_tile_size(&row_loc.pb_type);
                if let (Some(start_x), Some(start_y)) = (
                    self.eval_expr(&row_loc.start_x_expr, tile_width, tile_height),
                    self.eval_expr(&row_loc.start_y_expr, tile_width, tile_height),
                ) {
                    let incr_x = self.eval_expr(&row_loc.incr_x_expr, tile_width, tile_height)
                        .unwrap_or(self.width);
                    let repeat_y = row_loc.repeat_y_expr.as_ref()
                        .and_then(|expr| self.eval_expr(expr, tile_width, tile_height))
                        .unwrap_or(self.height);

                    for y in (start_y..self.height).step_by(repeat_y) {
                        for x in (start_x..self.width).step_by(incr_x) {
                            self.place_tile(y, x, &row_loc.pb_type);
                        }
                    }
                }
            }
            GridLocation::Region(region) => {
                let (tile_width, tile_height) = self.get_tile_size(&region.pb_type);
                if let (Some(start_x), Some(end_x), Some(start_y), Some(end_y)) = (
                    self.eval_expr(&region.start_x_expr, tile_width, tile_height),
                    self.eval_expr(&region.end_x_expr, tile_width, tile_height),
                    self.eval_expr(&region.start_y_expr, tile_width, tile_height),
                    self.eval_expr(&region.end_y_expr, tile_width, tile_height),
                ) {
                    let incr_x = self.eval_expr(&region.incr_x_expr, tile_width, tile_height)
                        .unwrap_or(end_x - start_x + 1);
                    let incr_y = self.eval_expr(&region.incr_y_expr, tile_width, tile_height)
                        .unwrap_or(end_y - start_y + 1);

                    for y in (start_y..=end_y.min(self.height - 1)).step_by(incr_y) {
                        for x in (start_x..=end_x.min(self.width - 1)).step_by(incr_x) {
                            self.place_tile(y, x, &region.pb_type);
                        }
                    }
                }
            }
        }
    }

    fn eval_expr(&self, expr: &str, tile_width: usize, tile_height: usize) -> Option<usize> {
        let expr = expr.trim();
        // Replace lowercase w/h with tile dimensions
        let expr = expr.replace('w', &tile_width.to_string());
        let expr = expr.replace('h', &tile_height.to_string());
        // Replace uppercase W/H with grid dimensions
        let expr = expr.replace('W', &self.width.to_string());
        let expr = expr.replace('H', &self.height.to_string());
        let expr = expr.replace(' ', "");
        self.eval_expr_recursive(&expr)
    }

    fn eval_expr_recursive(&self, expr: &str) -> Option<usize> {
        let expr = expr.trim();

        if let Ok(val) = expr.parse::<i32>() {
            return if val >= 0 { Some(val as usize) } else { None };
        }

        let chars: Vec<char> = expr.chars().collect();
        let len = chars.len();

        // Addition and subtraction (lowest precedence)
        let mut depth = 0;
        for i in (0..len).rev() {
            let ch = chars[i];
            match ch {
                ')' => depth += 1,
                '(' => depth -= 1,
                '+' | '-' if depth == 0 && i > 0 => {
                    let left: String = chars[..i].iter().collect();
                    let right: String = chars[i + 1..].iter().collect();

                    if let (Some(l), Some(r)) = (self.eval_expr_recursive(&left), self.eval_expr_recursive(&right)) {
                        return if ch == '+' {
                            Some(l + r)
                        } else if l >= r {
                            Some(l - r)
                        } else {
                            None
                        };
                    }
                }
                _ => {}
            }
        }

        // Multiplication and division (higher precedence)
        depth = 0;
        for i in (0..len).rev() {
            let ch = chars[i];
            match ch {
                ')' => depth += 1,
                '(' => depth -= 1,
                '*' | '/' if depth == 0 && i > 0 => {
                    let left: String = chars[..i].iter().collect();
                    let right: String = chars[i + 1..].iter().collect();

                    if let (Some(l), Some(r)) = (self.eval_expr_recursive(&left), self.eval_expr_recursive(&right)) {
                        return if ch == '*' {
                            Some(l * r)
                        } else if r > 0 {
                            Some(l / r)
                        } else {
                            None
                        };
                    }
                }
                _ => {}
            }
        }

        // Handle parentheses
        if expr.starts_with('(') && expr.ends_with(')') {
            return self.eval_expr_recursive(&expr[1..expr.len() - 1]);
        }

        None
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&GridCell> {
        if row < self.height && col < self.width {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }
}
