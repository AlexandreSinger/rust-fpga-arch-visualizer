use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

mod arch;
mod parse_complex_block_list;
mod parse_custom_switch_blocks;
mod parse_device;
mod parse_direct_list;
mod parse_error;
mod parse_layouts;
mod parse_metadata;
mod parse_models;
mod parse_port;
mod parse_segment_list;
mod parse_switch_list;
mod parse_tiles;
mod parse_timing;

pub use crate::arch::*;
pub use crate::parse_error::FPGAArchParseError;

use crate::parse_complex_block_list::parse_complex_block_list;
use crate::parse_custom_switch_blocks::parse_switchblocklist;
use crate::parse_device::parse_device;
use crate::parse_direct_list::parse_direct_list;
use crate::parse_layouts::parse_layouts;
use crate::parse_models::parse_models;
use crate::parse_segment_list::parse_segment_list;
use crate::parse_switch_list::parse_switch_list;
use crate::parse_tiles::parse_tiles;

fn parse_architecture(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<FPGAArch, FPGAArchParseError> {
    assert!(name.to_string() == "architecture");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut models: Option<Vec<Model>> = None;
    let mut tiles: Option<Vec<Tile>> = None;
    let mut layouts: Option<Vec<Layout>> = None;
    let mut device: Option<DeviceInfo> = None;
    let mut switch_list: Option<Vec<Switch>> = None;
    let mut segment_list: Option<Vec<Segment>> = None;
    let mut custom_switch_blocks: Option<Vec<CustomSwitchBlock>> = None;
    let mut direct_list: Option<Vec<GlobalDirect>> = None;
    let mut complex_block_list: Option<Vec<PBType>> = None;

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "models" => {
                        models = match models {
                            None => Some(parse_models(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "tiles" => {
                        tiles = match tiles {
                            None => Some(parse_tiles(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "layout" => {
                        layouts = match layouts {
                            None => Some(parse_layouts(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "device" => {
                        device = match device {
                            None => Some(parse_device(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "switchlist" => {
                        switch_list = match switch_list {
                            None => Some(parse_switch_list(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "segmentlist" => {
                        segment_list = match segment_list {
                            None => Some(parse_segment_list(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "switchblocklist" => {
                        custom_switch_blocks = match custom_switch_blocks {
                            None => Some(parse_switchblocklist(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "directlist" => {
                        direct_list = match direct_list {
                            None => Some(parse_direct_list(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "complexblocklist" => {
                        complex_block_list = match complex_block_list {
                            None => Some(parse_complex_block_list(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "power" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    }
                    "clocks" => {
                        // TODO: Implement.
                        // FIXME: Check that this is documented in VTR.
                        let _ = parser.skip();
                    }
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(
                            name.to_string(),
                            parser.position(),
                        ));
                    }
                };
            }
            Ok(XmlEvent::EndElement { name: end_name }) => match end_name.to_string().as_str() {
                "architecture" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        end_name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(
                    name.to_string(),
                ));
            }
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(
                    format!("{e:?}"),
                    parser.position(),
                ));
            }
            // There's more: https://docs.rs/xml/latest/xml/reader/enum.XmlEvent.html
            _ => {}
        };
    }

    let models = match models {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<models>".to_string(),
            ));
        }
    };
    let tiles = match tiles {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<tiles>".to_string(),
            ));
        }
    };
    let layouts = match layouts {
        Some(l) => l,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<layout>".to_string(),
            ));
        }
    };
    let device = match device {
        Some(d) => d,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<device>".to_string(),
            ));
        }
    };
    let switch_list = match switch_list {
        Some(s) => s,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<switchlist>".to_string(),
            ));
        }
    };
    let segment_list = match segment_list {
        Some(s) => s,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<segmentlist>".to_string(),
            ));
        }
    };
    let complex_block_list = match complex_block_list {
        Some(c) => c,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<complexblocklist>".to_string(),
            ));
        }
    };
    let custom_switch_blocks = custom_switch_blocks.unwrap_or_default();
    let direct_list = direct_list.unwrap_or_default();

    Ok(FPGAArch {
        models,
        tiles,
        layouts,
        device,
        switch_list,
        segment_list,
        custom_switch_blocks,
        direct_list,
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
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "architecture" => {
                        arch = Some(parse_architecture(&name, &attributes, &mut parser)?);
                    }
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(
                            format!("Invalid top-level tag: {name}, expected only <architecture>"),
                            parser.position(),
                        ));
                    }
                };
            }
            Ok(XmlEvent::EndElement { name }) => {
                return Err(FPGAArchParseError::UnexpectedEndTag(
                    name.to_string(),
                    parser.position(),
                ));
            }
            Ok(XmlEvent::EndDocument) => {
                break;
            }
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(
                    format!("{e:?}"),
                    parser.position(),
                ));
            }
            // There's more: https://docs.rs/xml/latest/xml/reader/enum.XmlEvent.html
            _ => {}
        };
    }

    // Return the architecture if it was provided. Error if no architecture was
    // provided in the description file.
    match arch {
        None => Err(FPGAArchParseError::MissingRequiredTag(String::from(
            "<architecture>",
        ))),
        Some(arch) => Ok(arch),
    }
}
