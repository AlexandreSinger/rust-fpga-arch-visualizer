//! # CRR Switch Block Parser
//!
//! A parser for Custom Routing Resource (CRR) switch block specifications defined in CSV format.
//!
//! This library provides functionality to parse and deserialize CSV files that describe the
//! connectivity of Custom Routing Resource switch blocks, which are fundamental components
//! in FPGA architecture. Each switch block definition specifies:
//!
//! - **Sink nodes**: the destination connection points with information about their direction,
//!   segment type, fan-in, and lane number
//! - **Source nodes**: the source connection points with their direction, segment type, tap number,
//!   and lane number
//! - **Connections**: the routing edges between source and sink nodes, including optional
//!   delay specifications
//!
//! ## Example
//!
//! ```
//! use crr_sb_parser::parse_csv_file;
//! use std::path::Path;
//!
//! let path = Path::new("switch_block.csv");
//! match parse_csv_file(path) {
//!     Ok(sb) => {
//!         println!("Parsed {} sink nodes", sb.sink_nodes.len());
//!         println!("Parsed {} source nodes", sb.source_nodes.len());
//!         println!("Parsed {} connections", sb.edges.len());
//!     },
//!     Err(e) => eprintln!("Parse error: {:?}", e),
//! }
//! ```

use std::{fs::File, path::Path};

mod crr_sb_des;
mod crr_sb_parse_error;
mod parse_common;
mod parse_sb_rows;
mod parse_sink_nodes;

pub use crate::crr_sb_des::*;
pub use crate::crr_sb_parse_error::CRRSBParseError;
use crate::{parse_sb_rows::parse_rows, parse_sink_nodes::parse_sink_nodes};

/// Parses a CSV file containing Custom Routing Resource (CRR) switch block definitions.
///
/// This function reads and parses a CSV file that specifies a switch block's connectivity,
/// extracting sink nodes, source nodes, and the connections between them. The CSV format
/// includes column headers describing sink nodes and row headers describing source nodes,
/// with the matrix cells containing connectivity information and optional delay values.
///
/// # Arguments
///
/// * `csv_file_path` - A path to the CSV file to parse
///
/// # Returns
///
/// * `Ok(CRRSwitchBlockDeserialized)` - A structure containing parsed switch block data with
///   sink nodes, source nodes, and their connections
/// * `Err(CRRSBParseError)` - A parse error if the file cannot be opened, read, or properly parsed
///
/// # Errors
///
/// This function returns an error if:
/// - The CSV file cannot be opened (file not found, permission denied, etc.)
/// - The CSV format is invalid or malformed
/// - The header or cell contents cannot be parsed into the expected data structures
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
