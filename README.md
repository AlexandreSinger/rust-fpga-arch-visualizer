# rust-fpga-arch-visualizer


## Motivation

Computer Aided Design (CAD) software has been mostly dominated by C++ due to its high performance and prior adoption. Due to this decision, many of these tools suffer from poor memory safety, brought on by how pointers are used within the C++ language, especially in parsing code which reads custom files written by users. This has led to open-source CAD software that is prone to crashes which make the tools challenging to use, which is especially a concern for commercial applications.

The open-source Verilog to Routing (VTR) project is a collection of CAD tools used for researching and designing Field-Programmable Gate Arrays (FPGAs). It is a popular infrastructure used for implementing a circuit (written in Verilog) onto a general class of FPGAs. To allow VTR to target many different FPGAs, VTR allows the user to describe their FPGA architecture through an XML-based Architecture Description File. This file is used by many commercial companies who use VTR’s architecture description file to represent their bespoke FPGA architectures. VTR is also used to explore new and exciting FPGA architectures, which may not have been physically implemented in silicon yet.

One major challenge faced by architecture designers who use VTR is how to efficiently write and debug their architecture description files. Although VTR provides a visualizer as part of the Versatile Place and Route (VPR) tool, the placer and router for the CAD flow, this visualizer was designed specifically for visualizing FPGA CAD algorithms and not for developing FPGA architectures. This visualizer only shows the grid-level view of the FPGA and does not contain detailed information on the inner-components of the FPGA architecture, which designers care deeply about. The current visualizer is also tied to the VPR flow, meaning that you must run the placement and routing algorithms to see the architecture, which can be slow and challenging to work with. FPGA architectures are also getting bigger, causing VPR to pre-compute more information not used for visualization, which further slows down the process of visualization.

The goal for this project is to provide an FPGA architecture visualizer to the VTR project, written in Rust. This visualizer will parse the architecture description file, provided by the user, and visualize the major components of the FPGA architecture. This will provide a detailed view to the FPGA designer, ensuring that the FPGA model matches their FPGA’s physical layout. Due to being written in Rust, this visualizer tool will be less prone to errors which will prevent user-level crashes.

As FPGA architectures continue to become larger and more complex, there is a growing need for fast FPGA architecture visualizers which show the key details that FPGA architects care about. By designing a custom FPGA architecture visualizer in Rust, we will provide a fast and safe visualizer to the open-source FPGA community.


## Objective and Key Features

The main objective of this project is to design a Rust-based FPGA Visualizer that can parse a description of an FPGA architecture and provide an interactive user interface for FPGA architects to visualize and analyze an FPGA architecture.

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

- Alex: XML parsing engine + Specialized FPGA database

- Maggie: FPGA global visualization, user interface

- Jack: FPGA intra-tile visualization, user interface


## Tentative Plan



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
