use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::common::{Position, TextPosition};
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
pub enum FPGAArchParseError {
    ArchFileOpenError(String),
    MissingRequiredTag(String),
    MissingRequiredAttribute(String, TextPosition),
    InvalidTag(String, TextPosition),
    XMLParseError(String, TextPosition),
    UnknownAttribute(String, TextPosition),
    DuplicateTag(String, TextPosition),
    DuplicateAttribute(String, TextPosition),
    UnexpectedEndTag(String, TextPosition),
    AttributeParseError(String, TextPosition),
    UnexpectedEndOfDocument(String),
}

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

fn parse_port(tag_name: &OwnedName,
              attributes: &[OwnedAttribute],
              parser: &mut EventReader<BufReader<File>>) -> Result<Port, FPGAArchParseError> {
    let mut port_name: Option<String> = None;
    let mut num_pins: Option<i32> = None;
    let mut equivalent: Option<PinEquivalence> = None;
    let mut is_non_clock_global: Option<bool> = None;
    let mut port_class: Option<PortClass> = None;

    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                port_name = match port_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "num_pins" => {
                num_pins = match num_pins {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "equivalent" => {
                equivalent = match equivalent {
                    None => match a.value.as_str() {
                        "none" => Some(PinEquivalence::None),
                        "full" => Some(PinEquivalence::Full),
                        "instance" => Some(PinEquivalence::Instance),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("Unknown pin equivalence: {}", a.value), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "is_non_clock_global" => {
                is_non_clock_global = match is_non_clock_global {
                    None => match tag_name.to_string().as_str() {
                        "input" => match a.value.parse() {
                            Ok(v) => Some(v),
                            Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                        },
                        _ => return Err(FPGAArchParseError::AttributeParseError("is_non_clock_global attribute only valid in input tag.".to_string(), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "port_class" => {
                port_class = match port_class {
                    None => match a.value.as_str() {
                        "lut_in" => Some(PortClass::LutIn),
                        "lut_out" => Some(PortClass::LutOut),
                        "D" => Some(PortClass::FlipFlopD),
                        "Q" => Some(PortClass::FlipFlopQ),
                        "clock" => Some(PortClass::Clock),
                        "address" => Some(PortClass::MemoryAddress),
                        "data_in" => Some(PortClass::MemoryDataIn),
                        "write_en" => Some(PortClass::MemoryWriteEn),
                        "data_out" => Some(PortClass::MemoryDataOut),
                        "address1" => Some(PortClass::MemoryAddressFirst),
                        "data_in1" => Some(PortClass::MemoryDataInFirst),
                        "write_en1" => Some(PortClass::MemoryWriteEnFirst),
                        "data_out1" => Some(PortClass::MemoryDataOutFirst),
                        "address2" => Some(PortClass::MemoryAddressSecond),
                        "data_in2" => Some(PortClass::MemoryDataInSecond),
                        "write_en2" => Some(PortClass::MemoryWriteEnSecond),
                        "data_out2" => Some(PortClass::MemoryDataOutSecond),
                        "read_en1" => Some(PortClass::MemoryReadEnFirst),
                        "read_en2" => Some(PortClass::MemoryReadEnSecond),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("Unknown port class: {}", a.value), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let port_name = match port_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let num_pins = match num_pins {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("num_pins".to_string(), parser.position())),
    };
    let equivalent = equivalent.unwrap_or(PinEquivalence::None);
    let is_non_clock_global = is_non_clock_global.unwrap_or(false);
    let port_class = port_class.unwrap_or(PortClass::None);

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() != tag_name.to_string() {
                    return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position()));
                }
                break;
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(tag_name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        };
    }

    match tag_name.to_string().as_ref() {
        "input" => Ok(Port::Input(InputPort {
            name: port_name,
            num_pins,
            equivalent,
            is_non_clock_global,
            port_class,
        })),
        "output" => Ok(Port::Output(OutputPort {
            name: port_name,
            num_pins,
            equivalent,
            port_class,
        })),
        "clock" => Ok(Port::Clock(ClockPort {
            name: port_name,
            num_pins,
            equivalent,
            port_class,
        })),
        _ => Err(FPGAArchParseError::InvalidTag(format!("Unknown port tag: {tag_name}"), parser.position())),
    }
}

fn parse_tile_site(name: &OwnedName,
                   attributes: &[OwnedAttribute],
                   parser: &mut EventReader<BufReader<File>>) -> Result<TileSite, FPGAArchParseError> {
    assert!(name.to_string() == "site");

    let mut site_pb_type: Option<String> = None;
    let mut site_pin_mapping: Option<TileSitePinMapping> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "pb_type" => {
                site_pb_type = match site_pb_type {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "pin_mapping" => {
                site_pin_mapping = match site_pin_mapping {
                    None => match a.value.as_str() {
                        "direct" => Some(TileSitePinMapping::Direct),
                        "custom" => Some(TileSitePinMapping::Custom),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("Unknown site pin mapping: {}", a.value), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let site_pb_type = match site_pb_type {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("pb_type".to_string(), parser.position())),
    };
    let site_pin_mapping = site_pin_mapping.unwrap_or(TileSitePinMapping::Direct);

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "site" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument("fc".to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        };
    }

    Ok(TileSite {
        pb_type: site_pb_type,
        pin_mapping: site_pin_mapping,
    })
}

fn parse_equivalent_sites(name: &OwnedName,
                          attributes: &[OwnedAttribute],
                          parser: &mut EventReader<BufReader<File>>) -> Result<Vec<TileSite>, FPGAArchParseError> {
    assert!(name.to_string() == "equivalent_sites");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut equivalent_sites: Vec<TileSite> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "site" => {
                        equivalent_sites.push(parse_tile_site(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "equivalent_sites" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    // TODO: Check the documentation. Is it allowed for equivalent sites to be empty?

    Ok(equivalent_sites)
}

fn create_sub_tile_io_fc(ty: &str,
                         val: &str,
                         parser: &EventReader<BufReader<File>>) -> Result<SubTileIOFC, FPGAArchParseError> {
    match ty {
        "frac" => {
            Ok(SubTileIOFC::Frac(SubTileFracFC {
                val: match val.parse() {
                    Ok(v) => v,
                    Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{val}: {e}"), parser.position())),
                },
            }))
        },
        "abs" => {
            Ok(SubTileIOFC::Abs(SubTileAbsFC {
                val: match val.parse() {
                    Ok(v) => v,
                    Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{val}: {e}"), parser.position())),
                },
            }))
        },
        _ => Err(FPGAArchParseError::AttributeParseError(format!("Unknown fc_type: {}", ty), parser.position())),
    }
}

fn parse_sub_tile_fc(name: &OwnedName,
                     attributes: &[OwnedAttribute],
                     parser: &mut EventReader<BufReader<File>>) -> Result<SubTileFC, FPGAArchParseError> {
    assert!(name.to_string() == "fc");

    let mut in_type: Option<String> = None;
    let mut in_val: Option<String> = None;
    let mut out_type: Option<String> = None;
    let mut out_val: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "in_type" => {
                in_type = match in_type {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "in_val" => {
                in_val = match in_val {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "out_type" => {
                out_type = match out_type {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "out_val" => {
                out_val = match out_val {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let in_type = match in_type {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("in_type".to_string(), parser.position())),
    };
    let in_val = match in_val {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("in_val".to_string(), parser.position())),
    };
    let out_type = match out_type {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("out_type".to_string(), parser.position())),
    };
    let out_val = match out_val {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("out_val".to_string(), parser.position())),
    };

    let in_fc = create_sub_tile_io_fc(&in_type, &in_val, parser)?;
    let out_fc = create_sub_tile_io_fc(&out_type, &out_val, parser)?;

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                match name.to_string().as_str() {
                    "fc_override" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "fc" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument("fc".to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        };
    }

    Ok(SubTileFC {
        in_fc,
        out_fc,
    })
}

fn parse_pin_loc(name: &OwnedName,
                 attributes: &[OwnedAttribute],
                 parser: &mut EventReader<BufReader<File>>) -> Result<PinLoc, FPGAArchParseError> {
    assert!(name.to_string() == "loc");

    let mut side: Option<PinSide> = None;
    let mut xoffset: Option<i32> = None;
    let mut yoffset: Option<i32> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "side" => {
                side = match side {
                    None => match a.value.as_str() {
                        "left" => Some(PinSide::Left),
                        "right" => Some(PinSide::Right),
                        "top" => Some(PinSide::Top),
                        "bottom" => Some(PinSide::Bottom),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("Unknown pin side: {}", a.value), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "xoffset" => {
                xoffset = match xoffset {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "yoffset" => {
                yoffset = match yoffset {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let side = match side {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("side".to_string(), parser.position())),
    };
    let xoffset = xoffset.unwrap_or_default();
    let yoffset = yoffset.unwrap_or_default();

    // Parse the pin strings.
    let mut pin_strings: Option<Vec<String>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(text)) => {
                pin_strings = match pin_strings {
                    None => Some(text.split_whitespace().map(|s| s.to_string()).collect()),
                    Some(_) => return Err(FPGAArchParseError::InvalidTag("Duplicate characters within loc tag.".to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "loc" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        };
    }

    // FIXME: The Stratix-IV has cases where a loc is provided with no
    //        pin strings. Need to update the documentation to make this
    //        clear what to do in this case.
    // For now, just make the pin strings empty.
    let pin_strings = pin_strings.unwrap_or_default();

    Ok(PinLoc {
        side,
        xoffset,
        yoffset,
        pin_strings,
    })
}

fn parse_sub_tile_pin_locations(name: &OwnedName,
                                attributes: &[OwnedAttribute],
                                parser: &mut EventReader<BufReader<File>>) -> Result<SubTilePinLocations, FPGAArchParseError> {
    assert!(name.to_string() == "pinlocations");

    let mut pattern: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "pattern" => {
                pattern = match pattern {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let pattern = match pattern {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("pattern".to_string(), parser.position())),
    };

    let mut pin_locs: Vec<PinLoc> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "loc" => {
                        // If pin locations are defined for any patter other than
                        // custom, something is wrong.
                        if pattern != "custom" {
                            return Err(FPGAArchParseError::InvalidTag("Pin locations can only be given for custom pattern".to_string(), parser.position()));
                        }
                        pin_locs.push(parse_pin_loc(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "pinlocations" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    match pattern.as_str() {
        "spread" => Ok(SubTilePinLocations::Spread),
        "perimeter" => Ok(SubTilePinLocations::Perimeter),
        "spread_inputs_perimeter_outputs" => Ok(SubTilePinLocations::SpreadInputsPerimeterOutputs),
        "custom" => {
            Ok(SubTilePinLocations::Custom(CustomPinLocations{
                pin_locations: pin_locs,
            }))
        },
        _ => Err(FPGAArchParseError::AttributeParseError(format!("Unknown spreadpattern for pinlocations: {}", pattern), parser.position())),
    }
}

fn parse_sub_tile(name: &OwnedName,
                  attributes: &[OwnedAttribute],
                  parser: &mut EventReader<BufReader<File>>) -> Result<SubTile, FPGAArchParseError> {
    assert!(name.to_string() == "sub_tile");

    let mut sub_tile_name: Option<String> = None;
    let mut sub_tile_capacity: Option<i32> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                sub_tile_name = match sub_tile_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "capacity" => {
                sub_tile_capacity = match sub_tile_capacity {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let sub_tile_name = match sub_tile_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let sub_tile_capacity = sub_tile_capacity.unwrap_or(1);

    let mut equivalent_sites: Option<Vec<TileSite>> = None;
    let mut ports: Vec<Port> = Vec::new();
    let mut sub_tile_fc: Option<SubTileFC> = None;
    let mut pin_locations: Option<SubTilePinLocations> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "equivalent_sites" => {
                        equivalent_sites = match equivalent_sites {
                            None => Some(parse_equivalent_sites(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "input" | "output" | "clock" => {
                        ports.push(parse_port(&name, &attributes, parser)?);
                    },
                    "fc" => {
                        sub_tile_fc = match sub_tile_fc {
                            None => Some(parse_sub_tile_fc(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "pinlocations" => {
                        pin_locations = match pin_locations {
                            None => Some(parse_sub_tile_pin_locations(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "sub_tile" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    let equivalent_sites = match equivalent_sites {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<equivalent_sites>".to_string())),
    };
    let sub_tile_fc = match sub_tile_fc {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<fc>".to_string())),
    };
    let pin_locations = match pin_locations {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<pinlocations>".to_string())),
    };

    Ok(SubTile {
        name: sub_tile_name,
        capacity: sub_tile_capacity,
        equivalent_sites,
        ports,
        fc: sub_tile_fc,
        pin_locations,
    })
}

fn parse_tile(name: &OwnedName,
              attributes: &[OwnedAttribute],
              parser: &mut EventReader<BufReader<File>>) -> Result<Tile, FPGAArchParseError> {
    assert!(name.to_string() == "tile");

    let mut tile_name: Option<String> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;
    let mut area: Option<f32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                tile_name = match tile_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "width" => {
                width = match width {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "height" => {
                height = match height {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "area" => {
                area = match area {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        }
    }

    let tile_name = match tile_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
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
                        sub_tiles.push(parse_sub_tile(&name, &attributes, parser)?);
                    },
                    "input" | "output" | "clock" => {
                        ports.push(parse_port(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "tile" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(Tile {
        name: tile_name,
        ports,
        sub_tiles,
        width,
        height,
        area,
    })
}

fn parse_tiles(name: &OwnedName,
               attributes: &[OwnedAttribute],
               parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Tile>, FPGAArchParseError> {
    assert!(name.to_string() == "tiles");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    // Iterate over the parser until we reach the EndElement for tile.
    let mut tiles: Vec<Tile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "tile" => {
                        tiles.push(parse_tile(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "tiles" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(tiles)
}

fn parse_grid_location(name: &OwnedName,
                       attributes: &[OwnedAttribute],
                       parser: &mut EventReader<BufReader<File>>) -> Result<GridLocation, FPGAArchParseError> {

    let mut pb_type: Option<String> = None;
    let mut priority: Option<i32> = None;
    let mut x_expr: Option<String> = None;
    let mut y_expr: Option<String> = None;
    let mut start_x_expr: Option<String> = None;
    let mut end_x_expr: Option<String> = None;
    let mut repeat_x_expr: Option<String> = None;
    let mut incr_x_expr: Option<String> = None;
    let mut start_y_expr: Option<String> = None;
    let mut end_y_expr: Option<String> = None;
    let mut repeat_y_expr: Option<String> = None;
    let mut incr_y_expr: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                pb_type = match pb_type {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "priority" => {
                priority = match priority {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "x" => {
                x_expr = match x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "y" => {
                y_expr = match y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "startx" => {
                start_x_expr = match start_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "endx" => {
                end_x_expr = match end_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "repeatx" => {
                repeat_x_expr = match repeat_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "incrx" => {
                incr_x_expr = match incr_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "starty" => {
                start_y_expr = match start_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "endy" => {
                end_y_expr = match end_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "repeaty" => {
                repeat_y_expr = match repeat_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "incry" => {
                incr_y_expr = match incr_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let pb_type = match pb_type {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("type".to_string(), parser.position())),
    };
    let priority = match priority {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("priority".to_string(), parser.position())),
    };

    let start_x_expr = start_x_expr.unwrap_or(String::from("0"));
    let end_x_expr = end_x_expr.unwrap_or(String::from("W - 1"));
    let incr_x_expr = incr_x_expr.unwrap_or(String::from("w"));
    let start_y_expr = start_y_expr.unwrap_or(String::from("0"));
    let end_y_expr = end_y_expr.unwrap_or(String::from("H - 1"));
    let incr_y_expr = incr_y_expr.unwrap_or(String::from("h"));

    // Skip the contents of the grid location tag.
    // TODO: Should parse metadata tag.
    let _ = parser.skip();

    match name.to_string().as_ref() {
        "perimeter" => {
            Ok(GridLocation::Perimeter(PerimeterGridLocation {
                pb_type,
                priority,
            }))
        },
        "corners" => {
            Ok(GridLocation::Corners(CornersGridLocation {
                pb_type,
                priority,
            }))
        },
        "fill" => {
            Ok(GridLocation::Fill(FillGridLocation {
                pb_type,
                priority,
            }))
        },
        "single" => {
            let x_expr = match x_expr {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("x".to_string(), parser.position())),
            };
            let y_expr = match y_expr {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("y".to_string(), parser.position())),
            };
            Ok(GridLocation::Single(SingleGridLocation {
                pb_type,
                priority,
                x_expr,
                y_expr,
            }))
        },
        "col" => {
            Ok(GridLocation::Col(ColGridLocation {
                pb_type,
                priority,
                start_x_expr,
                repeat_x_expr,
                start_y_expr,
                incr_y_expr,
            }))
        },
        "row" => {
            Ok(GridLocation::Row(RowGridLocation {
                pb_type,
                priority,
                start_x_expr,
                incr_x_expr,
                start_y_expr,
                repeat_y_expr,
            }))
        },
        "region" => {
            Ok(GridLocation::Region(RegionGridLocation {
                pb_type,
                priority,
                start_x_expr,
                end_x_expr,
                repeat_x_expr,
                incr_x_expr,
                start_y_expr,
                end_y_expr,
                repeat_y_expr,
                incr_y_expr,
            }))
        },
        _ => Err(FPGAArchParseError::InvalidTag(format!("Unknown grid location: {name}"), parser.position())),
    }
}

fn parse_grid_location_list(layout_type_name: &OwnedName,
                            parser: &mut EventReader<BufReader<File>>) -> Result<Vec<GridLocation>, FPGAArchParseError> {
    let mut grid_locations: Vec<GridLocation> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                grid_locations.push(parse_grid_location(&name, &attributes, parser)?);
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == layout_type_name.to_string() {
                    break;
                } else {
                    return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position()));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(layout_type_name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(grid_locations)
}

fn parse_auto_layout(name: &OwnedName,
                     attributes: &[OwnedAttribute],
                     parser: &mut EventReader<BufReader<File>>) -> Result<AutoLayout, FPGAArchParseError> {
    assert!(name.to_string() == "auto_layout");

    let mut aspect_ratio: Option<f32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "aspect_ratio" => {
                aspect_ratio = match aspect_ratio {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        }
    }

    let aspect_ratio = aspect_ratio.unwrap_or(1.0);

    let grid_locations = parse_grid_location_list(name, parser)?;

    Ok(AutoLayout {
        aspect_ratio,
        grid_locations,
    })
}

fn parse_fixed_layout(name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<FixedLayout, FPGAArchParseError> {
    assert!(name.to_string() == "fixed_layout");

    let mut layout_name: Option<String> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                layout_name = match layout_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "width" => {
                width = match width {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "height" => {
                height = match height {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        }
    }

    let layout_name = match layout_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let width = match width {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("width".to_string(), parser.position())),
    };
    let height = match height {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("height".to_string(), parser.position())),
    };

    let grid_locations = parse_grid_location_list(name, parser)?;

    Ok(FixedLayout {
        name: layout_name,
        width,
        height,
        grid_locations,
    })
}

fn parse_layouts(name: &OwnedName,
                 attributes: &[OwnedAttribute],
                 parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Layout>, FPGAArchParseError> {
    assert!(name.to_string() == "layout");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut layouts: Vec<Layout> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "auto_layout" => {
                        layouts.push(Layout::AutoLayout(parse_auto_layout(&name, &attributes, parser)?));
                    },
                    "fixed_layout" => {
                        layouts.push(Layout::FixedLayout(parse_fixed_layout(&name, &attributes, parser)?));
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "layout" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(layouts)
}

fn parse_chan_w_dist(name: &str,
                     attributes: &Vec<OwnedAttribute>,
                     parser: &mut EventReader<BufReader<File>>) -> Result<ChanWDist, FPGAArchParseError> {
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
                "gaussian" => Ok(ChanWDist::Gaussian(GaussianChanWDist {
                    peak: peak.unwrap(),
                    width: width.unwrap(),
                    xpeak: xpeak.unwrap(),
                    dc: dc.unwrap(),
                })),
                "uniform" => Ok(ChanWDist::Uniform(UniformChanWDist {
                    peak: peak.unwrap(),
                })),
                "pulse" => Ok(ChanWDist::Pulse(PulseChanWDist {
                    peak: peak.unwrap(),
                    width: width.unwrap(),
                    xpeak: xpeak.unwrap(),
                    dc: dc.unwrap(),
                })),
                "delta" => Ok(ChanWDist::Delta(DeltaChanWDist {
                    peak: peak.unwrap(),
                    xpeak: xpeak.unwrap(),
                    dc: dc.unwrap(),
                })),
                _ => panic!("Unknown distr for chan_w_distr: {}", distr_str),
            }
        },
        None => panic!("No distr provided for chan_w_distr"),
    }
}

fn parse_device(name: &OwnedName,
                attributes: &[OwnedAttribute],
                parser: &mut EventReader<BufReader<File>>) -> Result<DeviceInfo, FPGAArchParseError> {
    assert!(name.to_string() == "device");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

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
                                            x_distr = Some(parse_chan_w_dist(&name.to_string(), &attributes, parser)?);
                                        },
                                        "y" => {
                                            assert!(y_distr.is_none());
                                            y_distr = Some(parse_chan_w_dist(&name.to_string(), &attributes, parser)?);
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
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            // TODO: Handle the other cases.
            _ => {},
        }
    };

    Ok(DeviceInfo {
        r_min_w_nmos: r_min_w_nmos.unwrap(),
        r_min_w_pmos: r_min_w_pmos.unwrap(),
        input_switch_name: input_switch_name.unwrap(),
        grid_logic_tile_area: grid_logic_tile_area.unwrap(),
        sb_type: sb_type.unwrap(),
        sb_fs,
        x_distr: x_distr.unwrap(),
        y_distr: y_distr.unwrap(),
    })
}

fn parse_segment(name: &OwnedName,
                 attributes: &[OwnedAttribute],
                 parser: &mut EventReader<BufReader<File>>) -> Result<Segment, FPGAArchParseError> {
    assert!(name.to_string() == "segment");

    let mut axis: Option<SegmentAxis> = None;
    let mut name: Option<String> = None;
    let mut length: Option<i32> = None;
    let mut segment_type: Option<SegmentType> = None;
    let mut res_type: Option<SegmentResourceType> = None;
    let mut freq: Option<f32> = None;
    let mut r_metal: Option<f32> = None;
    let mut c_metal: Option<f32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "axis" => {
                axis = match axis {
                    None => match a.value.as_ref() {
                        "x" => Some(SegmentAxis::X),
                        "y" => Some(SegmentAxis::Y),
                        "z" => Some(SegmentAxis::Z),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: Unknown segment axis"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "name" => {
                name = match name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "length" => {
                length = match length {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "type" => {
                segment_type = match segment_type {
                    None => match a.value.as_ref() {
                        "bidir" => Some(SegmentType::Bidir),
                        "unidir" => Some(SegmentType::Unidir),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: Unknown segment type"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "res_type" => {
                res_type = match res_type {
                    None => match a.value.as_ref() {
                        "GCLK" => Some(SegmentResourceType::Gclk),
                        "GENERAL" => Some(SegmentResourceType::General),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: Unknown segment resource type"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "freq" => {
                freq = match freq {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "Rmetal" => {
                r_metal = match r_metal {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "Cmetal" => {
                c_metal = match c_metal {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    // DOCUMENTATION ISSUE: Some architectures do not specify names. This either
    //                      needs to be enforced or documented as optional.
    let name = match name {
        Some(n) => n,
        None => String::from("UnnamedSegment"),
    };
    let axis = axis.unwrap_or(SegmentAxis::XY);
    let length = match length {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("length".to_string(), parser.position())),
    };
    let segment_type = match segment_type {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("type".to_string(), parser.position())),
    };
    let freq = match freq {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("freq".to_string(), parser.position())),
    };
    let r_metal = match r_metal {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("Rmetal".to_string(), parser.position())),
    };
    let c_metal = match c_metal {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("Cmetal".to_string(), parser.position())),
    };

    let res_type = res_type.unwrap_or(SegmentResourceType::General);

    // TODO: Need to parse the mux, sb, and cb tags. For now just ignore.
    let _ = parser.skip();

    Ok(Segment {
        name,
        axis,
        length,
        segment_type,
        res_type,
        freq,
        r_metal,
        c_metal,
    })
}

fn parse_segment_list(name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Segment>, FPGAArchParseError> {
    assert!(name.to_string() == "segmentlist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut segments: Vec<Segment> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "segment" => {
                        segments.push(parse_segment(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "segmentlist" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(segments)
}

fn parse_pack_pattern(name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<PackPattern, FPGAArchParseError> {
    assert!(name.to_string() == "pack_pattern");

    let mut pattern_name: Option<String> = None;
    let mut in_port: Option<String> = None;
    let mut out_port: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                pattern_name = match pattern_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "in_port" => {
                in_port = match in_port {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "out_port" => {
                out_port = match out_port {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let pattern_name = match pattern_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let in_port = match in_port {
        Some(i) => i,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("in_port".to_string(), parser.position())),
    };
    let out_port = match out_port {
        Some(o) => o,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("out_port".to_string(), parser.position())),
    };

    match parser.next() {
        Ok(XmlEvent::StartElement { name, .. }) => {
            return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
        },
        Ok(XmlEvent::EndElement { name }) => {
            if name.to_string() != "pack_pattern" {
                return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position()));
            }
        },
        Ok(XmlEvent::EndDocument) => {
            return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
        },
        Err(e) => {
            return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
        },
        _ => {},
    };

    Ok(PackPattern {
        name: pattern_name,
        in_port,
        out_port,
    })
}

fn parse_interconnect(name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<Interconnect, FPGAArchParseError> {

    let mut inter_name: Option<String> = None;
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                inter_name = match inter_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "input" => {
                input = match input {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "output" => {
                output = match output {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let inter_name = match inter_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let input = match input {
        Some(i) => i,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("input".to_string(), parser.position())),
    };
    let output = match output {
        Some(o) => o,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("output".to_string(), parser.position())),
    };

    let mut pack_patterns: Vec<PackPattern> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "pack_pattern" => {
                        pack_patterns.push(parse_pack_pattern(&name, &attributes, parser)?);
                    },
                    "delay_constant" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "delay_matrix" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "metadata" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "direct" | "mux" | "complete" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    match name.to_string().as_ref() {
        "direct" => Ok(Interconnect::Direct(DirectInterconnect {
            name: inter_name,
            input,
            output,
            pack_patterns,
        })),
        "mux" => Ok(Interconnect::Mux(MuxInterconnect {
            name: inter_name,
            input,
            output,
            pack_patterns,
        })),
        "complete" => Ok(Interconnect::Complete(CompleteInterconnect {
            name: inter_name,
            input,
            output,
            pack_patterns,
        })),
        _ => Err(FPGAArchParseError::InvalidTag(format!("Unknown interconnect tag: {name}"), parser.position())),
    }
}

fn parse_interconnects(name: &OwnedName,
                       attributes: &[OwnedAttribute],
                       parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Interconnect>, FPGAArchParseError> {
    assert!(name.to_string() == "interconnect");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut interconnects: Vec<Interconnect> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "direct" | "mux" | "complete" => {
                        interconnects.push(parse_interconnect(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "interconnect" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(interconnects)
}

fn parse_pb_mode(name: &OwnedName,
                 attributes: &[OwnedAttribute],
                 parser: &mut EventReader<BufReader<File>>) -> Result<PBMode, FPGAArchParseError> {
    assert!(name.to_string() == "mode");

    let mut mode_name: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                mode_name = match mode_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let mode_name = match mode_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };

    let mut pb_types: Vec<PBType> = Vec::new();
    let mut interconnects: Option<Vec<Interconnect>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name, &attributes, parser)?);
                    },
                    "interconnect" => {
                        interconnects = match interconnects {
                            None => Some(parse_interconnects(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "metadata" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "mode" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    // TODO: The documentation is not very clear on if this is required or not.
    //       Assuming that it is not.
    let interconnects = interconnects.unwrap_or_default();

    Ok(PBMode {
        name: mode_name,
        pb_types,
        interconnects,
    })
}

fn parse_pb_type(name: &OwnedName,
                 attributes: &[OwnedAttribute],
                 parser: &mut EventReader<BufReader<File>>) -> Result<PBType, FPGAArchParseError> {
    assert!(name.to_string() == "pb_type");

    let mut pb_type_name: Option<String> = None;
    let mut num_pb: Option<i32> = None;
    let mut blif_model: Option<String> = None;
    let mut class: Option<PBTypeClass> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                pb_type_name = match pb_type_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "num_pb" => {
                num_pb = match num_pb {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "blif_model" => {
                blif_model = match blif_model {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "class" => {
                class = match class {
                    None => match a.value.to_string().as_ref() {
                        "lut" => Some(PBTypeClass::Lut),
                        "flipflop" => Some(PBTypeClass::FlipFlop),
                        "memory" => Some(PBTypeClass::Memory),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: Unknown port class"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }

    let pb_type_name = match pb_type_name {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let num_pb = num_pb.unwrap_or(1);
    let class = match class {
        None => PBTypeClass::None,
        Some(c) => c,
    };

    let mut pb_ports: Vec<Port> = Vec::new();
    let mut pb_types: Vec<PBType> = Vec::new();
    let mut pb_modes: Vec<PBMode> = Vec::new();
    let mut interconnects: Option<Vec<Interconnect>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "input" | "output" | "clock" => {
                        pb_ports.push(parse_port(&name, &attributes, parser)?);
                    },
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name, &attributes, parser)?);
                    },
                    "mode" => {
                        pb_modes.push(parse_pb_mode(&name, &attributes, parser)?);
                    },
                    "interconnect" => {
                        interconnects = match interconnects {
                            None => Some(parse_interconnects(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "power" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "delay_constant" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "delay_matrix" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "T_setup" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "T_hold" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "T_clock_to_Q" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "metadata" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "pinlocations" | "fc" => {
                        // This one is strange. This should not be in the pb_types.
                        // The ZA architectures have this here for some reason.
                        // FIXME: Talk to ZA, I think this is a mistake in their arch
                        //        files.
                        //        Will skip for now without error so we can support
                        //        their arch files.
                        let _ = parser.skip();
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "pb_type" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    // TODO: The documentation is not very clear on if this is required or not.
    //       Assuming that it is not.
    let interconnects = interconnects.unwrap_or_default();

    Ok(PBType {
        name: pb_type_name,
        num_pb,
        blif_model,
        class,
        ports: pb_ports,
        modes: pb_modes,
        pb_types,
        interconnects,
    })
}

fn parse_complex_block_list(name: &OwnedName,
                            attributes: &[OwnedAttribute],
                            parser: &mut EventReader<BufReader<File>>) -> Result<Vec<PBType>, FPGAArchParseError> {
    assert!(name.to_string() == "complexblocklist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut complex_block_list: Vec<PBType> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        complex_block_list.push(parse_pb_type(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "complexblocklist" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(complex_block_list)
}

pub fn parse_architecture(name: &OwnedName,
                          attributes: &[OwnedAttribute],
                          parser: &mut EventReader<BufReader<File>>) -> Result<FPGAArch, FPGAArchParseError> {
    assert!(name.to_string() == "architecture");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut tiles: Option<Vec<Tile>> = None;
    let mut layouts: Option<Vec<Layout>> = None;
    let mut device: Option<DeviceInfo> = None;
    let mut segment_list: Option<Vec<Segment>> = None;
    let mut complex_block_list: Option<Vec<PBType>> = None;

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "models" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "tiles" => {
                        tiles = match tiles {
                            None => Some(parse_tiles(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "layout" => {
                        layouts = match layouts {
                            None => Some(parse_layouts(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "device" => {
                        device = match device {
                            None => Some(parse_device(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "switchlist" => {
                        // TODO: Implement.
                        let _ = parser.skip();
                    },
                    "segmentlist" => {
                        segment_list = match segment_list {
                            None => Some(parse_segment_list(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "switchblocklist" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "directlist" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "complexblocklist" => {
                        complex_block_list = match complex_block_list {
                            None => Some(parse_complex_block_list(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
                    },
                    "power" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    "clocks" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name: _ }) => {
                match name.to_string().as_str() {
                    "architecture" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            // There's more: https://docs.rs/xml/latest/xml/reader/enum.XmlEvent.html
            _ => {},
        };
    }

    let tiles = match tiles {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<tiles>".to_string())),
    };
    let layouts = match layouts {
        Some(l) => l,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<layout>".to_string())),
    };
    let device = match device {
        Some(d) => d,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<device>".to_string())),
    };
    let segment_list = match segment_list {
        Some(s) => s,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<segmentlist>".to_string())),
    };
    let complex_block_list = match complex_block_list {
        Some(c) => c,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<complexblocklist>".to_string())),
    };

    Ok(FPGAArch {
        models: Vec::new(),
        tiles,
        layouts,
        device,
        switch_list: Vec::new(),
        segment_list,
        complex_block_list,
    })
}

pub fn parse(arch_file: &Path) -> Result<FPGAArch, FPGAArchParseError> {
    // Try to open the file.
    let file = File::open(arch_file);
    let file = match file {
        Ok(f) => f,
        Err(error) => return Err(FPGAArchParseError::ArchFileOpenError(format!("{error:?}"))),
    };

    // Create an XML event reader.
    // Buffering is used for performance.
    let file = BufReader::new(file);
    let mut parser = EventReader::new(file);

    // Parse the top-level tags.
    // At the top-level, we only expect the architecture tag.
    let mut arch: Option<FPGAArch> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "architecture" => {
                        arch = Some(parse_architecture(&name, &attributes, &mut parser)?);
                    },
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(format!("Invalid top-level tag: {name}, expected only <architecture>"), parser.position()));
                    },
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndDocument) => {
                break;
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            // There's more: https://docs.rs/xml/latest/xml/reader/enum.XmlEvent.html
            _ => {},
        };
    }

    // Return the architecture if it was provided. Error if no architecture was
    // provided in the description file.
    match arch {
        None => Err(FPGAArchParseError::MissingRequiredTag(String::from("<architecture>"))),
        Some(arch) => Ok(arch),
    }
}
