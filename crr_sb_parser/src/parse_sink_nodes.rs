use std::fs::File;

use csv::StringRecordsIter;

use crate::{
    CRRSBParseError,
    crr_sb_des::CRRSwitchSinkNodeInfo,
    parse_common::{parse_crr_fan_in, parse_crr_lane_num, parse_crr_switch_dir},
};

fn parse_fan_in_cell(cell_str: &str) -> Result<Option<usize>, CRRSBParseError> {
    match cell_str {
        "" => Ok(None),
        _ => Ok(Some(parse_crr_fan_in(cell_str)?)),
    }
}

fn parse_header_row(
    csv_records: &mut StringRecordsIter<'_, File>,
    row_description: &str,
) -> Result<csv::StringRecord, CRRSBParseError> {
    match csv_records.next() {
        Some(Ok(row)) => Ok(row),
        Some(Err(e)) => Err(CRRSBParseError::CSVParseError(e.to_string())),
        None => Err(CRRSBParseError::SBHeaderRowMissing(format!(
            "{} column header row missing.",
            row_description
        ))),
    }
}

pub fn parse_sink_nodes(
    csv_records: &mut StringRecordsIter<'_, File>,
) -> Result<Vec<CRRSwitchSinkNodeInfo>, CRRSBParseError> {
    let mut sink_nodes: Vec<CRRSwitchSinkNodeInfo> = Vec::new();

    // Parse the first four rows which are assumed to be the sink node headers.
    let dir_row = parse_header_row(csv_records, "Dir")?;
    let segment_type_row = parse_header_row(csv_records, "Segment type")?;
    let fan_in_row = parse_header_row(csv_records, "Fan-in")?;
    let lane_num_row = parse_header_row(csv_records, "lane-num")?;

    // Check that all rows have the same length. This protects the code below
    // from throwing a panic.
    let num_cols = dir_row.len();
    if segment_type_row.len() != num_cols
        || fan_in_row.len() != num_cols
        || lane_num_row.len() != num_cols
    {
        return Err(CRRSBParseError::SBHeaderCellParseError(
            "Header rows have inconsistent column counts.".to_string(),
        ));
    }

    // Check that the first four cells in each row exist and are empty.
    if num_cols < 4 {
        return Err(CRRSBParseError::SBHeaderCellParseError(
            "Header rows have invalid number of columns, should be at least 4.".to_string(),
        ));
    }
    for i in 0..4 {
        if !dir_row[i].trim().is_empty()
            || !segment_type_row[i].trim().is_empty()
            || !fan_in_row[i].trim().is_empty()
            || !lane_num_row[i].trim().is_empty()
        {
            return Err(CRRSBParseError::SBHeaderCellParseError(
                "The first 4 cells of the header rows are expected to be empty.".to_string(),
            ));
        }
    }

    // Create a sink node info entry for each column.
    for i in 4..num_cols {
        sink_nodes.push(CRRSwitchSinkNodeInfo {
            dir: parse_crr_switch_dir(dir_row[i].trim())?,
            segment_type: segment_type_row[i].trim().to_string(),
            fan_in: parse_fan_in_cell(fan_in_row[i].trim())?,
            lane_num: parse_crr_lane_num(lane_num_row[i].trim())?,
        });
    }

    Ok(sink_nodes)
}
