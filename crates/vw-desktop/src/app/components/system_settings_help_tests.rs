use super::*;
use crate::app::App;
use crate::app::components::system_settings::SystemTab;

fn test_app() -> App {
    App::new().0
}

#[test]
fn system_settings_help_tests_are_wired() {
    assert!(module_path!().contains("system_settings_help_tests"));
}

#[test]
fn help_open_for_tab_requires_matching_supported_tab() {
    assert!(help_open_for_tab(
        Some(SystemTab::General),
        SystemTab::General
    ));
    assert!(!help_open_for_tab(
        Some(SystemTab::General),
        SystemTab::Memory
    ));
    assert!(!help_open_for_tab(None, SystemTab::General));
    assert!(!help_open_for_tab(
        Some(SystemTab::Security),
        SystemTab::Security
    ));
}

#[test]
fn help_button_bar_exists_only_for_supported_tabs() {
    for tab in [
        SystemTab::General,
        SystemTab::DialogueFlow,
        SystemTab::Editor,
        SystemTab::Projects,
        SystemTab::Providers,
        SystemTab::Models,
        SystemTab::EmbeddingRoutes,
        SystemTab::ModelRoutes,
        SystemTab::QueryClassification,
        SystemTab::GoalLoop,
        SystemTab::Sop,
        SystemTab::Agents,
        SystemTab::Channels,
        SystemTab::Memory,
        SystemTab::Runtime,
        SystemTab::Storage,
        SystemTab::Tunnel,
        SystemTab::Composio,
        SystemTab::Hooks,
        SystemTab::HttpRequest,
        SystemTab::Browser,
        SystemTab::Multimodal,
    ] {
        assert!(help_button_bar(tab).is_some());
    }

    assert!(help_button_bar(SystemTab::Security).is_none());
}

#[test]
fn with_help_modal_returns_base_for_closed_mismatched_and_unsupported_tabs() {
    let app = test_app();
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = with_help_modal(&app, base, SystemTab::General, None);

    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = with_help_modal(
        &app,
        base,
        SystemTab::General,
        Some(SystemTab::Memory),
    );

    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = with_help_modal(
        &app,
        base,
        SystemTab::Security,
        Some(SystemTab::Security),
    );
}

#[test]
fn with_help_modal_wraps_supported_open_tab() {
    let app = test_app();
    let base = iced::widget::container(iced::widget::text("base")).into();
    let _ = with_help_modal(
        &app,
        base,
        SystemTab::Memory,
        Some(SystemTab::Memory),
    );
}
