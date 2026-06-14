use super::windows::{
    install_windows, start_windows, status_windows, stop_windows, uninstall_windows,
    windows_task_name,
};

fn temp_config() -> (tempfile::TempDir, crate::app::agent::config::Config) {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut config = crate::app::agent::config::Config::default();
    config.config_path = dir.path().join("vibewindow.json");
    (dir, config)
}

#[test]
fn windows_task_name_is_stable() {
    assert_eq!(windows_task_name(), "VibeWindow Daemon");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn install_windows_writes_wrapper_before_schtasks_failure() {
    let (_dir, config) = temp_config();
    let result = install_windows(&config);

    let logs_dir = config.config_path.parent().unwrap().join("logs");
    let wrapper = logs_dir.join("vibewindow-daemon.cmd");
    let stdout_log = logs_dir.join("daemon.stdout.log");
    let stderr_log = logs_dir.join("daemon.stderr.log");
    let content = std::fs::read_to_string(&wrapper).expect("wrapper should be written");
    let exe = std::env::current_exe().expect("current exe");

    assert!(result.is_err(), "non-Windows hosts should not have schtasks available");
    assert_eq!(
        content,
        format!(
            "@echo off\r\n\"{}\" daemon >>\"{}\" 2>>\"{}\"",
            exe.display(),
            stdout_log.display(),
            stderr_log.display()
        )
    );
}

#[cfg(not(target_os = "windows"))]
#[test]
fn start_windows_reports_spawn_failure_when_schtasks_is_missing() {
    let err = start_windows().expect_err("schtasks should be unavailable off Windows");
    assert!(err.to_string().contains("Failed to spawn command"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn stop_and_status_windows_are_idempotent_when_schtasks_is_missing() {
    stop_windows().expect("stop ignores command failure");
    status_windows().expect("status maps command failure to not installed output");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn uninstall_windows_removes_existing_wrapper_and_ignores_scheduler_errors() {
    let (_dir, config) = temp_config();
    let wrapper = config.config_path.parent().unwrap().join("logs").join("vibewindow-daemon.cmd");
    std::fs::create_dir_all(wrapper.parent().unwrap()).expect("logs dir");
    std::fs::write(&wrapper, "@echo off\r\n").expect("wrapper fixture");

    uninstall_windows(&config).expect("uninstall should ignore scheduler delete errors");

    assert!(!wrapper.exists());
}
