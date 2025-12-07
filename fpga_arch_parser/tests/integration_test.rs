
use std::path::{PathBuf, absolute};

use fpga_arch_parser::{
    FPGAArchParseError,
    Layout,
    GridLocation,
    Port,
    SubTileIOFC,
    TileSitePinMapping,
    SubTilePinLocations,
    SBType,
    ChanWDist,
    SegmentType,
    SwitchType,
    SwitchBufSize,
};

#[test]
fn test_k4_n4_90nm_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k4_N4_90nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check models.
    assert_eq!(res.models.len(), 0);

    // Check tiles.
    assert_eq!(res.tiles.len(), 2);
    assert_eq!(res.tiles[0].name,"io");
    assert_eq!(res.tiles[1].name,"clb");
    assert_eq!(res.tiles[0].sub_tiles.len(), 1);
    assert_eq!(res.tiles[0].sub_tiles[0].name, "io");
    assert_eq!(res.tiles[0].sub_tiles[0].capacity, 3);
    assert_eq!(res.tiles[0].sub_tiles[0].equivalent_sites.len(), 1);
    assert_eq!(res.tiles[0].sub_tiles[0].equivalent_sites[0].pb_type, "io");
    assert!(matches!(res.tiles[0].sub_tiles[0].equivalent_sites[0].pin_mapping, TileSitePinMapping::Direct));
    assert_eq!(res.tiles[0].sub_tiles[0].ports.len(), 3);
    assert!(matches!(res.tiles[0].sub_tiles[0].ports[0], Port::Input { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].ports[1], Port::Output { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].ports[2], Port::Clock { .. }));
    // TODO: Add stronger tests for ports.
    assert!(matches!(res.tiles[0].sub_tiles[0].fc.in_fc, SubTileIOFC::Frac { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].fc.out_fc, SubTileIOFC::Frac { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].pin_locations, SubTilePinLocations::Custom { .. }));
    assert_eq!(res.tiles[1].sub_tiles.len(), 1);
    assert_eq!(res.tiles[1].sub_tiles[0].name, "clb");
    assert_eq!(res.tiles[1].sub_tiles[0].capacity, 1);
    assert_eq!(res.tiles[1].sub_tiles[0].equivalent_sites.len(), 1);
    assert_eq!(res.tiles[1].sub_tiles[0].equivalent_sites[0].pb_type, "clb");
    assert!(matches!(res.tiles[1].sub_tiles[0].equivalent_sites[0].pin_mapping, TileSitePinMapping::Direct));
    assert!(matches!(res.tiles[1].sub_tiles[0].pin_locations, SubTilePinLocations::Spread));

    // Check layouts.
    assert_eq!(res.layouts.len(), 1);
    assert!(matches!(res.layouts[0], Layout::AutoLayout { .. }));
    match &res.layouts[0] {
        Layout::AutoLayout( auto_layout ) => {
            assert_eq!(auto_layout.aspect_ratio, 1.0);
            assert_eq!(auto_layout.grid_locations.len(), 3);
            assert!(matches!(auto_layout.grid_locations[0], GridLocation::Perimeter { .. }));
            assert!(matches!(auto_layout.grid_locations[1], GridLocation::Corners { .. }));
            assert!(matches!(auto_layout.grid_locations[2], GridLocation::Fill { .. }));
            // TODO: Check the priority and the pb_types are correct.
        },
        _ => panic!("Should never hit this.")
    }

    // Check device.
    assert_eq!(res.device.sizing.r_min_w_nmos, 4_220.93);
    assert_eq!(res.device.sizing.r_min_w_pmos, 11_207.6);
    assert!(matches!(res.device.chan_width_distr.x_distr, ChanWDist::Uniform { .. }));
    assert!(matches!(res.device.chan_width_distr.y_distr, ChanWDist::Uniform { .. }));
    assert!(matches!(res.device.switch_block.sb_type, SBType::Wilton));
    assert_eq!(res.device.switch_block.sb_fs, Some(3));
    assert_eq!(res.device.connection_block.input_switch_name, "ipin_cblock");

    // Check switch list.
    assert_eq!(res.switch_list.len(), 2);
    let switch1 = &res.switch_list[0];
    assert!(matches!(switch1.sw_type, SwitchType::Mux));
    assert_eq!(switch1.name, "0");
    assert_eq!(switch1.resistance, 0.0);
    assert_eq!(switch1.c_in, 0.0);
    assert_eq!(switch1.c_out, 0.0);
    assert_eq!(switch1.t_del, Some(6.244e-11));
    assert_eq!(switch1.mux_trans_size, Some(1.835460));
    match switch1.buf_size {
        SwitchBufSize::Val(v) => assert_eq!(v, 10.498600),
        SwitchBufSize::Auto => panic!("switch1 buf size expected to be Val"),
    };
    let switch2 = &res.switch_list[1];
    assert!(matches!(switch2.sw_type, SwitchType::Mux));
    assert_eq!(switch2.name, "ipin_cblock");
    assert_eq!(switch2.resistance, 1055.232544);
    assert_eq!(switch2.c_in, 0.0);
    assert_eq!(switch2.c_out, 0.0);
    assert_eq!(switch2.t_del, Some(8.045e-11));
    assert_eq!(switch2.mux_trans_size, Some(0.983352));
    assert!(matches!(switch2.buf_size, SwitchBufSize::Auto));

    // Check segment list
    assert_eq!(res.segment_list.len(), 1);
    assert_eq!(res.segment_list[0].freq, 1.0);
    assert_eq!(res.segment_list[0].length, 1);
    assert!(matches!(res.segment_list[0].segment_type, SegmentType::Unidir));
    assert_eq!(res.segment_list[0].r_metal, 0.0);
    assert_eq!(res.segment_list[0].c_metal, 0.0);

    // Check complex block list.
    assert_eq!(res.complex_block_list.len(), 2);
    let clb0 = &res.complex_block_list[0];
    assert_eq!(clb0.name, "io");
    assert_eq!(clb0.num_pb, 1);
    assert!(clb0.blif_model.is_none());
    assert_eq!(clb0.ports.len(), 3);
    assert!(matches!(clb0.ports[0], Port::Input { .. }));
    assert!(matches!(clb0.ports[1], Port::Output { .. }));
    assert!(matches!(clb0.ports[2], Port::Clock { .. }));
    // TODO: Add stronger tests for pb_type ports.
    assert!(clb0.pb_types.is_empty());
    assert_eq!(clb0.modes.len(), 2);
    let clb0_mode0 = &clb0.modes[0];
    assert_eq!(clb0_mode0.name, "inpad");
    assert_eq!(clb0_mode0.pb_types.len(), 1);
    assert_eq!(clb0_mode0.pb_types[0].name, "inpad");
    assert!(clb0_mode0.pb_types[0].blif_model.is_some());
    assert_eq!(clb0_mode0.interconnects.len(), 1);
    assert_eq!(clb0.modes[1].name, "outpad");
    assert_eq!(clb0.modes[1].pb_types.len(), 1);
    // TODO: Make these pb_type heirarchy checks more robust.
    assert_eq!(res.complex_block_list[1].name, "clb");

    // TODO: Collect stats on the architecture and ensure they match what is
    //       expected.

    Ok(())
}

#[test]
fn test_k4_n8_legacy_45nm_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k4_N8_legacy_45nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let _res = fpga_arch_parser::parse(&input_xml)?;

    Ok(())
}

#[test]
fn test_k6_n10_40nm_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6_N10_40nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let _res = fpga_arch_parser::parse(&input_xml)?;

    Ok(())
}

#[test]
fn test_k6_n10_sparse_crossbar_40nm_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6_N10_sparse_crossbar_40nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let _res = fpga_arch_parser::parse(&input_xml)?;

    Ok(())
}

#[test]
fn test_k6_frac_n10_40nm_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6_frac_N10_40nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let _res = fpga_arch_parser::parse(&input_xml)?;

    Ok(())
}

#[test]
fn test_vtr_flagship_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6_frac_N10_frac_chain_mem32K_40nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check models.
    assert_eq!(res.models.len(), 4);
    let multiply_model = &res.models[0];
    assert_eq!(multiply_model.name, "multiply");
    assert_eq!(multiply_model.input_ports.len(), 2);
    let multiply_model_port_a = &multiply_model.input_ports[0];
    assert_eq!(multiply_model_port_a.name, "a");
    assert_eq!(multiply_model_port_a.combinational_sink_ports.len(), 1);
    assert_eq!(multiply_model_port_a.combinational_sink_ports[0], "out");
    let multiply_model_port_b = &multiply_model.input_ports[1];
    assert_eq!(multiply_model_port_b.name, "b");
    assert_eq!(multiply_model_port_b.combinational_sink_ports.len(), 1);
    assert_eq!(multiply_model_port_b.combinational_sink_ports[0], "out");
    assert_eq!(multiply_model.output_ports.len(), 1);
    let multiply_model_port_out = &multiply_model.output_ports[0];
    assert_eq!(multiply_model_port_out.name, "out");

    // Check tiles
    let tiles = &res.tiles;
    assert_eq!(tiles.len(), 4);
    let tile1 = &res.tiles[0];
    assert_eq!(tile1.name, "io");
    assert_eq!(tile1.width, 1);
    assert_eq!(tile1.height, 1);
    assert_eq!(tile1.area, Some(0.0));
    let tile2 = &res.tiles[1];
    assert_eq!(tile2.name, "clb");
    assert_eq!(tile2.width, 1);
    assert_eq!(tile2.height, 1);
    assert_eq!(tile2.area, Some(53_894.0));
    let tile3 = &res.tiles[2];
    assert_eq!(tile3.name, "mult_36");
    assert_eq!(tile3.width, 1);
    assert_eq!(tile3.height, 4);
    assert_eq!(tile3.area, Some(396_000.0));
    let tile4 = &res.tiles[3];
    assert_eq!(tile4.name, "memory");
    assert_eq!(tile4.width, 1);
    assert_eq!(tile4.height, 6);
    assert_eq!(tile4.area, Some(548_000.0));

    let io_tile = &tiles[0];
    assert_eq!(io_tile.name, "io");
    assert_eq!(io_tile.sub_tiles.len(), 1);
    let io_subtile = &io_tile.sub_tiles[0];
    assert_eq!(io_subtile.name, "io");
    assert_eq!(io_subtile.capacity, 8);

    // TODO: Add stronger tests for the tiles.
    Ok(())
}

#[test]
fn test_stratix_iv_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/stratixiv_arch.timing.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check tiles
    assert_eq!(res.tiles.len(), 6);

    // Check layouts
    assert_eq!(res.layouts.len(), 7);

    // Check segments
    assert_eq!(res.segment_list.len(), 2);
    let seg1 = &res.segment_list[0];
    assert_eq!(seg1.name, "L4");
    assert_eq!(seg1.freq, 260.0);
    assert_eq!(seg1.length, 4);
    assert!(matches!(seg1.segment_type, SegmentType::Unidir));
    assert_eq!(seg1.r_metal, 201.7);
    assert_eq!(seg1.c_metal, 18.0e-15);
    let seg2 = &res.segment_list[1];
    assert_eq!(seg2.name, "L16");
    assert_eq!(seg2.freq, 40.0);
    assert_eq!(seg2.length, 16);
    assert!(matches!(seg2.segment_type, SegmentType::Unidir));
    assert_eq!(seg2.r_metal, 50.42);
    assert_eq!(seg2.c_metal, 20.7e-15);

    // Check complex blocks.
    assert_eq!(res.complex_block_list.len(), 6);

    Ok(())
}

#[test]
fn test_z1000() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/z1000.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check tiles.
    assert_eq!(res.tiles.len(), 6);

    Ok(())
}

#[test]
fn test_z1010() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/z1010.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check tiles.
    assert_eq!(res.tiles.len(), 8);

    Ok(())
}

#[test]
fn test_z1060() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/z1060.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check tiles.
    assert_eq!(res.tiles.len(), 8);

    Ok(())
}
