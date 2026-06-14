use super::shared::{
    agent_sidebar_button_style, entry_kind_label, is_dark_theme, models_for_provider,
    with_selected_option,
};
use crate::app::state::{AgentSettingsEntryKind, DelegateAgentSettingsEntry};
use iced::widget::{button, text_editor};
use iced::{Background, Theme};

#[test]
fn with_selected_option_keeps_existing_selection_once() {
    let options = with_selected_option(vec!["alpha".to_string(), "beta".to_string()], "beta");

    assert_eq!(options, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn with_selected_option_appends_missing_selection() {
    let options = with_selected_option(vec!["alpha".to_string()], "beta");

    assert_eq!(options, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn with_selected_option_trims_empty_selection_sorts_and_dedups() {
    let options = with_selected_option(
        vec!["beta".to_string(), "alpha".to_string(), "alpha".to_string()],
        "   ",
    );

    assert_eq!(options, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn entry_kind_label_maps_all_entry_kinds() {
    let mut entry = DelegateAgentSettingsEntry {
        key: "main".to_string(),
        label: "Main".to_string(),
        kind: AgentSettingsEntryKind::Main,
        enabled: true,
        provider: String::new(),
        model: String::new(),
        system_prompt_editor: text_editor::Content::new(),
        api_key_input: String::new(),
        temperature: 0.7,
        compact_context: false,
        max_tool_iterations: 20,
        max_history_messages: 50,
        parallel_tools: false,
        tool_dispatcher: "auto".to_string(),
        max_depth: 1,
        agentic: false,
        allowed_tools: Vec::new(),
        allowed_skills: Vec::new(),
        max_iterations: 1,
    };

    assert_eq!(entry_kind_label(&entry), "主 Agent");
    entry.kind = AgentSettingsEntryKind::BuiltinWorker;
    assert_eq!(entry_kind_label(&entry), "内建 Worker");
    entry.kind = AgentSettingsEntryKind::Custom;
    assert_eq!(entry_kind_label(&entry), "自定义");
}

#[test]
fn models_for_provider_returns_models_or_empty() {
    let (mut app, _) = crate::app::App::new();
    app.agents_settings.provider_models = vec![crate::app::state::ProviderModelsSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        models: vec![
            crate::app::state::ModelSummary {
                id: "gpt-a".to_string(),
                name: "A".to_string(),
                enabled: true,
                toolcall: true,
                attachment: false,
                context_limit: 128_000,
                detail: serde_json::json!({}),
            },
            crate::app::state::ModelSummary {
                id: "gpt-b".to_string(),
                name: "B".to_string(),
                enabled: true,
                toolcall: false,
                attachment: true,
                context_limit: 64_000,
                detail: serde_json::json!({}),
            },
        ],
    }];

    assert_eq!(models_for_provider(&app, "openai"), vec!["gpt-a".to_string(), "gpt-b".to_string()]);
    assert!(models_for_provider(&app, "missing").is_empty());
}

#[test]
fn dark_theme_detection_and_sidebar_style_cover_branches() {
    assert!(!is_dark_theme(&Theme::Light));
    assert!(is_dark_theme(&Theme::Dark));

    let light_selected = agent_sidebar_button_style(&Theme::Light, button::Status::Active, true);
    let light_hovered = agent_sidebar_button_style(&Theme::Light, button::Status::Hovered, false);
    let dark_pressed = agent_sidebar_button_style(&Theme::Dark, button::Status::Pressed, false);

    assert!(matches!(light_selected.background, Some(Background::Color(_))));
    assert!(matches!(light_hovered.background, Some(Background::Color(_))));
    assert!(matches!(dark_pressed.background, Some(Background::Color(_))));
    assert_eq!(light_selected.border.width, 1.0);
    assert_eq!(light_selected.border.radius.top_left, 14.0);
}
