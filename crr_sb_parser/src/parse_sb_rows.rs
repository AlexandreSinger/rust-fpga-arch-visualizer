use std::fs::File;

use csv::{StringRecord, StringRecordsIter};

use crate::{CRRSBParseError, crr_sb_des::{CRRSwitchCell, CRRSwitchCellData, CRRSwitchRowHeaderInfo}, parse_common::{parse_crr_lane_num, parse_crr_switch_dir, parse_crr_tap_num}};

fn parse_row_header(row: &StringRecord) -> Result<CRRSwitchRowHeaderInfo, CRRSBParseError> {
    // FIXME: Check that the row has at least 4 columns.

    Ok(CRRSwitchRowHeaderInfo {
        dir: parse_crr_switch_dir(row[0].trim())?,
        segment_type: row[1].trim().to_string(),
        lane_num: parse_crr_lane_num(row[2].trim())?,
        tap_num: parse_crr_tap_num(row[3].trim())?,
    })
}

fn parse_row_cell_data(cell_str: &str) -> Result<CRRSwitchCellData, CRRSBParseError> {
    match cell_str {
        "x" => Ok(CRRSwitchCellData::Connection),
        _ => match cell_str.parse() {
            Ok(delay) => Ok(CRRSwitchCellData::DelaySpecified { delay }),
            Err(e) => Err(CRRSBParseError::SBSWCellParseError(e.to_string())),
        }
    }
}

fn parse_row_cells(row: &StringRecord, row_idx: usize) -> Result<Vec<CRRSwitchCell>, CRRSBParseError> {
    let mut row_cells: Vec<CRRSwitchCell> = Vec::new();
    let num_cols = row.len();

    for i in 4..num_cols {
        let cell_str = row[i].trim();
        if cell_str.is_empty() {
            continue;
        }

        row_cells.push(CRRSwitchCell { 
            row_idx,
            col_idx: i - 4,
            data: parse_row_cell_data(cell_str)?,
        });
    }

    Ok(row_cells)
}

pub fn parse_rows(csv_records: &mut StringRecordsIter<'_, File>) -> Result<(Vec<CRRSwitchRowHeaderInfo>, Vec<CRRSwitchCell>), CRRSBParseError> {
    let mut row_headers: Vec<CRRSwitchRowHeaderInfo> = Vec::new();
    let mut cells: Vec<CRRSwitchCell> = Vec::new();

    let mut row_idx: usize = 0;
    for row in csv_records {
        let row = match row {
            Ok(v) => v,
            Err(e) => {
                return Err(CRRSBParseError::CSVParseError(e.to_string()));
            },
        };
        // FIXME: We somehow need to verify that the rows have the correct length.

        row_headers.push(parse_row_header(&row)?);
        cells.append(&mut parse_row_cells(&row, row_idx)?);

        row_idx += 1;
    }

    Ok((row_headers, cells))
}