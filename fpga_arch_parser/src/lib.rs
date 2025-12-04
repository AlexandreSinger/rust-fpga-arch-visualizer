use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

mod arch;
mod parse_error;
mod parse_metadata;
mod parse_port;
mod parse_tiles;
mod parse_layouts;
mod parse_device;
mod parse_switch_list;
mod parse_segment_list;
mod parse_complex_block_list;

pub use crate::parse_error::FPGAArchParseError;
pub use crate::arch::*;

use crate::parse_port::parse_port;
use crate::parse_tiles::parse_tiles;
use crate::parse_layouts::parse_layouts;
use crate::parse_device::parse_device;
use crate::parse_switch_list::parse_switch_list;
use crate::parse_segment_list::parse_segment_list;
use crate::parse_complex_block_list::parse_complex_block_list;

fn parse_architecture(name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<FPGAArch, FPGAArchParseError> {
    assert!(name.to_string() == "architecture");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut tiles: Option<Vec<Tile>> = None;
    let mut layouts: Option<Vec<Layout>> = None;
    let mut device: Option<DeviceInfo> = None;
    let mut switch_list: Option<Vec<Switch>> = None;
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
                        switch_list = match switch_list {
                            None => Some(parse_switch_list(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(format!("<{name}>"), parser.position())),
                        }
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
    let switch_list = match switch_list {
        Some(s) => s,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<switchlist>".to_string())),
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
        switch_list,
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
