use eframe::egui;
use fpga_arch_parser::{FPGAArch, PBType, PBTypeClass, Port, Tile};

pub fn render_hierarchy_tree(ui: &mut egui::Ui, arch: &FPGAArch, tile: &Tile) {
    for sub_tile in &tile.sub_tiles {
        egui::CollapsingHeader::new(format!("SubTile: {}", sub_tile.name))
            .default_open(true)
            .show(ui, |ui| {
                ui.label(format!("Capacity: {}", sub_tile.capacity));
                for site in &sub_tile.equivalent_sites {
                    ui.label(format!("Equivalent Site PB Type: {}", site.pb_type));
                    // Find the PBType definition
                    if let Some(pb_type) = arch
                        .complex_block_list
                        .iter()
                        .find(|pb| pb.name == site.pb_type)
                    {
                        ui.push_id(format!("pb_{}", pb_type.name), |ui| {
                            render_pb_type_tree_node(ui, pb_type);
                        });
                    } else {
                        ui.colored_label(egui::Color32::RED, "PBType not found!");
                    }
                }
            });
    }
}

fn render_interconnects(ui: &mut egui::Ui, interconnects: &[fpga_arch_parser::Interconnect]) {
    for inter in interconnects {
        let (kind, name, input, output, pack_patterns) = match inter {
            fpga_arch_parser::Interconnect::Direct(d) => {
                ("Direct", &d.name, &d.input, &d.output, &d.pack_patterns)
            }
            fpga_arch_parser::Interconnect::Mux(m) => {
                ("Mux", &m.name, &m.input, &m.output, &m.pack_patterns)
            }
            fpga_arch_parser::Interconnect::Complete(c) => {
                ("Complete", &c.name, &c.input, &c.output, &c.pack_patterns)
            }
        };
        ui.horizontal(|ui| {
            ui.label(format!("{}: {} ({} -> {})", kind, name, input, output));
            if !pack_patterns.is_empty() {
                for pp in pack_patterns {
                    ui.label(format!("[Pack: {}]", pp.name));
                }
            }
        });
    }
}

fn render_pb_type_tree_node(ui: &mut egui::Ui, pb_type: &PBType) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("PB Type: {}", pb_type.name)).strong());
            ui.label(format!("(Num: {})", pb_type.num_pb));
            if let Some(class) = &pb_type.blif_model {
                ui.label(format!("Model: {}", class));
            }
            match pb_type.class {
                PBTypeClass::Lut => {
                    ui.label(egui::RichText::new("[LUT]").color(egui::Color32::GOLD));
                }
                PBTypeClass::FlipFlop => {
                    ui.label(egui::RichText::new("[FF]").color(egui::Color32::LIGHT_BLUE));
                }
                PBTypeClass::Memory => {
                    ui.label(egui::RichText::new("[MEM]").color(egui::Color32::GREEN));
                }
                PBTypeClass::None => {}
            }
        });

        ui.indent("ports", |ui| {
            for port in &pb_type.ports {
                let (direction, name, num_pins) = match port {
                    Port::Input(p) => ("In", &p.name, p.num_pins),
                    Port::Output(p) => ("Out", &p.name, p.num_pins),
                    Port::Clock(p) => ("Clock", &p.name, p.num_pins),
                };
                ui.label(format!("{} Port: {} [{}]", direction, name, num_pins));
            }
        });

        // Display direct children
        if !pb_type.modes.is_empty() {
            // If there are modes, children are inside modes
            ui.indent("modes", |ui| {
                for mode in &pb_type.modes {
                    egui::CollapsingHeader::new(format!("Mode: {}", mode.name))
                        .show(ui, |ui| {
                            for child_pb in &mode.pb_types {
                                render_pb_type_tree_node(ui, child_pb);
                            }
                            if !mode.interconnects.is_empty() {
                                egui::CollapsingHeader::new("Interconnects")
                                    .show(ui, |ui| {
                                        render_interconnects(ui, &mode.interconnects);
                                    });
                            }
                        });
                }
            });
        } else if !pb_type.pb_types.is_empty() {
            // If no modes, show direct children
            ui.indent("children", |ui| {
                for child_pb in &pb_type.pb_types {
                    render_pb_type_tree_node(ui, child_pb);
                }
            });
        }

        // Display top-level interconnects
        if !pb_type.interconnects.is_empty() {
            ui.indent("interconnects", |ui| {
                egui::CollapsingHeader::new("Interconnects")
                    .default_open(true)
                    .show(ui, |ui| {
                        render_interconnects(ui, &pb_type.interconnects);
                    });
            });
        }
    });
}
