use std::{fs::File, path::Path};

mod crr_sb_des;
mod crr_sb_parse_error;
mod parse_sb_col_headers;
mod parse_sb_rows;
mod parse_common;

pub use crate::crr_sb_des::*;
pub use crate::crr_sb_parse_error::CRRSBParseError;
use crate::{parse_sb_col_headers::parse_sink_nodes, parse_sb_rows::parse_rows};

pub fn parse_csv_file(csv_file_path: &Path) -> Result<CRRSwitchBlockDeserialized, CRRSBParseError> {
    // Try to open the file.
    let file = File::open(csv_file_path);
    let file = match file {
        Ok(f) => f,
        Err(error) => return Err(CRRSBParseError::SBFileOpenError(format!("{error:?}"))),
    };

    // Create a reader.
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    // Get an iterator to the records.
    let mut csv_records = rdr.records();

    // Parse the column headers, row headers, and cells.
    let sink_nodes = parse_sink_nodes(&mut csv_records)?;
    let (source_nodes, edges) = parse_rows(&mut csv_records)?;
    Ok(CRRSwitchBlockDeserialized {
        sink_nodes,
        source_nodes,
        edges,
    })
}