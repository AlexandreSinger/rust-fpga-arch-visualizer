# rust-fpga-arch-visualizer


## Motivation

Computer Aided Design (CAD) software has been mostly dominated by C++ due to its high performance and prior adoption. Due to this decision, many of these tools suffer from poor memory safety, brought on by how pointers are used within the C++ language, especially in parsing code which reads custom files written by users. This has led to open-source CAD software that is prone to crashes which make the tools challenging to use, which is especially a concern for commercial applications.

The open-source Verilog to Routing (VTR) project is a collection of CAD tools used for researching and designing Field-Programmable Gate Arrays (FPGAs). It is a popular infrastructure used for implementing a circuit (written in Verilog) onto a general class of FPGAs. To allow VTR to target many different FPGAs, VTR allows the user to describe their FPGA architecture through an XML-based Architecture Description File. This file is used by many commercial companies who use VTR’s architecture description file to represent their bespoke FPGA architectures. VTR is also used to explore new and exciting FPGA architectures, which may not have been physically implemented in silicon yet.

One major challenge faced by architecture designers who use VTR is how to efficiently write and debug their architecture description files. Although VTR provides a visualizer as part of the Versatile Place and Route (VPR) tool, the placer and router for the CAD flow, this visualizer was designed specifically for visualizing FPGA CAD algorithms and not for developing FPGA architectures. This visualizer only shows the grid-level view of the FPGA and does not contain detailed information on the inner-components of the FPGA architecture, which designers care deeply about. The current visualizer is also tied to the VPR flow, meaning that you must run the placement and routing algorithms to see the architecture, which can be slow and challenging to work with. FPGA architectures are also getting bigger, causing VPR to pre-compute more information not used for visualization, which further slows down the process of visualization.

The goal for this project is to provide an FPGA architecture visualizer to the VTR project, written in Rust. This visualizer will parse the architecture description file, provided by the user, and visualize the major components of the FPGA architecture. This will provide a detailed view to the FPGA designer, ensuring that the FPGA model matches their FPGA’s physical layout. Due to being written in Rust, this visualizer tool will be less prone to errors which will prevent user-level crashes.

As FPGA architectures continue to become larger and more complex, there is a growing need for fast FPGA architecture visualizers which show the key details that FPGA architects care about. By designing a custom FPGA architecture visualizer in Rust, we will provide a fast and safe visualizer to the open-source FPGA community.


## Objective and Key Features

The main objective of this project is to design a Rust-based FPGA Visualizer that can parse a description of an FPGA architecture and provide an interactive user interface for FPGA architects to visualize and analyze an FPGA architecture.

Currently, the tools within VTR can only visualize a global view of the FPGA architecture, which shows a simplified view of the tiles on the FPGA and the global routing connecting those tiles together.
However, FPGA architects require a more detailed view within the FPGA tiles, such as the primitive elements within the tile and the local routing interconnecting those primitives.
Thus, FPGA architects often have to read the architecture description file themselves, which is tedious and prone to error.
The visualization currently within VTR is also very slow since it is tied to the algorithms used to place and route a circuit onto the FPGA.
This project aims to fix these issues by providing a detailed visualizer for different FPGA architectures described by an architect; providing an improved interface for FPGA architecture design and exploration.

While the Rust programming language has been widely used in system programming, web services, and embedded systems, few efforts have been made to explore its application in the field of computer architecture. The Rust ecosystem lacks native support for computer architectures and CAD tools, which are traditionally dominated by C/C++ implementations. This project addresses this gap by introducing a modern, open-source tool that brings the safety and performance benefits of Rust into FPGA research and development. It not only provides a functional tool for FPGA exploration but also lays the foundation for a future Rust-based CAD tooling framework that could greatly benefit the research and development of FPGA devices.

### Key features include:

- FPGA description XML parsing engine: 
  - Develop a parsing engine that is able to parse FPGA architecture description XML files, as described in the [VTR arichecture description specification](https://docs.verilogtorouting.org/en/latest/arch/).
  - Extract information for logic blocks, routing resources, and local / global interconnects.
  - Since the architecture description file used in VTR is under active development and constantly evolving, this parsing engine must be extensible to allow future description features to be added.

- Specialized FPGA database: 
  - Design efficient and type-safe Rust data structures to represent the parsed FPGA architecture.
  - Store grid layouts, routing connections, and blocks hierarchically for visualization and analysis.
  - Ensure fast and convenient data access for visualization.

- FPGA Visualization:
  - Render both the general (global) FPGA grid layout and intra-tile (local) components (primitives and local interconnect).
  - Support visualization for the local interconnect structures between logic blocks; which has not been done before for VTR architecture visualization.
  - Provide clear insights for users into the structure of the FPGA for efficient debugging and design.

- Interactive User interface: 
  - Implement an intuitive, cross-platform user interface.
  - Enable responsive user interaction, such as zooming, panning, and highlighting.
  - Allows users to explore, analyze, and experiment with the FPGA architectures.

### Work Distribution

The work will be divided generally as follows to ensure a fair distribution of work and a reasonable workload throughout the term. More details will be provided in the next **Tentative Plan** section.

- Alex: XML parsing engine + specialized FPGA database.

- Maggie: Global FPGA layout and inter-tile routing visualization, user interface.

- Jack: Local (intra-tile) primitive and routing visualization, user interface.


## Tentative Plan

Weekly team meeting: Fridays, 3:00–4:00 PM

| Task | Person Responsible | Timeline |
| ---- | ------------------ | -------- |
| **Milestone 0: Team Kickoff** | | |
| Study [VTR documentation](https://docs.verilogtorouting.org/en/latest/arch/) and analyze FPGA architecture XML format | All | Week 1: Oct 6 - Oct 13 |
| Research Rust XML parsing crates and explore Qt integration options for visualization in Rust | All | Week 1: Oct 6 - Oct 13 |
| **Milestone 1: Core Design**| | |
| Design a modular and memory-safe FPGA database schema in Rust | Alex | Week 2 - 3: Oct 13 - Oct 27 |
| Define a data serialization format and create sample datasets for visualization testing | Alex | Week 2 - 3: Oct 13 - Oct 27 |
| Sketch visualization mockups design (confirm with the team) | Jack & Maggie| Week 2 - 3: Oct 13 - Oct 27 |
| **Key deliverable: Present a design report/slides to VTR Industry Sync Up Meeting for feedback** | All| November 6, 1:00 PM |
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
