use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

fn parse_switch_t_del(name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<SwitchTDel, FPGAArchParseError> {
    assert!(name.to_string() == "Tdel");

    let mut num_inputs: Option<i32> = None;
    let mut delay: Option<f32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "num_inputs" => {
                num_inputs = match num_inputs {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "delay" => {
                delay = match delay {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        }
    }
    let num_inputs = match num_inputs {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("num_inputs".to_string(), parser.position())),
    };
    let delay = match delay {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("delay".to_string(), parser.position())),
    };

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "Tdel" => break,
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

    Ok(SwitchTDel {
        num_inputs,
        delay,
    })
}

fn parse_switch(name: &OwnedName,
                attributes: &[OwnedAttribute],
                parser: &mut EventReader<BufReader<File>>) -> Result<Switch, FPGAArchParseError> {
    assert!(name.to_string() == "switch");

    let mut sw_type: Option<SwitchType> = None;
    let mut sw_name: Option<String> = None;
    let mut resistance: Option<f32> = None;
    let mut c_in: Option<f32> = None;
    let mut c_out: Option<f32> = None;
    let mut c_internal: Option<f32> = None;
    let mut t_del: Option<f32> = None;
    let mut buf_size: Option<SwitchBufSize> = None;
    let mut mux_trans_size: Option<f32> = None;
    let mut power_buf_size: Option<i32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                sw_type = match sw_type {
                    None => match a.value.to_string().as_ref() {
                        "mux" => Some(SwitchType::Mux),
                        "tristate" => Some(SwitchType::Tristate),
                        "pass_gate" => Some(SwitchType::PassGate),
                        "short" => Some(SwitchType::Short),
                        "buffer" => Some(SwitchType::Buffer),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: Unknown switch type."), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "name" => {
                sw_name = match sw_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "R" => {
                resistance = match resistance {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "Cin" => {
                c_in = match c_in {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "Cout" => {
                c_out = match c_out {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "Cinternal" => {
                c_internal = match c_internal {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "Tdel" => {
                t_del = match t_del {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "buf_size" => {
                buf_size = match buf_size {
                    None => match a.value.as_ref() {
                        "auto" => Some(SwitchBufSize::Auto),
                        _ => match a.value.parse() {
                            Ok(v) => Some(SwitchBufSize::Val(v)),
                            Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                        },
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "mux_trans_size" => {
                mux_trans_size = match mux_trans_size {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "power_buf_size" => {
                power_buf_size = match power_buf_size {
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

    let sw_type = match sw_type {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("type".to_string(), parser.position())),
    };
    let sw_name = match sw_name {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let resistance = match resistance {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("R".to_string(), parser.position())),
    };
    let c_in = match c_in {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("Cin".to_string(), parser.position())),
    };
    let c_out = match c_out {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("Cout".to_string(), parser.position())),
    };
    let buf_size = buf_size.unwrap_or(SwitchBufSize::Auto);

    let mut t_del_tags: Vec<SwitchTDel> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "Tdel" => {
                        t_del_tags.push(parse_switch_t_del(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "switch" => break,
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

    Ok(Switch {
        sw_type,
        name: sw_name,
        resistance,
        c_in,
        c_out,
        c_internal,
        t_del,
        buf_size,
        mux_trans_size,
        power_buf_size,
        t_del_tags,
    })
}

pub fn parse_switch_list(name: &OwnedName,
                         attributes: &[OwnedAttribute],
                         parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Switch>, FPGAArchParseError> {
    assert!(name.to_string() == "switchlist");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut switch_list: Vec<Switch> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "switch" => {
                        switch_list.push(parse_switch(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "switchlist" => break,
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

    // TODO: It is not clear what should happen if the switch list is empty.
    //       Should confirm with the documentation.

    Ok(switch_list)
}
