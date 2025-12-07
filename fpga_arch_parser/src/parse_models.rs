use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

fn parse_model_port(tag_name: &OwnedName,
                    attributes: &[OwnedAttribute],
                    parser: &mut EventReader<BufReader<File>>) -> Result<ModelPort, FPGAArchParseError> {
    assert!(tag_name.to_string() == "port");

    let mut name: Option<String> = None;
    let mut is_clock: Option<bool> = None;
    let mut clock: Option<String> = None;
    let mut combinational_sink_ports: Option<Vec<String>> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                name = match name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "is_clock" => {
                is_clock = match is_clock {
                    None => match a.value.as_ref() {
                        "0" => Some(false),
                        "1" => Some(true),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("is_clock can only be 0 or 1, found: {}", a.value), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "clock" => {
                clock = match clock {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "combinational_sink_ports" => {
                combinational_sink_ports = match combinational_sink_ports {
                    None => Some(a.value.split_whitespace().map(str::to_string).collect()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }
    let name = match name {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let is_clock = is_clock.unwrap_or(false);
    let combinational_sink_ports = combinational_sink_ports.unwrap_or_default();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position()));
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_ref() {
                    "port" => break,
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

    Ok(ModelPort {
        name,
        is_clock,
        clock,
        combinational_sink_ports,
    })
}

fn parse_model_port_list(tag_name: &OwnedName,
                         attributes: &[OwnedAttribute],
                         parser: &mut EventReader<BufReader<File>>) -> Result<Vec<ModelPort>, FPGAArchParseError> {
    assert!(tag_name.to_string() == "input_ports" || tag_name.to_string() == "output_ports");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut ports: Vec<ModelPort> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "port" => {
                        ports.push(parse_model_port(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == tag_name.to_string() {
                    break;
                } else {
                    return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position()));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(tag_name.to_string()));
            },
            Err(e) => {
                return Err(FPGAArchParseError::XMLParseError(format!("{e:?}"), parser.position()));
            },
            _ => {},
        }
    };

    Ok(ports)
}

fn parse_model(tag_name: &OwnedName,
               attributes: &[OwnedAttribute],
               parser: &mut EventReader<BufReader<File>>) -> Result<Model, FPGAArchParseError> {
    assert!(tag_name.to_string() == "model");

    let mut name: Option<String> = None; 
    let mut never_prune: Option<bool> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                name = match name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "never_prune" => {
                never_prune = match never_prune {
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
    let name = match name {
        Some(p) => p,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };
    let never_prune = never_prune.unwrap_or(false);

    let mut input_ports: Option<Vec<ModelPort>> = None;
    let mut output_ports: Option<Vec<ModelPort>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "input_ports" => {
                        input_ports = match input_ports {
                            None => Some(parse_model_port_list(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "output_ports" => {
                        output_ports = match output_ports {
                            None => Some(parse_model_port_list(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "model" => break,
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
    let input_ports = match input_ports {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<input_ports>".to_string())),
    };
    let output_ports = match output_ports {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<output_ports>".to_string())),
    };

    Ok(Model {
        name,
        never_prune,
        input_ports,
        output_ports,
    })
}

pub fn parse_models(name: &OwnedName,
                    attributes: &[OwnedAttribute],
                    parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Model>, FPGAArchParseError> {
    assert!(name.to_string() == "models");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut models: Vec<Model> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "model" => {
                        models.push(parse_model(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "models" => break,
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

    Ok(models)
}
