use super::state::{EnabledCheck, FormatterInfo, FormatterStatus, State};
use std::collections::HashMap;
use std::sync::Mutex;

#[test]
fn state_can_hold_formatter_info_and_status() {
    let formatter = FormatterInfo {
        name: "demo".to_string(),
        command: vec!["demo".to_string()],
        environment: HashMap::new(),
        extensions: vec![".demo".to_string()],
        enabled: EnabledCheck::Always,
    };
    let state = State {
        enabled: Mutex::new(HashMap::new()),
        formatters: HashMap::from([("demo".to_string(), formatter)]),
    };
    let status = FormatterStatus {
        name: "demo".to_string(),
        extensions: vec![".demo".to_string()],
        enabled: true,
    };

    assert!(state.formatters.contains_key("demo"));
    assert_eq!(serde_json::to_value(status).expect("serialize")["enabled"], true);
}

#[test]
fn formatter_status_round_trips_through_json() {
    let status = FormatterStatus {
        name: "rustfmt".to_string(),
        extensions: vec![".rs".to_string()],
        enabled: false,
    };

    let json = serde_json::to_string(&status).expect("serialize status");
    let parsed: FormatterStatus = serde_json::from_str(&json).expect("deserialize status");

    assert_eq!(parsed.name, "rustfmt");
    assert_eq!(parsed.extensions, vec![".rs"]);
    assert!(!parsed.enabled);
}

#[test]
fn state_default_starts_empty() {
    let state = State::default();

    assert!(state.enabled.lock().unwrap().is_empty());
    assert!(state.formatters.is_empty());
}

#[test]
fn enabled_check_variants_are_debug_and_cloneable() {
    let variants = vec![
        EnabledCheck::Always,
        EnabledCheck::Which("tool"),
        EnabledCheck::FileUpAny(&["package.json"]),
        EnabledCheck::Prettier,
        EnabledCheck::Oxfmt,
        EnabledCheck::Biome,
        EnabledCheck::ClangFormat,
        EnabledCheck::Ruff,
        EnabledCheck::UvFormat,
        EnabledCheck::Pint,
        EnabledCheck::Ocamlformat,
        EnabledCheck::RLangAir,
    ];

    for variant in variants {
        let cloned = variant.clone();
        assert!(!format!("{cloned:?}").is_empty());
    }
}
