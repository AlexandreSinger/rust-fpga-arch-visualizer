use std::fs::File;
use std::io::BufReader;

use xml::common::Position;
use xml::reader::{EventReader, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;

use crate::parse_error::*;
use crate::arch::*;

fn parse_device_sizing(name: &OwnedName,
                       attributes: &[OwnedAttribute],
                       parser: &mut EventReader<BufReader<File>>) -> Result<DeviceSizingInfo, FPGAArchParseError> {
    assert!(name.to_string() == "sizing");

    let mut r_min_w_nmos: Option<f32> = None;
    let mut r_min_w_pmos: Option<f32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "R_minW_nmos" => {
                r_min_w_nmos = match r_min_w_nmos {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "R_minW_pmos" => {
                r_min_w_pmos = match r_min_w_pmos {
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
    let r_min_w_nmos = match r_min_w_nmos {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("R_minW_nmos".to_string(), parser.position())),
    };
    let r_min_w_pmos = match r_min_w_pmos {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("R_minW_pmos".to_string(), parser.position())),
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
        }
    };

    Ok(DeviceSizingInfo {
        r_min_w_nmos,
        r_min_w_pmos,
    })
}

fn parse_device_connection_block(name: &OwnedName,
                                 attributes: &[OwnedAttribute],
                                 parser: &mut EventReader<BufReader<File>>) -> Result<DeviceConnectionBlockInfo, FPGAArchParseError> {
    assert!(name.to_string() == "connection_block");

    let mut input_switch_name: Option<String> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "input_switch_name" => {
                input_switch_name = match input_switch_name {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            _ => return Err(FPGAArchParseError::UnknownAttribute(a.to_string(), parser.position())),
        };
    }
    let input_switch_name = match input_switch_name {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("input_switch_name".to_string(), parser.position())),
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
        }
    };

    Ok(DeviceConnectionBlockInfo {
        input_switch_name,
    })
}

fn parse_device_area(name: &OwnedName,
                     attributes: &[OwnedAttribute],
                     parser: &mut EventReader<BufReader<File>>) -> Result<DeviceAreaInfo, FPGAArchParseError> {
    assert!(name.to_string() == "area");

    let mut grid_logic_tile_area: Option<f32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "grid_logic_tile_area" => {
                grid_logic_tile_area = match grid_logic_tile_area {
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
    let grid_logic_tile_area = match grid_logic_tile_area {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("grid_logic_tile_area".to_string(), parser.position())),
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
        }
    };

    Ok(DeviceAreaInfo {
        grid_logic_tile_area,
    })
}

fn parse_device_switch_block(name: &OwnedName,
                             attributes: &[OwnedAttribute],
                             parser: &mut EventReader<BufReader<File>>) -> Result<DeviceSwitchBlockInfo, FPGAArchParseError> {
    assert!(name.to_string() == "switch_block");

    let mut sb_type: Option<SBType> = None;
    let mut sb_fs: Option<i32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "type" => {
                sb_type = match sb_type {
                    None => match a.value.as_ref() {
                        "wilton" => Some(SBType::Wilton),
                        "subset" => Some(SBType::Subset),
                        "universal" => Some(SBType::Universal),
                        "custom" => Some(SBType::Custom),
                        _ => return Err(FPGAArchParseError::AttributeParseError(format!("Unknown switch_block type: {}", a.value), parser.position())),

                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "fs" => {
                sb_fs = match sb_fs {
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
    let sb_type = match sb_type {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("type".to_string(), parser.position())),
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
        }
    };

    Ok(DeviceSwitchBlockInfo {
        sb_type,
        sb_fs,
    })
}

fn parse_chan_w_dist(name: &OwnedName,
                     attributes: &[OwnedAttribute],
                     parser: &mut EventReader<BufReader<File>>) -> Result<ChanWDist, FPGAArchParseError> {
    assert!(name.to_string() == "x" || name.to_string() == "y");

    let mut distr: Option<String> = None;
    let mut peak: Option<f32> = None;
    let mut width: Option<f32> = None;
    let mut xpeak: Option<f32> = None;
    let mut dc: Option<f32> = None;
    for a in attributes {
        match a.name.to_string().as_ref() {
            "distr" => {
                distr = match distr {
                    None => Some(a.value.clone()),
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "peak" => {
                peak = match peak {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "width" => {
                width = match width {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "xpeak" => {
                xpeak = match xpeak {
                    None => match a.value.parse() {
                        Ok(v) => Some(v),
                        Err(e) => return Err(FPGAArchParseError::AttributeParseError(format!("{a}: {e}"), parser.position())),
                    },
                    Some(_) => return Err(FPGAArchParseError::DuplicateAttribute(a.to_string(), parser.position())),
                }
            },
            "dc" => {
                dc = match dc {
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

    let distr = match distr {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("distr".to_string(), parser.position())),
    };
    let peak = match peak {
        Some(n) => n,
        None => return Err(FPGAArchParseError::MissingRequiredAttribute("peak".to_string(), parser.position())),
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
        }
    };

    match distr.as_str() {
        "gaussian" => {
            let width = match width {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("width".to_string(), parser.position())),
            };
            let xpeak = match xpeak {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("xpeak".to_string(), parser.position())),
            };
            let dc = match dc {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("dc".to_string(), parser.position())),
            };
            Ok(ChanWDist::Gaussian(GaussianChanWDist {
                peak,
                width,
                xpeak,
                dc,
            }))
        },
        "uniform" => {
            Ok(ChanWDist::Uniform(UniformChanWDist {
                peak,
            }))
        },
        "pulse" => {
            let width = match width {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("width".to_string(), parser.position())),
            };
            let xpeak = match xpeak {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("xpeak".to_string(), parser.position())),
            };
            let dc = match dc {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("dc".to_string(), parser.position())),
            };
            Ok(ChanWDist::Pulse(PulseChanWDist {
                peak,
                width,
                xpeak,
                dc,
            }))
        },
        "delta" => {
            let xpeak = match xpeak {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("xpeak".to_string(), parser.position())),
            };
            let dc = match dc {
                Some(n) => n,
                None => return Err(FPGAArchParseError::MissingRequiredAttribute("dc".to_string(), parser.position())),
            };
            Ok(ChanWDist::Delta(DeltaChanWDist {
                peak,
                xpeak,
                dc,
            }))
        },
        _ => Err(FPGAArchParseError::AttributeParseError(format!("Unknown distr for chan_w_distr: {distr}"), parser.position())),
    }
}

fn parse_device_chan_w_distr(name: &OwnedName,
                             attributes: &[OwnedAttribute],
                             parser: &mut EventReader<BufReader<File>>) -> Result<DeviceChanWidthDistrInfo, FPGAArchParseError> {
    assert!(name.to_string() == "chan_width_distr");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut x_distr: Option<ChanWDist> = None;
    let mut y_distr: Option<ChanWDist> = None;
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "x" => {
                        x_distr = match x_distr {
                            None => Some(parse_chan_w_dist(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "y" => {
                        y_distr = match y_distr {
                            None => Some(parse_chan_w_dist(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "chan_width_distr" => break,
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

    let x_distr = match x_distr {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<x>".to_string())),
    };
    let y_distr = match y_distr {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<y>".to_string())),
    };

    Ok(DeviceChanWidthDistrInfo {
        x_distr,
        y_distr,
    })
}

pub fn parse_device(name: &OwnedName,
                attributes: &[OwnedAttribute],
                parser: &mut EventReader<BufReader<File>>) -> Result<DeviceInfo, FPGAArchParseError> {
    assert!(name.to_string() == "device");
    if !attributes.is_empty() {
        return Err(FPGAArchParseError::UnknownAttribute(String::from("Expected to be empty"), parser.position()));
    }

    let mut sizing: Option<DeviceSizingInfo> = None;
    let mut connection_block: Option<DeviceConnectionBlockInfo> = None;
    let mut area: Option<DeviceAreaInfo> = None;
    let mut switch_block: Option<DeviceSwitchBlockInfo> = None;
    let mut chan_width_distr: Option<DeviceChanWidthDistrInfo> = None;

    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.to_string().as_str() {
                    "sizing" => {
                        sizing = match sizing {
                            None => Some(parse_device_sizing(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "connection_block" => {
                        connection_block = match connection_block {
                            None => Some(parse_device_connection_block(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "area" => {
                        area = match area {
                            None => Some(parse_device_area(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "switch_block" => {
                        switch_block = match switch_block {
                            None => Some(parse_device_switch_block(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    "chan_width_distr" => {
                        chan_width_distr = match chan_width_distr {
                            None => Some(parse_device_chan_w_distr(&name, &attributes, parser)?),
                            Some(_) => return Err(FPGAArchParseError::DuplicateTag(name.to_string(), parser.position())),
                        }
                    },
                    _ => return Err(FPGAArchParseError::InvalidTag(name.to_string(), parser.position())),
                };
            },
            Ok(XmlEvent::EndElement { name }) => {
                match name.to_string().as_str() {
                    "device" => break,
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

    let sizing = match sizing {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<sizing>".to_string())),
    };
    let area = match area {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<area>".to_string())),
    };
    let connection_block = match connection_block {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<connection_block>".to_string())),
    };
    let switch_block = match switch_block {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<switch_block>".to_string())),
    };
    let chan_width_distr = match chan_width_distr {
        Some(t) => t,
        None => return Err(FPGAArchParseError::MissingRequiredTag("<chan_width_distr>".to_string())),
    };

    Ok(DeviceInfo {
        sizing,
        area,
        connection_block,
        switch_block,
        chan_width_distr,
    })
}

