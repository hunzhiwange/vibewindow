#[test]
fn cleanup_stats_ignores_targets_covered_by_existing_directory() {
    let mut stats = super::CleanupStats::default();
    stats.track_directory("/tmp");
    stats.track_matching_files("/tmp", &["log"]);

    assert_eq!(stats.targets.len(), 1);
    assert_eq!(stats.targets[0].kind, super::ScanDetailKind::Directory);
}

#[test]
fn cleanup_stats_summary_lines_use_formatted_totals() {
    let stats = super::CleanupStats::default();

    assert_eq!(stats.summary_line(), "本次预计清理垃圾数据：0 B");
    assert_eq!(stats.actual_removed_line(), "本次实际删除垃圾数据：0 B");
}

#[test]
fn cleanup_stats_replaces_covered_child_with_parent_directory() {
    let mut stats = super::CleanupStats::default();
    stats.track_matching_files("/tmp/vw-cleaner", &["dmg"]);
    stats.track_directory("/tmp/vw-cleaner");

    assert_eq!(stats.targets.len(), 1);
    assert_eq!(stats.targets[0].kind, super::ScanDetailKind::Directory);
}

#[test]
fn cleanup_target_covers_matching_extension_target_inside_directory() {
    let target = super::CleanupTarget {
        path: std::path::PathBuf::from("/tmp/vw-cleaner"),
        kind: super::ScanDetailKind::Directory,
        before_bytes: 0,
    };

    assert!(target.covers(
        std::path::Path::new("/tmp/vw-cleaner/downloads"),
        super::ScanDetailKind::FileExtensions(&["zip"])
    ));
}

#[test]
fn command_output_formats_success_stdout_and_stderr() {
    let output = std::process::Output {
        status: successful_status(),
        stdout: b"removed cache\n".to_vec(),
        stderr: b"warning\n".to_vec(),
    };

    assert_eq!(
        super::command_output("用户目录清理", &output),
        "用户目录清理: 成功\nstdout:\nremoved cache\nstderr:\nwarning"
    );
}

#[test]
fn command_output_formats_failure_without_streams() {
    let output =
        std::process::Output { status: failed_status(), stdout: Vec::new(), stderr: Vec::new() };

    assert_eq!(super::command_output("系统目录清理", &output), "系统目录清理: 失败");
}

#[test]
fn escape_applescript_escapes_quotes_and_backslashes() {
    assert_eq!(super::escape_applescript(r#"rm "a\b""#), r#"rm \"a\\b\""#);
}

#[test]
fn quote_for_single_argument_doubles_embedded_single_quotes() {
    assert_eq!(super::quote_for_single_argument("it's ok"), "'it''s ok'");
}

#[test]
fn cancelled_publishes_log_and_returns_true() {
    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let progress = std::sync::Arc::new(parking_lot::Mutex::new(String::new()));
    let mut log = vec!["开始".to_string()];

    assert!(super::cancelled(&cancel_flag, &mut log, &progress));
    assert_eq!(log.last().map(String::as_str), Some("清理任务已取消，已停止后续步骤。"));
    assert!(progress.lock().contains("清理任务已取消"));
}

#[test]
fn publish_log_joins_entries_with_blank_lines() {
    let progress = std::sync::Arc::new(parking_lot::Mutex::new(String::new()));

    super::publish_log(&["one".to_string(), "two".to_string()], &progress);

    assert_eq!(&*progress.lock(), "one\n\ntwo");
}

#[test]
fn format_bytes_uses_binary_units() {
    assert_eq!(super::format_bytes(0), "0 B");
    assert_eq!(super::format_bytes(1023), "1023 B");
    assert_eq!(super::format_bytes(1024), "1.00 KB");
    assert_eq!(super::format_bytes(5 * 1024 * 1024), "5.00 MB");
}

#[test]
fn execute_cleanup_reports_unsupported_platform_on_linux() {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        return;
    }

    let request: vw_api_types::cleaner::CleanerCleanupRequest =
        serde_json::from_value(serde_json::json!({})).expect("default cleanup request");
    let result = super::execute_cleanup(
        request,
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        std::sync::Arc::new(parking_lot::Mutex::new(String::new())),
    );

    assert_eq!(result, Err(super::unsupported_platform_message()));
}

#[cfg(unix)]
fn successful_status() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(0)
}

#[cfg(unix)]
fn failed_status() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(256)
}

#[cfg(windows)]
fn successful_status() -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(0)
}

#[cfg(windows)]
fn failed_status() -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(1)
}
