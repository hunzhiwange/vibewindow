#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("agents_tests"));
}

use super::{ordered_agent_keys, AgentsSettingsState, DelegateAgentSettingsEntry};
use crate::app::state::{AgentSettingsEntryKind, MAIN_AGENT_KEY};
use std::collections::HashMap;
use vw_config_types::agent::DelegateAgentConfig;

#[test]
fn delegate_agent_entry_uses_main_kind_and_forces_main_enabled() {
    let config = DelegateAgentConfig { enabled: false, label: Some("Primary".to_string()), ..Default::default() };

    let entry = DelegateAgentSettingsEntry::from_config(MAIN_AGENT_KEY, Some(config));

    assert_eq!(entry.kind, AgentSettingsEntryKind::Main);
    assert!(entry.enabled);
    assert_eq!(entry.label, "Primary");
}

#[test]
fn delegate_agent_entry_clamps_numeric_config_values() {
    let config = DelegateAgentConfig {
        temperature: Some(5.0),
        max_depth: 0,
        max_iterations: 500,
        provider: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        system_prompt: Some("  prompt  ".to_string()),
        api_key: Some("secret".to_string()),
        allowed_tools: vec!["read".to_string()],
        allowed_skills: vec!["skill-a".to_string()],
        ..Default::default()
    };

    let entry = DelegateAgentSettingsEntry::from_config("custom", Some(config));

    assert_eq!(entry.kind, AgentSettingsEntryKind::Custom);
    assert_eq!(entry.temperature, 2.0);
    assert_eq!(entry.max_depth, 1);
    assert_eq!(entry.max_iterations, 100);
    assert_eq!(entry.system_prompt_editor.text(), "  prompt  ");
    assert_eq!(entry.api_key_input, "secret");
    assert_eq!(entry.allowed_tools, vec!["read"]);
    assert_eq!(entry.allowed_skills, vec!["skill-a"]);
}

#[test]
fn ordered_agent_keys_puts_builtin_keys_before_sorted_custom_keys() {
    let mut configured = HashMap::new();
    configured.insert("zeta".to_string(), DelegateAgentConfig::default());
    configured.insert("alpha".to_string(), DelegateAgentConfig::default());

    let keys = ordered_agent_keys(&configured);
    let alpha_index = keys.iter().position(|key| key == "alpha").expect("alpha key missing");
    let zeta_index = keys.iter().position(|key| key == "zeta").expect("zeta key missing");
    let main_index = keys.iter().position(|key| key == MAIN_AGENT_KEY).expect("main key missing");

    assert!(main_index < alpha_index);
    assert!(alpha_index < zeta_index);
}

#[test]
fn agents_settings_default_selects_main_and_builds_workspace_identity_files() {
    let state = AgentsSettingsState::default();

    assert_eq!(state.selected_agent, MAIN_AGENT_KEY);
    assert!(!state.entries.is_empty());
    assert!(state.entries.iter().any(|entry| entry.key == MAIN_AGENT_KEY));
    assert!(state.workspace_identity_files.iter().any(|file| file.file_name == "AGENTS.md"));
    assert!(state.save_error.is_none());
}
