use std::fs;
use std::process::Command;

fn femfetch() -> Command {
    Command::new(env!("CARGO_BIN_EXE_femfetch"))
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("femfetch-{}-{name}", std::process::id()))
}

#[test]
fn json_output_is_valid_and_versioned() {
    let output = femfetch()
        .args(["--no-config", "--json", "--modules", "os,memory"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["version"], 1);
    assert_eq!(parsed["modules"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["modules"][0]["name"], "os");
}

#[test]
fn cli_theme_overrides_config() {
    let path = temp_path("precedence.toml");
    fs::write(&path, "theme = \"not-a-theme\"\n").unwrap();
    let output = femfetch()
        .args([
            "--config",
            path.to_str().unwrap(),
            "--theme",
            "mint",
            "--plain",
            "--modules",
            "os",
        ])
        .output()
        .unwrap();
    let _ = fs::remove_file(path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn malformed_config_is_a_clean_error() {
    let path = temp_path("malformed.toml");
    fs::write(&path, "compact = maybe\n").unwrap();
    let output = femfetch()
        .args(["--config", path.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = fs::remove_file(path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("expected true or false"));
    assert!(output.stdout.is_empty());
}

#[test]
fn html_export_is_self_contained_and_escaped() {
    let path = temp_path("report.html");
    let output = femfetch()
        .args([
            "--no-config",
            "--modules",
            "os",
            "--export",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let html = fs::read_to_string(&path).unwrap();
    let _ = fs::remove_file(path);
    assert!(html.starts_with("<!doctype html>"));
    assert!(!html.contains("<script"));
    assert!(!html.contains("http://"));
    assert!(!html.contains("https://"));
}

#[test]
fn narrow_plain_output_respects_columns() {
    let output = femfetch()
        .env("COLUMNS", "18")
        .args(["--no-config", "--plain", "--no-color", "--modules", "cpu"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.trim_end().chars().count() <= 18);
}
