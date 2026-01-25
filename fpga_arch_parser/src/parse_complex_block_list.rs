use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

use crate::parse_metadata::parse_metadata;
use crate::parse_port::parse_port;
use crate::parse_timing::parse_clock_to_q;
use crate::parse_timing::parse_delay_constant;
use crate::parse_timing::parse_delay_matrix;
use crate::parse_timing::parse_t_hold;
use crate::parse_timing::parse_t_setup;

fn parse_pack_pattern(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<PackPattern, FPGAArchParseError> {
    assert!(name.to_string() == "pack_pattern");

    let mut pattern_name: Option<String> = None;
    let mut in_port: Option<String> = None;
    let mut out_port: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                pattern_name = match pattern_name {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "in_port" => {
                in_port = match in_port {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "out_port" => {
                out_port = match out_port {
                    None => Some(a.value.clone()),
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

    let pattern_name = match pattern_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };
    let in_port = match in_port {
        Some(i) => i,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "in_port".to_string(),
                parser.position(),
            ));
        }
    };
    let out_port = match out_port {
        Some(o) => o,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "out_port".to_string(),
                parser.position(),
            ));
        }
    };

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(
                    name.to_string(),
                    parser.position(),
                ));
            }
            Ok(XmlEvent::EndElement { name: end_name }) => {
                if end_name.to_string() == name.to_string() {
                    break;
                } else {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            }
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
            _ => {}
        }
    }

    Ok(PackPattern {
        name: pattern_name,
        in_port,
        out_port,
    })
}

fn parse_interconnect(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Interconnect, FPGAArchParseError> {
    let mut inter_name: Option<String> = None;
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                inter_name = match inter_name {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "input" => {
                input = match input {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "output" => {
                output = match output {
                    None => Some(a.value.clone()),
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

    let inter_name = match inter_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };
    let input = match input {
        Some(i) => i,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "input".to_string(),
                parser.position(),
            ));
        }
    };
    let output = match output {
        Some(o) => o,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "output".to_string(),
                parser.position(),
            ));
        }
    };

    let mut pack_patterns: Vec<PackPattern> = Vec::new();
    let mut delays: Vec<DelayInfo> = Vec::new();
    let mut metadata: Option<Vec<Metadata>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "pack_pattern" => {
                        pack_patterns.push(parse_pack_pattern(&name, &attributes, parser)?);
                    }
                    "delay_constant" => {
                        delays.push(parse_delay_constant(&name, &attributes, parser)?);
                    }
                    "delay_matrix" => {
                        delays.push(parse_delay_matrix(&name, &attributes, parser)?);
                    }
                    "metadata" => {
                        metadata = match metadata {
                            None => Some(parse_metadata(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(
                            name.to_string(),
                            parser.position(),
                        ));
                    }
                };
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "direct" | "mux" | "complete" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
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
            _ => {}
        }
    }

    match name.to_string().as_ref() {
        "direct" => Ok(Interconnect::Direct(DirectInterconnect {
            name: inter_name,
            input,
            output,
            pack_patterns,
            delays,
            metadata,
        })),
        "mux" => Ok(Interconnect::Mux(MuxInterconnect {
            name: inter_name,
            input,
            output,
            pack_patterns,
            delays,
            metadata,
        })),
        "complete" => Ok(Interconnect::Complete(CompleteInterconnect {
            name: inter_name,
            input,
            output,
            pack_patterns,
            delays,
            metadata,
        })),
        _ => Err(FPGAArchParseError::InvalidTag(
            format!("Unknown interconnect tag: {name}"),
            parser.position(),
        )),
    }
}

fn parse_interconnects(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<Interconnect>, FPGAArchParseError> {
    assert!(name.to_string() == "interconnect");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut interconnects: Vec<Interconnect> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "direct" | "mux" | "complete" => {
                        interconnects.push(parse_interconnect(&name, &attributes, parser)?);
                    }
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(
                            name.to_string(),
                            parser.position(),
                        ));
                    }
                };
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "interconnect" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
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
            _ => {}
        }
    }

    Ok(interconnects)
}

fn parse_pb_mode(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<PBMode, FPGAArchParseError> {
    assert!(name.to_string() == "mode");

    let mut mode_name: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                mode_name = match mode_name {
                    None => Some(a.value.clone()),
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

    let mode_name = match mode_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };

    let mut pb_types: Vec<PBType> = Vec::new();
    let mut interconnects: Option<Vec<Interconnect>> = None;
    let mut metadata: Option<Vec<Metadata>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name, &attributes, parser)?);
                    }
                    "interconnect" => {
                        interconnects = match interconnects {
                            None => Some(parse_interconnects(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    name.to_string(),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "metadata" => {
                        metadata = match metadata {
                            None => Some(parse_metadata(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    name.to_string(),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(
                            name.to_string(),
                            parser.position(),
                        ));
                    }
                };
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "mode" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
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
            _ => {}
        }
    }

    // TODO: The documentation is not very clear on if this is required or not.
    //       Assuming that it is not.
    let interconnects = interconnects.unwrap_or_default();

    Ok(PBMode {
        name: mode_name,
        pb_types,
        interconnects,
        metadata,
    })
}

fn parse_pb_type(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<PBType, FPGAArchParseError> {
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
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "num_pb" => {
                num_pb = match num_pb {
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
            "blif_model" => {
                blif_model = match blif_model {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "class" => {
                class = match class {
                    None => match a.value.to_string().as_ref() {
                        "lut" => Some(PBTypeClass::Lut),
                        "flipflop" => Some(PBTypeClass::FlipFlop),
                        "memory" => Some(PBTypeClass::Memory),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: Unknown port class"),
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
            _ => {
                return Err(FPGAArchParseError::UnknownAttribute(
                    a.to_string(),
                    parser.position(),
                ));
            }
        };
    }

    let pb_type_name = match pb_type_name {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
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
    let mut delays: Vec<DelayInfo> = Vec::new();
    let mut timing_constraints: Vec<TimingConstraintInfo> = Vec::new();
    let mut metadata: Option<Vec<Metadata>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "input" | "output" | "clock" => {
                        pb_ports.push(parse_port(&name, &attributes, parser)?);
                    }
                    "pb_type" => {
                        pb_types.push(parse_pb_type(&name, &attributes, parser)?);
                    }
                    "mode" => {
                        pb_modes.push(parse_pb_mode(&name, &attributes, parser)?);
                    }
                    "interconnect" => {
                        interconnects = match interconnects {
                            None => Some(parse_interconnects(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    name.to_string(),
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
                    "delay_constant" => {
                        delays.push(parse_delay_constant(&name, &attributes, parser)?);
                    }
                    "delay_matrix" => {
                        delays.push(parse_delay_matrix(&name, &attributes, parser)?);
                    }
                    "T_setup" => {
                        timing_constraints.push(parse_t_setup(&name, &attributes, parser)?);
                    }
                    "T_hold" => {
                        timing_constraints.push(parse_t_hold(&name, &attributes, parser)?);
                    }
                    "T_clock_to_Q" => {
                        timing_constraints.push(parse_clock_to_q(&name, &attributes, parser)?);
                    }
                    "metadata" => {
                        metadata = match metadata {
                            None => Some(parse_metadata(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    name.to_string(),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "pinlocations" | "fc" => {
                        // This one is strange. This should not be in the pb_types.
                        // The ZA architectures have this here for some reason.
                        // FIXME: Talk to ZA, I think this is a mistake in their arch
                        //        files.
                        //        Will skip for now without error so we can support
                        //        their arch files.
                        // TODO: Print a warning.
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
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "pb_type" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
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
            _ => {}
        }
    }

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
        delays,
        timing_constraints,
        metadata,
    })
}

pub fn parse_complex_block_list(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<PBType>, FPGAArchParseError> {
    assert!(name.to_string() == "complexblocklist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut complex_block_list: Vec<PBType> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "pb_type" => {
                        complex_block_list.push(parse_pb_type(&name, &attributes, parser)?);
                    }
                    _ => {
                        return Err(FPGAArchParseError::InvalidTag(
                            name.to_string(),
                            parser.position(),
                        ));
                    }
                };
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "complexblocklist" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
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
            _ => {}
        }
    }

    Ok(complex_block_list)
}
