#[derive(Debug, PartialEq)]
pub enum CRRSwitchDir {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug)]
pub struct CRRSwitchSourceNodeInfo {
    pub dir: CRRSwitchDir,
    pub segment_type: String,
    pub lane_num: usize,
    pub tap_num: usize,
}

#[derive(Debug)]
pub struct CRRSwitchSinkNodeInfo {
    pub dir: CRRSwitchDir,
    pub segment_type: String,
    pub fan_in: Option<usize>,
    pub lane_num: usize,
}

#[derive(Debug)]
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
