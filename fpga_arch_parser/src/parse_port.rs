use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

pub fn parse_port(tag_name: &OwnedName,
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

