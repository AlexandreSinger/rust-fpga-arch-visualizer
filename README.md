# rust-fpga-arch-visualizer


## Motivation

Computer Aided Design (CAD) software has been mostly dominated by C++ due to its high performance and prior adoption. Due to this decision, many of these tools suffer from poor memory safety, brought on by how pointers are used within the C++ language, especially in parsing code which reads custom files written by users. This has led to open-source CAD software that is prone to crashes which make the tools challenging to use, which is especially a concern for commercial applications.

The open-source Verilog to Routing (VTR) project is a collection of CAD tools used for researching and designing Field-Programmable Gate Arrays (FPGAs). It is a popular infrastructure used for implementing a circuit (written in Verilog) onto a general class of FPGAs. To allow VTR to target many different FPGAs, VTR allows the user to describe their FPGA architecture through an XML-based Architecture Description File. This file is used by many commercial companies who use VTR’s architecture description file to represent their bespoke FPGA architectures. VTR is also used to explore new and exciting FPGA architectures, which may not have been physically implemented in silicon yet.

One major challenge faced by architecture designers who use VTR is how to efficiently write and debug their architecture description files. Although VTR provides a visualizer as part of the Versatile Place and Route (VPR) tool, the placer and router for the CAD flow, this visualizer was designed specifically for visualizing FPGA CAD algorithms and not for developing FPGA architectures. This visualizer only shows the grid-level view of the FPGA and does not contain detailed information on the inner-components of the FPGA architecture, which designers care deeply about. The current visualizer is also tied to the VPR flow, meaning that you must run the placement and routing algorithms to see the architecture, which can be slow and challenging to work with. FPGA architectures are also getting bigger, causing VPR to pre-compute more information not used for visualization, which further slows down the process of visualization.

The goal for this project is to provide an FPGA architecture visualizer to the VTR project, written in Rust. This visualizer will parse the architecture description file, provided by the user, and visualize the major components of the FPGA architecture. This will provide a detailed view to the FPGA designer, ensuring that the FPGA model matches their FPGA’s physical layout. Due to being written in Rust, this visualizer tool will be less prone to errors which will prevent user-level crashes.

As FPGA architectures continue to become larger and more complex, there is a growing need for fast FPGA architecture visualizers which show the key details that FPGA architects care about. By designing a custom FPGA architecture visualizer in Rust, we will provide a fast and safe visualizer to the open-source FPGA community.


## Objective and Key Features



## Tentative Plan


