use crate::{CRRSBParseError, crr_sb_des::CRRSwitchDir};

pub fn parse_crr_switch_dir(dir_str: &str) -> Result<CRRSwitchDir, CRRSBParseError> {
    match dir_str {
        "left" => Ok(CRRSwitchDir::Left),
        "right" => Ok(CRRSwitchDir::Right),
        "top" => Ok(CRRSwitchDir::Top),
        "bottom" => Ok(CRRSwitchDir::Bottom),
        _ => Err(CRRSBParseError::SBHeaderCellParseError("Invlaid dir string.".to_string()))
    }
}

pub fn parse_crr_lane_num(lane_num_str: &str) -> Result<usize, CRRSBParseError> {
    match lane_num_str.parse() {
        Ok(v) => Ok(v),
        Err(e) => Err(CRRSBParseError::SBHeaderCellParseError(format!("Invalid lane num string: {e}.")))
    }
}

pub fn parse_crr_tap_num(lane_num_str: &str) -> Result<usize, CRRSBParseError> {
    match lane_num_str.parse() {
        Ok(v) => Ok(v),
        Err(e) => Err(CRRSBParseError::SBHeaderCellParseError(format!("Invalid tap num string: {e}.")))
    }
}

pub fn parse_crr_fan_in(fan_in_str: &str) -> Result<usize, CRRSBParseError> {
    match fan_in_str.parse() {
        Ok(v) => Ok(v),
        Err(e) => Err(CRRSBParseError::SBHeaderCellParseError(format!("Invalid fan-in string: {e}.")))
    }
}