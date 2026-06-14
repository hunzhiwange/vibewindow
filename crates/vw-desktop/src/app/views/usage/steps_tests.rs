use iced::Theme;

use crate::app::App;
use crate::app::models::{ChatSession, ChatSessionStep, TokenUsage};

fn usage_data_with_steps(steps: Vec<ChatSessionStep>) -> super::UsageData {
    super::UsageData {
        total_tokens: 0,
        session: Some(ChatSession {
            id: "session-1".to_string(),
            title: "Session".to_string(),
            messages: Vec::new(),
            message_ids: Vec::new(),
            calls: Vec::new(),
            steps,
            created_ms: 1,
            updated_ms: 2,
        }),
        session_id: "session-1".to_string(),
        message_count: 0,
        call_count: 0,
        session_title: "Session".to_string(),
        last_step_input_tokens: 0,
        last_step_output_tokens: 0,
        last_step_cached_tokens: 0,
        last_step_reasoning_tokens: 0,
        last_step_total_tokens: 0,
        user_msgs: 0,
        assistant_msgs: 0,
        system_msgs: 0,
        tool_msgs: 0,
    }
}

fn step(index: u32, finished_ms: Option<u64>) -> ChatSessionStep {
    ChatSessionStep {
        index,
        started_ms: 1_700_000_000_000 + u64::from(index) * 1_000,
        finished_ms,
        start_snapshot_path: Some(format!("/tmp/session/{index}-start.snap")),
        finish_snapshot_path: Some(format!("/tmp/session/{index}-finish.snap")),
        usage: TokenUsage {
            input_tokens: 10,
            output_tokens: 20,
            cached_tokens: 3,
            reasoning_tokens: 4,
        },
        cost_usd: Some(0.012345),
        finish_reason: Some("stop".to_string()),
        model: Some("model-a".to_string()),
    }
}

#[test]
fn seg_color_covers_all_segments() {
    for seg in [
        super::BarSeg::User,
        super::BarSeg::Assistant,
        super::BarSeg::Tool,
        super::BarSeg::Other,
        super::BarSeg::Prompt,
        super::BarSeg::Answer,
    ] {
        let color = super::seg_color(&Theme::Dark, seg);
        assert!(color.a > 0.0);
    }
}

#[test]
fn stacked_bar_handles_empty_zero_and_weighted_values() {
    let _empty = super::stacked_bar(&[]);
    let _zero = super::stacked_bar(&[(0, super::BarSeg::User), (0, super::BarSeg::Tool)]);
    let _weighted =
        super::stacked_bar(&[(1, super::BarSeg::Prompt), (999, super::BarSeg::Answer)]);
}

#[test]
fn legend_item_handles_zero_and_nonzero_totals() {
    let _zero = super::legend_item("输入", 0, 0, super::BarSeg::Prompt);
    let _half = super::legend_item("输出", 25, 50, super::BarSeg::Answer);
}

#[test]
fn build_steps_panel_handles_empty_steps() {
    let (app, _task) = App::new();
    let data = usage_data_with_steps(Vec::new());

    let _panel = super::build_steps_panel(&app, &data);
}

#[test]
fn build_steps_panel_handles_collapsed_and_expanded_steps() {
    let (mut app, _task) = App::new();
    app.usage_step_expanded.insert(1);
    let data = usage_data_with_steps(vec![step(1, Some(1_700_000_002_000)), step(2, None)]);

    let _panel = super::build_steps_panel(&app, &data);
}

#[test]
fn build_steps_panel_handles_expanded_step_without_optional_fields() {
    let (mut app, _task) = App::new();
    app.usage_step_expanded.insert(3);
    let mut s = step(3, None);
    s.start_snapshot_path = None;
    s.finish_snapshot_path = None;
    s.cost_usd = None;
    s.finish_reason = None;
    s.model = None;
    s.usage = TokenUsage::default();
    let data = usage_data_with_steps(vec![s]);

    let _panel = super::build_steps_panel(&app, &data);
}
