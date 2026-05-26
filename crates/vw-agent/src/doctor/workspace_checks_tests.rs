use super::workspace_checks::{parse_df_available_mb, workspace_probe_path};

#[test]
fn parse_df_available_mb_reads_last_data_line() {
    let stdout = "Filesystem 1M-blocks Used Available Use% Mounted on\n/dev/disk 100 20 80 20% /";

    assert_eq!(parse_df_available_mb(stdout), Some(80));
    assert_eq!(parse_df_available_mb(""), None);
}

#[test]
fn workspace_probe_path_stays_inside_workspace() {
    let root = tempfile::tempdir().expect("tempdir");
    let probe = workspace_probe_path(root.path());

    assert!(probe.starts_with(root.path()));
    assert!(probe.file_name().unwrap().to_string_lossy().starts_with(".vibewindow_doctor_probe_"));
}
