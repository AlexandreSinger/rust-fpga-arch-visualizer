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
