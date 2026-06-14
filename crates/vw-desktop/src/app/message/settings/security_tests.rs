use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn normalizes_and_updates_security_settings() {
    assert_eq!(normalize_security_sandbox_enabled(" TRUE "), "true");
    assert_eq!(normalize_security_sandbox_enabled("bad"), "auto");
    assert_eq!(normalize_security_sandbox_backend(" FIREJAIL "), "firejail");
    assert_eq!(normalize_security_sandbox_backend_for_enabled("docker", "false"), "none");
    assert_eq!(normalize_security_sandbox_backend_for_enabled("none", "true"), "auto");
    assert_eq!(normalize_security_otp_method("bad"), "totp");

    let mut app = app();
    let _ = update(&mut app, SettingsMessage::SecuritySandboxEnabledChanged("false".to_string()));
    assert_eq!(app.security_settings.sandbox_backend_input, "none");
    let _ = update(&mut app, SettingsMessage::SecuritySandboxEnabledChanged("true".to_string()));
    let _ = update(&mut app, SettingsMessage::SecuritySandboxBackendChanged("none".to_string()));
    assert_eq!(app.security_settings.sandbox_backend_input, "auto");
    let _ = update(
        &mut app,
        SettingsMessage::SecuritySandboxFirejailArgsChanged("--private".to_string()),
    );
    let _ = update(&mut app, SettingsMessage::SecurityResourcesMaxMemoryMbChanged(1));
    let _ = update(&mut app, SettingsMessage::SecurityResourcesMaxCpuTimeSecondsChanged(999_999));
    let _ = update(&mut app, SettingsMessage::SecurityResourcesMaxSubprocessesChanged(0));
    assert_eq!(app.security_settings.resources_max_memory_mb, 32);
    assert_eq!(app.security_settings.resources_max_cpu_time_seconds, 86_400);
    assert_eq!(app.security_settings.resources_max_subprocesses, 1);
    let _ = update(&mut app, SettingsMessage::SecurityResourcesMemoryMonitoringToggled(false));
    let _ = update(&mut app, SettingsMessage::SecurityAuditEnabledToggled(true));
    let _ =
        update(&mut app, SettingsMessage::SecurityAuditLogPathChanged(" audit.log ".to_string()));
    let _ = update(&mut app, SettingsMessage::SecurityAuditMaxSizeMbChanged(20_000));
    let _ = update(&mut app, SettingsMessage::SecurityAuditSignEventsToggled(true));
    assert_eq!(app.security_settings.audit_max_size_mb, 10_000);

    let _ = update(&mut app, SettingsMessage::SecurityOtpEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::SecurityOtpMethodChanged("bad".to_string()));
    let _ = update(&mut app, SettingsMessage::SecurityOtpTokenTtlSecsChanged(0));
    let _ = update(&mut app, SettingsMessage::SecurityOtpCacheValidSecsChanged(999_999));
    let _ = update(
        &mut app,
        SettingsMessage::SecurityOtpGatedActionsChanged("delete,deploy".to_string()),
    );
    let _ = update(
        &mut app,
        SettingsMessage::SecurityOtpGatedDomainsChanged("example.com".to_string()),
    );
    let _ = update(
        &mut app,
        SettingsMessage::SecurityOtpGatedDomainCategoriesChanged("prod".to_string()),
    );
    assert_eq!(app.security_settings.otp_method_input, "totp");
    assert_eq!(app.security_settings.otp_cache_valid_secs, 86_400);

    let _ = update(&mut app, SettingsMessage::SecurityEstopEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::SecurityEstopStateFileChanged(" state ".to_string()));
    let _ = update(&mut app, SettingsMessage::SecurityEstopRequireOtpToResumeToggled(true));
    let _ = update(&mut app, SettingsMessage::SecuritySyscallAnomalyEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::SecuritySyscallAnomalyStrictModeToggled(true));
    let _ =
        update(&mut app, SettingsMessage::SecuritySyscallAnomalyAlertOnUnknownSyscallToggled(true));
    let _ =
        update(&mut app, SettingsMessage::SecuritySyscallAnomalyMaxDeniedEventsPerMinuteChanged(0));
    let _ = update(
        &mut app,
        SettingsMessage::SecuritySyscallAnomalyMaxTotalEventsPerMinuteChanged(999_999),
    );
    let _ = update(&mut app, SettingsMessage::SecuritySyscallAnomalyMaxAlertsPerMinuteChanged(0));
    let _ = update(&mut app, SettingsMessage::SecuritySyscallAnomalyAlertCooldownSecsChanged(0));
    let _ = update(
        &mut app,
        SettingsMessage::SecuritySyscallAnomalyLogPathChanged(" sys.log ".to_string()),
    );
    let _ = update(
        &mut app,
        SettingsMessage::SecuritySyscallAnomalyBaselineSyscallsChanged("read,write".to_string()),
    );
    assert_eq!(app.security_settings.syscall_anomaly_max_total_events_per_minute, 100_000);
    let _ = update(&mut app, SettingsMessage::SecurityCanaryTokensToggled(true));
    let _ = update(&mut app, SettingsMessage::SecuritySemanticGuardToggled(true));
    let _ = update(
        &mut app,
        SettingsMessage::SecuritySemanticGuardCollectionChanged(" collection ".to_string()),
    );
    let _ = update(&mut app, SettingsMessage::SecuritySemanticGuardThresholdChanged(2.0));
    assert_eq!(app.security_settings.semantic_guard_threshold, 1.0);

    app.security_settings.sandbox_enabled_input = "bad".to_string();
    app.security_settings.otp_method_input = "bad".to_string();
    let _ = update(&mut app, SettingsMessage::SecuritySave);
    assert_eq!(app.security_settings.sandbox_enabled_input, "auto");
    assert_eq!(app.security_settings.otp_method_input, "totp");
    let _ = update(&mut app, SettingsMessage::SecurityHelpOpen);
    assert!(app.security_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::SecurityHelpClose);
    assert!(!app.security_settings.show_help_modal);
}
