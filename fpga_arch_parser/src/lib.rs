use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

pub struct Model {

}

pub enum PinEquivalence {
    None,
    Full,
    Instance,
}

pub enum PortClass {
    None,
    LutIn,
    LutOut,
    FlipFlopD,
    FlipFlopQ,
    Clock,
    MemoryAddress,
    MemoryDataIn,
    MemoryWriteEn,
    MemoryDataOut,
    MemoryAddressFirst,
    MemoryDataInFirst,
    MemoryWriteEnFirst,
    MemoryDataOutFirst,
    MemoryAddressSecond,
    MemoryDataInSecond,
    MemoryWriteEnSecond,
    MemoryDataOutSecond,
    // FIXME: These are not documented by VTR. Documentation needs to be updated.
    MemoryReadEnFirst,
    MemoryReadEnSecond,
}

pub struct InputPort {
    pub name: String,
    pub num_pins: i32,
    pub equivalent: PinEquivalence,
    pub is_non_clock_global: bool,
    pub port_class: PortClass,
}

pub struct OutputPort {
    pub name: String,
    pub num_pins: i32,
    pub equivalent: PinEquivalence,
    pub port_class: PortClass,
}

pub struct ClockPort {
    pub name: String,
    pub num_pins: i32,
    pub equivalent: PinEquivalence,
    pub port_class: PortClass,
}

pub enum Port {
    Input(InputPort),
    Output(OutputPort),
    Clock(ClockPort),
}

pub enum TileSitePinMapping {
    Direct,
    Custom,
}

pub struct TileSite {
    pub pb_type: String,
    pub pin_mapping: TileSitePinMapping,
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

pub enum PinSide {
    Left,
    Right,
    Bottom,
    Top,
}

pub struct PinLoc {
    pub side: PinSide,
    pub xoffset: i32,
    pub yoffset: i32,
    pub pin_strings: Vec<String>,
}

pub struct CustomPinLocations {
    pub pin_locations: Vec<PinLoc>,
}

pub enum SubTilePinLocations {
    Spread,
    Perimeter,
    SpreadInputsPerimeterOutputs,
    Custom(CustomPinLocations),
}

pub struct SubTile {
    pub name: String,
    pub capacity: i32,
    pub equivalent_sites: Vec<TileSite>,
    pub ports: Vec<Port>,
    pub fc: SubTileFC,
    pub pin_locations: SubTilePinLocations,
}

pub struct Tile {
    pub name: String,
    // FIXME: Documentation. It is not clear from the documentation if tiles should
    //        have ports or not. The ZA archs do have ports, but most do not.
    pub ports: Vec<Port>,
    pub sub_tiles: Vec<SubTile>,
    pub width: i32,
    pub height: i32,
    pub area: Option<f32>,
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

pub struct SingleGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub x_expr: String,
    pub y_expr: String,
}

pub struct ColGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub repeat_x_expr: Option<String>,
    pub start_y_expr: String,
    pub incr_y_expr: String,
}

pub struct RowGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub incr_x_expr: String,
    pub start_y_expr: String,
    pub repeat_y_expr: Option<String>,
}

pub struct RegionGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub end_x_expr: String,
    pub repeat_x_expr: Option<String>,
    pub incr_x_expr: String,
    pub start_y_expr: String,
    pub end_y_expr: String,
    pub repeat_y_expr: Option<String>,
    pub incr_y_expr: String,
}

pub enum GridLocation {
    Fill(FillGridLocation),
    Perimeter(PerimeterGridLocation),
    Corners(CornersGridLocation),
    Single(SingleGridLocation),
    Col(ColGridLocation),
    Row(RowGridLocation),
    Region(RegionGridLocation),
}

pub struct AutoLayout {
    pub aspect_ratio: f32,
    pub grid_locations: Vec<GridLocation>,
}

pub struct FixedLayout {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub grid_locations: Vec<GridLocation>,
}

pub enum Layout {
    AutoLayout(AutoLayout),
    FixedLayout(FixedLayout),
}

pub enum SBType {
    Wilton,
    Subset,
    Universal,
    Custom,
}

pub struct GaussianChanWDist {
    pub peak: f32,
    pub width: f32,
    pub xpeak: f32,
    pub dc: f32,
}

pub struct UniformChanWDist {
    pub peak: f32,
}

pub struct PulseChanWDist {
    pub peak: f32,
    pub width: f32,
    pub xpeak: f32,
    pub dc: f32,
}

pub struct DeltaChanWDist {
    pub peak: f32,
    pub xpeak: f32,
    pub dc: f32,
}

pub enum ChanWDist {
    Gaussian(GaussianChanWDist),
    Uniform(UniformChanWDist),
    Pulse(PulseChanWDist),
    Delta(DeltaChanWDist),
}

pub struct DeviceInfo {
    // Sizing.
    pub r_min_w_nmos: f32,
    pub r_min_w_pmos: f32,
    // Connection block.
    pub input_switch_name: String,
    // Area.
    pub grid_logic_tile_area: f32,
    // Switch block.
    pub sb_type: SBType,
    //      NOTE: SB fs is required if the sb type is non-custom.
    pub sb_fs: Option<i32>,
    // Chan width distribution.
    pub x_distr: ChanWDist,
    pub y_distr: ChanWDist,
    // TODO: default_fc
}

pub struct Switch {

}

pub enum SegmentAxis {
    X,
    Y,
    XY,
    Z,
}

pub enum SegmentType {
    Bidir,
    Unidir,
}

pub enum SegmentResourceType {
    Gclk,
    General,
}

pub struct Segment {
    pub axis: SegmentAxis,
    pub name: String,
    pub length: i32,
    pub segment_type: SegmentType,
    pub res_type: SegmentResourceType,
    pub freq: f32,
    pub r_metal: f32,
    pub c_metal: f32,
}

pub struct PackPattern {
    pub name: String,
    pub in_port: String,
    pub out_port: String,
}

pub struct CompleteInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    // FIXME: The documentation needs to be updated. The documentation says there
    //        may be a single pack pattern; however, an interconnect may have many
    //        pack patterns.
    pub pack_patterns: Vec<PackPattern>,
}

pub struct DirectInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    pub pack_patterns: Vec<PackPattern>,
}

pub struct MuxInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    pub pack_patterns: Vec<PackPattern>,
}

pub enum Interconnect {
    Complete(CompleteInterconnect),
    Direct(DirectInterconnect),
    Mux(MuxInterconnect),
}

pub struct PBMode {
    pub name: String,
    pub pb_types: Vec<PBType>,
    pub interconnects: Vec<Interconnect>,
}

pub enum PBTypeClass {
    None,
    Lut,
    FlipFlop,
    Memory,
}

pub struct PBType {
    pub name: String,
    pub num_pb: i32,
    pub blif_model: Option<String>,
    pub class: PBTypeClass,
    pub ports: Vec<Port>,
    pub modes: Vec<PBMode>,
    pub pb_types: Vec<PBType>,
    pub interconnects: Vec<Interconnect>,
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
            _ => panic!("Unnexpected attribute in port: {}", a.name),
        };
    }

    assert!(port_name.is_some());
    assert!(num_pins.is_some());

    let pin_equivalance = match equivalent.as_str() {
        "none" => PinEquivalence::None,
        "full" => PinEquivalence::Full,
        "instance" => PinEquivalence::Instance,
        _ => panic!("Unknown pin equivalance: {}", equivalent),
    };

    let port_class = match port_class {
        None => PortClass::None,
        Some(class) => match class.as_str() {
            "lut_in" => PortClass::LutIn,
            "lut_out" => PortClass::LutOut,
            "D" => PortClass::FlipFlopD,
            "Q" => PortClass::FlipFlopQ,
            "clock" => PortClass::Clock,
            "address" => PortClass::MemoryAddress,
            "data_in" => PortClass::MemoryDataIn,
            "write_en" => PortClass::MemoryWriteEn,
            "data_out" => PortClass::MemoryDataOut,
            "address1" => PortClass::MemoryAddressFirst,
            "data_in1" => PortClass::MemoryDataInFirst,
            "write_en1" => PortClass::MemoryWriteEnFirst,
            "data_out1" => PortClass::MemoryDataOutFirst,
            "address2" => PortClass::MemoryAddressSecond,
            "data_in2" => PortClass::MemoryDataInSecond,
            "write_en2" => PortClass::MemoryWriteEnSecond,
            "data_out2" => PortClass::MemoryDataOutSecond,
            "read_en1" => PortClass::MemoryReadEnFirst,
            "read_en2" => PortClass::MemoryReadEnSecond,
            _ => panic!("Unknown port class: {}", class),
        },
    };

    // TODO: Check that non-clock global is only set for inputs.

    match name {
        "input" => Port::Input(InputPort {
            name: port_name.unwrap(),
            num_pins: num_pins.unwrap(),
            equivalent: pin_equivalance,
            is_non_clock_global,
            port_class,
        }),
        "output" => Port::Output(OutputPort {
            name: port_name.unwrap(),
            num_pins: num_pins.unwrap(),
            equivalent: pin_equivalance,
            port_class,
        }),
        "clock" => Port::Clock(ClockPort {
            name: port_name.unwrap(),
            num_pins: num_pins.unwrap(),
            equivalent: pin_equivalance,
            port_class,
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

    let site_pin_mapping = match site_pin_mapping.as_str() {
        "direct" => TileSitePinMapping::Direct,
        "custom" => TileSitePinMapping::Custom,
        _ => panic!("Unknown site pin mapping: {}", site_pin_mapping),
    };

    TileSite {
        pb_type: site_pb_type.unwrap(),
        pin_mapping: site_pin_mapping,
    }
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

    equivalent_sites
}

fn create_sub_tile_io_fc(ty: &str, val: &str) -> SubTileIOFC {
    match ty {
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
            _ => panic!("Unknown fc attribute: {}", a.name),
        };
    }

    assert!(in_type.is_some());
    assert!(in_val.is_some());
    assert!(out_type.is_some());
    assert!(out_val.is_some());

    let in_fc = create_sub_tile_io_fc(&in_type.unwrap(), &in_val.unwrap());
    let out_fc = create_sub_tile_io_fc(&out_type.unwrap(), &out_val.unwrap());

    SubTileFC {
        in_fc,
        out_fc,
    }
}

fn parse_pin_loc(_name: &str,
                 attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) -> PinLoc {
    let mut side: Option<String> = None;
    let mut xoffset: Option<i32> = None;
    let mut yoffset: Option<i32> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "side" => {
                assert!(side.is_none());
                side = Some(a.value.clone());
            },
            "xoffset" => {
                assert!(xoffset.is_none());
                xoffset = Some(a.value.parse().expect("xoffset expected to be an i32."));
            },
            "yoffset" => {
                assert!(yoffset.is_none());
                yoffset = Some(a.value.parse().expect("yoffset expected to be an i32."));
            },
            _ => panic!("Unnexpected attribute in loc: {}", a.name),
        };
    }

    let xoffset = xoffset.unwrap_or_default();
    let yoffset = yoffset.unwrap_or_default();

    let side = match side {
        Some(side) => match side.as_str() {
            "left" => PinSide::Left,
            "right" => PinSide::Right,
            "top" => PinSide::Top,
            "bottom" => PinSide::Bottom,
            _ => panic!("Unknown pin side: {}", side),
        },
        None => panic!("loc tag has no side attribute."),
    };

    // Parse the pin strings.
    let pin_strings: Vec<String>;
    match parser.next() {
        Ok(XmlEvent::Characters(text)) => {
            pin_strings = text.split_whitespace().map(|s| s.to_string()).collect();

            // Parse the end loc tag. This is just to make this method clean.
            match parser.next() {
                Ok(XmlEvent::EndElement { name }) => {
                    assert!(name.to_string() == "loc");
                },
                _ => panic!("Unexpected tag in loc tag"),
            };
        },
        Ok(XmlEvent::EndElement { name }) => {
            assert!(name.to_string() == "loc");

            // FIXME: The Stratix-IV has cases where a loc is provided with no
            //        pin strings. Need to update the documentation to make this
            //        clear what to do in this case.
            // For now, just make the pin strings empty.
            pin_strings = Vec::new();
        },
        _ => panic!("Unexpected XML element found in loc tag"),
    };

    PinLoc {
        side,
        xoffset,
        yoffset,
        pin_strings,
    }
}

fn parse_sub_tile_pin_locations(_name: &str,
                                attributes: &Vec<OwnedAttribute>,
                                parser: &mut EventReader<BufReader<File>>) -> SubTilePinLocations {
    let mut pattern: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "pattern" => {
                assert!(pattern.is_none());
                pattern = Some(a.value.clone());
            },
            _ => panic!("Unknown pin locations attribute: {}", a.name),
        };
    }

    let mut pin_locs: Vec<PinLoc> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "loc" => {
                        pin_locs.push(parse_pin_loc(&name.to_string(), &attributes, parser));
                    },
                    _ => panic!("Unnexpected tag in pinlocations: {}", name),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "pinlocations" {
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

    // TODO: If pin locs is defined for any pattern other than custom, something
    //       is wrong.

    match pattern {
        Some(pattern) => match pattern.as_str() {
            "spread" => SubTilePinLocations::Spread,
            "perimeter" => SubTilePinLocations::Perimeter,
            "spread_inputs_perimeter_outputs" => SubTilePinLocations::SpreadInputsPerimeterOutputs,
            "custom" => {
                SubTilePinLocations::Custom(CustomPinLocations{
                    pin_locations: pin_locs,
                })
            },
            _ => panic!("Unknown spreadpattern for pinlocations: {}", pattern),
        },
        None => panic!("pinlocations tag has no pattern attribute."),
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
    let mut pin_locations: Option<SubTilePinLocations> = None;
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
                    },
                    "pinlocations" => {
                        pin_locations = Some(parse_sub_tile_pin_locations(&name.to_string(), &attributes, parser));
                    },
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
    assert!(pin_locations.is_some());

    SubTile {
        name: sub_tile_name.unwrap(),
        capacity: sub_tile_capacity,
        equivalent_sites: equivalent_sites.unwrap(),
        ports,
        fc: sub_tile_fc.unwrap(),
        pin_locations: pin_locations.unwrap(),
    }
}

fn parse_tile(name: &str,
              attributes: &Vec<OwnedAttribute>,
              parser: &mut EventReader<BufReader<File>>) -> Tile {

    assert!(name == "tile");

    let mut tile_name: Option<String> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;
    let mut area: Option<f32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(tile_name.is_none());
                tile_name = Some(a.value.clone());
            },
            "width" => {
                assert!(width.is_none());
                width = Some(a.value.parse().expect("Tile width expected to be i32 type."));
            },
            "height" => {
                assert!(height.is_none());
                height = Some(a.value.parse().expect("Tile height expected to be i32 type."));
            },
            "area" => {
                assert!(area.is_none());
                area = Some(a.value.parse().expect("Tile area expected to be f32 type."));
            },
            _ => panic!("Unnexpected attribute in tile tag: {}", a),
        }
    }

    let tile_name = match tile_name {
        Some(n) => n,
        None => panic!("Tile name required but not given."),
    };

    // If the width or height is not provided, they are assumed to be 1.
    let width = width.unwrap_or(1);
    let height = height.unwrap_or(1);

    let mut ports: Vec<Port> = Vec::new();
    let mut sub_tiles: Vec<SubTile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "sub_tile" => {
                        sub_tiles.push(parse_sub_tile(&name.to_string(), &attributes, parser));
                    },
                    "input" | "output" | "clock" => {
                        ports.push(parse_port(&name.to_string(), &attributes));
                    },
                    _ => {
                        panic!("Unnexpected tag in tile: {}.", name);
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

    Tile {
        name: tile_name,
        ports,
        sub_tiles,
        width,
        height,
        area,
    }
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
                if name.to_string().as_str() == "tile" {
                    tiles.push(parse_tile(&name.to_string(), &attributes, parser));
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

    tiles
}

fn parse_grid_location(name: &str,
                       attributes: &Vec<OwnedAttribute>,
                       parser: &mut EventReader<BufReader<File>>) -> GridLocation {

    let mut pb_type: Option<String> = None;
    let mut priority: Option<i32> = None;
    let mut x_expr: Option<String> = None;
    let mut y_expr: Option<String> = None;
    let mut start_x_expr = String::from("0");
    let mut end_x_expr = String::from("W - 1");
    let mut repeat_x_expr: Option<String> = None;
    let mut incr_x_expr = String::from("w");
    let mut start_y_expr = String::from("0");
    let mut end_y_expr = String::from("H - 1");
    let mut repeat_y_expr: Option<String> = None;
    let mut incr_y_expr = String::from("h");

    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                pb_type = Some(a.value.clone());
            },
            "priority" => {
                priority = Some(a.value.parse().expect("Not a valid number"));
            },
            "x" => {
                assert!(x_expr.is_none());
                x_expr = Some(a.value.clone());
            },
            "y" => {
                assert!(y_expr.is_none());
                y_expr = Some(a.value.clone());
            },
            "startx" => {
                start_x_expr = a.value.clone();
            },
            "endx" => {
                end_x_expr = a.value.clone();
            },
            "repeatx" => {
                repeat_x_expr = Some(a.value.clone());
            },
            "incrx" => {
                incr_x_expr = a.value.clone();
            },
            "starty" => {
                start_y_expr = a.value.clone();
            },
            "endy" => {
                end_y_expr = a.value.clone();
            },
            "repeaty" => {
                repeat_y_expr = Some(a.value.clone());
            },
            "incry" => {
                incr_y_expr = a.value.clone();
            },
            _ => panic!("Unnexpected attribute in grid location: {}", a),
        };
    }

    if pb_type.is_none() || priority.is_none() {
        panic!("Grid location {name} missing type and/or priority");
    }

    // Skip the contents of the grid location tag.
    // TODO: Should parse metadata tag.
    let _ = parser.skip();

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
        "single" => {
            GridLocation::Single(SingleGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
                x_expr: x_expr.unwrap(),
                y_expr: y_expr.unwrap(),
            })
        },
        "col" => {
            GridLocation::Col(ColGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
                start_x_expr,
                repeat_x_expr,
                start_y_expr,
                incr_y_expr,
            })
        },
        "row" => {
            GridLocation::Row(RowGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
                start_x_expr,
                incr_x_expr,
                start_y_expr,
                repeat_y_expr,
            })
        },
        "region" => {
            GridLocation::Region(RegionGridLocation {
                pb_type: pb_type.unwrap(),
                priority: priority.unwrap(),
                start_x_expr,
                end_x_expr,
                repeat_x_expr,
                incr_x_expr,
                start_y_expr,
                end_y_expr,
                repeat_y_expr,
                incr_y_expr,
            })
        },
        _ => {
            panic!("Unknown grid location: {}", name);
        },
    }
}

fn parse_grid_location_list(layout_type_name: &str,
                            parser: &mut EventReader<BufReader<File>>) -> Vec<GridLocation> {
    let mut grid_locations: Vec<GridLocation> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                grid_locations.push(parse_grid_location(&name.to_string(), &attributes, parser));
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == layout_type_name {
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

    grid_locations
}

fn parse_auto_layout(name: &str,
                     attributes: &Vec<OwnedAttribute>,
                     parser: &mut EventReader<BufReader<File>>) -> AutoLayout {

    let mut aspect_ratio: f32 = 1.0;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "aspect_ratio" => {
                aspect_ratio = a.value.parse().expect("Invalid aspect ratio");
            },
            _ => {
                panic!("Unknown attribute for auto layout: {}", a.name);
            },
        }
    }

    let grid_locations = parse_grid_location_list(name, parser);

    AutoLayout {
        aspect_ratio,
        grid_locations,
    }
}

fn parse_fixed_layout(name: &str,
                      attributes: &Vec<OwnedAttribute>,
                      parser: &mut EventReader<BufReader<File>>) -> FixedLayout {
    let mut layout_name: Option<String> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(layout_name.is_none());
                layout_name = Some(a.value.clone());
            },
            "width" => {
                assert!(width.is_none());
                width = Some(a.value.parse().expect("Width for fixed layout expected to be i32."));
            },
            "height" => {
                assert!(height.is_none());
                height = Some(a.value.parse().expect("Height for fixed layout expected to be i32."));
            },
            _ => {
                panic!("Unknown attribute for fixed layout: {}", a.name);
            },
        }
    }

    let grid_locations = parse_grid_location_list(name, parser);

    FixedLayout {
        name: layout_name.unwrap(),
        width: width.unwrap(),
        height: height.unwrap(),
        grid_locations,
    }
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
                        layouts.push(Layout::FixedLayout(parse_fixed_layout(&name.to_string(), &attributes, parser)));
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

    layouts
}

fn parse_chan_w_dist(name: &str,
                     attributes: &Vec<OwnedAttribute>,
                     parser: &mut EventReader<BufReader<File>>) -> ChanWDist {
    assert!(name == "x" || name == "y");
    let mut distr: Option<String> = None;
    let mut peak: Option<f32> = None;
    let mut width: Option<f32> = None;
    let mut xpeak: Option<f32> = None;
    let mut dc: Option<f32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "distr" => {
                assert!(distr.is_none());
                distr = Some(a.value.clone());
            },
            "peak" => {
                assert!(peak.is_none());
                peak = Some(a.value.parse().expect("chan_w_dist peak expected to be f32"));
            },
            "width" => {
                assert!(width.is_none());
                width = Some(a.value.parse().expect("chan_w_dist width expected to be f32"));
            },
            "xpeak" => {
                assert!(xpeak.is_none());
                xpeak = Some(a.value.parse().expect("chan_w_dist xpeak expected to be f32"));
            },
            "dc" => {
                assert!(dc.is_none());
                dc = Some(a.value.parse().expect("chan_w_dist dc expected to be f32"));
            },
            _ => panic!("Unexpected attribute in chan_w_distr: {}", a),
        };
    }

    match parser.next() {
        Ok(XmlEvent::EndElement { name: end_name }) => {
            assert!(end_name.to_string() == name);
        },
        _ => panic!("Unnexpected tag in chan_w_distr x/y tag"),
    };

    match distr {
        Some(distr_str) => {
            match distr_str.as_str() {
                "gaussian" => ChanWDist::Gaussian(GaussianChanWDist {
                    peak: peak.unwrap(),
                    width: width.unwrap(),
                    xpeak: xpeak.unwrap(),
                    dc: dc.unwrap(),
                }),
                "uniform" => ChanWDist::Uniform(UniformChanWDist {
                    peak: peak.unwrap(),
                }),
                "pulse" => ChanWDist::Pulse(PulseChanWDist {
                    peak: peak.unwrap(),
                    width: width.unwrap(),
                    xpeak: xpeak.unwrap(),
                    dc: dc.unwrap(),
                }),
                "delta" => ChanWDist::Delta(DeltaChanWDist {
                    peak: peak.unwrap(),
                    xpeak: xpeak.unwrap(),
                    dc: dc.unwrap(),
                }),
                _ => panic!("Unknown distr for chan_w_distr: {}", distr_str),
            }
        },
        None => panic!("No distr provided for chan_w_distr"),
    }
}

fn parse_device(name: &str,
                attributes: &[OwnedAttribute],
                parser: &mut EventReader<BufReader<File>>) -> DeviceInfo {
    assert!(name == "device");
    assert!(attributes.is_empty());

    let mut r_min_w_nmos: Option<f32> = None;
    let mut r_min_w_pmos: Option<f32> = None;
    let mut grid_logic_tile_area: Option<f32> = None;
    let mut x_distr: Option<ChanWDist> = None;
    let mut y_distr: Option<ChanWDist> = None;
    let mut sb_type: Option<SBType> = None;
    let mut sb_fs: Option<i32> = None;
    let mut input_switch_name: Option<String> = None;

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "sizing" => {
                        for a in attributes {
                            match a.name.to_string().as_ref() {
                                "R_minW_nmos" => {
                                    assert!(r_min_w_nmos.is_none());
                                    r_min_w_nmos = Some(a.value.parse().expect("R_minW_nmos expected to be f32 type"));
                                },
                                "R_minW_pmos" => {
                                    assert!(r_min_w_pmos.is_none());
                                    r_min_w_pmos = Some(a.value.parse().expect("R_minW_pmos expected to be f32 type"));
                                },
                                _ => panic!("Unknown attribute for sizing tag: {}", a),
                            };
                        }
                        match parser.next() {
                            Ok(XmlEvent::EndElement { name: end_name }) => {
                                assert!(end_name.to_string() == "sizing");
                            },
                            _ => panic!("Unnexpected tag in sizing tag"),
                        };
                    },
                    "area" => {
                        for a in attributes {
                            match a.name.to_string().as_ref() {
                                "grid_logic_tile_area" => {
                                    assert!(grid_logic_tile_area.is_none());
                                    grid_logic_tile_area = Some(a.value.parse().expect("grid_logic_tile_area expected to be f32 type"));
                                },
                                _ => panic!("Unknown attribute for area tag: {}", a),
                            };
                        }
                        match parser.next() {
                            Ok(XmlEvent::EndElement { name: end_name }) => {
                                assert!(end_name.to_string() == "area");
                            },
                            _ => panic!("Unnexpected tag in area tag"),
                        };
                    },
                    "chan_width_distr" => {
                        loop {
                            match parser.next() {
                                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                                    match name.to_string().as_str() {
                                        "x" => {
                                            assert!(x_distr.is_none());
                                            x_distr = Some(parse_chan_w_dist(&name.to_string(), &attributes, parser));
                                        },
                                        "y" => {
                                            assert!(y_distr.is_none());
                                            y_distr = Some(parse_chan_w_dist(&name.to_string(), &attributes, parser));
                                        },
                                        _ => panic!("Unexpected tag in chan_width_distr: {}", name),
                                    };
                                },
                                Ok(XmlEvent::EndElement { name }) => {
                                    match name.to_string().as_str() {
                                        "chan_width_distr" => break,
                                        _ => panic!("Unexpected end tag in chan_width_distr: {}", name),
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

                    },
                    "switch_block" => {
                        for a in attributes {
                            match a.name.to_string().as_ref() {
                                "type" => {
                                    assert!(sb_type.is_none());
                                    sb_type = match a.value.as_ref() {
                                        "wilton" => Some(SBType::Wilton),
                                        "subset" => Some(SBType::Subset),
                                        "universal" => Some(SBType::Universal),
                                        "custom" => Some(SBType::Custom),
                                        _ => panic!("Unknown switch_block type: {}", a.value),
                                    };
                                },
                                "fs" => {
                                    assert!(sb_fs.is_none());
                                    sb_fs = Some(a.value.parse().expect("switch_block fs expected to be i32 type, got"));
                                },
                                _ => panic!("Unknown attribute for area tag: {}", a),
                            };
                        }
                        match parser.next() {
                            Ok(XmlEvent::EndElement { name: end_name }) => {
                                assert!(end_name.to_string() == "switch_block");
                            },
                            _ => panic!("Unnexpected tag in switch_block tag"),
                        };
                    },
                    "connection_block" => {
                        for a in attributes {
                            match a.name.to_string().as_ref() {
                                "input_switch_name" => {
                                    assert!(input_switch_name.is_none());
                                    input_switch_name = Some(a.value.clone());
                                },
                                _ => panic!("Unknown attribute for connection_block tag: {}", a),
                            };
                        }
                        match parser.next() {
                            Ok(XmlEvent::EndElement { name: end_name }) => {
                                assert!(end_name.to_string() == "connection_block");
                            },
                            _ => panic!("Unnexpected tag in connection_block tag"),
                        };

                    },
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "device" {
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

    DeviceInfo {
        r_min_w_nmos: r_min_w_nmos.unwrap(),
        r_min_w_pmos: r_min_w_pmos.unwrap(),
        input_switch_name: input_switch_name.unwrap(),
        grid_logic_tile_area: grid_logic_tile_area.unwrap(),
        sb_type: sb_type.unwrap(),
        sb_fs,
        x_distr: x_distr.unwrap(),
        y_distr: y_distr.unwrap(),
    }
}

fn parse_segment(name: &str,
                 attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) -> Segment {
    assert!(name == "segment");

    let mut axis = SegmentAxis::XY;
    let mut name: Option<String> = None;
    let mut length: Option<i32> = None;
    let mut segment_type: Option<SegmentType> = None;
    let mut res_type = SegmentResourceType::General;
    let mut freq: Option<f32> = None;
    let mut r_metal: Option<f32> = None;
    let mut c_metal: Option<f32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "axis" => {
                axis = match a.value.as_ref() {
                    "x" => SegmentAxis::X,
                    "y" => SegmentAxis::Y,
                    "z" => SegmentAxis::Z,
                    _ => panic!("Unknown segment axis for segment: {}", a.value),
                }
            },
            "name" => {
                assert!(name.is_none());
                name = Some(a.value.clone());
            },
            "length" => {
                assert!(length.is_none());
                length = Some(a.value.parse().expect("Segment length expected to be i32 type"));
            },
            "type" => {
                assert!(segment_type.is_none());
                segment_type = Some(match a.value.as_ref() {
                    "bidir" => SegmentType::Bidir,
                    "unidir" => SegmentType::Unidir,
                    _ => panic!("Unknown segment type: {}", a.value),
                });
            },
            "res_type" => {
                res_type = match a.value.as_ref() {
                    "GCLK" => SegmentResourceType::Gclk,
                    "GENERAL" => SegmentResourceType::General,
                    _ => panic!("Unknown segment resource type: {}", a.value),
                };
            },
            "freq" => {
                assert!(freq.is_none());
                freq = Some(a.value.parse().expect("Segment frequency expected to be f32 type"));
            },
            "Rmetal" => {
                assert!(r_metal.is_none());
                r_metal = Some(a.value.parse().expect("Segment Rmetal expected to be f32 type"));
            },
            "Cmetal" => {
                assert!(c_metal.is_none());
                c_metal = Some(a.value.parse().expect("Segment Cmetal expected to be f32 type"));
            },
            _ => panic!("Unknown attribute in segment tag: {}", a),
        };
    }

    // TODO: Need to parse the mux, sb, and cb tags. For now just ignore.
    let _ = parser.skip();

    // DOCUMENTATION ISSUE: Some architectures do not specify names. This either
    //                      needs to be enforced or documented as optional.
    let name = match name {
        Some(n) => n,
        None => String::from("UnnamedSegment"),
    };

    Segment {
        name,
        axis,
        length: length.unwrap(),
        segment_type: segment_type.unwrap(),
        res_type,
        freq: freq.unwrap(),
        r_metal: r_metal.unwrap(),
        c_metal: c_metal.unwrap(),
    }
}

fn parse_segment_list(name: &str,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Vec<Segment> {
    assert!(name == "segmentlist");
    assert!(attributes.is_empty());

    let mut segments: Vec<Segment> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "segment" => {
                        segments.push(parse_segment(&name.to_string(), &attributes, parser));
                    },
                    _ => panic!("Unnexpected tag in segmentlist: {}", name),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "segmentlist" => break,
                    _ => panic!("Unnexpected end element in segmentlist: {}", name),
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

    segments
}

fn parse_pack_pattern(name: &str,
                      attributes: &Vec<OwnedAttribute>,
                      parser: &mut EventReader<BufReader<File>>) -> PackPattern {
    assert!(name == "pack_pattern");

    let mut pattern_name: Option<String> = None;
    let mut in_port: Option<String> = None;
    let mut out_port: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(pattern_name.is_none());
                pattern_name = Some(a.value.clone());
            },
            "in_port" => {
                assert!(in_port.is_none());
                in_port = Some(a.value.clone());
            },
            "out_port" => {
                assert!(out_port.is_none());
                out_port = Some(a.value.clone());
            },
            _ => panic!("Unknown attribute for pack_pattern: {}", a),
        };
    }

    assert!(pattern_name.is_some());
    assert!(in_port.is_some());
    assert!(out_port.is_some());

    match parser.next() {
        Ok(XmlEvent::EndElement { name }) => {
            assert!(name.to_string() == "pack_pattern");
        },
        _ => panic!("Unnexpected end tag."),
    };

    PackPattern {
        name: pattern_name.unwrap(),
        in_port: in_port.unwrap(),
        out_port: out_port.unwrap(),
    }
}

fn parse_interconnect(name: &str,
                      attributes: &Vec<OwnedAttribute>,
                      parser: &mut EventReader<BufReader<File>>) -> Interconnect {

    let mut inter_name: Option<String> = None;
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                assert!(inter_name.is_none());
                inter_name = Some(a.value.clone());
            },
            "input" => {
                assert!(input.is_none());
                input = Some(a.value.clone());
            },
            "output" => {
                assert!(output.is_none());
                output = Some(a.value.clone());
            },
            _ => panic!("Unknown attribute for {} tag: {}", name, a),
        };
    }

    assert!(inter_name.is_some());
    assert!(input.is_some());
    assert!(output.is_some());

    let mut pack_patterns: Vec<PackPattern> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if name.to_string().as_str() == "pack_pattern" {
                    pack_patterns.push(parse_pack_pattern(&name.to_string(), &attributes, parser));
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "direct" | "mux" | "complete" => break,
                    _ => {},
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

    match name {
        "direct" => Interconnect::Direct(DirectInterconnect {
            name: inter_name.unwrap(),
            input: input.unwrap(),
            output: output.unwrap(),
            pack_patterns,
        }),
        "mux" => Interconnect::Mux(MuxInterconnect {
            name: inter_name.unwrap(),
            input: input.unwrap(),
            output: output.unwrap(),
            pack_patterns,
        }),
        "complete" => Interconnect::Complete(CompleteInterconnect {
            name: inter_name.unwrap(),
            input: input.unwrap(),
            output: output.unwrap(),
            pack_patterns,
        }),
        _ => panic!("Unknown interconnect tag: {}", name),
    }
}

fn parse_interconnects(name: &str,
                       attributes: &[OwnedAttribute],
                       parser: &mut EventReader<BufReader<File>>) -> Vec<Interconnect> {
    assert!(name == "interconnect");
    assert!(attributes.is_empty());

    let mut interconnects: Vec<Interconnect> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "direct" | "mux" | "complete" => {
                        interconnects.push(parse_interconnect(&name.to_string(), &attributes, parser));
                    },
                    _ => {},
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == "interconnect" {
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

    interconnects
}

fn parse_pb_mode(_name: &str,
                 attributes: &Vec<OwnedAttribute>,
                 parser: &mut EventReader<BufReader<File>>) -> PBMode {
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
    let mut interconnects: Vec<Interconnect> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name.to_string(), &attributes, parser));
                    },
                    "interconnect" => {
                        // TODO: Check that there is only a single interconnect tag.
                        interconnects = parse_interconnects(&name.to_string(), &attributes, parser);
                    }
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

    PBMode {
        name: mode_name.unwrap(),
        pb_types,
        interconnects,
    }
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
            _ => panic!("Unknown attribute in pb_type: {}", a.name),
        };
    }
    assert!(pb_type_name.is_some());

    let mut pb_ports: Vec<Port> = Vec::new();
    let mut pb_types: Vec<PBType> = Vec::new();
    let mut pb_modes: Vec<PBMode> = Vec::new();
    let mut interconnects: Vec<Interconnect> = Vec::new();
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
                    },
                    "interconnect" => {
                        // TODO: Check that there is only a single interconnect tag.
                        interconnects = parse_interconnects(&name.to_string(), &attributes, parser);
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

    let pb_class = match class {
        None => PBTypeClass::None,
        Some(class_name) => match class_name.as_str() {
            "lut" => PBTypeClass::Lut,
            "flipflop" => PBTypeClass::FlipFlop,
            "memory" => PBTypeClass::Memory,
            _ => panic!("Unknown PB class: {}", class_name),
        },
    };

    PBType {
        name: pb_type_name.unwrap(),
        num_pb,
        blif_model,
        class: pb_class,
        ports: pb_ports,
        modes: pb_modes,
        pb_types,
        interconnects,
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

    complex_block_list
}

// TODO: This result type should be changed to something better than std::io
pub fn parse(arch_file: &Path) -> std::io::Result<FPGAArch> {
    let file = File::open(arch_file)?;
    // Buffering is used for performance.
    let file = BufReader::new(file);

    let mut tiles: Vec<Tile> = Vec::new();
    let mut layouts: Vec<Layout> = Vec::new();
    let mut device: Option<DeviceInfo> = None;
    let mut segment_list: Vec<Segment> = Vec::new();
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
                        assert!(device.is_none());
                        device = Some(parse_device(&name.to_string(), &attributes, &mut parser));
                    },
                    "switchlist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "segmentlist" => {
                        assert!(segment_list.is_empty());
                        segment_list = parse_segment_list(&name.to_string(), &attributes, &mut parser);
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

    assert!(device.is_some());

    Ok(FPGAArch {
        models: Vec::new(),
        tiles,
        layouts,
        device: device.unwrap(),
        switch_list: Vec::new(),
        segment_list,
        complex_block_list,
    })
}
