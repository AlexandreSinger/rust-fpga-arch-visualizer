
use std::path::{PathBuf, absolute};

use fpga_arch_parser::{Layout, GridLocation, Port, SubTileIOFC};

#[test]
fn test_k4_n4_90nm() {
    let input_xml_relative = PathBuf::from("tests/k4_N4_90nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml);
    assert!(res.is_ok());

    let res = res.unwrap();

    // Check tiles.
    assert!(res.tiles.len() == 2);
    assert!(res.tiles[0].name == "io");
    assert!(res.tiles[1].name == "clb");
    assert!(res.tiles[0].sub_tiles.len() == 1);
    assert!(res.tiles[0].sub_tiles[0].name == "io");
    assert!(res.tiles[0].sub_tiles[0].capacity == 3);
    assert!(res.tiles[0].sub_tiles[0].equivalent_sites.len() == 1);
    assert!(res.tiles[0].sub_tiles[0].equivalent_sites[0].pb_type == "io");
    assert!(res.tiles[0].sub_tiles[0].equivalent_sites[0].pin_mapping == "direct");
    assert!(res.tiles[0].sub_tiles[0].ports.len() == 3);
    assert!(matches!(res.tiles[0].sub_tiles[0].ports[0], Port::Input { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].ports[1], Port::Output { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].ports[2], Port::Clock { .. }));
    // TODO: Add stronger tests for ports.
    assert!(matches!(res.tiles[0].sub_tiles[0].fc.in_fc, SubTileIOFC::Frac { .. }));
    assert!(matches!(res.tiles[0].sub_tiles[0].fc.out_fc, SubTileIOFC::Frac { .. }));
    assert!(res.tiles[1].sub_tiles.len() == 1);
    assert!(res.tiles[1].sub_tiles[0].name == "clb");
    assert!(res.tiles[1].sub_tiles[0].capacity == 1);
    assert!(res.tiles[1].sub_tiles[0].equivalent_sites.len() == 1);
    assert!(res.tiles[1].sub_tiles[0].equivalent_sites[0].pb_type == "clb");
    assert!(res.tiles[1].sub_tiles[0].equivalent_sites[0].pin_mapping == "direct");

    // Check layouts.
    assert!(res.layouts.len() == 1);
    assert!(matches!(res.layouts[0], Layout::AutoLayout { .. }));
    match &res.layouts[0] {
        Layout::AutoLayout( auto_layout ) => {
            assert!(auto_layout.aspect_ratio == 1.0);
            assert!(auto_layout.grid_locations.len() == 3);
            assert!(matches!(auto_layout.grid_locations[0], GridLocation::Perimeter { .. }));
            assert!(matches!(auto_layout.grid_locations[1], GridLocation::Corners { .. }));
            assert!(matches!(auto_layout.grid_locations[2], GridLocation::Fill { .. }));
            // TODO: Check the priority and the pb_types are correct.
        },
        _ => panic!("Should never hit this.")
    }

    // Check complex block list.
    assert!(res.complex_block_list.len() == 2);
    assert!(res.complex_block_list[0].name == "io");
    assert!(res.complex_block_list[0].num_pb == 1);
    assert!(res.complex_block_list[0].blif_model.is_none());
    assert!(res.complex_block_list[0].class.is_none());
    assert!(res.complex_block_list[0].ports.len() == 3);
    assert!(matches!(res.complex_block_list[0].ports[0], Port::Input { .. }));
    assert!(matches!(res.complex_block_list[0].ports[1], Port::Output { .. }));
    assert!(matches!(res.complex_block_list[0].ports[2], Port::Clock { .. }));
    // TODO: Add stronger tests for pb_type ports.
    assert!(res.complex_block_list[0].pb_types.len() == 0);
    assert!(res.complex_block_list[0].modes.len() == 2);
    assert!(res.complex_block_list[0].modes[0].name == "inpad");
    assert!(res.complex_block_list[0].modes[0].pb_types.len() == 1);
    assert!(res.complex_block_list[0].modes[0].pb_types[0].name == "inpad");
    assert!(res.complex_block_list[0].modes[0].pb_types[0].blif_model.is_some());
    assert!(res.complex_block_list[0].modes[1].name == "outpad");
    assert!(res.complex_block_list[0].modes[1].pb_types.len() == 1);
    // TODO: Make these pb_type heirarchy checks more robust.
    assert!(res.complex_block_list[1].name == "clb");

    // TODO: Collect stats on the architecture and ensure they match what is
    //       expected.
}
