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
