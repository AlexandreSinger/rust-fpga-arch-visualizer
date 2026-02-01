use fpga_arch_parser::{AutoLayout, FPGAArch, GridLocation};
use std::collections::HashMap;

// A single cell in the FPGA grid
#[derive(Debug, Clone, PartialEq)]
pub enum GridCell {
    Empty,
    // Anchor cell: the top-left cell of a tile (stores tile name, width, height)
    BlockAnchor {
        pb_type: String,
        width: usize,
        height: usize,
    },
    // Occupied cell: part of a multi-cell tile, points to the anchor's coordinates
    BlockOccupied {
        pb_type: String,
        anchor_row: usize,
        anchor_col: usize,
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

    // The priority of each cell currently placed on the grid.
    // This is used when building the grid.
    grid_priorities: Vec<Vec<i32>>,
}

impl DeviceGrid {
    fn build_tile_size_map(arch: &FPGAArch) -> HashMap<String, (usize, usize)> {
        let mut tile_sizes = HashMap::new();
        for tile in &arch.tiles {
            tile_sizes.insert(
                tile.name.clone(),
                (tile.width as usize, tile.height as usize),
            );
        }
        tile_sizes
    }

    pub fn from_auto_layout_with_dimensions(arch: &FPGAArch, width: usize, height: usize) -> Self {
        let auto_layout = match arch.layouts.layout_list.first() {
            Some(fpga_arch_parser::Layout::AutoLayout(al)) => al,
            _ => panic!("Expected AutoLayout"),
        };

        let tile_sizes = Self::build_tile_size_map(arch);
        Self::from_auto_layout_impl(auto_layout, width, height, tile_sizes)
    }

    pub fn from_fixed_layout(arch: &FPGAArch, layout_index: usize) -> Self {
        let fixed_layout = match arch.layouts.layout_list.get(layout_index) {
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
            grid_priorities: vec![vec![i32::MIN; width]; height],
        };

        for grid_location in &fixed_layout.grid_locations {
            grid.apply_grid_location(grid_location);
        }

        grid
    }

    fn from_auto_layout_impl(
        auto_layout: &AutoLayout,
        width: usize,
        height: usize,
        tile_sizes: HashMap<String, (usize, usize)>,
    ) -> Self {
        let mut grid = Self {
            width,
            height,
            cells: vec![vec![GridCell::Empty; width]; height],
            tile_sizes,
            grid_priorities: vec![vec![i32::MIN; width]; height],
        };

        for grid_location in &auto_layout.grid_locations {
            grid.apply_grid_location(grid_location);
        }

        grid
    }

    fn get_tile_size(&self, pb_type: &str) -> (usize, usize) {
        self.tile_sizes.get(pb_type).copied().unwrap_or((1, 1))
    }

    /// Place a multi-cell tile at the given position
    /// Returns true if successfully placed, false if there wasn't enough space
    fn place_tile(&mut self, row: usize, col: usize, pb_type: &str, priority: i32) -> bool {
        let (tile_width, tile_height) = self.get_tile_size(pb_type);

        // Check if there's enough space
        if row + tile_height > self.height || col + tile_width > self.width {
            return false;
        }

        // Get the max priority of all tiles that will be intersected.
        let mut max_priority = i32::MIN;
        for dy in 0..tile_height {
            for dx in 0..tile_width {
                let check_row = row + dy;
                let check_col = col + dx;
                if check_row < self.height && check_col < self.width {
                    max_priority = std::cmp::max(max_priority, self.grid_priorities[check_row][check_col]);
                }
            }
        }

        // If this has a lower priority, we do not override.
        // NOTE: To match VTR's functionality, in the case of a tie, we override
        //       since we want later XML tags to override ties.
        if priority < max_priority {
            return false;
        }

        // Find all tiles that will be intersected and clear them entirely
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
                        GridCell::BlockOccupied {
                            anchor_row,
                            anchor_col,
                            ..
                        } => {
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
            if let GridCell::BlockAnchor { width, height, .. } = &self.cells[anchor_row][anchor_col]
            {
                let old_width = *width;
                let old_height = *height;
                // Clear the entire old tile
                for dy in 0..old_height {
                    for dx in 0..old_width {
                        let clear_row = anchor_row + dy;
                        let clear_col = anchor_col + dx;
                        if clear_row < self.height && clear_col < self.width {
                            self.cells[clear_row][clear_col] = GridCell::Empty;
                            // NOTE: This is not the best idea since there may have been a lower
                            //       priority tile that is being overwritten; however, this is
                            //       how VPR appears to handle this currently. We want to match
                            //       VPR as much as possible.
                            self.grid_priorities[clear_row][clear_col] = i32::MIN;
                        }
                    }
                }
            }
        }

        // Place the anchor cell
        if pb_type == "EMPTY" {
            self.cells[row][col] = GridCell::Empty;
        } else {
            self.cells[row][col] = GridCell::BlockAnchor {
                pb_type: pb_type.to_string(),
                width: tile_width,
                height: tile_height,
            };
        }
        self.grid_priorities[row][col] = priority;

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
                    self.grid_priorities[row + dy][col + dx] = priority;
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
                        if self.place_tile(row, col, &fill.pb_type, fill.priority) {
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
                    if self.place_tile(0, col, &perimeter.pb_type, perimeter.priority) {
                        col += tile_width;
                    } else {
                        col += 1;
                    }
                }

                // Bottom edge
                if self.height > 1 {
                    let mut col = 0;
                    while col < self.width {
                        if self.place_tile(self.height - 1, col, &perimeter.pb_type, perimeter.priority) {
                            col += tile_width;
                        } else {
                            col += 1;
                        }
                    }
                }

                // Left edge
                let mut row = 0;
                while row < self.height {
                    if self.place_tile(row, 0, &perimeter.pb_type, perimeter.priority) {
                        row += tile_height;
                    } else {
                        row += 1;
                    }
                }

                // Right edge
                if self.width > 1 {
                    let mut row = 0;
                    while row < self.height {
                        if self.place_tile(row, self.width - 1, &perimeter.pb_type, perimeter.priority) {
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
                        self.place_tile(row, col, &corners.pb_type, corners.priority);
                    }
                }
            }
            GridLocation::Single(single) => {
                let (tile_width, tile_height) = self.get_tile_size(&single.pb_type);
                if let (Some(x), Some(y)) = (
                    self.eval_expr(&single.x_expr, tile_width, tile_height),
                    self.eval_expr(&single.y_expr, tile_width, tile_height),
                ) && y < self.height
                    && x < self.width
                {
                    self.place_tile(y, x, &single.pb_type, single.priority);
                }
            }
            GridLocation::Col(col_loc) => {
                let (tile_width, tile_height) = self.get_tile_size(&col_loc.pb_type);
                if let (Some(start_x), Some(start_y)) = (
                    self.eval_expr(&col_loc.start_x_expr, tile_width, tile_height),
                    self.eval_expr(&col_loc.start_y_expr, tile_width, tile_height),
                ) {
                    let incr_y = self
                        .eval_expr(&col_loc.incr_y_expr, tile_width, tile_height)
                        .unwrap_or(tile_height);
                    let repeat_x = col_loc
                        .repeat_x_expr
                        .as_ref()
                        .and_then(|expr| self.eval_expr(expr, tile_width, tile_height))
                        .unwrap_or(self.width);

                    for x in (start_x..self.width).step_by(repeat_x) {
                        for y in (start_y..self.height).step_by(incr_y) {
                            self.place_tile(y, x, &col_loc.pb_type, col_loc.priority);
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
                    let incr_x = self
                        .eval_expr(&row_loc.incr_x_expr, tile_width, tile_height)
                        .unwrap_or(tile_width);
                    let repeat_y = row_loc
                        .repeat_y_expr
                        .as_ref()
                        .and_then(|expr| self.eval_expr(expr, tile_width, tile_height))
                        .unwrap_or(self.height);

                    for y in (start_y..self.height).step_by(repeat_y) {
                        for x in (start_x..self.width).step_by(incr_x) {
                            self.place_tile(y, x, &row_loc.pb_type, row_loc.priority);
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
                    let incr_x = self
                        .eval_expr(&region.incr_x_expr, tile_width, tile_height)
                        .unwrap_or(end_x - start_x + 1);
                    let incr_y = self
                        .eval_expr(&region.incr_y_expr, tile_width, tile_height)
                        .unwrap_or(end_y - start_y + 1);

                    for y in (start_y..=end_y.min(self.height - 1)).step_by(incr_y) {
                        for x in (start_x..=end_x.min(self.width - 1)).step_by(incr_x) {
                            self.place_tile(y, x, &region.pb_type, region.priority);
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
        eval_expr_recursive(&expr)
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&GridCell> {
        if row < self.height && col < self.width {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }
}

fn eval_expr_recursive(expr: &str) -> Option<usize> {
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

                if let (Some(l), Some(r)) =
                    (eval_expr_recursive(&left), eval_expr_recursive(&right))
                {
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

                if let (Some(l), Some(r)) =
                    (eval_expr_recursive(&left), eval_expr_recursive(&right))
                {
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
        return eval_expr_recursive(&expr[1..expr.len() - 1]);
    }

    None
}
