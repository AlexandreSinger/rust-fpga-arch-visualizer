use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::{Position, TextPosition};
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

fn parse_port_class(value: &str, position: TextPosition) -> Result<PortClass, FPGAArchParseError> {
    // Try to parse as simple port class first
    match value {
        "lut_in" => return Ok(PortClass::LutIn),
        "lut_out" => return Ok(PortClass::LutOut),
        "D" => return Ok(PortClass::FlipFlopD),
        "Q" => return Ok(PortClass::FlipFlopQ),
        "clock" => return Ok(PortClass::Clock),
        _ => {}
    }

    // Try to parse as memory port class with optional integer suffix
    // Memory classes: address, data_in, write_en, data_out, read_en
    let memory_prefixes = ["address", "data_in", "write_en", "data_out", "read_en"];

    for prefix in &memory_prefixes {
        if let Some(suffix) = value.strip_prefix(prefix) {
            let port_num = if suffix.is_empty() {
                // Default to 1 if no number suffix
                1
            } else {
                // Parse the numeric suffix
                match suffix.parse::<i32>() {
                    Ok(num) => num,
                    Err(_) => {
                        return Err(FPGAArchParseError::AttributeParseError(
                            format!("Unknown port class: {}", value),
                            position,
                        ));
                    }
                }
            };

            return Ok(match *prefix {
                "address" => PortClass::MemoryAddress(port_num),
                "data_in" => PortClass::MemoryDataIn(port_num),
                "write_en" => PortClass::MemoryWriteEn(port_num),
                "data_out" => PortClass::MemoryDataOut(port_num),
                "read_en" => PortClass::MemoryReadEn(port_num),
                _ => unreachable!(),
            });
        }
    }

    Err(FPGAArchParseError::AttributeParseError(
        format!("Unknown port class: {}", value),
        position,
    ))
}

pub fn parse_port(
    tag_name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Port, FPGAArchParseError> {
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
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "num_pins" => {
                num_pins = match num_pins {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: {e}"),
                                parser.position(),
                            ));
                        }
                    },
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "equivalent" => {
                equivalent = match equivalent {
                    None => match a.value.as_str() {
                        "none" => Some(PinEquivalence::None),
                        "full" => Some(PinEquivalence::Full),
                        "instance" => Some(PinEquivalence::Instance),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("Unknown pin equivalence: {}", a.value),
                                parser.position(),
                            ));
                        }
                    },
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "is_non_clock_global" => {
                is_non_clock_global = match is_non_clock_global {
                    None => match tag_name.to_string().as_str() {
                        "input" => match a.value.parse() {
                            Ok(v) => Some(v),
                            Err(e) => {
                                return Err(FPGAArchParseError::AttributeParseError(
                                    format!("{a}: {e}"),
                                    parser.position(),
                                ));
                            }
                        },
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                "is_non_clock_global attribute only valid in input tag."
                                    .to_string(),
                                parser.position(),
                            ));
                        }
                    },
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "port_class" => {
                port_class = match port_class {
                    None => match parse_port_class(&a.value, parser.position()) {
                        Ok(pc) => Some(pc),
                        Err(e) => return Err(e),
                    },
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            _ => {
                return Err(FPGAArchParseError::UnknownAttribute(
                    a.to_string(),
                    parser.position(),
                ));
            }
        };
    }

    let port_name = match port_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };
    let num_pins = match num_pins {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "num_pins".to_string(),
                parser.position(),
            ));
        }
    };
    let equivalent = equivalent.unwrap_or(PinEquivalence::None);
    let is_non_clock_global = is_non_clock_global.unwrap_or(false);
    let port_class = port_class.unwrap_or(PortClass::None);

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(
                    name.to_string(),
                    parser.position(),
                ));
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() != tag_name.to_string() {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
                break;
            }
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(
                    tag_name.to_string(),
                ));
            }
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(
                    format!("{e:?}"),
                    parser.position(),
                ));
            }
            _ => {}
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
        _ => Err(FPGAArchParseError::InvalidTag(
            format!("Unknown port tag: {tag_name}"),
            parser.position(),
        )),
    }
}
