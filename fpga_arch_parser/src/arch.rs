pub struct ModelPort {
    pub name: String,
    pub is_clock: bool,
    pub clock: Option<String>,
    pub combinational_sink_ports: Vec<String>,
}

pub struct Model {
    pub name: String,
    pub never_prune: bool,
    pub input_ports: Vec<ModelPort>,
    pub output_ports: Vec<ModelPort>,
}

pub struct Metadata {
    pub name: String,
    pub value: String,
}

pub enum PinEquivalence {
    None,
    Full,
    Instance,
}

pub enum PortClass {
    None,
    LutIn,
    LutOut,
    FlipFlopD,
    FlipFlopQ,
    Clock,
    // FIXME: These are not document well in VTR. Documentation needs to be updated.
    MemoryAddress(i32),
    MemoryDataIn(i32),
    MemoryWriteEn(i32),
    MemoryDataOut(i32),
    MemoryReadEn(i32),
}

pub struct InputPort {
    pub name: String,
    pub num_pins: i32,
    pub equivalent: PinEquivalence,
    pub is_non_clock_global: bool,
    pub port_class: PortClass,
}

pub struct OutputPort {
    pub name: String,
    pub num_pins: i32,
    pub equivalent: PinEquivalence,
    pub port_class: PortClass,
}

pub struct ClockPort {
    pub name: String,
    pub num_pins: i32,
    pub equivalent: PinEquivalence,
    pub port_class: PortClass,
}

pub enum Port {
    Input(InputPort),
    Output(OutputPort),
    Clock(ClockPort),
}

pub enum TileSitePinMapping {
    Direct,
    Custom,
}

pub struct TileSite {
    pub pb_type: String,
    pub pin_mapping: TileSitePinMapping,
}

pub enum SubTileIOFC {
    Frac(f32),
    Abs(i32),
}

pub struct SubTileFCOverride {
    pub fc: SubTileIOFC,
    pub port_name: Option<String>,
    pub segment_name: Option<String>,
}

pub struct SubTileFC {
    pub in_fc: SubTileIOFC,
    pub out_fc: SubTileIOFC,
    pub fc_overrides: Vec<SubTileFCOverride>,
}

pub enum PinSide {
    Left,
    Right,
    Bottom,
    Top,
}

pub struct PinLoc {
    pub side: PinSide,
    pub xoffset: i32,
    pub yoffset: i32,
    pub pin_strings: Vec<String>,
}

pub struct CustomPinLocations {
    pub pin_locations: Vec<PinLoc>,
}

pub enum SubTilePinLocations {
    Spread,
    Perimeter,
    SpreadInputsPerimeterOutputs,
    Custom(CustomPinLocations),
}

pub struct SubTile {
    pub name: String,
    pub capacity: i32,
    pub equivalent_sites: Vec<TileSite>,
    pub ports: Vec<Port>,
    pub fc: SubTileFC,
    pub pin_locations: SubTilePinLocations,
}

pub enum SwitchBlockLocationType {
    Full,
    Straight,
    Turns,
    None,
}

pub struct SwitchBlockLocation {
    pub sb_type: SwitchBlockLocationType,
    pub xoffset: i32,
    pub yoffset: i32,
    pub switch_override: Option<String>,
}

pub enum SwitchBlockLocationsPattern {
    ExternalFullInternalStraight,
    All,
    External,
    Internal,
    None,
    Custom(Vec<SwitchBlockLocation>),
}

pub struct SwitchBlockLocations {
    pub pattern: SwitchBlockLocationsPattern,
    pub internal_switch: Option<String>,
}

pub struct Tile {
    pub name: String,
    // FIXME: Documentation. It is not clear from the documentation if tiles should
    //        have ports or not. The ZA archs do have ports, but most do not.
    pub ports: Vec<Port>,
    pub sub_tiles: Vec<SubTile>,
    pub width: i32,
    pub height: i32,
    pub area: Option<f32>,
    pub switchblock_locations: Option<SwitchBlockLocations>,
}

// TODO: pb_type and priority is better served as a trait.
pub struct FillGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct PerimeterGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct CornersGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct SingleGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub x_expr: String,
    pub y_expr: String,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct ColGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub repeat_x_expr: Option<String>,
    pub start_y_expr: String,
    pub incr_y_expr: String,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct RowGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub incr_x_expr: String,
    pub start_y_expr: String,
    pub repeat_y_expr: Option<String>,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct RegionGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub end_x_expr: String,
    pub repeat_x_expr: Option<String>,
    pub incr_x_expr: String,
    pub start_y_expr: String,
    pub end_y_expr: String,
    pub repeat_y_expr: Option<String>,
    pub incr_y_expr: String,
    pub metadata: Option<Vec<Metadata>>,
}

pub enum GridLocation {
    Fill(FillGridLocation),
    Perimeter(PerimeterGridLocation),
    Corners(CornersGridLocation),
    Single(SingleGridLocation),
    Col(ColGridLocation),
    Row(RowGridLocation),
    Region(RegionGridLocation),
}

pub struct AutoLayout {
    pub aspect_ratio: f32,
    pub grid_locations: Vec<GridLocation>,
}

pub struct FixedLayout {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub grid_locations: Vec<GridLocation>,
}

pub enum Layout {
    AutoLayout(AutoLayout),
    FixedLayout(FixedLayout),
}

pub enum CustomSwitchBlockType {
    Unidir,
    Bidir,
}

pub enum CustomSwitchBlockLocation {
    Everywhere,
    Perimeter,
    Corner,
    Fringe,
    Core,
    // FIXME: This is undocumented!
    XYSpecified { x: i32, y: i32 },
}

pub enum CustomSwitchFuncType {
    LeftToTop,
    LeftToRight,
    LeftToBottom,
    TopToRight,
    TopToBottom,
    TopToLeft,
    RightToBottom,
    RightToLeft,
    RightToTop,
    BottomToLeft,
    BottomToTop,
    BottomToRight,
}

pub struct CustomSwitchFunc {
    pub func_type: CustomSwitchFuncType,
    pub formula: String,
}

pub struct CustomSwitchBlockConnPoint {
    pub segment_type: String,
    pub switchpoint: Vec<i32>,
}

pub enum CustomSwitchBlockWireConnOrder {
    Shuffled,
    Fixed,
}

pub struct CustomSwitchWireConn {
    pub num_conns: String,
    pub from_points: Vec<CustomSwitchBlockConnPoint>,
    pub to_points: Vec<CustomSwitchBlockConnPoint>,
    pub from_order: CustomSwitchBlockWireConnOrder,
    pub to_order: CustomSwitchBlockWireConnOrder,
    pub switch_override: Option<String>,
}

pub struct CustomSwitchBlock {
    pub name: String,
    pub sb_type: CustomSwitchBlockType,
    pub switchblock_location: CustomSwitchBlockLocation,
    pub switch_funcs: Vec<CustomSwitchFunc>,
    pub wireconns: Vec<CustomSwitchWireConn>,
}

#[derive(Debug)]
pub enum SBType {
    Wilton,
    Subset,
    Universal,
    Custom,
}

pub struct GaussianChanWDist {
    pub peak: f32,
    pub width: f32,
    pub xpeak: f32,
    pub dc: f32,
}

pub struct UniformChanWDist {
    pub peak: f32,
}

pub struct PulseChanWDist {
    pub peak: f32,
    pub width: f32,
    pub xpeak: f32,
    pub dc: f32,
}

pub struct DeltaChanWDist {
    pub peak: f32,
    pub xpeak: f32,
    pub dc: f32,
}

pub enum ChanWDist {
    Gaussian(GaussianChanWDist),
    Uniform(UniformChanWDist),
    Pulse(PulseChanWDist),
    Delta(DeltaChanWDist),
}

pub struct DeviceSizingInfo {
    pub r_min_w_nmos: f32,
    pub r_min_w_pmos: f32,
}

pub struct DeviceConnectionBlockInfo {
    pub input_switch_name: String,
}

pub struct DeviceAreaInfo {
    pub grid_logic_tile_area: f32,
}

pub struct DeviceSwitchBlockInfo {
    pub sb_type: SBType,
    //      NOTE: SB fs is required if the sb type is non-custom.
    pub sb_fs: Option<i32>,
}

pub struct DeviceChanWidthDistrInfo {
    pub x_distr: ChanWDist,
    pub y_distr: ChanWDist,
}

pub struct DeviceInfo {
    pub sizing: DeviceSizingInfo,
    pub connection_block: DeviceConnectionBlockInfo,
    pub area: DeviceAreaInfo,
    pub switch_block: DeviceSwitchBlockInfo,
    pub chan_width_distr: DeviceChanWidthDistrInfo,
    // TODO: default_fc
}

#[derive(Debug)]
pub enum SwitchType {
    Mux,
    Tristate,
    PassGate,
    Short,
    Buffer,
}

pub enum SwitchBufSize {
    Auto,
    Val(f32),
}

pub struct SwitchTDel {
    pub num_inputs: i32,
    pub delay: f32,
}

pub struct Switch {
    pub sw_type: SwitchType,
    pub name: String,
    pub resistance: f32,
    pub c_in: f32,
    pub c_out: f32,
    pub c_internal: Option<f32>,
    pub t_del: Option<f32>,
    pub buf_size: SwitchBufSize,
    pub mux_trans_size: Option<f32>,
    pub power_buf_size: Option<i32>,
    pub t_del_tags: Vec<SwitchTDel>,
}

#[derive(Debug)]
pub enum SegmentAxis {
    X,
    Y,
    XY,
    Z,
}

#[derive(Debug)]
pub enum SegmentType {
    Bidir,
    Unidir,
}

pub enum SegmentResourceType {
    Gclk,
    General,
}

pub enum SegmentSwitchPoints {
    Unidir {
        mux_inc: String,
        mux_dec: String,
    },
    Bidir {
        wire_switch: String,
        opin_switch: String,
    },
}

pub struct Segment {
    pub axis: SegmentAxis,
    pub name: String,
    pub length: i32,
    pub segment_type: SegmentType,
    pub res_type: SegmentResourceType,
    pub freq: f32,
    pub r_metal: f32,
    pub c_metal: f32,
    pub sb_pattern: Vec<bool>,
    pub cb_pattern: Vec<bool>,
    pub switch_points: SegmentSwitchPoints,
}

pub struct GlobalDirect {
    pub name: String,
    pub from_pin: String,
    pub to_pin: String,
    pub x_offset: i32,
    pub y_offset: i32,
    pub z_offset: i32,
    pub switch_name: Option<String>,
    pub from_side: Option<PinSide>,
    pub to_side: Option<PinSide>,
}

pub enum DelayType {
    Max,
    Min,
}

pub enum DelayInfo {
    Constant {
        min: f32,
        max: f32,
        in_port: String,
        out_port: String,
    },
    Matrix {
        delay_type: DelayType,
        // This matrix is [in_pin_idx][out_pin_idx]
        // TODO: The documentation should be more clear on this.
        matrix: Vec<Vec<f32>>,
        in_port: String,
        out_port: String,
    },
}

pub enum TimingConstraintType {
    Hold,
    Setup,
    ClockToQ,
}

pub struct TimingConstraintInfo {
    pub constraint_type: TimingConstraintType,
    // NOTE: Only ClockToQ can have two different min/max values right now.
    //       A bit of future proofing and cleanup to split max and min here.
    pub min_value: f32,
    pub max_value: f32,
    pub port: String,
    pub clock: String,
}

pub struct PackPattern {
    pub name: String,
    pub in_port: String,
    pub out_port: String,
}

pub struct CompleteInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    // FIXME: The documentation needs to be updated. The documentation says there
    //        may be a single pack pattern; however, an interconnect may have many
    //        pack patterns.
    pub pack_patterns: Vec<PackPattern>,
    pub delays: Vec<DelayInfo>,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct DirectInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    pub pack_patterns: Vec<PackPattern>,
    pub delays: Vec<DelayInfo>,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct MuxInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    pub pack_patterns: Vec<PackPattern>,
    pub delays: Vec<DelayInfo>,
    pub metadata: Option<Vec<Metadata>>,
}

pub enum Interconnect {
    Complete(CompleteInterconnect),
    Direct(DirectInterconnect),
    Mux(MuxInterconnect),
}

pub struct PBMode {
    pub name: String,
    pub pb_types: Vec<PBType>,
    pub interconnects: Vec<Interconnect>,
    pub metadata: Option<Vec<Metadata>>,
}

pub enum PBTypeClass {
    None,
    Lut,
    FlipFlop,
    Memory,
}

pub struct PBType {
    pub name: String,
    pub num_pb: i32,
    pub blif_model: Option<String>,
    pub class: PBTypeClass,
    pub ports: Vec<Port>,
    pub modes: Vec<PBMode>,
    pub pb_types: Vec<PBType>,
    pub interconnects: Vec<Interconnect>,
    pub delays: Vec<DelayInfo>,
    pub timing_constraints: Vec<TimingConstraintInfo>,
    pub metadata: Option<Vec<Metadata>>,
}

pub struct FPGAArch {
    pub models: Vec<Model>,
    pub tiles: Vec<Tile>,
    pub layouts: Vec<Layout>,
    pub device: DeviceInfo,
    pub switch_list: Vec<Switch>,
    pub segment_list: Vec<Segment>,
    pub custom_switch_blocks: Vec<CustomSwitchBlock>,
    pub direct_list: Vec<GlobalDirect>,
    pub complex_block_list: Vec<PBType>,
}
