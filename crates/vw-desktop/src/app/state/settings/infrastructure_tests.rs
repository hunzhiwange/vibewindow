use super::*;

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("infrastructure_tests"));
}

#[test]
fn multimodal_and_http_request_defaults_match_config_ui() {
    let multimodal = MultimodalSettingsState::default();
    assert_eq!(multimodal.max_images, 4);
    assert_eq!(multimodal.max_image_size_mb, 5);
    assert!(!multimodal.allow_remote_fetch);
    assert!(multimodal.save_error.is_none());

    let http = HttpRequestSettingsState::default();
    assert!(!http.enabled);
    assert!(http.allowed_domains.is_empty());
    assert!(http.new_allowed_domain_input.is_empty());
    assert_eq!(http.max_response_size, 1_000_000);
    assert_eq!(http.timeout_secs, 30);
    assert_eq!(http.user_agent, HttpRequestConfig::default().user_agent);
    assert!(http.save_error.is_none());
}

#[test]
fn acp_security_and_autonomy_defaults_are_populated() {
    let acp = AcpSettingsState::default();
    assert!(acp.catalog.is_empty());
    assert!(acp.enabled.is_empty());
    assert!(!acp.loading);
    assert!(acp.saving_agent.is_none());
    assert!(acp.save_error.is_none());
    assert!(acp.status_message.is_none());

    let security = SecuritySettingsState::default();
    assert_eq!(security.sandbox_enabled_input, "auto");
    assert_eq!(security.sandbox_backend_input, "auto");
    assert_eq!(security.resources_max_memory_mb, 512);
    assert_eq!(security.resources_max_cpu_time_seconds, 60);
    assert_eq!(security.resources_max_subprocesses, 10);
    assert!(security.resources_memory_monitoring);
    assert!(security.audit_enabled);
    assert_eq!(security.audit_log_path, "audit.log");
    assert_eq!(security.audit_max_size_mb, 100);
    assert!(!security.audit_sign_events);
    assert!(!security.otp_enabled);
    assert_eq!(security.otp_method_input, "totp");
    assert_eq!(security.otp_token_ttl_secs, 30);
    assert_eq!(security.otp_cache_valid_secs, 300);
    assert!(security.otp_gated_actions_input.contains("shell"));
    assert!(!security.estop_enabled);
    assert_eq!(security.estop_state_file, vw_config_types::paths::ESTOP_STATE_FILE_PATH);
    assert!(security.estop_require_otp_to_resume);
    assert!(security.syscall_anomaly_enabled);
    assert!(!security.syscall_anomaly_strict_mode);
    assert!(security.syscall_anomaly_alert_on_unknown_syscall);
    assert_eq!(security.syscall_anomaly_max_denied_events_per_minute, 5);
    assert_eq!(security.syscall_anomaly_max_total_events_per_minute, 120);
    assert_eq!(security.syscall_anomaly_max_alerts_per_minute, 30);
    assert_eq!(security.syscall_anomaly_alert_cooldown_secs, 20);
    assert_eq!(security.syscall_anomaly_log_path, "syscall-anomalies.log");
    assert!(security.canary_tokens);
    assert!(!security.semantic_guard);
    assert_eq!(security.semantic_guard_collection, "semantic_guard");
    assert_eq!(security.semantic_guard_threshold, 0.82);
    assert!(!security.show_help_modal);

    let autonomy = AutonomySettingsState::default();
    assert_eq!(autonomy.level, vw_config_types::security::AutonomyLevel::Supervised);
    assert!(autonomy.workspace_only);
    assert!(autonomy.allowed_commands_input.contains("cargo"));
    assert!(autonomy.forbidden_paths_input.contains("/etc"));
    assert_eq!(autonomy.max_actions_per_hour, 20);
    assert_eq!(autonomy.max_cost_per_day_cents, 500);
    assert!(autonomy.require_approval_for_medium_risk);
    assert!(autonomy.block_high_risk_commands);
    assert_eq!(
        autonomy.shell_redirect_policy,
        vw_config_types::security::ShellRedirectPolicy::Block
    );
    assert!(autonomy.auto_approve_input.contains("file_read"));
    assert!(autonomy.non_cli_excluded_tools_input.contains("shell"));
    assert_eq!(
        autonomy.non_cli_natural_language_approval_mode,
        vw_config_types::security::NonCliNaturalLanguageApprovalMode::Direct
    );
    assert!(!autonomy.show_help_modal);
    assert!(autonomy.save_error.is_none());
}

#[test]
fn observability_storage_proxy_and_tunnel_defaults_are_stable() {
    let observability = ObservabilitySettingsState::default();
    assert_eq!(observability.backend, "none");
    assert_eq!(observability.runtime_trace_mode, "none");
    assert_eq!(observability.runtime_trace_path_input, "state/runtime-trace.jsonl");
    assert_eq!(observability.runtime_trace_max_entries, 200);
    assert!(!observability.show_help_modal);

    let storage = StorageSettingsState::default();
    assert!(storage.provider.is_empty());
    assert_eq!(storage.schema, "public");
    assert_eq!(storage.table, "memories");
    assert!(!storage.tls);
    assert!(storage.save_error.is_none());

    let proxy = ProxySettingsState::default();
    assert!(!proxy.enabled);
    assert!(proxy.http_proxy.is_empty());
    assert!(proxy.https_proxy.is_empty());
    assert!(proxy.all_proxy.is_empty());
    assert_eq!(proxy.scope, vw_config_types::proxy::ProxyScope::Vibewindow);
    assert!(!proxy.show_help_modal);

    let tunnel = TunnelSettingsState::default();
    assert_eq!(tunnel.provider, "none");
    assert!(!tunnel.tailscale_funnel);
    assert!(tunnel.cloudflare_token.is_empty());
    assert!(tunnel.custom_url_pattern.is_empty());
    assert!(tunnel.save_error.is_none());
}

#[test]
fn composio_and_transcription_defaults_are_ready_for_empty_config() {
    let composio = ComposioSettingsState::default();
    assert!(!composio.enabled);
    assert!(composio.api_key_input.is_empty());
    assert_eq!(composio.entity_id_input, "default");
    assert!(composio.save_error.is_none());

    let transcription = TranscriptionSettingsState::default();
    assert!(!transcription.enabled);
    assert_eq!(transcription.api_url, "https://api.groq.com/openai/v1/audio/transcriptions");
    assert_eq!(transcription.model, "whisper-large-v3-turbo");
    assert!(transcription.language.is_empty());
    assert_eq!(transcription.max_duration_secs, 120);
    assert!(!transcription.show_help_modal);
    assert!(transcription.save_error.is_none());
}
