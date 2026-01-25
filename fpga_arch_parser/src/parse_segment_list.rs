use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

fn parse_pattern_int_list(
    text: &str,
    parser: &EventReader<BufReader<File>>,
) -> Result<Vec<bool>, FPGAArchParseError> {
    let mut list: Vec<bool> = Vec::new();

    for v in text.split_whitespace() {
        let point_val: i32 = match v.parse() {
            Ok(val) => val,
            Err(e) => {
                return Err(FPGAArchParseError::InvalidTag(
                    format!("Pattern int list parse error: {e}"),
                    parser.position(),
                ));
            }
        };
        match point_val {
            0 => list.push(false),
            1 => list.push(true),
            _ => {
                return Err(FPGAArchParseError::InvalidTag(
                    format!("Pattern int list expected to only have 0s and 1s. Found: {point_val}"),
                    parser.position(),
                ));
            }
        }
    }

    Ok(list)
}

fn parse_segment_pattern(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<bool>, FPGAArchParseError> {
    assert!(name.to_string() == "sb" || name.to_string() == "cb");

    let mut block_type: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                block_type = match block_type {
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
        }
    }
    let block_type = match block_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };
    if block_type != "pattern" {
        return Err(FPGAArchParseError::AttributeParseError(
            format!("{name} type must be pattern"),
            parser.position(),
        ));
    }

    let mut pattern: Option<Vec<bool>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(text)) => {
                pattern = match pattern {
                    None => Some(parse_pattern_int_list(&text, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::InvalidTag(
                            "Duplicate characters within sb.".to_string(),
                            parser.position(),
                        ));
                    }
                }
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
    let pattern = match pattern {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::InvalidTag(
                "Missing pattern int list".to_string(),
                parser.position(),
            ));
        }
    };

    Ok(pattern)
}

fn parse_segment_switch_point_descriptor(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<String, FPGAArchParseError> {
    let mut desc_name: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                desc_name = match desc_name {
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
        }
    }
    let desc_name = match desc_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };

    loop {
        match parser.next() {
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

    Ok(desc_name)
}

fn parse_segment(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Segment, FPGAArchParseError> {
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
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: Unknown segment axis"),
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
            "length" => {
                length = match length {
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
            "type" => {
                segment_type = match segment_type {
                    None => match a.value.as_ref() {
                        "bidir" => Some(SegmentType::Bidir),
                        "unidir" => Some(SegmentType::Unidir),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: Unknown segment type"),
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
            "res_type" => {
                res_type = match res_type {
                    None => match a.value.as_ref() {
                        "GCLK" => Some(SegmentResourceType::Gclk),
                        "GENERAL" => Some(SegmentResourceType::General),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("{a}: Unknown segment resource type"),
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
            "freq" => {
                freq = match freq {
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
            "Rmetal" => {
                r_metal = match r_metal {
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
            "Cmetal" => {
                c_metal = match c_metal {
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

    // DOCUMENTATION ISSUE: Some architectures do not specify names. This either
    //                      needs to be enforced or documented as optional.
    let name = match name {
        Some(n) => n,
        None => String::from("UnnamedSegment"),
    };
    let axis = axis.unwrap_or(SegmentAxis::XY);
    let length = match length {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "length".to_string(),
                parser.position(),
            ));
        }
    };
    let segment_type = match segment_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };
    let freq = match freq {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "freq".to_string(),
                parser.position(),
            ));
        }
    };
    let r_metal = match r_metal {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "Rmetal".to_string(),
                parser.position(),
            ));
        }
    };
    let c_metal = match c_metal {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "Cmetal".to_string(),
                parser.position(),
            ));
        }
    };

    let res_type = res_type.unwrap_or(SegmentResourceType::General);

    let mut sb_pattern: Option<Vec<bool>> = None;
    let mut cb_pattern: Option<Vec<bool>> = None;
    let mut mux: Option<String> = None;
    let mut mux_inc: Option<String> = None;
    let mut mux_dec: Option<String> = None;
    let mut wire_switch: Option<String> = None;
    let mut opin_switch: Option<String> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "sb" => {
                        sb_pattern = match sb_pattern {
                            None => Some(parse_segment_pattern(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        };
                    }
                    "cb" => {
                        cb_pattern = match cb_pattern {
                            None => Some(parse_segment_pattern(&name, &attributes, parser)?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        };
                    }
                    "mux" => {
                        mux = match mux {
                            None => Some(parse_segment_switch_point_descriptor(
                                &name,
                                &attributes,
                                parser,
                            )?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "mux_inc" => {
                        mux_inc = match mux_inc {
                            None => Some(parse_segment_switch_point_descriptor(
                                &name,
                                &attributes,
                                parser,
                            )?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "mux_dec" => {
                        mux_dec = match mux_dec {
                            None => Some(parse_segment_switch_point_descriptor(
                                &name,
                                &attributes,
                                parser,
                            )?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "wire_switch" => {
                        wire_switch = match wire_switch {
                            None => Some(parse_segment_switch_point_descriptor(
                                &name,
                                &attributes,
                                parser,
                            )?),
                            Some(_) => {
                                return Err(FPGAArchParseError::DuplicateTag(
                                    format!("<{name}>"),
                                    parser.position(),
                                ));
                            }
                        }
                    }
                    "opin_switch" => {
                        opin_switch = match opin_switch {
                            None => Some(parse_segment_switch_point_descriptor(
                                &name,
                                &attributes,
                                parser,
                            )?),
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
                "segment" => break,
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
    let sb_pattern = match sb_pattern {
        Some(p) => {
            if p.len() as i32 != length + 1 {
                return Err(FPGAArchParseError::InvalidTag(
                    "For a length L wire there must be L+1 entries separated by spaces for <sb>"
                        .to_string(),
                    parser.position(),
                ));
            } else {
                p
            }
        }
        None => return Err(FPGAArchParseError::MissingRequiredTag("<sb>".to_string())),
    };
    let cb_pattern = match cb_pattern {
        Some(p) => {
            if p.len() as i32 != length {
                return Err(FPGAArchParseError::InvalidTag(
                    "For a length L wire there must be L entries separated by spaces for <cb>"
                        .to_string(),
                    parser.position(),
                ));
            } else {
                p
            }
        }
        None => return Err(FPGAArchParseError::MissingRequiredTag("<sb>".to_string())),
    };
    let switch_points = match segment_type {
        SegmentType::Unidir => match mux {
            Some(mux_name) => {
                if mux_inc.is_some() || mux_dec.is_some() {
                    return Err(FPGAArchParseError::InvalidTag("For unidirectional segments, either <mux> tag or both <mux_inc> and <mux_dec> should be defined in the architecture file.".to_string(), parser.position()));
                }
                SegmentSwitchPoints::Unidir {
                    mux_inc: mux_name.clone(),
                    mux_dec: mux_name,
                }
            }
            None => {
                let mux_inc = match mux_inc {
                        Some(m) => m,
                        None => return Err(FPGAArchParseError::InvalidTag("For unidirectional segments, either <mux> tag or both <mux_inc> and <mux_dec> should be defined in the architecture file.".to_string(), parser.position())),
                    };
                let mux_dec = match mux_dec {
                        Some(m) => m,
                        None => return Err(FPGAArchParseError::InvalidTag("For unidirectional segments, either <mux> tag or both <mux_inc> and <mux_dec> should be defined in the architecture file.".to_string(), parser.position())),
                    };
                SegmentSwitchPoints::Unidir { mux_inc, mux_dec }
            }
        },
        SegmentType::Bidir => {
            let wire_switch = match wire_switch {
                Some(w) => w,
                None => {
                    return Err(FPGAArchParseError::MissingRequiredTag(
                        "<wire_switch>".to_string(),
                    ));
                }
            };
            let opin_switch = match opin_switch {
                Some(w) => w,
                None => {
                    return Err(FPGAArchParseError::MissingRequiredTag(
                        "<opin_switch>".to_string(),
                    ));
                }
            };
            SegmentSwitchPoints::Bidir {
                wire_switch,
                opin_switch,
            }
        }
    };

    Ok(Segment {
        name,
        axis,
        length,
        segment_type,
        res_type,
        freq,
        r_metal,
        c_metal,
        sb_pattern,
        cb_pattern,
        switch_points,
    })
}

pub fn parse_segment_list(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<Segment>, FPGAArchParseError> {
    assert!(name.to_string() == "segmentlist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut segments: Vec<Segment> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "segment" => {
                        segments.push(parse_segment(&name, &attributes, parser)?);
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
                "segmentlist" => break,
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

    Ok(segments)
}
