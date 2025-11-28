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
            let height = (default_size as f64 / aspect_ratio).round() as usize;
            (width, height.max(1)) // Ensure height is at least 1
        } else {
            // Height > width (portrait)
            let height = default_size;
            let width = (default_size as f64 * aspect_ratio).round() as usize;
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
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&GridCell> {
        if row < self.height && col < self.width {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }
}
