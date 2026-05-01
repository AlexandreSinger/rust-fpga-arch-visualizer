## [v0.3.7] - 2026-05-01

### Added
- Added scroll regions to side-bars so they are easier to see on small screens.
- Added CLI arguments to pre-select an architecture file and run parsing only (without the viewer opening).

### Fixed
- Fixed caching issue with web builds, so now the cache is always invalidated on new version releases.
- Fixed potential vulnerability in rand.
- Fixed the viewer freezing on Linux while loading a file by making file I/O non-blocking.
- Fixed dark mode in the tile view, the grid view, and the CRR Switch Block view.

## [v0.3.6] - 2026-03-29

### Added
- Basic NoC parsing and visualization support added.
- CRR view has been improved for the native build.
- FPS counter added to the top-right corner for debugging.
- Improved complex block graph parsing support.

## [v0.3.5] - 2026-03-22

### Added
- New Primitive View has been added which shows the timing characteristics of models and their associated primitives.

## [v0.3.4] - 2026-03-11

### Added
- Basic interposer support has been added.
- Sample architectures are provided in the binary to sample the viewer.
- 3D architectures are now supported.
- Files can now be dropped into the app instead of going through the file select prompt.

## [v0.3.3] - 2026-03-08

### Added
- Proper Tile View has been added which resolves issue where some complex blocks could not be viewed.
- WASM target is now supported.
- Initial support for Custom RR graphs.
- Fixed small oversight with shorts in parsing architecture files.
- Added automatic website deployment for the WASM build of the app.

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
