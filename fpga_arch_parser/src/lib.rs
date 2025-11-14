use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

pub struct Model {

}

pub struct InputPort {
    pub name: String,
    pub num_pins: i32,
    // TODO: This should be an enum.
    pub equivalent: String,
    pub is_non_clock_global: bool,
    pub port_class: Option<String>,
}

pub struct OutputPort {
    pub name: String,
    pub num_pins: i32,
    // TODO: This should be an enum.
    pub equivalent: String,
    pub is_non_clock_global: bool,
    pub port_class: Option<String>,
}

pub struct ClockPort {
    pub name: String,
    pub num_pins: i32,
    // TODO: This should be an enum.
    pub equivalent: String,
    pub is_non_clock_global: bool,
    pub port_class: Option<String>,
}

pub enum Port {
    Input(InputPort),
    Output(OutputPort),
    Clock(ClockPort),
}

pub struct TileSite {
    pub pb_type: String,
    pub pin_mapping: String,
}

pub struct SubTileFracFC {
    pub val: f32,
}

pub struct SubTileAbsFC {
    pub val: i32,
}

pub enum SubTileIOFC {
    Frac(SubTileFracFC),
    Abs(SubTileAbsFC),
}

pub struct SubTileFC {
    pub in_fc: SubTileIOFC,
    pub out_fc: SubTileIOFC,
}

pub struct SubTile {
    pub name: String,
    pub capacity: i32,
    pub equivalent_sites: Vec<TileSite>,
    pub ports: Vec<Port>,
    pub fc: SubTileFC,
}

pub struct Tile {
    pub name: String,
    pub sub_tiles: Vec<SubTile>,
}

// TODO: pb_type and priority is better served as a trait.
pub struct FillGridLocation {
    pub pb_type: String,
    pub priority: i32,
}

pub struct PerimeterGridLocation {
    pub pb_type: String,
    pub priority: i32,
}

pub struct CornersGridLocation {
    pub pb_type: String,
    pub priority: i32,
}

pub enum GridLocation {
    Fill(FillGridLocation),
    Perimeter(PerimeterGridLocation),
    Corners(CornersGridLocation),
}

pub struct AutoLayout {
    pub aspect_ratio: f32,
    pub grid_locations: Vec<GridLocation>,
}

pub struct FixedLayout {
    pub name: String,
    pub width: i32,
    pub height: i32,
}

pub enum Layout {
    AutoLayout(AutoLayout),
    FixedLayout(FixedLayout),
}

pub struct DeviceInfo {

}

pub struct Switch {

}

pub struct Segment {

}

pub struct PBMode {
    pub name: String,
    pub pb_types: Vec<PBType>,
}

pub struct PBType {
    pub name: String,
    pub num_pb: i32,
    pub blif_model: Option<String>,
    pub class: Option<String>,
    pub ports: Vec<Port>,
    pub modes: Vec<PBMode>,
    pub pb_types: Vec<PBType>,
}

pub struct FPGAArch {
    pub models: Vec<Model>,
    pub tiles: Vec<Tile>,
    pub layouts: Vec<Layout>,
    pub device: DeviceInfo,
    pub switch_list: Vec<Switch>,
    pub segment_list: Vec<Segment>,
    pub complex_block_list: Vec<PBType>,
}

fn parse_port(name: &str,
              attributes: &Vec<OwnedAttribute>) -> Port {
    let mut port_name: Option<String> = None;
    let mut num_pins: Option<i32> = None;
    let mut equivalent = String::from("none");
    let is_non_clock_global = false;
    let mut port_class: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => port_name = Some(a.value.clone()),
            "num_pins" => num_pins = Some(a.value.parse().expect("Num pins should be integer type")),
            "equivalent" => equivalent = a.value.clone(),
            "is_non_clock_global" => panic!("TODO: Handle is_non_clock_global"),
            "port_class" => {
                assert!(port_class.is_none());
                port_class = Some(a.value.clone());
            },
            _ => panic!("Unnexpected attribute in port: {}", a.name.to_string()),
        };
    }

    assert!(port_name.is_some());
    assert!(num_pins.is_some());

    match name {
        "input" => Port::Input(InputPort {
            name: port_name.unwrap(),
            num_pins: num_pins.unwrap(),
            equivalent: equivalent,
            is_non_clock_global: is_non_clock_global,
            port_class: port_class,
        }),
        "output" => Port::Output(OutputPort {
            name: port_name.unwrap(),
            num_pins: num_pins.unwrap(),
            equivalent: equivalent,
            is_non_clock_global: is_non_clock_global,
            port_class: port_class,
        }),
        "clock" => Port::Clock(ClockPort {
            name: port_name.unwrap(),
            num_pins: num_pins.unwrap(),
            equivalent: equivalent,
            is_non_clock_global: is_non_clock_global,
            port_class: port_class,
        }),
        _ => panic!("Unknown port tag: {}", name),
    }
}

fn parse_tile_site(_name: &str,
                   attributes: &Vec<OwnedAttribute>) -> TileSite {

    let mut site_pb_type: Option<String> = None;
    let mut site_pin_mapping = String::from("direct");
    for a in attributes {
        match a.name.to_string().as_str() {
            "pb_type" => {
                site_pb_type = Some(a.value.clone());
            },
            "pin_mapping" => {
                site_pin_mapping = a.value.clone();
            },
            _ => {
                panic!("Unnexpected attribute.");
            },
        };
    }


    return TileSite {
        pb_type: site_pb_type.unwrap(),
        pin_mapping: site_pin_mapping,
    };
}

fn parse_equivalent_sites(_name: &str,
                          _attributes: &Vec<OwnedAttribute>,
                          parser: &mut EventReader<BufReader<File>>) -> Vec<TileSite> {

    let mut equivalent_sites: Vec<TileSite> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "site" => {
                        equivalent_sites.push(parse_tile_site(&name.to_string(), &attributes));
                    },
                    _ => {
                        panic!("Unnexpected tag in equivalent_sites.");
                    },
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "equivalent_sites" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return equivalent_sites;
}

fn create_sub_tile_io_fc(ty: &str, val: &str) -> SubTileIOFC {
    return match ty {
        "frac" => {
            SubTileIOFC::Frac(SubTileFracFC {
                val: val.parse().expect("fc_val should be frac"),
            })
        },
        "abs" => {
            SubTileIOFC::Abs(SubTileAbsFC {
                val: val.parse().expect("fc_val should be abs"),
            })
        },
        _ => panic!("Unknown fc_type: {}", ty),
    }
}

fn parse_sub_tile_fc(_name: &str,
                     attributes: &Vec<OwnedAttribute>) -> SubTileFC {
    let mut in_type: Option<String> = None;
    let mut in_val: Option<String> = None;
    let mut out_type: Option<String> = None;
    let mut out_val: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "in_type" => {
                assert!(in_type.is_none());
                in_type = Some(a.value.clone());
            },
            "in_val" => {
                assert!(in_val.is_none());
                in_val = Some(a.value.clone());
            },
            "out_type" => {
                assert!(out_type.is_none());
                out_type = Some(a.value.clone());
            },
            "out_val" => {
                assert!(out_val.is_none());
                out_val = Some(a.value.clone());
            },
            _ => panic!("Unknown fc attribute: {}", a.name.to_string()),
        };
    }

    assert!(in_type.is_some());
    assert!(in_val.is_some());
    assert!(out_type.is_some());
    assert!(out_val.is_some());

    let in_fc = create_sub_tile_io_fc(&in_type.unwrap(), &in_val.unwrap());
    let out_fc = create_sub_tile_io_fc(&out_type.unwrap(), &out_val.unwrap());

    return SubTileFC {
        in_fc: in_fc,
        out_fc: out_fc,
    }
}

fn parse_sub_tile(_name: &str,
                  attributes: &Vec<OwnedAttribute>,
                  parser: &mut EventReader<BufReader<File>>) -> SubTile {

    let mut sub_tile_name: Option<String> = None;
    let mut sub_tile_capacity: i32 = 1;
    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                sub_tile_name = Some(a.value.clone());
            },
            "capacity" => {
                sub_tile_capacity = a.value.parse().expect("Invalid capacity");
            },
            _ => {},
        };
    }

    assert!(sub_tile_name.is_some());

    let mut equivalent_sites: Option<Vec<TileSite>> = None;
    let mut ports: Vec<Port> = Vec::new();
    let mut sub_tile_fc: Option<SubTileFC> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "equivalent_sites" => {
                        equivalent_sites = Some(parse_equivalent_sites(&name.to_string(), &attributes, parser));
                    },
                    "input" | "output" | "clock" => {
                        ports.push(parse_port(&name.to_string(), &attributes));
                    },
                    "fc" => {
                        assert!(sub_tile_fc.is_none());
                        sub_tile_fc = Some(parse_sub_tile_fc(&name.to_string(), &attributes));
                    }
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "sub_tile" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    assert!(equivalent_sites.is_some());
    assert!(sub_tile_fc.is_some());

    return SubTile {
        name: sub_tile_name.unwrap(),
        capacity: sub_tile_capacity,
        equivalent_sites: equivalent_sites.unwrap(),
        ports: ports,
        fc: sub_tile_fc.unwrap(),
    };
}

fn parse_tile(_name: &str,
              attributes: &Vec<OwnedAttribute>,
              parser: &mut EventReader<BufReader<File>>) -> Tile {

    // TODO: Verify the name and attributes are expected.

    let mut tile_name = String::new();
    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                tile_name = a.value.clone();
            },
            _ => {},
        };
    }

    let mut sub_tiles: Vec<SubTile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "sub_tile" => {
                        sub_tiles.push(parse_sub_tile(&name.to_string(), &attributes, parser));
                    },
                    _ => {
                        panic!("Unnexpected tag in tile.");
                    },
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "tile" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return Tile {
        name: tile_name,
        sub_tiles: sub_tiles,
    };
}

fn parse_tiles(_name: &str,
               _attributes: &Vec<OwnedAttribute>,
               parser: &mut EventReader<BufReader<File>>) -> Vec<Tile> {
    // TODO: Error check the name and attributes to ensure that they are corrrect.

    // Iterate over the parser until we reach the EndElement for tile.
    let mut tiles: Vec<Tile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "tile" => {
                        tiles.push(parse_tile(&name.to_string(), &attributes, parser));
                    },
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "tiles" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return tiles;
}

fn parse_grid_location(name: &str,
                       attributes: &Vec<OwnedAttribute>) -> GridLocation {

    let mut pb_type: Option<String> = None;
    let mut priority: Option<i32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                pb_type = Some(a.value.clone());
            },
            "priority" => {
                priority = Some(a.value.parse().expect("Not a valid number"));
            },
            _ => {},
        };
    }

    if pb_type.is_none() || priority.is_none() {
        panic!("Grid location {name} missing type and/or priority");
    }

    match name.to_string().as_ref() {
        "perimeter" => {
            GridLocation::Perimeter(PerimeterGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
            })
        },
        "corners" => {
            GridLocation::Corners(CornersGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
            })
        },
        "fill" => {
            GridLocation::Fill(FillGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
            })
        },
        _ => {
            panic!("Unknown grid location: {}", name.to_string());
        },
    }
}

fn parse_auto_layout(_name: &str,
                     attributes: &Vec<OwnedAttribute>,
                     parser: &mut EventReader<BufReader<File>>) -> AutoLayout {

    let mut aspect_ratio: f32 = 1.0;
    let mut grid_locations: Vec<GridLocation> = Vec::new();

    for a in attributes {
        match a.name.to_string().as_ref() {
            "aspect_ratio" => {
                aspect_ratio = a.value.parse().expect("Invalid aspect ratio");
            },
            _ => {
                panic!("Unknown attribute for auto layout: {}", a.name.to_string());
            },
        }
    }

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                grid_locations.push(parse_grid_location(&name.to_string(), &attributes));
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "auto_layout" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return AutoLayout {
        aspect_ratio: aspect_ratio,
        grid_locations: grid_locations,
    };
}

fn parse_layouts(_name: &str,
                 _attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) -> Vec<Layout> {

    // TODO: Error check the name and attributes.

    let mut layouts: Vec<Layout> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "auto_layout" => {
                        layouts.push(Layout::AutoLayout(parse_auto_layout(&name.to_string(), &attributes, parser)));
                    },
                    "fixed_layout" => {
                        // FIXME: Add error.
                        let _ = parser.skip();
                    },
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "layout" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return layouts;
}

fn parse_pb_mode(_name: &str,
                 attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) ->PBMode {
    let mut mode_name: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(mode_name.is_none());
                mode_name = Some(a.value.clone());
            },
            _ => panic!("Unknown attribute in pb_type"),
        };
    }

    assert!(mode_name.is_some());

    let mut pb_types: Vec<PBType> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name.to_string(), &attributes, parser));
                    },
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "mode" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return PBMode {
        name: mode_name.unwrap(),
        pb_types: pb_types,
    };
}

fn parse_pb_type(_name: &str,
                 attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) -> PBType {
    let mut pb_type_name: Option<String> = None;
    let mut num_pb: i32 = 1;
    let mut blif_model: Option<String> = None;
    let mut class: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(pb_type_name.is_none());
                pb_type_name = Some(a.value.clone());
            },
            "num_pb" => {
                num_pb = a.value.parse().expect("num_pb should be an integer.");
            },
            "blif_model" => {
                blif_model = Some(a.value.clone());
            },
            "class" => {
                class = Some(a.value.clone());
            },
            _ => panic!("Unknown attribute in pb_type: {}", a.name.to_string()),
        };
    }
    assert!(pb_type_name.is_some());

    let mut pb_ports: Vec<Port> = Vec::new();
    let mut pb_types: Vec<PBType> = Vec::new();
    let mut pb_modes: Vec<PBMode> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "input" | "output" | "clock" => {
                        pb_ports.push(parse_port(&name.to_string(), &attributes));
                    },
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name.to_string(), &attributes, parser));
                    },
                    "mode" => {
                        pb_modes.push(parse_pb_mode(&name.to_string(), &attributes, parser));
                    }
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "pb_type" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return PBType {
        name: pb_type_name.unwrap(),
        num_pb: num_pb,
        blif_model: blif_model,
        class: class,
        ports: pb_ports,
        modes: pb_modes,
        pb_types: pb_types,
    }
}

fn parse_complex_block_list(_name: &str,
                            _attributes: &Vec<OwnedAttribute>,
                            parser: &mut EventReader<BufReader<File>>) -> Vec<PBType> {

    // TODO: Error check the name and the attributes.

    let mut complex_block_list: Vec<PBType> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        complex_block_list.push(parse_pb_type(&name.to_string(), &attributes, parser));
                    },
                    _ => panic!("Invalid tag in complex block list."),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "complexblocklist" {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    return complex_block_list;
}

// TODO: This result type should be changed to something better than std::io
pub fn parse(arch_file: &Path) -> std::io::Result<FPGAArch> {
    let file = File::open(arch_file)?;
    // Buffering is used for performance.
    let file = BufReader::new(file);

    let mut tiles: Vec<Tile> = Vec::new();
    let mut layouts: Vec<Layout> = Vec::new();
    let mut complex_block_list: Vec<PBType> = Vec::new();

    // TODO: We should ignore comments and maybe whitespace.
    let mut parser = EventReader::new(file);

    // TODO: We should check that the first tag is the architecture tag.

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "models" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "tiles" => {
                        // TODO: Need to check that we do not see multiple tiles tags.
                        tiles = parse_tiles(&name.to_string(), &attributes, &mut parser);
                    },
                    "layout" => {
                        // TODO: Need to check that we do not see multiple layout tags.
                        layouts = parse_layouts(&name.to_string(), &attributes, &mut parser);
                    },
                    "device" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "switchlist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "segmentlist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "complexblocklist" => {
                        // TODO: Need to check that we do not see multiple complex block tags.
                        complex_block_list = parse_complex_block_list(&name.to_string(), &attributes, &mut parser);
                    },
                    _ => {
                        // TODO: Raise an error here if a tag is found that is
                        //       not of the above types.
                    },
                };
            },
            Ok(XmlEvent::EndElement { name: _ }) => {
                // TODO: We should never see an end element if the sub-parsers
                //       are doing their job. This would imply that there is a
                //       problem.
                //       The only end element we should see is the architecture
                //       tag.
            },
            Ok(XmlEvent::EndDocument) => {
                break;
            },
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            },
            // There's more: https://docs.rs/xml/latest/xml/reader/enum.XmlEvent.html
            _ => {},
        };
    }

    return Ok(FPGAArch {
        models: Vec::new(),
        tiles: tiles,
        layouts: layouts,
        device: DeviceInfo {},
        switch_list: Vec::new(),
        segment_list: Vec::new(),
        complex_block_list: complex_block_list,
    });
}
