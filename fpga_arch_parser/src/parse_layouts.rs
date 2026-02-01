use std::fs::File;
use std::io::BufReader;

use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::arch::*;
use crate::parse_error::*;

use crate::parse_metadata::parse_metadata;

fn parse_bool_attribute(
    value: &str,
    parser: &EventReader<BufReader<File>>,
) -> Result<bool, FPGAArchParseError> {
    match value.to_lowercase().as_str() {
        "true" | "on" | "1" | "yes" => Ok(true),
        "false" | "off" | "0" | "no" => Ok(false),
        _ => Err(FPGAArchParseError::AttributeParseError(
            format!("Invalid boolean value: {}", value),
            parser.position(),
        )),
    }
}

fn parse_grid_location(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<GridLocation, FPGAArchParseError> {
    let mut pb_type: Option<String> = None;
    let mut priority: Option<i32> = None;
    let mut x_expr: Option<String> = None;
    let mut y_expr: Option<String> = None;
    let mut start_x_expr: Option<String> = None;
    let mut end_x_expr: Option<String> = None;
    let mut repeat_x_expr: Option<String> = None;
    let mut incr_x_expr: Option<String> = None;
    let mut start_y_expr: Option<String> = None;
    let mut end_y_expr: Option<String> = None;
    let mut repeat_y_expr: Option<String> = None;
    let mut incr_y_expr: Option<String> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                pb_type = match pb_type {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "priority" => {
                priority = match priority {
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
            "x" => {
                x_expr = match x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "y" => {
                y_expr = match y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "startx" => {
                start_x_expr = match start_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "endx" => {
                end_x_expr = match end_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "repeatx" => {
                repeat_x_expr = match repeat_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "incrx" => {
                incr_x_expr = match incr_x_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "starty" => {
                start_y_expr = match start_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "endy" => {
                end_y_expr = match end_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "repeaty" => {
                repeat_y_expr = match repeat_y_expr {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "incry" => {
                incr_y_expr = match incr_y_expr {
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

    let pb_type = match pb_type {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "type".to_string(),
                parser.position(),
            ));
        }
    };
    let priority = match priority {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "priority".to_string(),
                parser.position(),
            ));
        }
    };

    let start_x_expr = start_x_expr.unwrap_or(String::from("0"));
    let end_x_expr = end_x_expr.unwrap_or(String::from("W - 1"));
    let incr_x_expr = incr_x_expr.unwrap_or(String::from("w"));
    let start_y_expr = start_y_expr.unwrap_or(String::from("0"));
    let end_y_expr = end_y_expr.unwrap_or(String::from("H - 1"));
    let incr_y_expr = incr_y_expr.unwrap_or(String::from("h"));

    let mut metadata: Option<Vec<Metadata>> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "metadata" => {
                        metadata = match metadata {
                            None => Some(parse_metadata(&name, &attributes, parser)?),
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

    match name.to_string().as_ref() {
        "perimeter" => Ok(GridLocation::Perimeter(PerimeterGridLocation {
            pb_type,
            priority,
            metadata,
        })),
        "corners" => Ok(GridLocation::Corners(CornersGridLocation {
            pb_type,
            priority,
            metadata,
        })),
        "fill" => Ok(GridLocation::Fill(FillGridLocation {
            pb_type,
            priority,
            metadata,
        })),
        "single" => {
            let x_expr = match x_expr {
                Some(n) => n,
                None => {
                    return Err(FPGAArchParseError::MissingRequiredAttribute(
                        "x".to_string(),
                        parser.position(),
                    ));
                }
            };
            let y_expr = match y_expr {
                Some(n) => n,
                None => {
                    return Err(FPGAArchParseError::MissingRequiredAttribute(
                        "y".to_string(),
                        parser.position(),
                    ));
                }
            };
            Ok(GridLocation::Single(SingleGridLocation {
                pb_type,
                priority,
                x_expr,
                y_expr,
                metadata,
            }))
        }
        "col" => Ok(GridLocation::Col(ColGridLocation {
            pb_type,
            priority,
            start_x_expr,
            repeat_x_expr,
            start_y_expr,
            incr_y_expr,
            metadata,
        })),
        "row" => Ok(GridLocation::Row(RowGridLocation {
            pb_type,
            priority,
            start_x_expr,
            incr_x_expr,
            start_y_expr,
            repeat_y_expr,
            metadata,
        })),
        "region" => Ok(GridLocation::Region(RegionGridLocation {
            pb_type,
            priority,
            start_x_expr,
            end_x_expr,
            repeat_x_expr,
            incr_x_expr,
            start_y_expr,
            end_y_expr,
            repeat_y_expr,
            incr_y_expr,
            metadata,
        })),
        _ => Err(FPGAArchParseError::InvalidTag(
            format!("Unknown grid location: {name}"),
            parser.position(),
        )),
    }
}

fn parse_grid_location_list(
    layout_type_name: &OwnedName,
    parser: &mut EventReader<BufReader<File>>,
) -> Result<Vec<GridLocation>, FPGAArchParseError> {
    let mut grid_locations: Vec<GridLocation> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                grid_locations.push(parse_grid_location(&name, &attributes, parser)?);
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.to_string() == layout_type_name.to_string() {
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
                    layout_type_name.to_string(),
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

    Ok(grid_locations)
}

fn parse_auto_layout(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<AutoLayout, FPGAArchParseError> {
    assert!(name.to_string() == "auto_layout");

    let mut aspect_ratio: Option<f32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "aspect_ratio" => {
                aspect_ratio = match aspect_ratio {
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
        }
    }

    let aspect_ratio = aspect_ratio.unwrap_or(1.0);

    let grid_locations = parse_grid_location_list(name, parser)?;

    Ok(AutoLayout {
        aspect_ratio,
        grid_locations,
    })
}

fn parse_fixed_layout(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<FixedLayout, FPGAArchParseError> {
    assert!(name.to_string() == "fixed_layout");

    let mut layout_name: Option<String> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "name" => {
                layout_name = match layout_name {
                    None => Some(a.value.clone()),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "width" => {
                width = match width {
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
            "height" => {
                height = match height {
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
        }
    }

    let layout_name = match layout_name {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "name".to_string(),
                parser.position(),
            ));
        }
    };
    let width = match width {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "width".to_string(),
                parser.position(),
            ));
        }
    };
    let height = match height {
        Some(n) => n,
        None => {
            return Err(FPGAArchParseError::MissingRequiredAttribute(
                "height".to_string(),
                parser.position(),
            ));
        }
    };

    let grid_locations = parse_grid_location_list(name, parser)?;

    Ok(FixedLayout {
        name: layout_name,
        width,
        height,
        grid_locations,
    })
}

pub fn parse_layouts(
    name: &OwnedName,
    attributes: &[OwnedAttribute],
    parser: &mut EventReader<BufReader<File>>,
) -> Result<DeviceLayouts, FPGAArchParseError> {
    assert!(name.to_string() == "layout");

    // Parse tileable configuration attributes from the layout tag
    let mut tileable: Option<bool> = None;
    let mut through_channel: Option<bool> = None;
    let mut shrink_boundary: Option<bool> = None;
    let mut perimeter_cb: Option<bool> = None;
    let mut opin2all_sides: Option<bool> = None;
    let mut concat_wire: Option<bool> = None;
    let mut concat_pass_wire: Option<bool> = None;

    for a in attributes {
        match a.name.to_string().as_ref() {
            "tileable" => {
                tileable = match tileable {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "through_channel" => {
                through_channel = match through_channel {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "shrink_boundary" => {
                shrink_boundary = match shrink_boundary {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "perimeter_cb" => {
                perimeter_cb = match perimeter_cb {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "opin2all_sides" => {
                opin2all_sides = match opin2all_sides {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "concat_wire" => {
                concat_wire = match concat_wire {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
                    Some(_) => {
                        return Err(FPGAArchParseError::DuplicateAttribute(
                            a.to_string(),
                            parser.position(),
                        ));
                    }
                }
            }
            "concat_pass_wire" => {
                concat_pass_wire = match concat_pass_wire {
                    None => Some(parse_bool_attribute(&a.value, parser)?),
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

    // Create tileable config if any tileable attributes were specified
    let tileable_config = if tileable.is_some()
        || through_channel.is_some()
        || shrink_boundary.is_some()
        || perimeter_cb.is_some()
        || opin2all_sides.is_some()
        || concat_wire.is_some()
        || concat_pass_wire.is_some()
    {
        Some(TileableLayoutConfig {
            tileable: tileable.unwrap_or(false),
            through_channel: through_channel.unwrap_or(false),
            shrink_boundary: shrink_boundary.unwrap_or(false),
            perimeter_cb: perimeter_cb.unwrap_or(false),
            opin2all_sides: opin2all_sides.unwrap_or(false),
            concat_wire: concat_wire.unwrap_or(false),
            concat_pass_wire: concat_pass_wire.unwrap_or(false),
        })
    } else {
        None
    };

    let mut layouts: Vec<Layout> = Vec::new();
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                match name.to_string().as_str() {
                    "auto_layout" => {
                        layouts.push(Layout::AutoLayout(parse_auto_layout(
                            &name,
                            &attributes,
                            parser,
                        )?));
                    }
                    "fixed_layout" => {
                        layouts.push(Layout::FixedLayout(parse_fixed_layout(
                            &name,
                            &attributes,
                            parser,
                        )?));
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
                "layout" => break,
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

    Ok(DeviceLayouts {
        layout_list: layouts,
        tileable_config,
    })
}
