use super::*;
use crate::app::agent::config::AuditConfig;

#[test]
fn audit_event_builder_preserves_actor_action_and_result() {
    let event = AuditEvent::new(AuditEventType::CommandExecution)
        .with_actor("agent".into(), Some("user-1".into()), Some("Ada".into()))
        .with_action("ls".into(), "low".into(), true, true)
        .with_result(true, Some(0), 7, None);

    assert_eq!(event.actor.unwrap().channel, "agent");
    assert_eq!(event.action.unwrap().command.as_deref(), Some("ls"));
    assert!(event.result.unwrap().success);
}

#[test]
fn audit_event_defaults_include_safe_security_context() {
    let event = AuditEvent::new(AuditEventType::SecurityEvent);

    assert!(!event.event_id.is_empty());
    assert!(event.actor.is_none());
    assert!(event.action.is_none());
    assert!(event.result.is_none());
    assert!(!event.security.policy_violation);
    assert_eq!(event.security.rate_limit_remaining, None);
    assert_eq!(event.security.sandbox_backend, None);
}

#[test]
fn audit_event_builder_sets_security_backend_and_failure_result() {
    let event = AuditEvent::new(AuditEventType::PolicyViolation)
        .with_security(Some("docker".into()))
        .with_result(false, Some(126), 12, Some("blocked".into()));

    assert_eq!(event.security.sandbox_backend.as_deref(), Some("docker"));
    let result = event.result.unwrap();
    assert!(!result.success);
    assert_eq!(result.exit_code, Some(126));
    assert_eq!(result.duration_ms, Some(12));
    assert_eq!(result.error.as_deref(), Some("blocked"));
}

#[test]
fn disabled_logger_does_not_create_or_write_log_file() {
    let dir = tempfile::tempdir().unwrap();
    let config = AuditConfig { enabled: false, ..AuditConfig::default() };
    let logger = AuditLogger::new(config, dir.path().to_path_buf()).unwrap();

    logger.log(&AuditEvent::new(AuditEventType::AuthSuccess)).unwrap();

    assert!(!dir.path().join("audit.log").exists());
}

#[test]
fn enabled_logger_initializes_nested_log_path_and_writes_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let config = AuditConfig {
        enabled: true,
        log_path: "logs/audit.jsonl".into(),
        max_size_mb: 10,
        ..AuditConfig::default()
    };
    let logger = AuditLogger::new(config, dir.path().to_path_buf()).unwrap();
    let event = AuditEvent::new(AuditEventType::ConfigChange).with_actor(
        "cli".into(),
        None,
        Some("operator".into()),
    );

    logger.log(&event).unwrap();

    let log_path = dir.path().join("logs/audit.jsonl");
    let raw = std::fs::read_to_string(log_path).unwrap();
    let parsed: AuditEvent = serde_json::from_str(raw.trim()).unwrap();
    assert_eq!(parsed.actor.unwrap().username.as_deref(), Some("operator"));
}

#[test]
fn log_command_compat_method_writes_command_execution_event() {
    let dir = tempfile::tempdir().unwrap();
    let config = AuditConfig { enabled: true, max_size_mb: 10, ..AuditConfig::default() };
    let logger = AuditLogger::new(config, dir.path().to_path_buf()).unwrap();

    logger.log_command("discord", "echo ok", "low", true, true, true, 33).unwrap();

    let raw = std::fs::read_to_string(dir.path().join("audit.log")).unwrap();
    let parsed: AuditEvent = serde_json::from_str(raw.trim()).unwrap();
    assert!(matches!(parsed.event_type, AuditEventType::CommandExecution));
    assert_eq!(parsed.actor.unwrap().channel, "discord");
    assert_eq!(parsed.action.unwrap().command.as_deref(), Some("echo ok"));
    assert_eq!(parsed.result.unwrap().duration_ms, Some(33));
}

#[test]
fn logger_rotates_when_existing_file_reaches_limit() {
    let dir = tempfile::tempdir().unwrap();
    let config = AuditConfig { enabled: true, max_size_mb: 0, ..AuditConfig::default() };
    let logger = AuditLogger::new(config, dir.path().to_path_buf()).unwrap();
    let log_path = dir.path().join("audit.log");
    std::fs::write(&log_path, "old\n").unwrap();

    logger.log(&AuditEvent::new(AuditEventType::SecurityEvent)).unwrap();

    assert!(std::path::PathBuf::from(format!("{}.1.log", log_path.display())).exists());
    let new_log = std::fs::read_to_string(log_path).unwrap();
    assert!(new_log.contains("security_event"));
}
