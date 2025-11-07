
use std::path::{PathBuf, absolute};

use fpga_arch_parser::{Layout, GridLocation};

#[test]
fn test_k4_n4_90nm() {
    let input_xml_relative = PathBuf::from("tests/k4_N4_90nm.xml");
    let input_xml = absolute(&input_xml_relative).expect("Failed to get absolute path");

    let res = fpga_arch_parser::parse(&input_xml);
    assert!(res.is_ok());

    let res = res.unwrap();

    assert!(res.tiles.len() == 2);
    assert!(res.tiles[0].name == "io");
    assert!(res.tiles[1].name == "clb");

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

    // TODO: Collect stats on the architecture and ensure they match what is
    //       expected.
}
