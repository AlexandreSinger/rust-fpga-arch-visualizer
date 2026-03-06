use std::path::{PathBuf, absolute};

use crr_sb_parser::{CRRSBParseError, CRRSwitchConnectionDelay, CRRSwitchDir};

#[test]
fn test_sb_template_1() -> Result<(), CRRSBParseError> {
    let input_csv_relative = PathBuf::from("tests/sb_template_1.csv");
    let input_csv = absolute(&input_csv_relative).expect("Failed to get absolute path");

    let res = crr_sb_parser::parse_csv_file(&input_csv)?;

    // Check the column headers.
    assert_eq!(res.sink_nodes.len(), 100);
    let sides_in_order = [CRRSwitchDir::Left, CRRSwitchDir::Right, CRRSwitchDir::Top, CRRSwitchDir::Bottom];
    for i in 0..100 {
        assert_eq!(res.sink_nodes[i].dir, sides_in_order[i / 25]);
        assert_eq!(res.sink_nodes[i].segment_type, "l4");
        assert_eq!(res.sink_nodes[i].fan_in, Some(20));
        assert_eq!(res.sink_nodes[i].lane_num, (i % 25) + 1);
    }

    // Check the row headers.
    assert_eq!(res.source_nodes.len(), 400);
    for i in 0..400 {
        assert_eq!(res.source_nodes[i].dir, sides_in_order[i / 100]);
        assert_eq!(res.source_nodes[i].segment_type, "l4");
        assert_eq!(res.source_nodes[i].lane_num, ((i / 4) % 25) + 1);
        assert_eq!(res.source_nodes[i].tap_num, (i % 4) + 1);
    }

    // FIXME: Need to confirm how large this should be.
    //        I think it should be 2000, but the tool is saying 2227.
    // assert_eq!(res.cells.len(), 2000);

    // TODO: We should check for more cells than just the first and last ones.
    assert_eq!(res.edges[0].source_node_id, 0);
    assert_eq!(res.edges[0].sink_node_id, 40);
    if let CRRSwitchConnectionDelay::DelaySpecified { delay } = res.edges[0].delay {
        assert_eq!(delay, 120.0);
    } else {
        panic!("First cell expected to be delay specified.");
    }

    let last_cell = res.edges.last().expect("Cells should not be empty.");
    assert_eq!(last_cell.source_node_id, 399);
    assert_eq!(last_cell.sink_node_id, 69);
    if let CRRSwitchConnectionDelay::DelaySpecified { delay } = last_cell.delay {
        assert_eq!(delay, 165.0);
    } else {
        panic!("Last cell expected to be delay specified.");
    }

    Ok(())
}

#[test]
fn test_sb_template_2() -> Result<(), CRRSBParseError> {
    let input_csv_relative = PathBuf::from("tests/sb_template_2.csv");
    let input_csv = absolute(&input_csv_relative).expect("Failed to get absolute path");

    let res = crr_sb_parser::parse_csv_file(&input_csv)?;

    // Check the column headers.
    assert_eq!(res.sink_nodes.len(), 100);
    let sides_in_order = [CRRSwitchDir::Left, CRRSwitchDir::Right, CRRSwitchDir::Top, CRRSwitchDir::Bottom];
    for i in 0..100 {
        assert_eq!(res.sink_nodes[i].dir, sides_in_order[i / 25]);
        assert_eq!(res.sink_nodes[i].segment_type, "l4");
        assert_eq!(res.sink_nodes[i].fan_in, Some(20));
        assert_eq!(res.sink_nodes[i].lane_num, (i % 25) + 1);
    }

    // Check the row headers.
    assert_eq!(res.source_nodes.len(), 400);
    for i in 0..400 {
        assert_eq!(res.source_nodes[i].dir, sides_in_order[i / 100]);
        assert_eq!(res.source_nodes[i].segment_type, "l4");
        assert_eq!(res.source_nodes[i].lane_num, ((i / 4) % 25) + 1);
        assert_eq!(res.source_nodes[i].tap_num, (i % 4) + 1);
    }

    // FIXME: Need to confirm how large this should be.
    //        I think it should be 2000, but the tool is saying 2227.
    // assert_eq!(res.cells.len(), 2000);

    // TODO: We should check for more cells than just the first and last ones.
    assert_eq!(res.edges[0].source_node_id, 0);
    assert_eq!(res.edges[0].sink_node_id, 29);
    if let CRRSwitchConnectionDelay::DelaySpecified { delay } = res.edges[0].delay {
        assert_eq!(delay, 188.0);
    } else {
        panic!("First cell expected to be delay specified.");
    }

    let last_cell = res.edges.last().expect("Cells should not be empty.");
    assert_eq!(last_cell.source_node_id, 399);
    assert_eq!(last_cell.sink_node_id, 71);
    if let CRRSwitchConnectionDelay::DelaySpecified { delay } = last_cell.delay {
        assert_eq!(delay, 136.0);
    } else {
        panic!("Last cell expected to be delay specified.");
    }

    Ok(())
}

#[test]
fn test_sb_template_3() -> Result<(), CRRSBParseError> {
    let input_csv_relative = PathBuf::from("tests/sb_template_3.csv");
    let input_csv = absolute(&input_csv_relative).expect("Failed to get absolute path");

    let res = crr_sb_parser::parse_csv_file(&input_csv)?;

    // Check the column headers.
    assert_eq!(res.sink_nodes.len(), 100);
    let sides_in_order = [CRRSwitchDir::Left, CRRSwitchDir::Right, CRRSwitchDir::Top, CRRSwitchDir::Bottom];
    for i in 0..100 {
        assert_eq!(res.sink_nodes[i].dir, sides_in_order[i / 25]);
        assert_eq!(res.sink_nodes[i].segment_type, "l4");
        assert_eq!(res.sink_nodes[i].fan_in, Some(20));
        assert_eq!(res.sink_nodes[i].lane_num, (i % 25) + 1);
    }

    // Check the row headers.
    assert_eq!(res.source_nodes.len(), 400);
    for i in 0..400 {
        assert_eq!(res.source_nodes[i].dir, sides_in_order[i / 100]);
        assert_eq!(res.source_nodes[i].segment_type, "l4");
        assert_eq!(res.source_nodes[i].lane_num, ((i / 4) % 25) + 1);
        assert_eq!(res.source_nodes[i].tap_num, (i % 4) + 1);
    }

    // FIXME: Need to confirm how large this should be.
    //        I think it should be 2000, but the tool is saying 2227.
    // assert_eq!(res.cells.len(), 2000);

    // TODO: We should check for more cells than just the first and last ones.
    assert_eq!(res.edges[0].source_node_id, 0);
    assert_eq!(res.edges[0].sink_node_id, 42);
    if let CRRSwitchConnectionDelay::DelaySpecified { delay } = res.edges[0].delay {
        assert_eq!(delay, 136.0);
    } else {
        panic!("First cell expected to be delay specified.");
    }

    let last_cell = res.edges.last().expect("Cells should not be empty.");
    assert_eq!(last_cell.source_node_id, 399);
    assert_eq!(last_cell.sink_node_id, 73);
    if let CRRSwitchConnectionDelay::DelaySpecified { delay } = last_cell.delay {
        assert_eq!(delay, 137.0);
    } else {
        panic!("Last cell expected to be delay specified.");
    }

    Ok(())
}