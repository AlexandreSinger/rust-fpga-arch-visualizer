use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

fn parse_switchblock_location(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<CustomSwitchBlockLocation, FPGAArchParseError> {
    assert!(name.to_string() == "switchblock_location");

    let mut switchblock_location: Option<CustomSwitchBlockLocation> = None;
    let mut x: Option<i32> = None;
    let mut y: Option<i32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                switchblock_location = match switchblock_location {
                    None => match a.value.to_string().as_ref() {
                        "EVERYWHERE" => Some(CustomSwitchBlockLocation::Everywhere),
                        "PERIMETER" => Some(CustomSwitchBlockLocation::Perimeter),
                        "CORNER" => Some(CustomSwitchBlockLocation::Corner),
                        "FRINGE" => Some(CustomSwitchBlockLocation::Fringe),
                        "CORE" => Some(CustomSwitchBlockLocation::Core),
                        // Special case for XY specified. We do not know the location yet.
                        // Just use a placeholder for now.
                        "XY_SPECIFIED" => {
                            Some(CustomSwitchBlockLocation::XYSpecified { x: -1, y: -1 })
                        }
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: Unknown custom switch block location"),
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
            "x" => {
                x = match x {
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
            "y" => {
                y = match y {
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
    let switchblock_location = match switchblock_location {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };
    // Add the x and y locations to XY specified or error.
    let switchblock_location = match switchblock_location {
        CustomSwitchBlockLocation::XYSpecified { .. } => match x {
            Some(x) => match y {
                Some(y) => CustomSwitchBlockLocation::XYSpecified { x, y },
                None => {
                    return Err(FPGAArchParseError::MissingRequiredAttribute(
                        "y".to_string(),
                        parser.position(),
                    ));
                }
            },
            None => {
                return Err(FPGAArchParseError::MissingRequiredAttribute(
                    "x".to_string(),
                    parser.position(),
                ));
            }
        },
        _ => {
            // x and y should not be specified for any other location type.
            if x.is_some() {
                return Err(FPGAArchParseError::UnknownAttribute(
                    "x".to_string(),
                    parser.position(),
                ));
            }
            if y.is_some() {
                return Err(FPGAArchParseError::UnknownAttribute(
                    "y".to_string(),
                    parser.position(),
                ));
            }
            switchblock_location
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

    Ok(switchblock_location)
}

fn parse_func(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<CustomSwitchFunc, FPGAArchParseError> {
    assert!(name.to_string() == "func");

    let mut func_type: Option<CustomSwitchFuncType> = None;
    let mut formula: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                func_type = match func_type {
                    None => match a.value.to_string().as_ref() {
                        "lt" => Some(CustomSwitchFuncType::LeftToTop),
                        "lr" => Some(CustomSwitchFuncType::LeftToRight),
                        "lb" => Some(CustomSwitchFuncType::LeftToBottom),
                        "tr" => Some(CustomSwitchFuncType::TopToRight),
                        "tb" => Some(CustomSwitchFuncType::TopToBottom),
                        "tl" => Some(CustomSwitchFuncType::TopToLeft),
                        "rb" => Some(CustomSwitchFuncType::RightToBottom),
                        "rl" => Some(CustomSwitchFuncType::RightToLeft),
                        "rt" => Some(CustomSwitchFuncType::RightToTop),
                        "bl" => Some(CustomSwitchFuncType::BottomToLeft),
                        "bt" => Some(CustomSwitchFuncType::BottomToTop),
                        "br" => Some(CustomSwitchFuncType::BottomToRight),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: Unknown custom switch function type"),
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
            "formula" => {
                // TODO: Need to verify that this formula is possible.
                formula = match formula {
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
    let func_type = match func_type {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };
    let formula = match formula {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "formula".to_string(),
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

    Ok(CustomSwitchFunc { func_type, formula })
}

fn parse_switchfuncs(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<CustomSwitchFunc>, FPGAArchParseError> {
    assert!(name.to_string() == "switchfuncs");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut switch_funcs: Vec<CustomSwitchFunc> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "func" => {
                        switch_funcs.push(parse_func(&name, &attributes, parser)?);
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
                "switchfuncs" => break,
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
    // FIXME: The documentation is not clear if switch_funcs is allowed to be empty.

    Ok(switch_funcs)
}

fn parse_switchpoint_list(
    switchpoint_list: &str,
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<i32>, FPGAArchParseError> {
    let mut switchpoints: Vec<i32> = Vec::new();

    for substr in switchpoint_list.split(',') {
        let switchpoint = match substr.parse() {
            Ok(v) => v,
            Err(e) => {
                return Err(FPGAArchParseError::AttributeParseError(
                    format!("switchpoint parse error: {e}"),
                    parser.position(),
                ));
            }
        };

        switchpoints.push(switchpoint);
    }

    Ok(switchpoints)
}

fn parse_conn_point(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<CustomSwitchBlockConnPoint, FPGAArchParseError> {
    assert!(name.to_string() == "from" || name.to_string() == "to");

    let mut segment_type: Option<String> = None;
    let mut switchpoint: Option<Vec<i32>> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                segment_type = match segment_type {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "switchpoint" => {
                switchpoint = match switchpoint {
                    None => Some(parse_switchpoint_list(&a.value, parser)?),
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
    let segment_type = match segment_type {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };
    let switchpoint = match switchpoint {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "switchpoint".to_string(),
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

    Ok(CustomSwitchBlockConnPoint {
        segment_type,
        switchpoint,
    })
}

fn parse_wireconn(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<CustomSwitchWireConn, FPGAArchParseError> {
    assert!(name.to_string() == "wireconn");

    let mut num_conns: Option<String> = None;
    let mut from_type: Option<Vec<String>> = None;
    let mut to_type: Option<Vec<String>> = None;
    let mut from_switchpoint: Option<Vec<i32>> = None;
    let mut to_switchpoint: Option<Vec<i32>> = None;
    let mut from_order: Option<CustomSwitchBlockWireConnOrder> = None;
    let mut to_order: Option<CustomSwitchBlockWireConnOrder> = None;
    let mut switch_override: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "num_conns" => {
                num_conns = match num_conns {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "from_type" => {
                from_type = match from_type {
                    None => Some(a.value.split(',').map(|s| s.trim().to_string()).collect()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "to_type" => {
                to_type = match to_type {
                    None => Some(a.value.split(',').map(|s| s.trim().to_string()).collect()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "from_switchpoint" => {
                from_switchpoint = match from_switchpoint {
                    None => Some(parse_switchpoint_list(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "to_switchpoint" => {
                to_switchpoint = match to_switchpoint {
                    None => Some(parse_switchpoint_list(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "from_order" => {
                from_order = match from_order {
                    None => match a.value.to_string().as_ref() {
                        "shuffled" => Some(CustomSwitchBlockWireConnOrder::Shuffled),
                        "fixed" => Some(CustomSwitchBlockWireConnOrder::Fixed),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: unknown custom switch block connection order."),
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
            "to_order" => {
                to_order = match to_order {
                    None => match a.value.to_string().as_ref() {
                        "shuffled" => Some(CustomSwitchBlockWireConnOrder::Shuffled),
                        "fixed" => Some(CustomSwitchBlockWireConnOrder::Fixed),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: unknown custom switch block connection order."),
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
            "switch_override" => {
                switch_override = match switch_override {
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
    let num_conns = match num_conns {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "num_conns".to_string(),
                parser.position(),
            ));
        }
    };
    let from_order = from_order.unwrap_or(CustomSwitchBlockWireConnOrder::Shuffled);
    // FIXME: The documentation is not clear about the default value of this.
    let to_order = to_order.unwrap_or(CustomSwitchBlockWireConnOrder::Shuffled);
    let mut from_points = match from_type {
        Some(from_types) => match from_switchpoint {
            Some(from_switchpoints) => {
                let mut from_points: Vec<CustomSwitchBlockConnPoint> = Vec::new();

                // FIXME: The documentation is really not clear what happens
                //        when multiple from types are defined. Made a best guess.
                for from_type in from_types {
                    from_points.push(CustomSwitchBlockConnPoint {
                        segment_type: from_type,
                        switchpoint: from_switchpoints.clone(),
                    });
                }

                from_points
            }
            None => {
                return Err(FPGAArchParseError::MissingRequiredAttribute(
                    "from_switchpoint".to_string(),
                    parser.position(),
                ));
            }
        },
        None => match from_switchpoint {
            Some(_) => {
                return Err(FPGAArchParseError::MissingRequiredAttribute(
                    "from_type".to_string(),
                    parser.position(),
                ));
            }
            None => Vec::new(),
        },
    };
    let mut to_points = match to_type {
        Some(to_types) => match to_switchpoint {
            Some(to_switchpoints) => {
                let mut to_points: Vec<CustomSwitchBlockConnPoint> = Vec::new();

                // FIXME: The documentation is really not clear what happens
                //        when multiple from types are defined. Made a best guess.
                for to_type in to_types {
                    to_points.push(CustomSwitchBlockConnPoint {
                        segment_type: to_type,
                        switchpoint: to_switchpoints.clone(),
                    });
                }

                to_points
            }
            None => {
                return Err(FPGAArchParseError::MissingRequiredAttribute(
                    "to_switchpoint".to_string(),
                    parser.position(),
                ));
            }
        },
        None => match to_switchpoint {
            Some(_) => {
                return Err(FPGAArchParseError::MissingRequiredAttribute(
                    "to_type".to_string(),
                    parser.position(),
                ));
            }
            None => Vec::new(),
        },
    };

    // FIXME: The documentation is not clear what should happen if from/to points
    //        are defined in the attributes and tags.

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "from" => {
                        from_points.push(parse_conn_point(&name, &attributes, parser)?);
                    }
                    "to" => {
                        to_points.push(parse_conn_point(&name, &attributes, parser)?);
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
                "wireconn" => break,
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

    if from_points.is_empty() {
        return Err(FPGAArchParseError::InvalidTag(
            "from_points is empty".to_string(),
            parser.position(),
        ));
    }
    if to_points.is_empty() {
        return Err(FPGAArchParseError::InvalidTag(
            "to_points is empty".to_string(),
            parser.position(),
        ));
    }

    Ok(CustomSwitchWireConn {
        num_conns,
        from_points,
        to_points,
        from_order,
        to_order,
        switch_override,
    })
}

fn parse_switchblock(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<CustomSwitchBlock, FPGAArchParseError> {
    assert!(name.to_string() == "switchblock");

    let mut name: Option<String> = None;
    let mut sb_type: Option<CustomSwitchBlockType> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                name = match name {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "type" => {
                sb_type = match sb_type {
                    None => match a.value.to_string().as_ref() {
                        "unidir" => Some(CustomSwitchBlockType::Unidir),
                        "bidir" => Some(CustomSwitchBlockType::Bidir),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: unknown custom switch block type"),
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
    let name = match name {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };
    let sb_type = match sb_type {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };

    let mut switchblock_location: Option<CustomSwitchBlockLocation> = None;
    let mut switch_funcs: Option<Vec<CustomSwitchFunc>> = None;
    let mut wireconns: Vec<CustomSwitchWireConn> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "switchblock_location" => {
                        switchblock_location = match switchblock_location {
                            None => Some(parse_switchblock_location(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "switchfuncs" => {
                        switch_funcs = match switch_funcs {
                            None => Some(parse_switchfuncs(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "wireconn" => {
                        wireconns.push(parse_wireconn(&name, &attributes, parser)?);
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
                "switchblock" => break,
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
    // FIXME: The documentation is not clear if this required.
    let switchblock_location = match switchblock_location {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<switchblock_location>".to_string(),
            ));
        }
    };
    // FIXME: The documentation is not clear if this is required.
    let switch_funcs = match switch_funcs {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<switchfuncs>".to_string(),
            ));
        }
    };

    Ok(CustomSwitchBlock {
        name,
        sb_type,
        switchblock_location,
        switch_funcs,
        wireconns,
    })
}

pub fn parse_switchblocklist(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<CustomSwitchBlock>, FPGAArchParseError> {
    assert!(name.to_string() == "switchblocklist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut custom_switch_blocks: Vec<CustomSwitchBlock> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "switchblock" => {
                        custom_switch_blocks.push(parse_switchblock(&name, &attributes, parser)?);
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
                "switchblocklist" => break,
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

    Ok(custom_switch_blocks)
}
