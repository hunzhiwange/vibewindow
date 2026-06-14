use super::bash_view::tool_bash_view;
use crate::app::App;

#[test]
fn bash_view_rejects_malformed_or_non_bash_payloads() {
    let app = App::new().0;

    assert!(tool_bash_view(&app, 0, 0, "tool read\n{}").is_none());
    assert!(tool_bash_view(&app, 0, 0, "bash without prefix").is_none());
    assert!(tool_bash_view(&app, 0, 0, "tool bash\nnot-json").is_none());
}

#[test]
fn bash_view_accepts_plain_command_input() {
    let app = App::new().0;
    let visible = r#"tool bash
{"input":"cargo test -p vw-desktop","output":"ok"}"#;

    assert!(tool_bash_view(&app, 1, 2, visible).is_some());
}

#[test]
fn bash_view_accepts_json_command_input_and_hovered_detail_slot() {
    let mut app = App::new().0;
    app.chat_tool_hovered_idx = Some((3_u64 << 32) | 4);
    let visible = r#"tool bash
{"input":"{\"command\":\"ls -la\"}","output":"ok"}"#;

    assert!(tool_bash_view(&app, 3, 4, visible).is_some());
}

#[test]
fn image_info_reuses_bash_view_detail_surface() {
    let app = App::new().0;
    let visible = r#"tool image_info
{"input":"{\"path\":\"/tmp/image.png\"}","output":"metadata"}"#;

    assert!(tool_bash_view(&app, 0, 1, visible).is_some());
}
