//! config_view.rs 测试模块。
//!
//! 这些测试固定相邻解析器、视图辅助函数或状态计算的行为，防止后续 UI 重排时破坏边界契约。

use serde_json::json;

/// 重新导出 use super::{parse_config_input, parse_config_result_from_output, summary_from_input, summary_from_result}，让上层模块通过稳定路径访问。
use super::{
    parse_config_input, parse_config_result_from_output, summary_from_input, summary_from_result,
    tool_config_view,
};
use crate::app::App;

/// 解析 config result from output reads get shape 的输入文本，返回后续视图可以直接消费的结构化结果。
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
fn parse_config_result_from_output_reads_get_shape() {
    let result = parse_config_result_from_output(
        r#"{"success":true,"operation":"get","setting":"browser.enabled","value":true}"#,
    )
    .expect("config get payload should parse");

    assert_eq!(result.setting.as_deref(), Some("browser.enabled"));
    assert_eq!(result.value, Some(json!(true)));
    assert_eq!(summary_from_result(&result), "browser.enabled = true");
}

#[test]
fn parse_config_result_from_output_rejects_non_json_text() {
    assert!(parse_config_result_from_output("plain text").is_none());
    assert!(parse_config_result_from_output("{not-json").is_none());
}

/// 解析 config result from output reads set shape 的输入文本，返回后续视图可以直接消费的结构化结果。
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
fn parse_config_result_from_output_reads_set_shape() {
    let result = parse_config_result_from_output(
        r#"{"success":true,"operation":"set","setting":"appUi.terminalTheme","previousValue":"system","newValue":"monokai"}"#,
    )
    .expect("config set payload should parse");

    assert_eq!(result.previous_value, Some(json!("system")));
    assert_eq!(result.new_value, Some(json!("monokai")));
    assert_eq!(summary_from_result(&result), "appUi.terminalTheme -> monokai");
}

#[test]
fn summary_from_result_handles_failures_and_overview_gets() {
    let failed = parse_config_result_from_output(
        r#"{"success":false,"operation":"set","error":"permission denied"}"#,
    )
    .expect("config failure payload should parse");
    assert_eq!(summary_from_result(&failed), "permission denied");

    let overview =
        parse_config_result_from_output(r#"{"success":true,"operation":"get","value":{"proxy":{},"model_routing":{},"runtime":{}}}"#)
            .expect("config overview should parse");
    assert_eq!(summary_from_result(&overview), "当前配置概览");
}

/// 解析 config input reads pending write request 的输入文本，返回后续视图可以直接消费的结构化结果。
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
fn parse_config_input_reads_pending_write_request() {
    let input = parse_config_input(r#"{"setting":"appUi.terminalTheme","value":"monokai"}"#)
        .expect("config input should parse");

    assert_eq!(input.setting.as_deref(), Some("appUi.terminalTheme"));
    assert_eq!(input.value, Some(json!("monokai")));
    assert_eq!(summary_from_input(&input), "设置 appUi.terminalTheme = monokai");
}

#[test]
fn summary_from_input_handles_reads_and_sections() {
    let read = parse_config_input(r#"{"setting":"browser.enabled"}"#)
        .expect("config read input should parse");
    assert_eq!(summary_from_input(&read), "读取 browser.enabled");

    let proxy =
        parse_config_input(r#"{"section":"proxy"}"#).expect("config section input should parse");
    assert_eq!(summary_from_input(&proxy), "读取 proxy 高级配置");

    let overview = parse_config_input(r#"{}"#).expect("empty config input should parse");
    assert_eq!(summary_from_input(&overview), "读取配置概览");
    assert!(parse_config_input("not-json").is_none());
}

#[test]
fn config_view_rejects_bad_tool_and_empty_completed_result() {
    let app = App::new().0;

    assert!(tool_config_view(&app, 0, 0, "tool bash\n{}").is_none());
    assert!(tool_config_view(&app, 0, 0, "tool config\nnot-json").is_none());
    assert!(
        tool_config_view(
            &app,
            0,
            0,
            r#"tool config
{"status":"success","output":"plain text"}"#
        )
        .is_none()
    );
}

#[test]
fn config_view_renders_running_success_and_error_states() {
    let mut app = App::new().0;
    app.chat_tool_hovered_idx = Some((1_u64 << 32) | 2);

    assert!(
        tool_config_view(
            &app,
            1,
            2,
            r#"tool config
{"status":"running","input":"{\"setting\":\"browser.enabled\"}"}"#
        )
        .is_some()
    );
    assert!(
        tool_config_view(
            &app,
            1,
            2,
            r#"tool config
{"result":{"data":{"success":true,"operation":"set","setting":"browser.enabled","previousValue":false,"newValue":true}}}"#
        )
        .is_some()
    );
    assert!(
        tool_config_view(
            &app,
            1,
            2,
            r#"tool config
{"status":"error","error":"bad config","result":{"data":{"success":false,"error":"bad config"}}}"#
        )
        .is_some()
    );
}
