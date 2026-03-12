use std::fs::File;

use csv::{StringRecord, StringRecordsIter};

use crate::{
    crr_sb_des::{CRRSwitchConnection, CRRSwitchConnectionDelay, CRRSwitchSourceNodeInfo}, parse_common::{parse_crr_lane_num, parse_crr_switch_dir, parse_crr_tap_num}, CRRSBParseError, CRRSwitchDir, CRRSwitchSourcePin
};

fn parse_source_pin(source_pin_str: &str, dir: CRRSwitchDir) -> Result<CRRSwitchSourcePin, CRRSBParseError> {
    match dir {
        CRRSwitchDir::OPIN => Ok(CRRSwitchSourcePin::Pin { pin_name: source_pin_str.to_string() }),
        CRRSwitchDir::IPIN => Err(CRRSBParseError::SBHeaderCellParseError("Source node cannot have IPIN dir.".to_string())),
        CRRSwitchDir::Left | CRRSwitchDir::Right | CRRSwitchDir::Bottom | CRRSwitchDir::Top => {
            Ok(CRRSwitchSourcePin::Tap { tap_num: parse_crr_tap_num(source_pin_str)? })
        }
    }
}

fn parse_source_info(row: &StringRecord) -> Result<CRRSwitchSourceNodeInfo, CRRSBParseError> {
    if row.len() < 4 {
        return Err(CRRSBParseError::SBHeaderColMissing(format!(
            "Found {} row header cols, expected 4.",
            row.len()
        )));
    }

    let dir = parse_crr_switch_dir(row[0].trim())?;

    Ok(CRRSwitchSourceNodeInfo {
        dir,
        segment_type: row[1].trim().to_string(),
        lane_num: parse_crr_lane_num(row[2].trim())?,
        source_pin: parse_source_pin(row[3].trim(), dir)?,
    })
}

fn parse_row_edge_delay(cell_str: &str) -> Result<CRRSwitchConnectionDelay, CRRSBParseError> {
    match cell_str {
        "x" => Ok(CRRSwitchConnectionDelay::Undefined),
        _ => match cell_str.parse() {
            Ok(delay) => Ok(CRRSwitchConnectionDelay::DelaySpecified { delay }),
            Err(e) => Err(CRRSBParseError::SBSWCellParseError(e.to_string())),
        },
    }
}

fn parse_row_edges(
    row: &StringRecord,
    row_idx: usize,
) -> Result<Vec<CRRSwitchConnection>, CRRSBParseError> {
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

pub fn parse_rows(
    csv_records: &mut StringRecordsIter<'_, File>,
) -> Result<(Vec<CRRSwitchSourceNodeInfo>, Vec<CRRSwitchConnection>), CRRSBParseError> {
    let mut source_nodes: Vec<CRRSwitchSourceNodeInfo> = Vec::new();
    let mut edges: Vec<CRRSwitchConnection> = Vec::new();

    for (row_idx, row) in csv_records.enumerate() {
        let row = match row {
            Ok(v) => v,
            Err(e) => {
                return Err(CRRSBParseError::CSVParseError(e.to_string()));
            }
        };
        // TODO: We somehow need to verify that the rows have the correct length.

        source_nodes.push(parse_source_info(&row)?);
        edges.append(&mut parse_row_edges(&row, row_idx)?);
    }

    Ok((source_nodes, edges))
}
