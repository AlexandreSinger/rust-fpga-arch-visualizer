use std::fs::File;

use csv::{StringRecord, StringRecordsIter};

use crate::{CRRSBParseError, crr_sb_des::{CRRSwitchConnection, CRRSwitchConnectionDelay, CRRSwitchSourceNodeInfo}, parse_common::{parse_crr_lane_num, parse_crr_switch_dir, parse_crr_tap_num}};

fn parse_source_info(row: &StringRecord) -> Result<CRRSwitchSourceNodeInfo, CRRSBParseError> {
    // FIXME: Check that the row has at least 4 columns.

    Ok(CRRSwitchSourceNodeInfo {
        dir: parse_crr_switch_dir(row[0].trim())?,
        segment_type: row[1].trim().to_string(),
        lane_num: parse_crr_lane_num(row[2].trim())?,
        tap_num: parse_crr_tap_num(row[3].trim())?,
    })
}

fn parse_row_edge_delay(cell_str: &str) -> Result<CRRSwitchConnectionDelay, CRRSBParseError> {
    match cell_str {
        "x" => Ok(CRRSwitchConnectionDelay::Undefined),
        _ => match cell_str.parse() {
            Ok(delay) => Ok(CRRSwitchConnectionDelay::DelaySpecified { delay }),
            Err(e) => Err(CRRSBParseError::SBSWCellParseError(e.to_string())),
        }
    }
}

fn parse_row_edges(row: &StringRecord, row_idx: usize) -> Result<Vec<CRRSwitchConnection>, CRRSBParseError> {
    let mut row_edges: Vec<CRRSwitchConnection> = Vec::new();
    let num_cols = row.len();

    for i in 4..num_cols {
        let cell_str = row[i].trim();
        if cell_str.is_empty() {
            continue;
        }

        row_edges.push(CRRSwitchConnection { 
            source_node_id: row_idx,
            sink_node_id: i - 4,
            delay: parse_row_edge_delay(cell_str)?,
        });
    }

    Ok(row_edges)
}

pub fn parse_rows(csv_records: &mut StringRecordsIter<'_, File>) -> Result<(Vec<CRRSwitchSourceNodeInfo>, Vec<CRRSwitchConnection>), CRRSBParseError> {
    let mut source_nodes: Vec<CRRSwitchSourceNodeInfo> = Vec::new();
    let mut edges: Vec<CRRSwitchConnection> = Vec::new();

    let mut row_idx: usize = 0;
    for row in csv_records {
        let row = match row {
            Ok(v) => v,
            Err(e) => {
                return Err(CRRSBParseError::CSVParseError(e.to_string()));
            },
        };
        // FIXME: We somehow need to verify that the rows have the correct length.

        source_nodes.push(parse_source_info(&row)?);
        edges.append(&mut parse_row_edges(&row, row_idx)?);

        row_idx += 1;
    }

    Ok((source_nodes, edges))
}