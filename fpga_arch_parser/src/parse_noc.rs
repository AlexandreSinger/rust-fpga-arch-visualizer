use std::collections::HashSet;
use std::io::BufRead;

use xml::common::Position;
use xml::reader::XmlEvent;
use xml::{EventReader, attribute::OwnedAttribute, name::OwnedName};

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

    // FIXME: Currently VTR cannot specify an individual NoC router's layer;
    //        however, the mesh topology can. There is a gap in the spec.
    Ok(NoCRouterInfo {
        id,
        position_x,
        position_y,
        layer: 0,
        connections,
    })
}

fn parse_mesh<R: BufRead>(
    tag_name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<R>,
) -> Result<NoCTopologyInfo, FPGAArchParseError> {
    assert!(tag_name.to_string() == "mesh");

    let mut start_x: Option<f32> = None;
    let mut end_x: Option<f32> = None;
    let mut start_y: Option<f32> = None;
    let mut end_y: Option<f32> = None;
    let mut start_layer: Option<i32> = None;
    let mut end_layer: Option<i32> = None;
    let mut size: Option<i32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "startx" => {
                start_x = match start_x {
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
            "endx" => {
                end_x = match end_x {
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
            "starty" => {
                start_y = match start_y {
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
            "endy" => {
                end_y = match end_y {
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
            "startlayer" => {
                start_layer = match start_layer {
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
            "endlayer" => {
                end_layer = match end_layer {
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
            "size" => {
                size = match size {
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

    let start_x = match start_x {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "startx".to_string(),
                parser.position(),
            ));
        }
    };
    let end_x = match end_x {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "endx".to_string(),
                parser.position(),
            ));
        }
    };
    let start_y = match start_y {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "starty".to_string(),
                parser.position(),
            ));
        }
    };
    let end_y = match end_y {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "endy".to_string(),
                parser.position(),
            ));
        }
    };
    let start_layer = start_layer.unwrap_or(0);
    let end_layer = end_layer.unwrap_or(0);
    let size = match size {
        Some(v) => v,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "size".to_string(),
                parser.position(),
            ));
        }
    };

    if size < 2 {
        return Err(FPGAArchParseError::InvalidTag(
            format!("NoC mesh size must be greater than 1, got {size}"),
            parser.position(),
        ));
    }
    if end_layer < start_layer {
        return Err(FPGAArchParseError::InvalidTag(
            format!("NoC mesh endlayer ({end_layer}) must be >= startlayer ({start_layer})"),
            parser.position(),
        ));
    }
    if start_layer < 0 {
        return Err(FPGAArchParseError::InvalidTag(
            format!("NoC mesh start_layer ({start_layer}) must be non-negative"),
            parser.position(),
        ));
    }

    // Consume the mesh end tag; the mesh element has no children.
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                return Err(FPGAArchParseError::InvalidTag(
                    name.to_string(),
                    parser.position(),
                ));
            }
            Ok(XmlEvent::EndElement { name }) => match name.to_string().as_ref() {
                "mesh" => break,
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

    // Generate the mesh topology. Router IDs start at 0 at the bottom-left
    // corner of the first layer and increase in row-major order across layers.
    let num_layers = (end_layer - start_layer + 1) as usize;
    let size = size as usize;
    let mut routers: Vec<NoCRouterInfo> = Vec::with_capacity(num_layers * size * size);

    for layer in 0..num_layers {
        for row in 0..size {
            for col in 0..size {
                let id = (layer * size * size + row * size + col) as i32;

                let pos_x = start_x + col as f32 * (end_x - start_x) / (size - 1) as f32;
                let pos_y = start_y + row as f32 * (end_y - start_y) / (size - 1) as f32;

                let mut connections: Vec<i32> = Vec::new();
                if col > 0 {
                    // Left neighbour.
                    connections.push((layer * size * size + row * size + (col - 1)) as i32);
                }
                if col < size - 1 {
                    // Right neighbour.
                    connections.push((layer * size * size + row * size + (col + 1)) as i32);
                }
                if row > 0 {
                    // Below neighbour.
                    connections.push((layer * size * size + (row - 1) * size + col) as i32);
                }
                if row < size - 1 {
                    // Above neighbour.
                    connections.push((layer * size * size + (row + 1) * size + col) as i32);
                }
                if layer > 0 {
                    // Neighbour on layer below.
                    connections.push(((layer - 1) * size * size + row * size + col) as i32);
                }
                if layer < num_layers - 1 {
                    // Neighbour on layer above.
                    connections.push(((layer + 1) * size * size + row * size + col) as i32);
                }

                routers.push(NoCRouterInfo {
                    id,
                    position_x: pos_x,
                    position_y: pos_y,
                    layer: start_layer as usize + layer,
                    connections,
                });
            }
        }
    }

    Ok(NoCTopologyInfo { routers })
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
            return Err(FPGAArchParseError::InvalidTag(
                format!("Found a NoC router with a duplicate ID: {}", router.id),
                parser.position(),
            ));
        }
        known_ids.insert(router.id);
    }

    // Verify that each router connects to a valid router.
    for router in &routers {
        for connection in &router.connections {
            if !known_ids.contains(connection) {
                return Err(FPGAArchParseError::InvalidTag(
                    format!(
                        "Found a NoC router {} with an invalid connection ID: {}",
                        router.id, connection
                    ),
                    parser.position(),
                ));
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
                "mesh" => {
                    topology = match topology {
                        None => Some(parse_mesh(&child_name, &child_attributes, parser)?),
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
                "<topology> or <mesh>".to_string(),
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
