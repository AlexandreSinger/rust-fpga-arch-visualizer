use std::path::{PathBuf, absolute};

use fpga_arch_parser::{
    ChanWDist, CustomSwitchBlockLocation, CustomSwitchBlockType, FPGAArchParseError, GridLocation,
    Layout, PBTypeClass, Port, SBType, SegmentType, SubTileIOFC, SubTilePinLocations,
    SwitchBlockLocationType, SwitchBlockLocationsPattern, SwitchBufSize, SwitchType,
    TileSitePinMapping,
};

#[test]
#[allow(clippy::excessive_precision)]
fn test_k4_n4_90nm_parse() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k4_N4_90nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check models.
    // Should have the 4 built-in models.
    assert_eq!(res.models.len(), 4);
    assert_eq!(res.models[0].name, ".input");
    assert_eq!(res.models[1].name, ".output");
    assert_eq!(res.models[2].name, ".latch");
    assert_eq!(res.models[3].name, ".names");

    // Check tiles.
    assert_eq!(res.tiles.len(), 2);
    assert_eq!(res.tiles[0].name, "io");
    assert_eq!(res.tiles[1].name, "clb");
    assert_eq!(res.tiles[0].sub_tiles.len(), 1);
    assert_eq!(res.tiles[0].sub_tiles[0].name, "io");
    assert_eq!(res.tiles[0].sub_tiles[0].capacity, 3);
    assert_eq!(res.tiles[0].sub_tiles[0].equivalent_sites.len(), 1);
    assert_eq!(res.tiles[0].sub_tiles[0].equivalent_sites[0].pb_type, "io");
    assert!(matches!(
        res.tiles[0].sub_tiles[0].equivalent_sites[0].pin_mapping,
        TileSitePinMapping::Direct
    ));
    assert_eq!(res.tiles[0].sub_tiles[0].ports.len(), 3);
    assert!(matches!(
        res.tiles[0].sub_tiles[0].ports[0],
        Port::Input { .. }
    ));
    assert!(matches!(
        res.tiles[0].sub_tiles[0].ports[1],
        Port::Output { .. }
    ));
    assert!(matches!(
        res.tiles[0].sub_tiles[0].ports[2],
        Port::Clock { .. }
    ));
    // TODO: Add stronger tests for ports.
    assert!(matches!(
        res.tiles[0].sub_tiles[0].fc.in_fc,
        SubTileIOFC::Frac { .. }
    ));
    assert!(matches!(
        res.tiles[0].sub_tiles[0].fc.out_fc,
        SubTileIOFC::Frac { .. }
    ));
    assert!(matches!(
        res.tiles[0].sub_tiles[0].pin_locations,
        SubTilePinLocations::Custom { .. }
    ));
    assert_eq!(res.tiles[1].sub_tiles.len(), 1);
    assert_eq!(res.tiles[1].sub_tiles[0].name, "clb");
    assert_eq!(res.tiles[1].sub_tiles[0].capacity, 1);
    assert_eq!(res.tiles[1].sub_tiles[0].equivalent_sites.len(), 1);
    assert_eq!(res.tiles[1].sub_tiles[0].equivalent_sites[0].pb_type, "clb");
    assert!(matches!(
        res.tiles[1].sub_tiles[0].equivalent_sites[0].pin_mapping,
        TileSitePinMapping::Direct
    ));
    assert!(matches!(
        res.tiles[1].sub_tiles[0].pin_locations,
        SubTilePinLocations::Spread
    ));

    // Check layouts.
    assert_eq!(res.layouts.layout_list.len(), 1);
    assert!(matches!(res.layouts.layout_list[0], Layout::AutoLayout(_)));
    match &res.layouts.layout_list[0] {
        Layout::AutoLayout(auto_layout) => {
            assert_eq!(auto_layout.aspect_ratio, 1.0);
            assert_eq!(auto_layout.layers.len(), 1);
            assert_eq!(auto_layout.layers[0].grid_locations.len(), 3);
            assert!(matches!(
                auto_layout.layers[0].grid_locations[0],
                GridLocation::Perimeter { .. }
            ));
            assert!(matches!(
                auto_layout.layers[0].grid_locations[1],
                GridLocation::Corners { .. }
            ));
            assert!(matches!(
                auto_layout.layers[0].grid_locations[2],
                GridLocation::Fill { .. }
            ));
            // TODO: Check the priority and the pb_types are correct.
        }
        _ => panic!("Should never hit this."),
    }

    // Check device.
    assert_eq!(res.device.sizing.r_min_w_nmos, 4_220.93);
    assert_eq!(res.device.sizing.r_min_w_pmos, 11_207.6);
    assert!(matches!(
        res.device.chan_width_distr.x_distr,
        ChanWDist::Uniform { .. }
    ));
    assert!(matches!(
        res.device.chan_width_distr.y_distr,
        ChanWDist::Uniform { .. }
    ));
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
    assert_eq!(switch1.mux_trans_size, Some(1.835_46));
    match switch1.buf_size {
        SwitchBufSize::Val(v) => assert_eq!(v, 10.498_6),
        SwitchBufSize::Auto => panic!("switch1 buf size expected to be Val"),
    };
    let switch2 = &res.switch_list[1];
    assert!(matches!(switch2.sw_type, SwitchType::Mux));
    assert_eq!(switch2.name, "ipin_cblock");
    assert_eq!(switch2.resistance, 1_055.232_544);
    assert_eq!(switch2.c_in, 0.0);
    assert_eq!(switch2.c_out, 0.0);
    assert_eq!(switch2.t_del, Some(8.045e-11));
    assert_eq!(switch2.mux_trans_size, Some(0.983352));
    assert!(matches!(switch2.buf_size, SwitchBufSize::Auto));

    // Check segment list
    assert_eq!(res.segment_list.len(), 1);
    assert_eq!(res.segment_list[0].freq, 1.0);
    assert_eq!(res.segment_list[0].length, 1);
    assert!(matches!(
        res.segment_list[0].segment_type,
        SegmentType::Unidir
    ));
    assert_eq!(res.segment_list[0].r_metal, 0.0);
    assert_eq!(res.segment_list[0].c_metal, 0.0);

    // Check custom switch blocks.
    assert_eq!(res.custom_switch_blocks.len(), 0);

    // Check direct list
    assert_eq!(res.direct_list.len(), 0);

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
    // There are 4 specified in the file and 4 built-in.
    assert_eq!(res.models.len(), 4 + 4);
    let multiply_model = &res.models[4];
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

    // Check direct list.
    assert_eq!(res.direct_list.len(), 1);
    let gdirect = &res.direct_list[0];
    assert_eq!(gdirect.name, "adder_carry");
    assert_eq!(gdirect.from_pin, "clb.cout");
    assert_eq!(gdirect.to_pin, "clb.cin");
    assert_eq!(gdirect.x_offset, 0);
    assert_eq!(gdirect.y_offset, -1);
    assert_eq!(gdirect.z_offset, 0);
    assert!(gdirect.switch_name.is_none());
    assert!(gdirect.from_side.is_none());
    assert!(gdirect.to_side.is_none());

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
    assert_eq!(res.layouts.layout_list.len(), 7);

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

    // Check custom switch blocks
    assert_eq!(res.custom_switch_blocks.len(), 400);
    let custom_sb0 = &res.custom_switch_blocks[0];
    assert_eq!(custom_sb0.name, "custom_switch_block_0_0");
    assert!(matches!(custom_sb0.sb_type, CustomSwitchBlockType::Unidir));
    assert!(matches!(
        custom_sb0.switchblock_location,
        CustomSwitchBlockLocation::XYSpecified { x: 0, y: 0 }
    ));
    assert_eq!(custom_sb0.switch_funcs.len(), 12);

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

#[test]
fn test_custom_sbloc() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/custom_sbloc.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check tiles.
    assert_eq!(res.tiles.len(), 4);

    // Test tile 0: io - no switchblock_locations specified
    assert_eq!(res.tiles[0].name, "io");
    assert!(res.tiles[0].switchblock_locations.is_none());

    // Test tile 1: clb - no switchblock_locations specified
    assert_eq!(res.tiles[1].name, "clb");
    assert!(res.tiles[1].switchblock_locations.is_none());

    // Test tile 2: mult_36 - custom switchblock_locations with custom pattern
    assert_eq!(res.tiles[2].name, "mult_36");
    let mult_sbloc = res.tiles[2].switchblock_locations.as_ref();
    assert!(mult_sbloc.is_some());
    let mult_sbloc = mult_sbloc.unwrap();

    // Check pattern is custom
    assert!(matches!(
        mult_sbloc.pattern,
        SwitchBlockLocationsPattern::Custom(_)
    ));

    // Check internal_switch is None
    assert!(mult_sbloc.internal_switch.is_none());

    // Check custom sb_loc entries
    if let SwitchBlockLocationsPattern::Custom(custom_locs) = &mult_sbloc.pattern {
        assert_eq!(custom_locs.len(), 9); // 5 full + 1 full + 1 straight + 1 turns + 1 none

        // Check first sb_loc (full, xoffset=0, yoffset=2)
        assert!(matches!(
            custom_locs[0].sb_type,
            SwitchBlockLocationType::Full
        ));
        assert_eq!(custom_locs[0].xoffset, 0);
        assert_eq!(custom_locs[0].yoffset, 2);
        assert!(custom_locs[0].switch_override.is_none());

        // Check straight sb_loc (straight, xoffset=0, yoffset=1)
        assert!(matches!(
            custom_locs[6].sb_type,
            SwitchBlockLocationType::Straight
        ));
        assert_eq!(custom_locs[6].xoffset, 0);
        assert_eq!(custom_locs[6].yoffset, 1);

        // Check turns sb_loc (turns, xoffset=1, yoffset=0)
        assert!(matches!(
            custom_locs[7].sb_type,
            SwitchBlockLocationType::Turns
        ));
        assert_eq!(custom_locs[7].xoffset, 1);
        assert_eq!(custom_locs[7].yoffset, 0);

        // Check none sb_loc (none, xoffset=1, yoffset=1)
        assert!(matches!(
            custom_locs[8].sb_type,
            SwitchBlockLocationType::None
        ));
        assert_eq!(custom_locs[8].xoffset, 1);
        assert_eq!(custom_locs[8].yoffset, 1);
    } else {
        panic!("Expected custom pattern for mult_36");
    }

    // Test tile 3: memory - external switchblock_locations
    assert_eq!(res.tiles[3].name, "memory");
    let mem_sbloc = res.tiles[3].switchblock_locations.as_ref();
    assert!(mem_sbloc.is_some());
    let mem_sbloc = mem_sbloc.unwrap();

    // Check pattern is external
    assert!(matches!(
        mem_sbloc.pattern,
        SwitchBlockLocationsPattern::External
    ));
    assert!(mem_sbloc.internal_switch.is_none());

    Ok(())
}

#[test]
fn test_vtr_flagship_tileable() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6_frac_N10_frac_chain_mem32K_40nm_tileable.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check tiles.
    assert_eq!(res.tiles.len(), 4);

    // Check that the tileable config is correct.
    if let Some(tileable_config) = res.layouts.tileable_config {
        assert!(tileable_config.tileable);
        assert!(tileable_config.through_channel);
        assert!(tileable_config.concat_pass_wire);
        assert!(!tileable_config.shrink_boundary);
        assert!(!tileable_config.perimeter_cb);
        assert!(!tileable_config.opin2all_sides);
        assert!(!tileable_config.concat_wire);
    } else {
        panic!("Expected tileable layout");
    }

    Ok(())
}

#[test]
fn test_k6_n10_40nm_interposer() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6_N10_40nm_interposer.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    assert_eq!(res.layouts.layout_list.len(), 2);

    if let Layout::FixedLayout(fixed_layout) = &res.layouts.layout_list[1] {
        assert_eq!(fixed_layout.name, "vtr_homogeneous_extra_small");
        assert_eq!(fixed_layout.width, 10);
        assert_eq!(fixed_layout.height, 10);
        assert_eq!(fixed_layout.layers.len(), 1);
        assert_eq!(fixed_layout.layers[0].grid_locations.len(), 5);
        if let GridLocation::InterposerCut(horizontal_cut) =
            &fixed_layout.layers[0].grid_locations[3]
        {
            assert_eq!(horizontal_cut.y, Some("4".to_string()));
            assert_eq!(horizontal_cut.x, None);
            assert_eq!(horizontal_cut.interdie_wires.len(), 2);
            let l_up_wire = &horizontal_cut.interdie_wires[0];
            assert_eq!(l_up_wire.sg_name, "interposer_sg");
            assert_eq!(l_up_wire.sg_link, "L_UP");
            assert_eq!(l_up_wire.offset_start, -1);
            assert_eq!(l_up_wire.offset_end, -1);
            assert_eq!(l_up_wire.offset_increment, 1);
            assert_eq!(l_up_wire.num, "4");
            let l_down_wire = &horizontal_cut.interdie_wires[1];
            assert_eq!(l_down_wire.sg_name, "interposer_sg");
            assert_eq!(l_down_wire.sg_link, "L_DOWN");
            assert_eq!(l_down_wire.offset_start, 1);
            assert_eq!(l_down_wire.offset_end, 1);
            assert_eq!(l_down_wire.offset_increment, -1);
            assert_eq!(l_down_wire.num, "4");
        } else {
            panic!("Fourth grid location is expected to be an interposer cut.");
        }
        if let GridLocation::InterposerCut(vertical_cut) = &fixed_layout.layers[0].grid_locations[4]
        {
            assert_eq!(vertical_cut.x, Some("4".to_string()));
            assert_eq!(vertical_cut.y, None);
            assert_eq!(vertical_cut.interdie_wires.len(), 2);
            let l_right_wire = &vertical_cut.interdie_wires[0];
            assert_eq!(l_right_wire.sg_name, "interposer_sg");
            assert_eq!(l_right_wire.sg_link, "L_RIGHT");
            assert_eq!(l_right_wire.offset_start, -1);
            assert_eq!(l_right_wire.offset_end, -1);
            assert_eq!(l_right_wire.offset_increment, 1);
            assert_eq!(l_right_wire.num, "4");
            let l_left_wire = &vertical_cut.interdie_wires[1];
            assert_eq!(l_left_wire.sg_name, "interposer_sg");
            assert_eq!(l_left_wire.sg_link, "L_LEFT");
            assert_eq!(l_left_wire.offset_start, 1);
            assert_eq!(l_left_wire.offset_end, 1);
            assert_eq!(l_left_wire.offset_increment, -1);
            assert_eq!(l_left_wire.num, "4");
        } else {
            panic!("Fifth grid location is expected to be an interposer cut.");
        }
    } else {
        panic!("Second layout expected to be the fixed layout.");
    }

    Ok(())
}

#[test]
fn test_koios_arch() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k6FracN10LB_mem20K_complexDSP_customSB_22nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    assert_eq!(res.tiles.len(), 4);
    assert_eq!(res.layouts.layout_list.len(), 6);

    Ok(())
}

#[test]
fn test_3d_k4_n4_90nm_opin_per_block() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/3d_k4_N4_90nm_opin_per_block.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    assert_eq!(res.tiles.len(), 2);
    assert_eq!(res.layouts.layout_list.len(), 2);

    if let Layout::FixedLayout(fixed_3d_layout) = &res.layouts.layout_list[1] {
        assert_eq!(fixed_3d_layout.name, "FPGA3D");
        assert_eq!(fixed_3d_layout.width, 11);
        assert_eq!(fixed_3d_layout.height, 11);
        assert_eq!(fixed_3d_layout.layers.len(), 2);
        assert_eq!(fixed_3d_layout.layers[0].die, 0);
        assert_eq!(fixed_3d_layout.layers[1].die, 1);

        assert_eq!(fixed_3d_layout.layers[0].grid_locations.len(), 2);
        assert!(matches!(
            fixed_3d_layout.layers[0].grid_locations[0],
            GridLocation::Perimeter { .. }
        ));
        assert!(matches!(
            fixed_3d_layout.layers[0].grid_locations[1],
            GridLocation::Fill { .. }
        ));

        assert_eq!(fixed_3d_layout.layers[1].grid_locations.len(), 3);
        assert!(matches!(
            fixed_3d_layout.layers[1].grid_locations[0],
            GridLocation::Region { .. }
        ));
        assert!(matches!(
            fixed_3d_layout.layers[1].grid_locations[1],
            GridLocation::Region { .. }
        ));
        assert!(matches!(
            fixed_3d_layout.layers[1].grid_locations[2],
            GridLocation::Fill { .. }
        ));
    } else {
        panic!("Second layout expected to be fixed.");
    }

    Ok(())
}

#[test]
fn mesh_noc_topology() -> Result<(), FPGAArchParseError> {
    let input_xml_relative =
        PathBuf::from("tests/k6_frac_N10_frac_chain_mem32K_40nm_with_a_2x2_mesh_noc_topology.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check NoC info.
    let noc = res.noc.as_ref().expect("Expected NoC info to be present");
    assert_eq!(noc.link_latency, 1e-9);
    assert_eq!(noc.router_latency, 2e-9);
    assert_eq!(noc.link_bandwidth, 1.2e9);
    assert_eq!(noc.noc_router_tile_name, "noc_router");

    // A 2x2 single-layer mesh produces 4 routers.
    // IDs are assigned row-major from the bottom-left corner:
    //   id=0 (row=0,col=0)  id=1 (row=0,col=1)
    //   id=2 (row=1,col=0)  id=3 (row=1,col=1)
    let routers = &noc.topology.routers;
    assert_eq!(routers.len(), 4);

    // Bottom-left: connected to right (1) and above (2).
    assert_eq!(routers[0].id, 0);
    assert_eq!(routers[0].position_x, 0.0);
    assert_eq!(routers[0].position_y, 0.0);
    assert_eq!(routers[0].layer, 0);
    assert_eq!(routers[0].connections, vec![1, 2]);

    // Bottom-right: connected to left (0) and above (3).
    assert_eq!(routers[1].id, 1);
    assert_eq!(routers[1].position_x, 5.0);
    assert_eq!(routers[1].position_y, 0.0);
    assert_eq!(routers[1].layer, 0);
    assert_eq!(routers[1].connections, vec![0, 3]);

    // Top-left: connected to right (3) and below (0).
    assert_eq!(routers[2].id, 2);
    assert_eq!(routers[2].position_x, 0.0);
    assert_eq!(routers[2].position_y, 5.0);
    assert_eq!(routers[2].layer, 0);
    assert_eq!(routers[2].connections, vec![3, 0]);

    // Top-right: connected to left (2) and below (1).
    assert_eq!(routers[3].id, 3);
    assert_eq!(routers[3].position_x, 5.0);
    assert_eq!(routers[3].position_y, 5.0);
    assert_eq!(routers[3].layer, 0);
    assert_eq!(routers[3].connections, vec![2, 1]);

    Ok(())
}

#[test]
fn test_k4_n4_90nm_clb_complex_block_graph() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from("tests/k4_N4_90nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");
    let res = fpga_arch_parser::parse(&input_xml)?;

    // One graph per complex block (io and clb).
    assert_eq!(res.complex_block_graphs.len(), 2);
    let g = &res.complex_block_graphs[1];

    // --- Root: clb ---
    let root_id = g.root_complex_block_node;
    let root = &g.complex_block_nodes[root_id];
    assert_eq!(root.name, "clb");
    assert!(root.parent_mode.is_none());
    assert!(root.primitive_info.is_none());

    // Ports: I[10], O[4], clk[1].
    assert_eq!(root.input_ports.len(), 1);
    assert_eq!(root.output_ports.len(), 1);
    assert_eq!(root.clock_ports.len(), 1);
    let port_i = &g.complex_block_ports[root.input_ports[0]];
    assert_eq!(port_i.name, "I");
    assert_eq!(port_i.pins.len(), 10);
    let port_o = &g.complex_block_ports[root.output_ports[0]];
    assert_eq!(port_o.name, "O");
    assert_eq!(port_o.pins.len(), 4);
    let port_clk = &g.complex_block_ports[root.clock_ports[0]];
    assert_eq!(port_clk.name, "clk");
    assert_eq!(port_clk.pins.len(), 1);

    // clb has one implicit mode (no named modes, but has direct pb_type children).
    assert_eq!(root.modes.len(), 1);
    let clb_mode = &g.complex_block_modes[root.modes[0]];

    // Mode children: 4 fle instances + 3 interconnects (crossbar, clks, clbouts1).
    assert_eq!(clb_mode.children_complex_blocks.len(), 7);

    // --- fle instances [0..3] ---
    for i in 0..4 {
        let fle = &g.complex_block_nodes[clb_mode.children_complex_blocks[i]];
        assert_eq!(fle.name, "fle");
        assert_eq!(fle.input_ports.len(), 1);
        assert_eq!(fle.output_ports.len(), 1);
        assert_eq!(fle.clock_ports.len(), 1);
        assert_eq!(g.complex_block_ports[fle.input_ports[0]].pins.len(), 4);
        assert_eq!(g.complex_block_ports[fle.output_ports[0]].pins.len(), 1);
        assert_eq!(g.complex_block_ports[fle.clock_ports[0]].pins.len(), 1);

        // fle has one explicit mode: n1_lut4.
        assert_eq!(fle.modes.len(), 1);
        let fle_mode = &g.complex_block_modes[fle.modes[0]];

        // Mode children: 1 ble4 + 3 interconnects (direct1, direct2, direct3).
        assert_eq!(fle_mode.children_complex_blocks.len(), 4);

        // --- ble4 ---
        let ble4 = &g.complex_block_nodes[fle_mode.children_complex_blocks[0]];
        assert_eq!(ble4.name, "ble4");
        assert_eq!(ble4.input_ports.len(), 1);
        assert_eq!(ble4.output_ports.len(), 1);
        assert_eq!(ble4.clock_ports.len(), 1);
        assert_eq!(g.complex_block_ports[ble4.input_ports[0]].pins.len(), 4);
        assert_eq!(g.complex_block_ports[ble4.output_ports[0]].pins.len(), 1);
        assert_eq!(g.complex_block_ports[ble4.clock_ports[0]].pins.len(), 1);

        // ble4 has one implicit mode.
        assert_eq!(ble4.modes.len(), 1);
        let ble4_mode = &g.complex_block_modes[ble4.modes[0]];

        // Mode children: lut4, ff, + 4 interconnects (direct1..3, mux1).
        assert_eq!(ble4_mode.children_complex_blocks.len(), 6);

        // lut4: leaf primitive, class Lut, ports in[4] out[1].
        let lut4 = &g.complex_block_nodes[ble4_mode.children_complex_blocks[0]];
        assert_eq!(lut4.name, "lut4");
        assert_eq!(lut4.input_ports.len(), 1);
        assert_eq!(lut4.output_ports.len(), 1);
        assert_eq!(lut4.clock_ports.len(), 0);
        assert_eq!(g.complex_block_ports[lut4.input_ports[0]].pins.len(), 4);
        assert_eq!(g.complex_block_ports[lut4.output_ports[0]].pins.len(), 1);
        let lut4_info = lut4
            .primitive_info
            .as_ref()
            .expect("lut4 must have primitive_info");
        assert_eq!(lut4_info.blif_model, ".names");
        assert!(matches!(lut4_info.class, PBTypeClass::Lut));

        // ff: leaf primitive, class FlipFlop, ports D[1] Q[1] clk[1].
        let ff = &g.complex_block_nodes[ble4_mode.children_complex_blocks[1]];
        assert_eq!(ff.name, "ff");
        assert_eq!(ff.input_ports.len(), 1);
        assert_eq!(ff.output_ports.len(), 1);
        assert_eq!(ff.clock_ports.len(), 1);
        assert_eq!(g.complex_block_ports[ff.input_ports[0]].pins.len(), 1);
        assert_eq!(g.complex_block_ports[ff.output_ports[0]].pins.len(), 1);
        assert_eq!(g.complex_block_ports[ff.clock_ports[0]].pins.len(), 1);
        let ff_info = ff
            .primitive_info
            .as_ref()
            .expect("ff must have primitive_info");
        assert_eq!(ff_info.blif_model, ".latch");
        assert!(matches!(ff_info.class, PBTypeClass::FlipFlop));

        // ble4 interconnects: direct1, direct2, direct3 (Direct), mux1 (Mux).
        let ble4_direct1 = &g.complex_block_nodes[ble4_mode.children_complex_blocks[2]];
        assert_eq!(ble4_direct1.name, "direct1");
        assert!(matches!(
            ble4_direct1.primitive_info.as_ref().unwrap().class,
            PBTypeClass::InterconnectDirect
        ));
        let ble4_mux1 = &g.complex_block_nodes[ble4_mode.children_complex_blocks[5]];
        assert_eq!(ble4_mux1.name, "mux1");
        assert!(matches!(
            ble4_mux1.primitive_info.as_ref().unwrap().class,
            PBTypeClass::InterconnectMux
        ));
        // mux1 has 2 input groups (ff.Q and lut4.out) → 2 input ports.
        assert_eq!(ble4_mux1.input_ports.len(), 2);
        assert_eq!(ble4_mux1.output_ports.len(), 1);

        // fle interconnects: direct1, direct2, direct3 (all Direct).
        for j in 1..4 {
            let fle_inter = &g.complex_block_nodes[fle_mode.children_complex_blocks[j]];
            assert!(matches!(
                fle_inter.primitive_info.as_ref().unwrap().class,
                PBTypeClass::InterconnectDirect
            ));
        }
    }

    // --- clb-level interconnects ---
    // crossbar: complete, 2 input groups (clb.I and fle[3:0].out), output fle[3:0].in.
    let crossbar = &g.complex_block_nodes[clb_mode.children_complex_blocks[4]];
    assert_eq!(crossbar.name, "crossbar");
    assert!(matches!(
        crossbar.primitive_info.as_ref().unwrap().class,
        PBTypeClass::InterconnectComplete
    ));
    assert_eq!(crossbar.input_ports.len(), 2);
    // input_0 mirrors clb.I (10 pins), input_1 mirrors fle[3:0].out (4 pins).
    assert_eq!(
        g.complex_block_ports[crossbar.input_ports[0]].pins.len(),
        10
    );
    assert_eq!(g.complex_block_ports[crossbar.input_ports[1]].pins.len(), 4);
    // output mirrors fle[3:0].in (4 fle × 4 pins = 16 pins).
    assert_eq!(crossbar.output_ports.len(), 1);
    assert_eq!(
        g.complex_block_ports[crossbar.output_ports[0]].pins.len(),
        16
    );

    // clks: complete, 1 input group (clb.clk), output fle[3:0].clk.
    let clks = &g.complex_block_nodes[clb_mode.children_complex_blocks[5]];
    assert_eq!(clks.name, "clks");
    assert!(matches!(
        clks.primitive_info.as_ref().unwrap().class,
        PBTypeClass::InterconnectComplete
    ));
    assert_eq!(clks.input_ports.len(), 1);
    assert_eq!(g.complex_block_ports[clks.input_ports[0]].pins.len(), 1);
    assert_eq!(clks.output_ports.len(), 1);
    assert_eq!(g.complex_block_ports[clks.output_ports[0]].pins.len(), 4);

    // clbouts1: direct, input fle[3:0].out (4 pins), output clb.O (4 pins).
    let clbouts1 = &g.complex_block_nodes[clb_mode.children_complex_blocks[6]];
    assert_eq!(clbouts1.name, "clbouts1");
    assert!(matches!(
        clbouts1.primitive_info.as_ref().unwrap().class,
        PBTypeClass::InterconnectDirect
    ));
    assert_eq!(clbouts1.input_ports.len(), 1);
    assert_eq!(g.complex_block_ports[clbouts1.input_ports[0]].pins.len(), 4);
    assert_eq!(clbouts1.output_ports.len(), 1);
    assert_eq!(
        g.complex_block_ports[clbouts1.output_ports[0]].pins.len(),
        4
    );

    // --- Net counts ---
    // crossbar: 10 + 4 input-side nets + 16 output-side nets = 30.
    // clks: 1 input-side + 4 output-side = 5.
    // clbouts1: 4 input-side + 4 output-side = 8.
    assert_eq!(clb_mode.interconnect.len(), 43);

    Ok(())
}

#[test]
fn embedded_star_noc_topology() -> Result<(), FPGAArchParseError> {
    let input_xml_relative = PathBuf::from(
        "tests/k6_frac_N10_frac_chain_mem32K_40nm_with_a_embedded_star_noc_topology.xml",
    );
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml)?;

    // Check NoC info.
    let noc = res.noc.as_ref().expect("Expected NoC info to be present");
    assert_eq!(noc.link_latency, 5.0);
    assert_eq!(noc.router_latency, 7.7);
    assert_eq!(noc.link_bandwidth, 10.0);
    assert_eq!(noc.noc_router_tile_name, "noc_router");

    // Check topology: star topology with 9 routers (8 leaves + 1 center).
    let routers = &noc.topology.routers;
    assert_eq!(routers.len(), 9);

    // Check the 8 leaf routers (ids 1-8), each connected only to the center (id 9).
    let leaf_positions = [
        (1, 1.5_f32, 1.5_f32),
        (2, 33.5, 1.5),
        (3, 1.5, 21.5),
        (4, 33.5, 21.5),
        (5, 16.5, 1.5),
        (6, 16.5, 21.5),
        (7, 1.5, 10.5),
        (8, 33.5, 10.5),
    ];
    for (i, (id, pos_x, pos_y)) in leaf_positions.iter().enumerate() {
        assert_eq!(routers[i].id, *id);
        assert_eq!(routers[i].position_x, *pos_x);
        assert_eq!(routers[i].position_y, *pos_y);
        assert_eq!(routers[i].layer, 0);
        assert_eq!(routers[i].connections, vec![9]);
    }

    // Check the center router (id 9), connected to all leaf routers.
    let center = &routers[8];
    assert_eq!(center.id, 9);
    assert_eq!(center.position_x, 16.5);
    assert_eq!(center.position_y, 10.5);
    assert_eq!(center.layer, 0);
    assert_eq!(center.connections, vec![1, 2, 3, 4, 5, 6, 7, 8]);

    Ok(())
}
