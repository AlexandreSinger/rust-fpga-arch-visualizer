# rust-fpga-arch-visualizer

<p>
<img src="./docs/images/demo-splash-layout.png" width="49%" />
<img src="./docs/images/demo-splash-complex-blocks.png" width="49%" />
</p>

## Install

### Pre-Built Binaries

#### macOS and Linux (not NixOS):

Run the following code in a terminal:
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/AlexandreSinger/rust-fpga-arch-visualizer/releases/latest/download/fpga_arch_viewer-installer.sh | sh
```

#### Windows PowerShell:

Run the following code in PowerShell:
```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/AlexandreSinger/rust-fpga-arch-visualizer/releases/latest/download/fpga_arch_viewer-installer.ps1 | iex"
```

### Build From Source

This project uses `cargo` as a build system. You will need to install `cargo` to
build this project from source.

Download the code for this repository and navigate into it. Then run the following
commands:
```sh
cargo build --release

./target/release/fpga_arch_viewer
```

## Motivation

Computer Aided Design (CAD) software has been mostly dominated by C++ due to its high performance and prior adoption. Due to this decision, many of these tools suffer from poor memory safety, brought on by how pointers are used within the C++ language, especially in parsing code which reads custom files written by users. This has led to open-source CAD software that is prone to crashes, making the tools challenging to use, which is especially a concern for commercial applications.

The open-source [Verilog to Routing](https://verilogtorouting.org/) (VTR) project is a collection of CAD tools used for researching and designing Field-Programmable Gate Arrays (FPGAs). It is a popular infrastructure used for implementing a circuit (written in Verilog) onto a general class of FPGAs. To allow VTR to target many different FPGAs, VTR allows the user to describe their FPGA architecture through an XML-based Architecture Description File. This file is used by many commercial companies who use VTR’s architecture description file to represent their bespoke FPGA architectures. VTR is also used to explore new and exciting FPGA architectures, which may not have been physically implemented in silicon yet.

One major challenge faced by architecture designers who use VTR is how to efficiently write and debug their architecture description files. Although VTR provides a visualizer as part of the Versatile Place and Route (VPR) tool, the placer and router for the CAD flow, this visualizer was designed specifically for visualizing FPGA CAD algorithms and not for developing FPGA architectures. This visualizer only shows the grid-level view of the FPGA and does not contain detailed information on the inner-components of the FPGA architecture, which designers care deeply about. The current visualizer is also tied to the VPR flow, meaning that you must run the placement and routing algorithms to see the architecture, which can be slow and challenging to work with. FPGA architectures are also getting bigger, causing VPR to pre-compute more information not used for visualization, which further slows down the process of visualization.

The goal for this project is to provide an FPGA architecture visualizer to the VTR project, written in Rust. This visualizer will parse the architecture description file, provided by the user, and visualize the major components of the FPGA architecture. This will provide a detailed view to the FPGA designer, ensuring that the FPGA model matches their FPGA’s physical layout. Due to being written in Rust, this visualizer tool will be less prone to errors which will prevent user-level crashes.

As FPGA architectures continue to become larger and more complex, there is a growing need for fast FPGA architecture visualizers which show the key details that FPGA architects care about. By designing a custom FPGA architecture visualizer in Rust, we will provide a fast and safe visualizer to the open-source FPGA community.


## Objective

The main objective of this project is to design a Rust-based FPGA visualizer that can parse a description of an FPGA architecture and provide an interactive user interface for FPGA architects to visualize and analyze their FPGA architectures.

Currently, the tools within VTR can only visualize a global view of the FPGA architecture, which shows a simplified view of the tiles on the FPGA and the global routing connecting those tiles together.
However, FPGA architects require a more detailed view within the FPGA tiles, such as the primitive elements within the tile and the local routing interconnecting those primitives.
Thus, FPGA architects often have to read the architecture description file themselves, which is tedious and prone to error.
The visualization currently within VTR is also very slow since it is tied to the algorithms used to place and route a circuit onto the FPGA.
This project aims to fix these issues by providing a detailed visualizer for different FPGA architectures described by an architect; providing an improved interface for FPGA architecture design and exploration.

While the Rust programming language has been widely used in system programming, web services, and embedded systems, few efforts have been made to explore its application in the field of computer architecture. The Rust ecosystem lacks native support for computer architectures and CAD tools, which are traditionally dominated by C/C++ implementations. This project addresses this gap by introducing a modern, open-source tool that brings the safety and performance benefits of Rust into FPGA research and development. It not only provides a functional tool for FPGA exploration but also lays the foundation for a future Rust-based CAD tooling framework that could greatly benefit the research and development of FPGA devices.


## Features

This project is a **Rust-based, standalone visualizer** for VTR FPGA Architecture Description XML files. It helps FPGA architects **explore and debug architecture XML** by providing interactive views of:
- the **global tile grid** (inter-tile structure), and
- the **tile internals** (intra-tile primitives + local interconnect).

The key technical contributions are split across two crates:
- `fpga_arch_parser`: XML parsing + typed architecture model
- `fpga_arch_viewer`: interactive GUI + rendering
Below is a detailed feature breakdown aligned with: **parsing**, **visual**, and **general**.

### Parsing

- **Spec-driven XML parsing**
  - `fpga_arch_parser::parse(path)` parses an architecture XML into a strongly-typed `FPGAArch` model.
  - The parser is organized into focused modules (e.g., device/layout/tiles/ports/interconnect lists/
  timing), which makes it easier to extend as the VTR spec evolves.

- **Type-safe architecture database**
  - Architecture is stored in Rust structs/enums, including:
    - hierarchical PBTypes (with per-mode children and per-mode interconnect)
    - ports and pin counts
    - tile and layout structures
  - This enables safe traversal during visualization and reduces crash-prone string handling.

- **Testcases / real architecture files**
  - Includes multiple realistic architecture XML test inputs under `fpga_arch_parser/tests/` (e.g., `k4_N4_90nm.xml`, `stratixiv_arch.timing.xml`, `z1000.xml`, `z1010.xml`, `z1060.xml`).

### Visual

#### Global / inter-tile view

- **Grid-level device visualization**
  - Renders the FPGA fabric as a 2D grid of tiles.
  - Tiles are shown using consistent **block/tile representations** with color mapping.

- **AutoLayout + FixedLayout support**
  - Supports switching between available layouts when the architecture provides them.
  - For fixed layouts, grid dimensions are derived from the architecture.
  - For auto layouts, use sliders to control grid width/height

- **Tile counts summary**
  - Shows derived metrics like aspect ratio and grid size.
  - Computes counts of tile types from the rendered grid and displays them in a table.

- **Select tile**
  - Clicking a tile in the grid selects it and switches into the intra-tile view for that tile.

#### Intra-tile view

- **Block representation**
  - Recursive PBType drawing for hierarchical structures (e.g., CLB → BLE array → LUT/FF/memory).
  - For blocks with multiple modes, a per-block mode selector lets you choose which mode’s children/interconnect to visualize.
  - Optional textual hierarchy tree panel.

- **Expand/collapse and “Expand All”**
  - Expand/collapse on a per-block basis:
    - Collapsed state draws a compact header-only representation.
    - Expanded state renders child PBTypes and (optionally) the local interconnect.
  - Global “Expand All” to fully open hierarchy for deep inspection.

- **Routing**
  - Draws pins/ports and local routing between blocks:
    - Direct Interconnect: draws point-to-point port routing between blocks.
    - Mux Interconnect: draws mux “nodes” and routes sources/sinks to/from the mux, includes routing heuristics to reduce wire crossings in dense fabrics.
    - Complete Interconnect: renders dense crossbar structures.
    - Draw Interconnect UI toggle to hide direct/mux/complete interconnects (Blocks-only mode) for readability in very dense tiles.
  - Highlighting：
    - Hover highlighting for wires and interconnect blocks.
    - Pin hitboxes and tooltips for pin name, e.g., in[3], clk[0], etc.

- **Zoom and pan**
  - Panning is constrained to the intra-tile canvas viewport.
  - Zoom scales rendering and interaction targets (pins, labels, interconnect blocks, routing heuristics, widget sizing).

### General

- **Settings and theme**
  - Settings page for appearance controls.
  - Light/dark mode support.

- **Navigation**
  - Back button to return to previous views/pages (e.g., intra-tile → grid).

- **Load architecture file from UI**
  - File picker to open architecture XML.
  - Window title shows the loaded filename (`FPGA Architecture Visualizer - <file>.xml`).

- **Cross-platform desktop app**
  - Built using `eframe/egui`, targeting a single Rust codebase for common desktop platforms.

- **Release / easy download**
  - Pre-built binary install scripts are documented for macOS/Linux and Windows.



## Tentative Plan

**Weekly team meeting**: Fridays, 3:00–4:00 PM

Below is our proposed plan for achieving the project objectives. It outlines the responsibilities of each team member and the steps we will take over the coming weeks to deliver a functional, fully integrated Rust-based FPGA visualizer.

Since VTR is an active open-source project, there are many stakeholders (in both industry and academia) who would be interested in this project.
Every Thursday at 1:00 PM, there is an Industry Sync Meeting attended by the many companies that use VTR and the researchers working on it.
We plan to present this project at these meetings periodically during this term to get feedback and report progress.

| Task | Person Responsible | Timeline |
| ---- | ------------------ | -------- |
| **Milestone 0: Team Kickoff** | | |
| Study the [VTR documentation](https://docs.verilogtorouting.org/en/latest/arch/) and analyze the FPGA architecture XML format | All | Week 1: Oct 6 - Oct 13 |
| Research Rust XML parsing crates and explore Qt integration options for visualization in Rust | All | Week 1: Oct 6 - Oct 13 |
| **Milestone 1: Core Design**| | |
| Design a modular and memory-safe FPGA database schema in Rust | Alex | Week 2 - 3: Oct 13 - Oct 27 |
| Define a data serialization format and create sample datasets for visualization testing | Alex | Week 2 - 3: Oct 13 - Oct 27 |
| Sketch visualization mockups design (confirm with the team) | Jack & Maggie| Week 2 - 3: Oct 13 - Oct 27 |
| **Key deliverable: Present a design report/slides to the VTR Industry Sync Up Meeting for feedback** | All| November 6, 1:00 PM |
| **Milestone 2: Implementation** | | |
| Implement XML parsing engine prototype | Alex | Week 4-7: Oct 27 - Nov 24 |
| Implement outside view (grid-level FPGA visualization) | Maggie | Week 4-5: Oct 27 - Nov 10 |
| Implement inside view (intra-tile visualization for LUTs, routing, etc.) | Jack| Week 4-7: Oct 27 - Nov 24 |
| Add user interface controls (zoom, pan, element highlighting, tooltips) | Maggie | Week 6-7: Nov 10 - Nov 24 |
| **Key deliverable: Functional prototype demonstrating FPGA visualization using sample data**| All | November 24 |
| **Milestone 3: System Integration & Testing** | | |
| Integrate parser and visualization modules | All | Week 8-9: Nov 24 - Dec 8|
| End-to-end validation using multiple real VTR architecture XML files | All | Week 8-9: Nov 24 - Dec 8|
| **Key deliverable: Fully integrated system with parser, database, and visualization modules** | All | December 8 |
| **Milestone 3: Finalization** | | |
| Prepare documentation, usage guide, and final presentation demo | All | Week 10: Dec 8 - 14 |
| **Key deliverable: Comprehensive documentation, and polished final demo ready for submission** | All | December 14 |
| **Final Deliverable Due** | All | Dec 15, 11:59 PM|

## Resources

This is a list of resources that will be helpful for working on this project.

* “Architecture description and packing for logic blocks with hierarchy, modes and complex interconnect”, Luu et al.:
  * https://dl-acm-org.myaccess.library.utoronto.ca/doi/10.1145/1950413.1950457
  * This is the paper that introduced the architecture description file that is currently used in VTR.
* VTR: FPGA Architecture Description Docs
  * https://docs.verilogtorouting.org/en/latest/arch/
  * This is the actual design specification docs used for VTR.
* VPR: Graphics Docs
  * https://docs.verilogtorouting.org/en/latest/vpr/graphics/
  * Documentation of the graphics capability currently available in the VPR tool for the architecture description file.
