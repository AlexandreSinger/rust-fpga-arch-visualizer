use fpga_arch_parser::FPGAArchParseError;
use std::io::{BufRead, BufReader};

fn get_file_line(file_path: &std::path::Path, line_num: u64) -> Option<String> {
    let file = std::fs::File::open(file_path).ok()?;
    let reader = BufReader::new(file);
    let target = line_num.saturating_sub(1) as usize;
    reader.lines().nth(target).and_then(Result::ok)
}

fn format_context_line(line: &str, column: u64) -> String {
    let column = column as usize;
    let mut result = format!("  {}\n", line);
    let offset = column.saturating_sub(1).min(line.len());
    result.push_str(&format!("  {}{}", " ".repeat(offset), "^"));
    result
}

pub(crate) fn format_parse_error(
    error: &FPGAArchParseError,
    file_path: Option<&std::path::Path>,
) -> String {
    match error {
        FPGAArchParseError::ArchFileOpenError(msg) => {
            format!("Failed to open architecture file:\n{}", msg)
        }
        FPGAArchParseError::MissingRequiredTag(tag) => {
            format!("Missing required XML tag: {}", tag)
        }
        FPGAArchParseError::MissingRequiredAttribute(attr, pos) => {
            let mut msg = format!(
                "Missing required attribute '{}' at line {}, column {}",
                attr,
                pos.row + 1,
                pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::InvalidTag(tag, pos) => {
            let mut msg = format!(
                "Invalid or unexpected tag '{}' at line {}, column {}",
                tag,
                pos.row + 1,
                pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::XMLParseError(msg_text, pos) => {
            let mut msg = format!(
                "XML parsing error at line {}, column {}:\n{}",
                pos.row + 1,
                pos.column + 1,
                msg_text
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::UnknownAttribute(attr, pos) => {
            let mut msg = format!(
                "Unknown attribute '{}' at line {}, column {}",
                attr,
                pos.row + 1,
                pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::DuplicateTag(tag, pos) => {
            let mut msg = format!(
                "Duplicate tag '{}' at line {}, column {}",
                tag,
                pos.row + 1,
                pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::DuplicateAttribute(attr, pos) => {
            let mut msg = format!(
                "Duplicate attribute '{}' at line {}, column {}",
                attr,
                pos.row + 1,
                pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::UnexpectedEndTag(tag, pos) => {
            let mut msg = format!(
                "Unexpected end tag '</{}>' at line {}, column {}",
                tag,
                pos.row + 1,
                pos.column + 1
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::AttributeParseError(msg_text, pos) => {
            let mut msg = format!(
                "Failed to parse attribute at line {}, column {}:\n{}",
                pos.row + 1,
                pos.column + 1,
                msg_text
            );
            if let Some(path) = file_path
                && let Some(line) = get_file_line(path, pos.row + 1)
            {
                msg.push_str("\n\n");
                msg.push_str(&format_context_line(&line, pos.column + 1));
            }
            msg
        }
        FPGAArchParseError::UnexpectedEndOfDocument(msg) => {
            format!("Unexpected end of document:\n{}", msg)
        }
        FPGAArchParseError::PinParsingError(msg) => {
            format!("Pin parsing error:\n{}", msg)
        }
    }
}
