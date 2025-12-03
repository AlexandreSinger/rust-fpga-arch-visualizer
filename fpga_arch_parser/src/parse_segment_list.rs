use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

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

pub fn parse_segment_list(name: &OwnedName,
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

