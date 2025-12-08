use xml::common::TextPosition;

#[derive(Debug)]
pub enum FPGAArchParseError {
    ArchFileOpenError(String),
    MissingRequiredTag(String),
    MissingRequiredAttribute(String, TextPosition),
    InvalidTag(String, TextPosition),
    XMLParseError(String, TextPosition),
    UnknownAttribute(String, TextPosition),
    DuplicateTag(String, TextPosition),
    DuplicateAttribute(String, TextPosition),
    UnexpectedEndTag(String, TextPosition),
    AttributeParseError(String, TextPosition),
    UnexpectedEndOfDocument(String),
}
