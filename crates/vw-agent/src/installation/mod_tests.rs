use super::*;
use serde::Deserialize;
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(unix)]
fn write_executable(dir: &Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(name);
    std::fs::write(&path, body).expect("write fake executable");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("chmod fake executable");
}

#[cfg(unix)]
fn prepend_path(dir: &Path) -> std::ffi::OsString {
    let old_path = std::env::var_os("PATH");
    let mut paths = vec![dir.to_path_buf()];
    if let Some(old_path) = old_path {
        paths.extend(std::env::split_paths(&old_path));
    }
    std::env::join_paths(paths).expect("join PATH")
}

fn one_shot_json_server(status: &str, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local server");
    let addr = listener.local_addr().expect("server addr");
    let status = status.to_string();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut request = [0_u8; 2048];
        let _ = stream.read(&mut request);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    });
    format!("http://{addr}")
}

#[test]
fn info_round_trips_through_json() {
    let info = Info { version: "1.2.3".to_string(), latest: "1.2.4".to_string() };

    let encoded = serde_json::to_value(&info).expect("serialize info");
    assert_eq!(encoded, json!({ "version": "1.2.3", "latest": "1.2.4" }));

    let decoded: Info = serde_json::from_value(encoded).expect("deserialize info");
    assert_eq!(decoded.version, "1.2.3");
    assert_eq!(decoded.latest, "1.2.4");
}

#[test]
fn event_definitions_are_stable() {
    assert_eq!(event::UPDATED.r#type, "installation.updated");
    assert_eq!(event::UPDATE_AVAILABLE.r#type, "installation.update-available");
}

#[test]
fn run_shell_applies_environment_overrides() {
    let output = run_shell(
        "printf '%s' \"$VW_INSTALLATION_TEST_VALUE\"",
        Some(&[("VW_INSTALLATION_TEST_VALUE", "ok")]),
    )
    .expect("shell output");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).expect("utf8 stdout"), "ok");
}

#[test]
fn run_cmd_surfaces_missing_binary_as_io_error() {
    let err = run_cmd(&["definitely-not-a-vibewindow-test-command"], None)
        .expect_err("missing command should fail");

    assert!(matches!(err, Error::Io(_)));
    assert!(!err.to_string().is_empty());
}

#[cfg(unix)]
#[test]
fn method_detects_package_manager_from_fake_global_list() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_path = std::env::var_os("PATH");
    let dir = tempfile::tempdir().expect("tempdir");
    write_executable(dir.path(), "npm", "#!/bin/sh\nprintf 'vibewindow-ai@1.2.3\\n'\n");
    unsafe {
        std::env::set_var("PATH", prepend_path(dir.path()));
    }

    assert_eq!(method(), Method::Npm);

    unsafe {
        if let Some(old_path) = old_path {
            std::env::set_var("PATH", old_path);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[cfg(unix)]
#[test]
fn npm_registry_trims_trailing_slash_from_fake_npm() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_path = std::env::var_os("PATH");
    let dir = tempfile::tempdir().expect("tempdir");
    write_executable(
        dir.path(),
        "npm",
        "#!/bin/sh\nif [ \"$1\" = config ]; then printf 'https://registry.example.test/\\n'; fi\n",
    );
    unsafe {
        std::env::set_var("PATH", prepend_path(dir.path()));
    }

    assert_eq!(npm_registry().expect("registry"), "https://registry.example.test");

    unsafe {
        if let Some(old_path) = old_path {
            std::env::set_var("PATH", old_path);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[cfg(unix)]
#[test]
fn latest_reads_npm_dist_tag_from_configured_registry() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_path = std::env::var_os("PATH");
    let old_channel = std::env::var_os("VIBEWINDOW_CHANNEL");
    let old_registry = std::env::var_os("VW_INSTALLATION_TEST_REGISTRY");
    let registry = one_shot_json_server("200 OK", r#"{"version":"7.8.9"}"#);
    let dir = tempfile::tempdir().expect("tempdir");
    write_executable(
        dir.path(),
        "npm",
        "#!/bin/sh\nif [ \"$1\" = config ]; then printf '%s/\\n' \"$VW_INSTALLATION_TEST_REGISTRY\"; fi\n",
    );
    unsafe {
        std::env::set_var("PATH", prepend_path(dir.path()));
        std::env::set_var("VIBEWINDOW_CHANNEL", "beta");
        std::env::set_var("VW_INSTALLATION_TEST_REGISTRY", registry);
    }

    assert_eq!(latest(Some(Method::Npm)).expect("latest npm version"), "7.8.9");

    unsafe {
        if let Some(old_path) = old_path {
            std::env::set_var("PATH", old_path);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(old_channel) = old_channel {
            std::env::set_var("VIBEWINDOW_CHANNEL", old_channel);
        } else {
            std::env::remove_var("VIBEWINDOW_CHANNEL");
        }
        if let Some(old_registry) = old_registry {
            std::env::set_var("VW_INSTALLATION_TEST_REGISTRY", old_registry);
        } else {
            std::env::remove_var("VW_INSTALLATION_TEST_REGISTRY");
        }
    }
}

#[cfg(unix)]
#[test]
fn latest_reads_brew_tap_formula_json_from_fake_brew() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_path = std::env::var_os("PATH");
    let dir = tempfile::tempdir().expect("tempdir");
    write_executable(
        dir.path(),
        "brew",
        r#"#!/bin/sh
case "$*" in
  "list --formula anomalyco/tap/vibewindow") printf 'vibewindow\n' ;;
  "info --json=v2 anomalyco/tap/vibewindow") printf '{"formulae":[{"versions":{"stable":"9.8.7"}}]}\n' ;;
  *) exit 1 ;;
esac
"#,
    );
    unsafe {
        std::env::set_var("PATH", prepend_path(dir.path()));
    }

    assert_eq!(latest(Some(Method::Brew)).expect("brew latest"), "9.8.7");

    unsafe {
        if let Some(old_path) = old_path {
            std::env::set_var("PATH", old_path);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[cfg(unix)]
#[test]
fn upgrade_runs_package_manager_command_and_records_failure_stderr() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_path = std::env::var_os("PATH");
    let old_args = std::env::var_os("VW_INSTALLATION_TEST_ARGS");
    let dir = tempfile::tempdir().expect("tempdir");
    let args_path = dir.path().join("npm.args");
    write_executable(
        dir.path(),
        "npm",
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$VW_INSTALLATION_TEST_ARGS\"\n",
    );
    unsafe {
        std::env::set_var("PATH", prepend_path(dir.path()));
        std::env::set_var("VW_INSTALLATION_TEST_ARGS", &args_path);
    }

    upgrade(Method::Npm, "1.2.3").expect("npm upgrade");
    assert_eq!(
        std::fs::read_to_string(&args_path).expect("args"),
        "install\n-g\nvibewindow-ai@1.2.3\n"
    );

    write_executable(dir.path(), "npm", "#!/bin/sh\nprintf 'package failed' >&2\nexit 9\n");
    let err = upgrade(Method::Npm, "2.0.0").expect_err("npm failure");
    assert!(matches!(err, Error::UpgradeFailed(_)));
    assert_eq!(err.to_string(), "package failed");

    unsafe {
        if let Some(old_path) = old_path {
            std::env::set_var("PATH", old_path);
        } else {
            std::env::remove_var("PATH");
        }
        if let Some(old_args) = old_args {
            std::env::set_var("VW_INSTALLATION_TEST_ARGS", old_args);
        } else {
            std::env::remove_var("VW_INSTALLATION_TEST_ARGS");
        }
    }
}

#[cfg(unix)]
#[test]
fn upgrade_choco_failure_uses_elevation_message() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let old_path = std::env::var_os("PATH");
    let dir = tempfile::tempdir().expect("tempdir");
    write_executable(dir.path(), "choco", "#!/bin/sh\nprintf 'raw stderr' >&2\nexit 5\n");
    unsafe {
        std::env::set_var("PATH", prepend_path(dir.path()));
    }

    let err = upgrade(Method::Choco, "1.0.0").expect_err("choco failure");
    assert_eq!(err.to_string(), "not running from an elevated command shell");

    unsafe {
        if let Some(old_path) = old_path {
            std::env::set_var("PATH", old_path);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[test]
fn http_get_json_decodes_success_and_surfaces_status_errors() {
    #[derive(Debug, Deserialize)]
    struct Payload {
        value: String,
    }

    let ok_url = one_shot_json_server("200 OK", r#"{"value":"decoded"}"#);
    let payload: Payload = http_get_json(&ok_url, Some(&[("X-Test", "yes")])).expect("json");
    assert_eq!(payload.value, "decoded");

    let err_url = one_shot_json_server("500 Internal Server Error", r#"{"error":"nope"}"#);
    let err = http_get_json::<Payload>(&err_url, None).expect_err("status error");
    assert!(matches!(err, Error::Http(_)));
}
