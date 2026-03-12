use crate::{CRRSBParseError, crr_sb_des::CRRSwitchDir};

pub fn parse_crr_switch_dir(dir_str: &str) -> Result<CRRSwitchDir, CRRSBParseError> {
    match dir_str.to_lowercase().as_str() {
        "left" => Ok(CRRSwitchDir::Left),
        "right" => Ok(CRRSwitchDir::Right),
        "top" => Ok(CRRSwitchDir::Top),
        "bottom" => Ok(CRRSwitchDir::Bottom),
        "ipin" => Ok(CRRSwitchDir::IPIN),
        "opin" => Ok(CRRSwitchDir::OPIN),
        _ => Err(CRRSBParseError::SBHeaderCellParseError(
            format!("Invalid dir string: {dir_str}."),
        )),
    }
}

fn parse_usize(value: &str, field_name: &str) -> Result<usize, CRRSBParseError> {
    value.parse().map_err(|e| {
        CRRSBParseError::SBHeaderCellParseError(format!("Invalid {field_name} string ({value}): {e}."))
    })
}

pub fn parse_crr_lane_num(lane_num_str: &str) -> Result<usize, CRRSBParseError> {
    parse_usize(lane_num_str, "lane num")
}

pub fn parse_crr_tap_num(tap_num_str: &str) -> Result<usize, CRRSBParseError> {
    parse_usize(tap_num_str, "tap num")
}

pub fn parse_crr_fan_in(fan_in_str: &str) -> Result<usize, CRRSBParseError> {
    parse_usize(fan_in_str, "fan-in")
}
