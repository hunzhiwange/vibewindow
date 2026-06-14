use super::{render_tool_card, tool_summary_cli, toolcards::compact_cli_path};
use ratatui::text::Line;
use serde_json::json;

fn line_text(line: &Line<'_>) -> String {
    line.spans.iter().map(|span| span.content.as_ref()).collect::<String>()
}

#[test]
fn compacts_long_paths_but_keeps_short_paths() {
    assert_eq!(compact_cli_path("/tmp/a.txt", 32), "/tmp/a.txt");
    let compact = compact_cli_path("/very/long/path/to/project/src/main.rs", 18);
    assert!(compact.starts_with(".../"));
    assert!(compact.ends_with("main.rs"));
}

#[test]
fn summarizes_common_tool_inputs() {
    let read = tool_summary_cli(
        "read",
        &json!({"filePath":"/tmp/demo.rs","offset":3,"limit":2}).to_string(),
    );
    let patch = tool_summary_cli(
        "apply_patch",
        &json!({"patchText":"*** Update File: src/lib.rs\n*** End Patch"}).to_string(),
    );
    let fallback = tool_summary_cli("bash", "echo hello");

    assert!(read.contains("line 3-4"));
    assert!(read.contains("demo.rs"));
    assert_eq!(patch, "1 项 · 新增 0 · 更新 1 · 删除 0");
    assert_eq!(fallback, "echo hello");
}

#[test]
fn renders_tool_card_with_status_and_primary_detail() {
    let input = json!({
        "status": "completed",
        "command": "cargo test",
        "workdir": "/tmp/project"
    })
    .to_string();

    let card = render_tool_card("bash", "测试命令", &input, false);

    assert_eq!(card.len(), 2);
    assert!(line_text(&card[0]).contains("运行"));
    assert!(line_text(&card[0]).contains("已完成"));
    assert!(line_text(&card[1]).contains("命令 cargo test"));
}
