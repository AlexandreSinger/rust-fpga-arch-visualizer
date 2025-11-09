use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

pub struct Model {

}

#[derive(Debug)]
pub struct Tile {
    pub name: String,
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

pub struct PBType {
    pub name: String,
    // TODO: Add the ports.
    // TODO: Add the modes as an optional vector.
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

    let new_tile = Tile {
        name: tile_name,
    };

    // Skip the contents of the tile for now.
    // TODO: Add error check here.
    let _ = parser.skip();

    return new_tile;
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

fn parse_pb_type(_name: &str,
                 attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) -> PBType {
    let mut pb_type_name: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(pb_type_name.is_none());
                pb_type_name = Some(a.value.clone());
            },
            _ => panic!("Unknown attribute in pb_type"),
        };
    }
    assert!(pb_type_name.is_some());

    let _ = parser.skip();

    return PBType {
        name: pb_type_name.unwrap(),
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

    println!("{:?}", tiles);

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
