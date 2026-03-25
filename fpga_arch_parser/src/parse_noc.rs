use std::collections::HashSet;
use std::io::BufRead;

use xml::common::Position;
use xml::{EventReader, attribute::OwnedAttribute, name::OwnedName};
use xml::reader::XmlEvent;

use crate::{FPGAArchParseError, NoCInfo, NoCRouterInfo, NoCTopologyInfo};

fn parse_router<R: BufRead>(
    tag_name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<R>,
) -> Result<NoCRouterInfo, FPGAArchParseError> {
    assert!(tag_name.to_string() == "router");

    let mut id: Option<i32> = None;
    let mut position_x: Option<f32> = None;
    let mut position_y: Option<f32> = None;
    let mut connections: Option<Vec<i32>> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "id" => {
                id = match id {
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
            "positionx" => {
                position_x = match position_x {
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
            "positiony" => {
                position_y = match position_y {
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
            "connections" => {
                connections = match connections {
                    None => {
                        let mut conn_ids: Vec<i32> = Vec::new();
                        for token in a.value.split_whitespace() {
                            match token.parse() {
                                Ok(v) => conn_ids.push(v),
                                Err(e) => {
                                    return Err(FPGAArchParseError::AttributeParseError(
                                        format!("{a}: {e}"),
                                        parser.position(),
                                    ));
                                }
                            }
                        }
                        Some(conn_ids)
                    }
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

    let id = match id {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "id".to_string(),
                parser.position(),
            ));
        }
    };
    let position_x = match position_x {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "positionx".to_string(),
                parser.position(),
            ));
        }
    };
    let position_y = match position_y {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "positiony".to_string(),
                parser.position(),
            ));
        }
    };
    let connections = match connections {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "connections".to_string(),
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
                "router" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(
                    tag_name.to_string(),
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

    Ok(NoCRouterInfo {
        id,
        position_x,
        position_y,
        connections,
    })
}

fn parse_topology<R: BufRead>(
    tag_name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<R>,
) -> Result<NoCTopologyInfo, FPGAArchParseError> {
    assert!(tag_name.to_string() == "topology");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(
            String::from("Expected to be empty"),
            parser.position(),
        ));
    }

    let mut routers: Vec<NoCRouterInfo> = Vec::new();

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => match name.to_string().as_str() {
                "router" => {
                    routers.push(parse_router(&name, &attributes, parser)?);
                }
                _ => {
                    return Err(FPGAArchParseError::InvalidTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_str() {
                "topology" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndDocument) => {
                return Err(FPGAArchParseError::UnexpectedEndOfDocument(
                    tag_name.to_string(),
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

    // Verify that each router's ID is unique.
    let mut known_ids: HashSet<i32> = HashSet::new();
    for router in &routers {
        if known_ids.contains(&router.id) {
            return Err(FPGAArchParseError::InvalidTag(format!("Found a NoC router with a duplicate ID: {}", router.id), parser.position()));
        }
        known_ids.insert(router.id);
    }

    // Verify that each router connects to a valid router.
    for router in &routers {
        for connection in &router.connections {
            if !known_ids.contains(&connection) {
                return Err(FPGAArchParseError::InvalidTag(format!("Found a NoC router {} with an invalid connection ID: {}", router.id, connection), parser.position()));
            }
        }
    }

    Ok(NoCTopologyInfo { routers })
}

pub fn parse_noc<R: BufRead>(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<R>,
) -> Result<NoCInfo, FPGAArchParseError> {
    assert!(name.to_string() == "noc");

    let mut link_bandwidth: Option<f32> = None;
    let mut link_latency: Option<f32> = None;
    let mut router_latency: Option<f32> = None;
    let mut noc_router_tile_name: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "link_bandwidth" => {
                link_bandwidth = match link_bandwidth {
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
            "link_latency" => {
                link_latency = match link_latency {
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
            "router_latency" => {
                router_latency = match router_latency {
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
            "noc_router_tile_name" => {
                noc_router_tile_name = match noc_router_tile_name {
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

    let link_bandwidth = match link_bandwidth {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "link_bandwidth".to_string(),
                parser.position(),
            ));
        }
    };
    let link_latency = match link_latency {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "link_latency".to_string(),
                parser.position(),
            ));
        }
    };
    let router_latency = match router_latency {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "router_latency".to_string(),
                parser.position(),
            ));
        }
    };
    let noc_router_tile_name = match noc_router_tile_name {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "noc_router_tile_name".to_string(),
                parser.position(),
            ));
        }
    };

    let mut topology: Option<NoCTopologyInfo> = None;

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name: child_name,
                attributes: child_attributes,
                ..
            }) => match child_name.to_string().as_str() {
                "topology" => {
                    topology = match topology {
                        None => Some(parse_topology(&child_name, &child_attributes, parser)?),
                        Some(_) => {
                            return Err(FPGAArchParseError::DuplicateTag(
                                format!("<{child_name}>"),
                                parser.position(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(FPGAArchParseError::InvalidTag(
                        child_name.to_string(),
                        parser.position(),
                    ));
                }
            },
            Ok(XmlEvent::EndElement { name: end_name }) => match end_name.to_string().as_str() {
                "noc" => break,
                _ => {
                    return Err(FPGAArchParseError::UnexpectedEndTag(
                        end_name.to_string(),
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

    let topology = match topology {
        Some(t) => t,
        None => {
            return Err(FPGAArchParseError::MissingRequiredTag(
                "<topology>".to_string(),
            ));
        }
    };

    Ok(NoCInfo {
        link_bandwidth,
        link_latency,
        router_latency,
        noc_router_tile_name,
        topology,
    })
}
