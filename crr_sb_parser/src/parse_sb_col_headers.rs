use std::fs::File;

use csv::StringRecordsIter;

use crate::{CRRSBParseError, crr_sb_des::CRRSwitchSinkNodeInfo, parse_common::{parse_crr_lane_num, parse_crr_switch_dir}};

pub fn parse_sink_nodes(csv_records: &mut StringRecordsIter<'_, File>) -> Result<Vec<CRRSwitchSinkNodeInfo>, CRRSBParseError> {
    let mut sink_nodes: Vec<CRRSwitchSinkNodeInfo> = Vec::new();

    let dir_row = match csv_records.next() {
        Some(Ok(row)) => row,
        _ => {return Err(CRRSBParseError::SBHeaderRowMissing("Dir column header row missing.".to_string()))},
    };

    let segment_type_row = match csv_records.next() {
        Some(Ok(row)) => row,
        _ => {return Err(CRRSBParseError::SBHeaderRowMissing("Segment type column header row missing.".to_string()))},
    };

    // FIXME: This row is optional, but it is not clear how the code will know if this
    //        is provided or not. For now, assume it is always provided. Ask Amin.
    let fan_in_row = match csv_records.next() {
        Some(Ok(row)) => row,
        _ => {return Err(CRRSBParseError::SBHeaderRowMissing("Fan-in column header row missing.".to_string()))},
    };

    let lane_num_row = match csv_records.next() {
        Some(Ok(row)) => row,
        _ => {return Err(CRRSBParseError::SBHeaderRowMissing("lane-num column header row missing.".to_string()))},
    };

    // FIXME: Verify that all rows above are the same length.
    //        Should be true by construction, but worth checking.
    let num_cols = dir_row.len();

    // TODO: Check that the first 4 cells of each row are empty.

    for i in 4..num_cols {
        sink_nodes.push(CRRSwitchSinkNodeInfo {
            dir: parse_crr_switch_dir(&dir_row[i].trim())?,
            segment_type: segment_type_row[i].trim().to_string(),
            // FIXME: Move these parsing functions to common.
            fan_in: match fan_in_row[i].trim().parse() {
                Ok(v) => Some(v),
                Err(e) => {
                    return Err(CRRSBParseError::SBHeaderCellParseError(
                        format!("{e}")
                    ));
                }
            },
            lane_num: parse_crr_lane_num(lane_num_row[i].trim())?,
        });
    }

    Ok(sink_nodes)
}