/// 重新导出 use super::{tool_emoji, tool_header_label, tool_icon, tool_inline_summary, tool_verb}，让上层模块通过稳定路径访问。
use super::{
    tool_emoji, tool_header_label, tool_icon, tool_icon_badge_size, tool_inline_summary, tool_verb,
};
/// 重新导出 use crate::app::assets::Icon，让上层模块通过稳定路径访问。
use crate::app::assets::Icon;

/// 生成 ls summary uses path，用于工具卡片或状态行的简短说明。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn ls_summary_uses_path() {
    let input = serde_json::json!({
        "path": "docs/agents"
    })
    .to_string();

    assert_eq!(tool_inline_summary("ls", &input).as_deref(), Some("docs/agents"));
}

/// 生成 image info summary uses path，用于工具卡片或状态行的简短说明。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn image_info_summary_uses_path() {
    let input = serde_json::json!({
        "path": "assets/demo.png"
    })
    .to_string();

    assert_eq!(tool_inline_summary("image_info", &input).as_deref(), Some("assets/demo.png"));
}

/// 生成 file tool icons use chevron right，用于工具卡片或状态行的简短说明。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn file_tool_icons_use_chevron_right() {
    assert_eq!(tool_icon("read"), Icon::ChevronRight);
    assert_eq!(tool_icon("file_write"), Icon::ChevronRight);
}

/// 生成 file tool chevrons use compact badge size，用于工具卡片或状态行的简短说明。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn file_tool_chevrons_use_compact_badge_size() {
    assert_eq!(tool_icon_badge_size("read"), 8.0);
    assert_eq!(tool_icon_badge_size("file_write"), 8.0);
    assert_eq!(tool_icon_badge_size("bash"), 14.0);
}

#[test]
fn aliases_share_tool_metadata() {
    assert_eq!(tool_emoji("read_file"), "📖");
    assert_eq!(tool_icon("grep_search"), Icon::Search);
    assert_eq!(tool_verb("apply_patch"), "编辑");
    assert_eq!(tool_header_label("browser_open"), "浏览");
}

#[test]
fn config_and_read_summaries_include_key_details() {
    let config_input = serde_json::json!({
        "setting": "model",
        "value": "gpt-5"
    })
    .to_string();
    let read_input = serde_json::json!({
        "filePath": "file:///tmp/demo.rs",
        "offset": 0,
        "limit": 20
    })
    .to_string();

    assert_eq!(tool_inline_summary("config", &config_input).as_deref(), Some("model = gpt-5"));
    assert_eq!(
        tool_inline_summary("read", &read_input).as_deref(),
        Some("tmp/demo.rs (offset=1, limit=20)")
    );
}

#[test]
fn search_web_and_agent_summaries_handle_multiple_shapes() {
    let search_input = serde_json::json!({
        "pattern": "App::new",
        "path": "crates/vw-desktop/src/app"
    })
    .to_string();
    let web_input = serde_json::json!({
        "urls": ["https://example.test"],
        "query": "release notes"
    })
    .to_string();
    let agent_input = serde_json::json!({
        "agent": "code-reviewer",
        "prompt": "Check the diff for regressions"
    })
    .to_string();

    assert_eq!(
        tool_inline_summary("grep", &search_input).as_deref(),
        Some("App::new in crates/vw-desktop/src/app")
    );
    assert_eq!(
        tool_inline_summary("web_fetch", &web_input).as_deref(),
        Some("release notes in https://example.test")
    );
    assert_eq!(
        tool_inline_summary("agent", &agent_input).as_deref(),
        Some("code-reviewer · Check the diff for regressions")
    );
}
