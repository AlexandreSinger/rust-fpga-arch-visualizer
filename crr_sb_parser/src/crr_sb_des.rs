#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CRRSwitchDir {
    Left,
    Right,
    Top,
    Bottom,
    IPIN,
    OPIN,
}

#[derive(Debug, PartialEq)]
pub enum CRRSwitchSourcePin {
    Tap { tap_num: usize },
    Pin { pin_name: String },
}

#[derive(Debug)]
pub struct CRRSwitchSourceNodeInfo {
    pub dir: CRRSwitchDir,
    pub segment_type: String,
    pub lane_num: usize,
    pub source_pin: CRRSwitchSourcePin,
}

#[derive(Debug)]
pub struct CRRSwitchSinkNodeInfo {
    pub dir: CRRSwitchDir,
    pub segment_type: String,
    pub fan_in: Option<usize>,
    pub lane_num: usize,
    pub target_pin: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum CRRSwitchConnectionDelay {
    Undefined,
    DelaySpecified { delay: f32 },
}

#[derive(Debug)]
pub struct CRRSwitchConnection {
    pub source_node_id: usize,
    pub sink_node_id: usize,
    pub delay: CRRSwitchConnectionDelay,
}

#[derive(Debug)]
pub struct CRRSwitchBlockDeserialized {
    pub sink_nodes: Vec<CRRSwitchSinkNodeInfo>,
    pub source_nodes: Vec<CRRSwitchSourceNodeInfo>,
    pub edges: Vec<CRRSwitchConnection>,
}
