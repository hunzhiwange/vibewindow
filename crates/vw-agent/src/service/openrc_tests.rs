use super::openrc::generate_openrc_script;

#[cfg(unix)]
use super::openrc::{
    build_openrc_writability_probe_command, current_uid, install_linux_openrc, is_root,
    shell_single_quote,
};

#[test]
fn generate_openrc_script_contains_runtime_user_paths_and_logs() {
    let script =
        generate_openrc_script("/usr/local/bin/vibewindow".as_ref(), "/etc/vibewindow".as_ref());

    assert!(script.starts_with("#!/sbin/openrc-run\n"));
    assert!(script.contains("name=\"vibewindow\""));
    assert!(script.contains("description=\"VibeWindow daemon\""));
    assert!(script.contains("command=\"/usr/local/bin/vibewindow\""));
    assert!(script.contains("command_args=\"--config-dir /etc/vibewindow daemon\""));
    assert!(script.contains("command_background=\"yes\""));
    assert!(script.contains("command_user=\"vibewindow:vibewindow\""));
    assert!(script.contains("pidfile=\"/run/${RC_SVCNAME}.pid\""));
    assert!(script.contains("umask 027"));
    assert!(script.contains("output_log=\"/var/log/vibewindow/access.log\""));
    assert!(script.contains("error_log=\"/var/log/vibewindow/error.log\""));
    assert!(script.contains("depend()"));
    assert!(script.contains("need net"));
    assert!(script.contains("after firewall"));
}

#[test]
fn generate_openrc_script_inlines_supplied_paths() {
    let script = generate_openrc_script(
        "/opt/Vibe Window/bin/vibewindow".as_ref(),
        "/etc/vibe window/config".as_ref(),
    );

    assert!(script.contains("command=\"/opt/Vibe Window/bin/vibewindow\""));
    assert!(script.contains("command_args=\"--config-dir /etc/vibe window/config daemon\""));
    assert!(!script.contains("VIBEWINDOW_CONFIG_DIR"));
    assert!(!script.contains("VIBEWINDOW_WORKSPACE"));
}

#[cfg(unix)]
#[test]
fn shell_single_quote_wraps_empty_plain_and_apostrophe_values() {
    assert_eq!(shell_single_quote(""), "''");
    assert_eq!(shell_single_quote("/etc/vibewindow"), "'/etc/vibewindow'");
    assert_eq!(shell_single_quote("/tmp/one two"), "'/tmp/one two'");
    assert_eq!(shell_single_quote("/tmp/weird'path"), "'/tmp/weird'\"'\"'path'");
    assert_eq!(shell_single_quote("a'b'c"), "'a'\"'\"'b'\"'\"'c'");
}

#[cfg(unix)]
#[test]
fn build_probe_command_uses_runuser_when_available() {
    let (program, args) =
        build_openrc_writability_probe_command("/etc/vibewindow/work space".as_ref(), true);

    assert_eq!(program, "runuser");
    assert_eq!(
        args,
        vec![
            "-u".to_string(),
            "vibewindow".to_string(),
            "--".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            "test -w '/etc/vibewindow/work space'".to_string(),
        ]
    );
}

#[cfg(unix)]
#[test]
fn build_probe_command_falls_back_to_su_with_quoted_path() {
    let (program, args) =
        build_openrc_writability_probe_command("/tmp/vibe'window".as_ref(), false);

    assert_eq!(program, "su");
    assert_eq!(
        args,
        vec![
            "-s".to_string(),
            "/bin/sh".to_string(),
            "-c".to_string(),
            "test -w '/tmp/vibe'\"'\"'window'".to_string(),
            "vibewindow".to_string(),
        ]
    );
}

#[cfg(unix)]
#[test]
fn root_detection_matches_current_uid() {
    assert_eq!(is_root(), current_uid() == Some(0));
}

#[cfg(unix)]
#[test]
fn openrc_install_requires_root_before_touching_system_paths() {
    if is_root() {
        return;
    }

    let err = install_linux_openrc(&crate::app::agent::config::Config::default())
        .expect_err("non-root OpenRC install should stop before side effects");

    assert!(err.to_string().contains("requires root privileges"));
    assert!(err.to_string().contains("sudo vibewindow service install"));
}
