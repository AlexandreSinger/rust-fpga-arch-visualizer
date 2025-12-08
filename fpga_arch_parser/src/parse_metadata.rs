use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

fn parse_meta(tag_name: &OwnedName,
              attributes: &[OwnedAttribute],
              parser: &mut EventReader<BufReader<File>>) -> Result<Metadata, FPGAArchParseError> {
    assert!(tag_name.to_string() == "meta");

    let mut name: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_str() {
            "name" => {
                name = match name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }
    let name = match name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("name".to_string(), parser.position())),
    };

    let mut value: Option<String> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::Characters(text)) => {
                value = match value {
                    None => Some(text),
                    Some(_) => return Err(FPGAArchParseError::InvalidTag("Duplicate characters within meta.".to_string(), parser.position())),
                }
            },
            Ok(XmlEvent::EndElement { name: end_name }) => {
                match end_name.to_string().as_ref() {
                    "meta" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
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

    // TODO: Documentation is not clear if the value should be empty. Will allow
    //       it since it is intuitive to not have a value (the name acts as a flag).
    let value = value.unwrap_or_default();

    Ok(Metadata {
        name,
        value,
    })
}

pub fn parse_metadata(tag_name: &OwnedName,
                      attributes: &[OwnedAttribute],
                      parser: &mut EventReader<BufReader<File>>) -> Result<Vec<Metadata>, FPGAArchParseError> {
    assert!(tag_name.to_string() == "metadata");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut metadata: Vec<Metadata> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "meta" => {
                        metadata.push(parse_meta(&name, &attributes, parser)?);
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "metadata" => break,
                    _ => return Err(FPGAArchParseError::UnexpectedEndTag(name.to_string(), parser.position())),
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

    Ok(metadata)
}
