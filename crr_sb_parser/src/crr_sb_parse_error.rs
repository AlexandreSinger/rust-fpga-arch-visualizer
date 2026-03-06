use std::fmt;

#[derive(Debug)]
pub enum CRRSBParseError {
    SBFileOpenError(String),
    SBHeaderRowMissing(String),
    SBHeaderColMissing(String),
    // TODO: We should create an error location struct for the csv cell x,y location.
    SBHeaderCellParseError(String),
    SBSWCellParseError(String),
    CSVParseError(String),
}

impl fmt::Display for CRRSBParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SBFileOpenError(msg) => write!(f, "Failed to open file: {msg}"),
            Self::SBHeaderRowMissing(msg) => write!(f, "Header row missing: {msg}"),
            Self::SBHeaderColMissing(msg) => write!(f, "Header column missing: {msg}"),
            Self::SBHeaderCellParseError(msg) => write!(f, "Header cell parse error: {msg}"),
            Self::SBSWCellParseError(msg) => write!(f, "Cell parse error: {msg}"),
            Self::CSVParseError(msg) => write!(f, "CSV parse error: {msg}"),
        }
    }
}

impl std::error::Error for CRRSBParseError {}
