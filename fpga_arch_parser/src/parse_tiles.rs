use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

use crate::parse_port::parse_port;

fn parse_tile_site(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<TileSite, FPGAArchParseError> {
    assert!(name.to_string() == "site");

    let mut site_pb_type: Option<String> = None;
    let mut site_pin_mapping: Option<TileSitePinMapping> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "pb_type" => {
                site_pb_type = match site_pb_type {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "pin_mapping" => {
                site_pin_mapping = match site_pin_mapping {
                    None => match a.value.as_str() {
                        "direct" => Some(TileSitePinMapping::Direct),
                        "custom" => Some(TileSitePinMapping::Custom),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("Unknown site pin mapping: {}", a.value),
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

    let site_pb_type = match site_pb_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "pb_type".to_string(),
                parser.position(),
            ));
        }
    };
    let site_pin_mapping = site_pin_mapping.unwrap_or(TileSitePinMapping::Direct);

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(
                    name.to_string(),
                    parser.position(),
                ));
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "site" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(
                    "site".to_string(),
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

    Ok(TileSite {
        pb_type: site_pb_type,
        pin_mapping: site_pin_mapping,
    })
}

fn parse_equivalent_sites(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<TileSite>, FPGAArchParseError> {
    assert!(name.to_string() == "equivalent_sites");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut equivalent_sites: Vec<TileSite> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "site" => {
                        equivalent_sites.push(parse_tile_site(&name, &attributes, parser)?);
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
                "equivalent_sites" => break,
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

    // TODO: Check the documentation. Is it allowed for equivalent sites to be empty?

    Ok(equivalent_sites)
}

fn create_sub_tile_io_fc(
    ty: &str,
    val: &str,
    parser: &EventReader<BufReader<File>>,
) -> Result<SubTileIOFC, FPGAArchParseError> {
    match ty {
        "frac" => Ok(SubTileIOFC::Frac(match val.parse() {
            Ok(v) => v,
            Err(e) => {
                return Err(FPGAArchParseError::AttributeParseError(
                    format!("{val}: {e}"),
                    parser.position(),
                ));
            }
        })),
        "abs" => Ok(SubTileIOFC::Abs(match val.parse() {
            Ok(v) => v,
            Err(e) => {
                return Err(FPGAArchParseError::AttributeParseError(
                    format!("{val}: {e}"),
                    parser.position(),
                ));
            }
        })),
        _ => Err(FPGAArchParseError::AttributeParseError(
            format!("Unknown fc_type: {}", ty),
            parser.position(),
        )),
    }
}

fn parse_sub_tile_fc_override(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<SubTileFCOverride, FPGAArchParseError> {
    assert!(name.to_string() == "fc_override");

    let mut fc_type: Option<String> = None;
    let mut fc_val: Option<String> = None;
    let mut port_name: Option<String> = None;
    let mut segment_name: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_str() {
            "fc_type" => {
                fc_type = match fc_type {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "fc_val" => {
                fc_val = match fc_val {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "port_name" => {
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
            "segment_name" => {
                segment_name = match segment_name {
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
    let fc_type = match fc_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "fc_type".to_string(),
                parser.position(),
            ));
        }
    };
    let fc_val = match fc_val {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "fc_val".to_string(),
                parser.position(),
            ));
        }
    };
    let fc = create_sub_tile_io_fc(&fc_type, &fc_val, parser)?;
    if port_name.is_none() && segment_name.is_none() {
        return Err(FPGAArchParseError::MissingRequiredAttribute(
            "At least one of port_name or segment_name must be specified.".to_string(),
            parser.position(),
        ));
    }

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(
                    name.to_string(),
                    parser.position(),
                ));
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "fc_override" => break,
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
        };
    }

    Ok(SubTileFCOverride {
        fc,
        port_name,
        segment_name,
    })
}

fn parse_sub_tile_fc(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<SubTileFC, FPGAArchParseError> {
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
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "in_val" => {
                in_val = match in_val {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "out_type" => {
                out_type = match out_type {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "out_val" => {
                out_val = match out_val {
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

    let in_type = match in_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "in_type".to_string(),
                parser.position(),
            ));
        }
    };
    let in_val = match in_val {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "in_val".to_string(),
                parser.position(),
            ));
        }
    };
    let out_type = match out_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "out_type".to_string(),
                parser.position(),
            ));
        }
    };
    let out_val = match out_val {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "out_val".to_string(),
                parser.position(),
            ));
        }
    };

    let in_fc = create_sub_tile_io_fc(&in_type, &in_val, parser)?;
    let out_fc = create_sub_tile_io_fc(&out_type, &out_val, parser)?;

    let mut fc_overrides: Vec<SubTileFCOverride> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "fc_override" => {
                        fc_overrides.push(parse_sub_tile_fc_override(&name, &attributes, parser)?);
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
                "fc" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(
                    "fc".to_string(),
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

    Ok(SubTileFC {
        in_fc,
        out_fc,
        fc_overrides,
    })
}

fn parse_pin_loc(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<PinLoc, FPGAArchParseError> {
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
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("Unknown pin side: {}", a.value),
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
            "xoffset" => {
                xoffset = match xoffset {
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
            "yoffset" => {
                yoffset = match yoffset {
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
            _ => {
                return Err(FPGAArchParseError::UnknownAttribute(
                    a.to_string(),
                    parser.position(),
                ));
            }
        };
    }

    let side = match side {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "side".to_string(),
                parser.position(),
            ));
        }
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
                    Some(_) => {
                        return Err(FPGAArchParseError::InvalidTag(
                            "Duplicate characters within loc tag.".to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "loc" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(
                    name.to_string(),
                    parser.position(),
                ));
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

fn parse_sub_tile_pin_locations(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<SubTilePinLocations, FPGAArchParseError> {
    assert!(name.to_string() == "pinlocations");

    let mut pattern: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "pattern" => {
                pattern = match pattern {
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

    let pattern = match pattern {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "pattern".to_string(),
                parser.position(),
            ));
        }
    };

    let mut pin_locs: Vec<PinLoc> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "loc" => {
                        // If pin locations are defined for any patter other than
                        // custom, something is wrong.
                        if pattern != "custom" {
                            return Err(FPGAArchParseError::InvalidTag(
                                "Pin locations can only be given for custom pattern".to_string(),
                                parser.position(),
                            ));
                        }
                        pin_locs.push(parse_pin_loc(&name, &attributes, parser)?);
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
                "pinlocations" => break,
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

    match pattern.as_str() {
        "spread" => Ok(SubTilePinLocations::Spread),
        "perimeter" => Ok(SubTilePinLocations::Perimeter),
        "spread_inputs_perimeter_outputs" => Ok(SubTilePinLocations::SpreadInputsPerimeterOutputs),
        "custom" => Ok(SubTilePinLocations::Custom(CustomPinLocations {
            pin_locations: pin_locs,
        })),
        _ => Err(FPGAArchParseError::AttributeParseError(
            format!("Unknown spreadpattern for pinlocations: {}", pattern),
            parser.position(),
        )),
    }
}

fn parse_sub_tile(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<SubTile, FPGAArchParseError> {
    assert!(name.to_string() == "sub_tile");

    let mut sub_tile_name: Option<String> = None;
    let mut sub_tile_capacity: Option<i32> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                sub_tile_name = match sub_tile_name {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "capacity" => {
                sub_tile_capacity = match sub_tile_capacity {
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
            _ => {
                return Err(FPGAArchParseError::UnknownAttribute(
                    a.to_string(),
                    parser.position(),
                ));
            }
        };
    }

    let sub_tile_name = match sub_tile_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };
    let sub_tile_capacity = sub_tile_capacity.unwrap_or(1);

    let mut equivalent_sites: Option<Vec<TileSite>> = None;
    let mut ports: Vec<Port> = Vec::new();
    let mut sub_tile_fc: Option<SubTileFC> = None;
    let mut pin_locations: Option<SubTilePinLocations> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "equivalent_sites" => {
                        equivalent_sites = match equivalent_sites {
                            None => Some(parse_equivalent_sites(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "input" | "output" | "clock" => {
                        ports.push(parse_port(&name, &attributes, parser)?);
                    }
                    "fc" => {
                        sub_tile_fc = match sub_tile_fc {
                            None => Some(parse_sub_tile_fc(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "pinlocations" => {
                        pin_locations = match pin_locations {
                            None => Some(parse_sub_tile_pin_locations(&name, &attributes, parser)?),
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
                "sub_tile" => break,
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

    let equivalent_sites = match equivalent_sites {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<equivalent_sites>".to_string(),
            ));
        }
    };
    let sub_tile_fc = match sub_tile_fc {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<fc>".to_string())),
    };
    let pin_locations = match pin_locations {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<pinlocations>".to_string(),
            ));
        }
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

fn parse_tile(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Tile, FPGAArchParseError> {
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
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "width" => {
                width = match width {
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
            "height" => {
                height = match height {
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
            "area" => {
                area = match area {
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
            _ => {
                return Err(FPGAArchParseError::UnknownAttribute(
                    a.to_string(),
                    parser.position(),
                ));
            }
        }
    }

    let tile_name = match tile_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };

    // If the width or height is not provided, they are assumed to be 1.
    let width = width.unwrap_or(1);
    let height = height.unwrap_or(1);

    let mut ports: Vec<Port> = Vec::new();
    let mut sub_tiles: Vec<SubTile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "sub_tile" => {
                        sub_tiles.push(parse_sub_tile(&name, &attributes, parser)?);
                    }
                    "input" | "output" | "clock" => {
                        ports.push(parse_port(&name, &attributes, parser)?);
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
                "tile" => break,
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

    Ok(Tile {
        name: tile_name,
        ports,
        sub_tiles,
        width,
        height,
        area,
    })
}

pub fn parse_tiles(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<Tile>, FPGAArchParseError> {
    assert!(name.to_string() == "tiles");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    // Iterate over the parser until we reach the EndElement for tile.
    let mut tiles: Vec<Tile> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "tile" => {
                        tiles.push(parse_tile(&name, &attributes, parser)?);
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
                "tiles" => break,
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

    Ok(tiles)
}
