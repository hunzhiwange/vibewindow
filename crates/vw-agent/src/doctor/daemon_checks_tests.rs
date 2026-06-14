use super::Severity;
use super::daemon_checks::check_daemon_state;
use crate::app::agent::config::Config;
use chrono::{Duration as ChronoDuration, Utc};
use tempfile::TempDir;

fn test_config(tmp: &TempDir) -> Config {
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config").join("vibewindow.json"),
        ..Config::default()
    };
    std::fs::create_dir_all(config.config_path.parent().unwrap()).unwrap();
    config
}

fn write_state(config: &Config, value: serde_json::Value) {
    let path = crate::app::agent::daemon::state_file_path(config);
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}

#[test]
fn missing_state_file_reports_daemon_error() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let mut items = Vec::new();

    check_daemon_state(&config, &mut items);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].severity, Severity::Error);
    assert!(items[0].message.contains("state file not found"));
}

#[test]
fn invalid_state_json_is_reported() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let path = crate::app::agent::daemon::state_file_path(&config);
    std::fs::write(path, "{not-json").unwrap();
    let mut items = Vec::new();

    check_daemon_state(&config, &mut items);

    assert!(items.iter().any(|item| {
        item.severity == Severity::Error && item.message.contains("invalid state JSON")
    }));
}

#[test]
fn fresh_daemon_scheduler_and_channel_are_reported_ok() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let now = Utc::now().to_rfc3339();
    write_state(
        &config,
        serde_json::json!({
            "updated_at": now,
            "components": {
                "scheduler": {
                    "status": "ok",
                    "last_ok": now,
                },
                "channel:telegram": {
                    "status": "ok",
                    "last_ok": now,
                }
            }
        }),
    );
    let mut items = Vec::new();

    check_daemon_state(&config, &mut items);

    assert!(
        items.iter().any(|item| {
            item.severity == Severity::Ok && item.message.contains("heartbeat fresh")
        })
    );
    assert!(items.iter().any(|item| {
        item.severity == Severity::Ok && item.message.contains("scheduler healthy")
    }));
    assert!(items.iter().any(|item| {
        item.severity == Severity::Ok && item.message.contains("channel:telegram fresh")
    }));
}

#[test]
fn stale_scheduler_and_channel_are_reported_unhealthy() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let now = Utc::now();
    let stale_daemon = (now - ChronoDuration::seconds(31)).to_rfc3339();
    let stale_scheduler = (now - ChronoDuration::seconds(121)).to_rfc3339();
    let stale_channel = (now - ChronoDuration::seconds(301)).to_rfc3339();
    write_state(
        &config,
        serde_json::json!({
            "updated_at": stale_daemon,
            "components": {
                "scheduler": {
                    "status": "ok",
                    "last_ok": stale_scheduler,
                },
                "channel:slack": {
                    "status": "ok",
                    "last_ok": stale_channel,
                }
            }
        }),
    );
    let mut items = Vec::new();

    check_daemon_state(&config, &mut items);

    assert!(items.iter().any(|item| {
        item.severity == Severity::Error && item.message.contains("heartbeat stale")
    }));
    assert!(items.iter().any(|item| {
        item.severity == Severity::Error && item.message.contains("scheduler unhealthy")
    }));
    assert!(items.iter().any(|item| {
        item.severity == Severity::Error && item.message.contains("channel:slack stale")
    }));
    assert!(items.iter().any(|item| {
        item.severity == Severity::Warn && item.message.contains("1 channels, 1 stale")
    }));
}

#[test]
fn missing_components_emit_compatibility_warnings() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    write_state(
        &config,
        serde_json::json!({
            "updated_at": Utc::now().to_rfc3339(),
            "components": {}
        }),
    );
    let mut items = Vec::new();

    check_daemon_state(&config, &mut items);

    assert!(items.iter().any(|item| {
        item.severity == Severity::Warn && item.message.contains("scheduler component not tracked")
    }));
    assert!(items.iter().any(|item| {
        item.severity == Severity::Warn && item.message.contains("no channel components tracked")
    }));
}

#[test]
fn invalid_daemon_timestamp_is_reported() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    write_state(
        &config,
        serde_json::json!({
            "updated_at": "not-a-date",
            "components": {}
        }),
    );
    let mut items = Vec::new();

    check_daemon_state(&config, &mut items);

    assert!(items.iter().any(|item| {
        item.severity == Severity::Error && item.message.contains("invalid daemon timestamp")
    }));
}
