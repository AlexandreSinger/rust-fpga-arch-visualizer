use std::process::Command;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_fpga_arch_viewer")
}

fn valid_arch() -> &'static str {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fpga_arch_parser/tests/k4_N4_90nm.xml"
    )
}

fn invalid_arch() -> &'static str {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/invalid_arch.xml"
    )
}

#[test]
fn parse_only_valid_exits_zero() {
    let status = Command::new(binary())
        .args(["--parse-only", valid_arch()])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn parse_only_valid_prints_success() {
    let output = Command::new(binary())
        .args(["--parse-only", valid_arch()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Successfully parsed"));
}

#[test]
fn parse_only_invalid_exits_nonzero() {
    let status = Command::new(binary())
        .args(["--parse-only", invalid_arch()])
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn parse_only_invalid_reports_error_to_stderr() {
    let output = Command::new(binary())
        .args(["--parse-only", invalid_arch()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Parse error"));
}

#[test]
fn parse_only_missing_file_exits_nonzero() {
    let status = Command::new(binary())
        .args(["--parse-only", "/nonexistent/path/arch.xml"])
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn parse_only_flag_without_path_exits_nonzero() {
    let status = Command::new(binary())
        .arg("--parse-only")
        .status()
        .unwrap();
    assert!(!status.success());
}
