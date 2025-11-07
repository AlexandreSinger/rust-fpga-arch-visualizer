
use std::path::{PathBuf, absolute};

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

    // TODO: Collect stats on the architecture and ensure they match what is
    //       expected.
}
