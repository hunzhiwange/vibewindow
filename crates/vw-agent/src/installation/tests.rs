use super::*;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

#[test]
fn method_as_str_is_stable() {
    let cases = [
        (Method::Curl, "curl"),
        (Method::Npm, "npm"),
        (Method::Yarn, "yarn"),
        (Method::Pnpm, "pnpm"),
        (Method::Bun, "bun"),
        (Method::Brew, "brew"),
        (Method::Scoop, "scoop"),
        (Method::Choco, "choco"),
        (Method::Unknown, "unknown"),
    ];

    for (method, name) in cases {
        assert_eq!(method.as_str(), name);
    }
}

#[test]
fn upgrade_failed_error_displays_stderr_only() {
    let err = UpgradeFailedError { stderr: "upgrade failed".into() };
    assert_eq!(err.to_string(), "upgrade failed");
}

#[test]
fn local_version_and_channel_defaults_are_stable() {
    unsafe {
        std::env::remove_var("VIBEWINDOW_VERSION");
        std::env::remove_var("VIBEWINDOW_CHANNEL");
    }

    assert_eq!(version(), "local");
    assert_eq!(channel(), "local");
    assert!(is_local());
    assert!(is_preview());
    assert!(user_agent().starts_with("vibewindow/local/local/"));
}

#[test]
fn upgrade_rejects_unsupported_install_methods_without_running_commands() {
    let unsupported = upgrade(Method::Yarn, "1.2.3").expect_err("yarn cannot upgrade");
    assert_eq!(unsupported.to_string(), "Unknown method: yarn");
    let unsupported = upgrade(Method::Unknown, "1.2.3").expect_err("unknown cannot upgrade");
    assert_eq!(unsupported.to_string(), "Unknown method: unknown");
}

#[test]
fn run_helpers_cover_empty_and_failing_commands() {
    assert_eq!(run_quiet(Vec::new()), "");

    let err = run_cmd(&[], None).expect_err("missing command should be rejected");
    assert_eq!(err.to_string(), "missing command");

    let shell = run_shell("printf stdout; printf stderr >&2; exit 7", None).unwrap();
    assert_eq!(String::from_utf8_lossy(&shell.stdout), "stdout");
    assert_eq!(String::from_utf8_lossy(&shell.stderr), "stderr");
    assert!(!shell.status.success());
}

#[test]
fn extra_maps_include_base_pairs_and_output_streams() {
    #[cfg(unix)]
    let status = std::process::ExitStatus::from_raw(5 << 8);
    #[cfg(windows)]
    let status = std::process::ExitStatus::from_raw(5);

    let output = Output { status, stdout: b"out".to_vec(), stderr: b"err".to_vec() };
    let map = extra_from_output(&output, [("method", Value::String("npm".into()))]);

    assert_eq!(map.get("method"), Some(&Value::String("npm".into())));
    assert_eq!(map.get("stdout"), Some(&Value::String("out".into())));
    assert_eq!(map.get("stderr"), Some(&Value::String("err".into())));
    assert_eq!(map.get("exit_code"), Some(&Value::Number(5.into())));
}

#[test]
fn error_display_delegates_to_inner_errors() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad").unwrap_err();
    let err = Error::from(json_err);
    assert!(!err.to_string().is_empty());

    let utf8_err = String::from_utf8(vec![0xff]).unwrap_err();
    let err = Error::from(utf8_err);
    assert!(err.to_string().contains("invalid utf-8"));
}
