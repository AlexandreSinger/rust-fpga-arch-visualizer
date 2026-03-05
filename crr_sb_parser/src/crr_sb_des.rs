
#[derive(Debug, PartialEq)]
pub enum CRRSwitchDir {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct CRRSwitchRowHeaderInfo {
    pub dir: CRRSwitchDir,
    pub segment_type: String,
    pub lane_num: usize,
    pub tap_num: usize,
}

pub struct CRRSwitchColHeaderInfo {
    pub dir: CRRSwitchDir,
    pub segment_type: String,
    pub fan_in: Option<usize>,
    pub lane_num: usize,
}

pub enum CRRSwitchCellData {
    Connection,
    DelaySpecified {
        delay: f32,
    }
}

pub struct CRRSwitchCell {
    pub row_idx: usize,
    pub col_idx: usize,
    pub data: CRRSwitchCellData,
}

pub struct CRRSwitchBlockDeserialized {
    pub col_headers: Vec<CRRSwitchColHeaderInfo>,
    pub row_headers: Vec<CRRSwitchRowHeaderInfo>,
    pub cells: Vec<CRRSwitchCell>,
}