use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn parsers_and_runtime_update_paths() {
    assert_eq!(normalize_runtime_kind(" DOCKER "), "docker");
    assert_eq!(normalize_runtime_kind("bad"), "native");
    assert_eq!(normalize_reasoning_enabled("TRUE"), "true");
    assert_eq!(normalize_reasoning_enabled("bad"), "auto");
    assert_eq!(parse_reasoning_level("x-high").unwrap(), Some("xhigh".to_string()));
    assert!(parse_reasoning_level("extreme").is_err());
    assert_eq!(parse_optional_u64("42", "field").unwrap(), Some(42));
    assert!(parse_optional_u64("-1", "field").is_err());
    assert_eq!(parse_optional_f64("1.5", "field").unwrap(), Some(1.5));
    assert!(parse_optional_f64("0", "field").is_err());
    assert_eq!(parse_required_u64("0", "field", 1, 9).unwrap(), 1);
    assert_eq!(parse_csv_lines("a, b\nc"), vec!["a", "b", "c"]);
    assert!(parse_module_sha256_map("bad").is_err());
    assert_eq!(parse_module_sha256_map("m:abc").unwrap().get("m").unwrap(), "abc");

    let mut app = app();
    let _ =
        update(&mut app, SettingsMessage::Runtime(RuntimeMessage::KindChanged("bad".to_string())));
    assert_eq!(app.runtime_settings.kind, "native");
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerImageChanged(" image ".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerNetworkChanged("host".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerMemoryLimitMbChanged("bad".to_string())),
    );
    assert!(app.runtime_settings.save_error.as_deref().unwrap_or("").contains("Docker 内存限制"));
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerMemoryLimitMbChanged("256".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerCpuLimitChanged("2.5".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerReadOnlyRootfsToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerMountWorkspaceToggled(false)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::DockerAllowedWorkspaceRootsChanged(
            "/a,/b".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmToolsDirChanged("tools".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmFuelLimitChanged("0".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmMemoryLimitMbChanged("64".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmMaxModuleSizeMbChanged("32".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmAllowWorkspaceReadToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmAllowWorkspaceWriteToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmAllowedHostsChanged(
            "example.com".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmRequireWorkspaceRelativeToolsDirToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmRejectSymlinkModulesToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmRejectSymlinkToolsDirToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmStrictHostValidationToggled(true)),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmCapabilityEscalationModeChanged(
            "clamp".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmModuleHashPolicyChanged(
            "enforce".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::WasmModuleSha256Changed("bad".to_string())),
    );
    assert!(app.runtime_settings.save_error.as_deref().unwrap_or("").contains("module:sha256"));
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::ReasoningEnabledChanged("TRUE".to_string())),
    );
    let _ = update(
        &mut app,
        SettingsMessage::Runtime(RuntimeMessage::ReasoningLevelChanged("x_high".to_string())),
    );
    assert_eq!(app.runtime_settings.reasoning_enabled_input, "true");
    assert_eq!(app.runtime_settings.wasm_capability_escalation_mode, "clamp");
    app.runtime_settings.save_error = Some("old".to_string());
    let _ = update(&mut app, SettingsMessage::Runtime(RuntimeMessage::Refresh));
    assert!(app.runtime_settings.save_error.is_none());
}
