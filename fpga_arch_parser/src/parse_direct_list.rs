use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

fn parse_direct(
    tag_name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<GlobalDirect, FPGAArchParseError> {
    assert!(tag_name.to_string() == "direct");

    let mut name: Option<String> = None;
    let mut from_pin: Option<String> = None;
    let mut to_pin: Option<String> = None;
    let mut x_offset: Option<i32> = None;
    let mut y_offset: Option<i32> = None;
    let mut z_offset: Option<i32> = None;
    let mut switch_name: Option<String> = None;
    let mut from_side: Option<PinSide> = None;
    let mut to_side: Option<PinSide> = None;
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
            "from_pin" => {
                from_pin = match from_pin {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "to_pin" => {
                to_pin = match to_pin {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "x_offset" => {
                x_offset = match x_offset {
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
            "y_offset" => {
                y_offset = match y_offset {
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
            "z_offset" => {
                z_offset = match z_offset {
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
            "switch_name" => {
                switch_name = match switch_name {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "from_side" => {
                from_side = match from_side {
                    None => match a.value.as_ref() {
                        "left" => Some(PinSide::Left),
                        "right" => Some(PinSide::Right),
                        "top" => Some(PinSide::Top),
                        "bottom" => Some(PinSide::Bottom),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("Unknown side: {}", a.value),
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
            "to_side" => {
                to_side = match to_side {
                    None => match a.value.as_ref() {
                        "left" => Some(PinSide::Left),
                        "right" => Some(PinSide::Right),
                        "top" => Some(PinSide::Top),
                        "bottom" => Some(PinSide::Bottom),
                        _ => {
                            return Err(FPGAArchParseError::AttributeParseError(
                                format!("Unknown side: {}", a.value),
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
    let from_pin = match from_pin {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "from_pin".to_string(),
                parser.position(),
            ));
        }
    };
    let to_pin = match to_pin {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "to_pin".to_string(),
                parser.position(),
            ));
        }
    };
    let x_offset = match x_offset {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "x_offset".to_string(),
                parser.position(),
            ));
        }
    };
    let y_offset = match y_offset {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "y_offset".to_string(),
                parser.position(),
            ));
        }
    };
    let z_offset = match z_offset {
        Some(p) => p,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "z_offset".to_string(),
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
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_ref() {
                "direct" => break,
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

    Ok(GlobalDirect {
        name,
        from_pin,
        to_pin,
        x_offset,
        y_offset,
        z_offset,
        switch_name,
        from_side,
        to_side,
    })
}

pub fn parse_direct_list(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<GlobalDirect>, FPGAArchParseError> {
    assert!(name.to_string() == "directlist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut direct_list: Vec<GlobalDirect> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "direct" => {
                        direct_list.push(parse_direct(&name, &attributes, parser)?);
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
                "directlist" => break,
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

    Ok(direct_list)
}
