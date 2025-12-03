
pub struct Model {

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
    MemoryAddress,
    MemoryDataIn,
    MemoryWriteEn,
    MemoryDataOut,
    MemoryAddressFirst,
    MemoryDataInFirst,
    MemoryWriteEnFirst,
    MemoryDataOutFirst,
    MemoryAddressSecond,
    MemoryDataInSecond,
    MemoryWriteEnSecond,
    MemoryDataOutSecond,
    // FIXME: These are not documented by VTR. Documentation needs to be updated.
    MemoryReadEnFirst,
    MemoryReadEnSecond,
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

pub struct SubTileFracFC {
    pub val: f32,
}

pub struct SubTileAbsFC {
    pub val: i32,
}

pub enum SubTileIOFC {
    Frac(SubTileFracFC),
    Abs(SubTileAbsFC),
}

pub struct SubTileFC {
    pub in_fc: SubTileIOFC,
    pub out_fc: SubTileIOFC,
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

pub struct Tile {
    pub name: String,
    // FIXME: Documentation. It is not clear from the documentation if tiles should
    //        have ports or not. The ZA archs do have ports, but most do not.
    pub ports: Vec<Port>,
    pub sub_tiles: Vec<SubTile>,
    pub width: i32,
    pub height: i32,
    pub area: Option<f32>,
}

// TODO: pb_type and priority is better served as a trait.
pub struct FillGridLocation {
    pub pb_type: String,
    pub priority: i32,
}

pub struct PerimeterGridLocation {
    pub pb_type: String,
    pub priority: i32,
}

pub struct CornersGridLocation {
    pub pb_type: String,
    pub priority: i32,
}

pub struct SingleGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub x_expr: String,
    pub y_expr: String,
}

pub struct ColGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub repeat_x_expr: Option<String>,
    pub start_y_expr: String,
    pub incr_y_expr: String,
}

pub struct RowGridLocation {
    pub pb_type: String,
    pub priority: i32,
    pub start_x_expr: String,
    pub incr_x_expr: String,
    pub start_y_expr: String,
    pub repeat_y_expr: Option<String>,
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

pub struct Switch {

}

pub enum SegmentAxis {
    X,
    Y,
    XY,
    Z,
}

pub enum SegmentType {
    Bidir,
    Unidir,
}

pub enum SegmentResourceType {
    Gclk,
    General,
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
}

pub struct DirectInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    pub pack_patterns: Vec<PackPattern>,
}

pub struct MuxInterconnect {
    pub name: String,
    pub input: String,
    pub output: String,
    pub pack_patterns: Vec<PackPattern>,
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
}

pub struct FPGAArch {
    pub models: Vec<Model>,
    pub tiles: Vec<Tile>,
    pub layouts: Vec<Layout>,
    pub device: DeviceInfo,
    pub switch_list: Vec<Switch>,
    pub segment_list: Vec<Segment>,
    pub complex_block_list: Vec<PBType>,
}
