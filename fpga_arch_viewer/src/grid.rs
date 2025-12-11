use fpga_arch_parser::{AutoLayout, GridLocation};

// A single cell in the FPGA grid
#[derive(Debug, Clone, PartialEq)]
pub enum GridCell {
    Empty,
    Block(String), // pb_type name (e.g., "io", "clb")
}

// FPGA device grid
#[derive(Debug, Clone)]
pub struct DeviceGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GridCell>>,
}

impl DeviceGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![GridCell::Empty; width]; height];
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn from_auto_layout(auto_layout: &AutoLayout, default_size: usize) -> Self {
        // Calculate width and height based on aspect_ratio
        // aspect_ratio = width / height
        let aspect_ratio = auto_layout.aspect_ratio;

        let (width, height) = if aspect_ratio >= 1.0 {
            // Width >= height (landscape or square)
            let width = default_size;
            let height = (default_size as f32 / aspect_ratio).round() as usize;
            (width, height.max(1)) // Ensure height is at least 1
        } else {
            // Height > width (portrait)
            let height = default_size;
            let width = (default_size as f32 * aspect_ratio).round() as usize;
            (width.max(1), height) // Ensure width is at least 1
        };

        let mut grid = Self::new(width, height);

        // (priority, index) pairs and sort by priority
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

    fn apply_grid_location(&mut self, location: &GridLocation) {
        match location {
            GridLocation::Fill(fill) => {
                for row in 0..self.height {
                    for col in 0..self.width {
                        if fill.pb_type == "EMPTY" {
                            self.cells[row][col] = GridCell::Empty;
                        } else {
                            self.cells[row][col] = GridCell::Block(fill.pb_type.clone());
                        }
                    }
                }
            }
            GridLocation::Perimeter(perimeter) => {
                for col in 0..self.width {
                    if perimeter.pb_type == "EMPTY" {
                        self.cells[0][col] = GridCell::Empty;
                    } else {
                        self.cells[0][col] = GridCell::Block(perimeter.pb_type.clone());
                    }

                    if perimeter.pb_type == "EMPTY" {
                        self.cells[self.height - 1][col] = GridCell::Empty;
                    } else {
                        self.cells[self.height - 1][col] = GridCell::Block(perimeter.pb_type.clone());
                    }
                }

                for row in 0..self.height {
                    if perimeter.pb_type == "EMPTY" {
                        self.cells[row][0] = GridCell::Empty;
                    } else {
                        self.cells[row][0] = GridCell::Block(perimeter.pb_type.clone());
                    }

                    if perimeter.pb_type == "EMPTY" {
                        self.cells[row][self.width - 1] = GridCell::Empty;
                    } else {
                        self.cells[row][self.width - 1] = GridCell::Block(perimeter.pb_type.clone());
                    }
                }
            }
            GridLocation::Corners(corners) => {
                let corners_positions = [
                    (0, 0),
                    (0, self.width - 1),
                    (self.height - 1, 0),
                    (self.height - 1, self.width - 1),
                ];

                for (row, col) in corners_positions {
                    if corners.pb_type == "EMPTY" {
                        self.cells[row][col] = GridCell::Empty;
                    } else {
                        self.cells[row][col] = GridCell::Block(corners.pb_type.clone());
                    }
                }
            }
            GridLocation::Single(single) => {
                if let (Some(x), Some(y)) = (
                    self.eval_expr(&single.x_expr),
                    self.eval_expr(&single.y_expr),
                ) {
                    if y < self.height && x < self.width {
                        if single.pb_type == "EMPTY" {
                            self.cells[y][x] = GridCell::Empty;
                        } else {
                            self.cells[y][x] = GridCell::Block(single.pb_type.clone());
                        }
                    }
                }
            }
            GridLocation::Col(col) => {
                if let (Some(start_x), Some(start_y)) = (
                    self.eval_expr(&col.start_x_expr),
                    self.eval_expr(&col.start_y_expr),
                ) {
                    let incr_y = self.eval_expr(&col.incr_y_expr).unwrap_or(1);
                    let repeat_x = col.repeat_x_expr.as_ref()
                        .and_then(|expr| self.eval_expr(expr))
                        .unwrap_or(1);

                    for x_offset in 0..repeat_x {
                        let x = start_x + x_offset;
                        if x < self.width {
                            let mut y = start_y;
                            while y < self.height {
                                if col.pb_type == "EMPTY" {
                                    self.cells[y][x] = GridCell::Empty;
                                } else {
                                    self.cells[y][x] = GridCell::Block(col.pb_type.clone());
                                }
                                y += incr_y;
                            }
                        }
                    }
                }
            }
            GridLocation::Row(row) => {
                if let (Some(start_x), Some(start_y)) = (
                    self.eval_expr(&row.start_x_expr),
                    self.eval_expr(&row.start_y_expr),
                ) {
                    let incr_x = self.eval_expr(&row.incr_x_expr).unwrap_or(1);
                    let repeat_y = row.repeat_y_expr.as_ref()
                        .and_then(|expr| self.eval_expr(expr))
                        .unwrap_or(1);

                    for y_offset in 0..repeat_y {
                        let y = start_y + y_offset;
                        if y < self.height {
                            let mut x = start_x;
                            while x < self.width {
                                if row.pb_type == "EMPTY" {
                                    self.cells[y][x] = GridCell::Empty;
                                } else {
                                    self.cells[y][x] = GridCell::Block(row.pb_type.clone());
                                }
                                x += incr_x;
                            }
                        }
                    }
                }
            }
            GridLocation::Region(region) => {
                if let (Some(start_x), Some(end_x), Some(start_y), Some(end_y)) = (
                    self.eval_expr(&region.start_x_expr),
                    self.eval_expr(&region.end_x_expr),
                    self.eval_expr(&region.start_y_expr),
                    self.eval_expr(&region.end_y_expr),
                ) {
                    let incr_x = self.eval_expr(&region.incr_x_expr).unwrap_or(1);
                    let incr_y = self.eval_expr(&region.incr_y_expr).unwrap_or(1);

                    let repeat_x = region.repeat_x_expr.as_ref()
                        .and_then(|expr| self.eval_expr(expr))
                        .unwrap_or(1);
                    let repeat_y = region.repeat_y_expr.as_ref()
                        .and_then(|expr| self.eval_expr(expr))
                        .unwrap_or(1);

                    for x_repeat in 0..repeat_x {
                        for y_repeat in 0..repeat_y {
                            let mut y = start_y + (y_repeat * (end_y - start_y + incr_y));
                            while y <= end_y + (y_repeat * (end_y - start_y + incr_y)) && y < self.height {
                                let mut x = start_x + (x_repeat * (end_x - start_x + incr_x));
                                while x <= end_x + (x_repeat * (end_x - start_x + incr_x)) && x < self.width {
                                    if region.pb_type == "EMPTY" {
                                        self.cells[y][x] = GridCell::Empty;
                                    } else {
                                        self.cells[y][x] = GridCell::Block(region.pb_type.clone());
                                    }
                                    x += incr_x;
                                }
                                y += incr_y;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Evaluates expressions containing integers, W (width), H (height), and basic operators (+, -, /, *)
    fn eval_expr(&self, expr: &str) -> Option<usize> {
        let expr = expr.trim().to_uppercase();

        let expr = expr.replace('W', &self.width.to_string());
        let expr = expr.replace('H', &self.height.to_string());

        let expr = expr.replace(' ', "");

        self.eval_expr_recursive(&expr)
    }

    fn eval_expr_recursive(&self, expr: &str) -> Option<usize> {
        let expr = expr.trim();

        // Base case: try to parse as an integer
        if let Ok(val) = expr.parse::<i32>() {
            return if val >= 0 { Some(val as usize) } else { None };
        }

        let chars: Vec<char> = expr.chars().collect();
        let len = chars.len();

        // Addition and subtraction
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
                        } else {
                            if l >= r {
                                Some(l - r)
                            } else {
                                None
                            }
                        };
                    }
                }
                _ => {}
            }
        }

        // Multiplication and division
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
                        } else {
                            if r > 0 {
                                Some(l / r)
                            } else {
                                None
                            }
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
