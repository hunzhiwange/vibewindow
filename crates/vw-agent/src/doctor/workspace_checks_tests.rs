use super::workspace_checks::{check_workspace, parse_df_available_mb, workspace_probe_path};
use super::{DiagItem, Severity};
use crate::app::agent::config::Config;

fn config_for_workspace(path: std::path::PathBuf) -> Config {
    Config { workspace_dir: path, ..Config::default() }
}

#[test]
fn parse_df_available_mb_reads_last_data_line() {
    let stdout = "Filesystem 1M-blocks Used Available Use% Mounted on\n/dev/disk 100 20 80 20% /";

    assert_eq!(parse_df_available_mb(stdout), Some(80));
    assert_eq!(parse_df_available_mb(""), None);
    assert_eq!(
        parse_df_available_mb(
            "Filesystem 1M-blocks Used Available Use% Mounted on\n/dev/disk 100 20 nope 20% /"
        ),
        None
    );
    assert_eq!(parse_df_available_mb("Filesystem 1M-blocks Used Available"), None);
}

#[test]
fn workspace_probe_path_stays_inside_workspace() {
    let root = tempfile::tempdir().expect("tempdir");
    let probe = workspace_probe_path(root.path());

    assert!(probe.starts_with(root.path()));
    assert!(probe.file_name().unwrap().to_string_lossy().starts_with(".vibewindow_doctor_probe_"));
}

#[test]
fn check_workspace_reports_missing_directory_and_stops() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");
    let config = config_for_workspace(missing.clone());
    let mut items = Vec::<DiagItem>::new();

    check_workspace(&config, &mut items);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].severity, Severity::Error);
    assert!(items[0].message.contains(&missing.display().to_string()));
}

#[test]
fn check_workspace_reports_writable_directory_and_optional_docs() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("SOUL.md"), "soul").expect("write soul");
    let config = config_for_workspace(temp.path().to_path_buf());
    let mut items = Vec::<DiagItem>::new();

    check_workspace(&config, &mut items);

    assert!(items.iter().any(|item| {
        item.severity == Severity::Ok && item.message.starts_with("directory exists:")
    }));
    assert!(
        items.iter().any(|item| {
            item.severity == Severity::Ok && item.message == "directory is writable"
        })
    );
    assert!(
        items.iter().any(|item| item.severity == Severity::Ok && item.message == "SOUL.md present")
    );
    assert!(items.iter().any(|item| {
        item.severity == Severity::Warn && item.message == "AGENTS.md not found (optional)"
    }));
    assert!(std::fs::read_dir(temp.path()).expect("read tempdir").flatten().all(|entry| {
        !entry.file_name().to_string_lossy().starts_with(".vibewindow_doctor_probe_")
    }));
}
