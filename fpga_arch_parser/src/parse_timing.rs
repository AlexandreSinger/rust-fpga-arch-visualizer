use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

pub fn parse_delay_constant(name: &OwnedName,
                            attributes: &[OwnedAttribute],
                            parser: &mut EventReader<BufReader<File>>) -> Result<DelayInfo, FPGAArchParseError> {
    assert!(name.to_string() == "delay_constant");

    let mut min: Option<f32> = None;
    let mut max: Option<f32> = None;
    let mut in_port: Option<String> = None;
    let mut out_port: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "min" => {
                min = match min {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "max" => {
                max = match max {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
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
    let in_port = match in_port {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("in_port".to_string(), parser.position())),
    };
    let out_port = match out_port {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("out_port".to_string(), parser.position())),
    };
    let (min, max) = match min {
        Some(min) => match max {
            Some(max) => (min, max),
            None => (min, min),
        },
        None => match max {
            Some(max) => (max, max),
            None => return Err(FPGAArchParseError::MissingRequiredAttribute("At least one of the max or min attributes must be specified".to_string(), parser.position())),
        },
    };

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_ref() {
                    "delay_constant" => break,
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
        };
    }

    Ok(DelayInfo::Constant {
        min,
        max,
        in_port,
        out_port,
    })
}

fn parse_matrix_text(text: &str,
                     parser: &EventReader<BufReader<File>>) -> Result<Vec<Vec<f32>>, FPGAArchParseError> {
    let mut matrix: Vec<Vec<f32>> = Vec::new();

    for line in text.split('\n') {
        let mut matrix_row: Vec<f32> = Vec::new();
        for v in line.split_whitespace() {
            let element_val: f32 = match v.parse() {
                Ok(val) => val,
                Err(e) => return Err(FPGAArchParseError::InvalidTag(format!("Matrix text parse error: {e}"), parser.position())),
            };
            matrix_row.push(element_val);
        }
        matrix.push(matrix_row);
    }

    Ok(matrix)
}

pub fn parse_delay_matrix(name: &OwnedName,
                          attributes: &[OwnedAttribute],
                          parser: &mut EventReader<BufReader<File>>) -> Result<DelayInfo, FPGAArchParseError> {
    assert!(name.to_string() == "delay_matrix");

    let mut delay_type: Option<DelayType> = None;
    let mut in_port: Option<String> = None;
    let mut out_port: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                delay_type = match delay_type {
                    None => match a.value.as_ref() {
                        "max" => Some(DelayType::Max),
                        "min" => Some(DelayType::Min),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("Unknown delay type: {}", a.value), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                };
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
    let delay_type = match delay_type {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("type".to_string(), parser.position())),
    };
    let in_port = match in_port {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("in_port".to_string(), parser.position())),
    };
    let out_port = match out_port {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("out_port".to_string(), parser.position())),
    };

    let mut matrix: Option<Vec<Vec<f32>>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(text)) => {
                matrix = match matrix {
                    None => Some(parse_matrix_text(&text, parser)?),
                    Some(_) => return Err(FPGAArchParseError::InvalidTag("Duplicate characters within delay_matrix.".to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndElement { name: end_name }) => {
                if end_name.to_string() == name.to_string() {
                    break;
                } else {
                    return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position()));
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
    let matrix = match matrix {
        Some(p) => p,
        None => return Err(FPGAArchParseError::InvalidTag("delay_matrix tag must contain text representation of the matrix data".to_string(), parser.position())),
    };

    Ok(DelayInfo::Matrix {
        delay_type,
        matrix,
        in_port,
        out_port,
    })
}

fn parse_setup_or_hold(name: &OwnedName,
                       attributes: &[OwnedAttribute],
                       constraint_type: TimingConstraintType,
                       parser: &mut EventReader<BufReader<File>>) -> Result<TimingConstraintInfo, FPGAArchParseError> {
    assert!(name.to_string() == "T_setup" || name.to_string() == "T_hold");

    let mut value: Option<f32> = None;
    let mut port: Option<String> = None;
    let mut clock: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "value" => {
                value = match value {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "port" => {
                port = match port {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "clock" => {
                clock = match clock {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }
    let value = match value {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("value".to_string(), parser.position())),
    };
    let port = match port {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("port".to_string(), parser.position())),
    };
    let clock = match clock {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("clock".to_string(), parser.position())),
    };

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name: end_name }) => {
                if end_name.to_string() == name.to_string() {
                    break;
                } else {
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
    }

    Ok(TimingConstraintInfo {
        constraint_type,
        min_value: value,
        max_value: value,
        port,
        clock,
    })
}

pub fn parse_t_setup(name: &OwnedName,
                     attributes: &[OwnedAttribute],
                     parser: &mut EventReader<BufReader<File>>) -> Result<TimingConstraintInfo, FPGAArchParseError> {
    assert!(name.to_string() == "T_setup");

    parse_setup_or_hold(name, attributes, TimingConstraintType::Setup, parser)
}

pub fn parse_t_hold(name: &OwnedName,
                    attributes: &[OwnedAttribute],
                    parser: &mut EventReader<BufReader<File>>) -> Result<TimingConstraintInfo, FPGAArchParseError> {
    assert!(name.to_string() == "T_hold");

    parse_setup_or_hold(name, attributes, TimingConstraintType::Hold, parser)
}

pub fn parse_clock_to_q(name: &OwnedName,
                        attributes: &[OwnedAttribute],
                        parser: &mut EventReader<BufReader<File>>) -> Result<TimingConstraintInfo, FPGAArchParseError> {
    assert!(name.to_string() == "T_clock_to_Q");

    let mut min: Option<f32> = None;
    let mut max: Option<f32> = None;
    let mut port: Option<String> = None;
    let mut clock: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "min" => {
                min = match min {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "max" => {
                max = match max {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "port" => {
                port = match port {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "clock" => {
                clock = match clock {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }
    let (min_value, max_value) = match min {
        Some(min) => match max {
            Some(max) => (min, max),
            None => (min, min),
        },
        None => match max {
            Some(max) => (max, max),
            None => return Err(FPGAArchParseError::MissingRequiredAttribute("At least one of the max or min attributes must be specified".to_string(), parser.position())),
        },
    };
    let port = match port {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("port".to_string(), parser.position())),
    };
    let clock = match clock {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("clock".to_string(), parser.position())),
    };

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name: end_name }) => {
                if end_name.to_string() == name.to_string() {
                    break;
                } else {
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
    }

    Ok(TimingConstraintInfo {
        constraint_type: TimingConstraintType::ClockToQ,
        min_value,
        max_value,
        port,
        clock,
    })
}
