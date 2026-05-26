use super::session_commands::{handle_show_model, handle_show_providers};
use crate::app::agent::channels::ChannelRouteSelection;
use std::path::Path;

#[test]
fn handle_show_providers_marks_current_provider() {
    let current = ChannelRouteSelection {
        provider: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        task_mode_enabled: false,
    };

    let response = handle_show_providers(&current);

    assert!(response.contains("openai"));
    assert!(response.contains("gpt-4.1"));
}

#[test]
fn handle_show_model_includes_current_model() {
    let current = ChannelRouteSelection {
        provider: "openai".to_string(),
        model: "gpt-4.1".to_string(),
        task_mode_enabled: false,
    };

    let response = handle_show_model(&current, Path::new("/tmp"));

    assert!(response.contains("gpt-4.1"));
}
