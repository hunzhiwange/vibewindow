use super::usage_button::usage_button;
use crate::app::models::{ChatSessionStep, TokenUsage};
use crate::app::state::UsageModelInfo;
use crate::app::{App, Message};

fn test_app() -> App {
    App::new().0
}

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn model_info() -> UsageModelInfo {
    UsageModelInfo {
        provider_id: "provider".to_string(),
        provider_name: "Provider".to_string(),
        model_id: "model".to_string(),
        model_name: "Model".to_string(),
        context_limit: 1_000,
        output_limit: 100,
        cost_input_per_million: 1.0,
        cost_output_per_million: 3.0,
        cost_cache_read_per_million: 0.1,
        cost_cache_write_per_million: 0.2,
    }
}

fn step(input_tokens: i64) -> ChatSessionStep {
    ChatSessionStep {
        index: 1,
        started_ms: 1,
        finished_ms: None,
        start_snapshot_path: None,
        finish_snapshot_path: None,
        usage: TokenUsage { input_tokens, ..Default::default() },
        cost_usd: None,
        finish_reason: None,
        model: None,
    }
}

#[test]
fn task_741_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("usage_button_tests.rs"));
}

#[test]
fn usage_button_builds_with_empty_usage_state() {
    let app = test_app();

    keep(usage_button(&app));
}

#[test]
fn usage_button_builds_with_context_tokens_and_cost_data() {
    let mut app = test_app();
    app.usage_model_info = Some(model_info());
    app.active_session_view_state.steps.push(step(725));
    app.usage.input_tokens = 100;
    app.usage.output_tokens = 50;
    app.usage.cached_tokens = 25;
    app.usage.reasoning_tokens = 25;

    keep(usage_button(&app));
}
