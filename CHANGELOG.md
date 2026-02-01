## [v0.3.2] - 2026-02-01

### Added
- Visualizer app now has a basic icon.
- Added support for the Stratix-10 and 7-Series architecture captures.
- Added basic support for tileable architectures.
- Optimized grid rendering to allow for larger grids to be visualized.

### Infra
- Improved CI infrastructure

## [v0.3.1] - 2026-01-24

### Added
- An error window now opens when the architecture file fails to parse.
- Basic zoom controls were added to the grid view.
- Open and refresh buttons were added to the nav.

### Infra
- Updated packages to more recent versions.

## [v0.3.0] - 2026-01-17

### Added
- Summary view for the architecture description

### Infra
- Refactored large parts of the viewer code to make adding new views easier.

## [v0.2.0] - 2025-12-15

### Added
- Major release for basic features.
- Able to visualize the layout and complex blocks for most VTR architectures.

## [v0.1.2] - 2025-12-13

### Added
- Support for different tiles and grid locations in layouts.
- Support for fixed layouts.
- Tile counts are now shown for layouts.
- General cleanups to layout visualization.
- Support for complete interconnect.
- Improved drawing of interconnect wires.
- Added Zoom feature to complex block drawing.
- General cleanups to complex block visualization.

## [v0.1.1] - 2025-12-07

### Added
- Improved the error handling of the architecture XML parsing.
- Improved the architecture description coverage.

## [v0.1.0] - 2025-12-01

### Added
- First release of the major features such as:
    - Drawing the global tile layout (just IO and LB for now).
    - Intra-tile routing.
    - Basic parsing for general architecture files.
