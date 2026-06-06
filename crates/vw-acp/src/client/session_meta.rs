//! ACP 会话选项 meta 构造。

use super::*;

/// 将会话选项转换为 ACP `meta` 扩展字段。
///
/// 当前只在存在有效选项时返回 `Some`，避免向代理发送空扩展对象。空字符串工具
/// 名或模型名会被过滤。
pub(super) fn build_session_options_meta(options: Option<&AcpSessionOptions>) -> Option<acp::Meta> {
    let options = options?;
    let mut claude_code_options = Map::new();
    if let Some(model) = options.model.as_ref().filter(|value| !value.trim().is_empty()) {
        claude_code_options.insert("model".to_string(), Value::String(model.clone()));
    }
    if let Some(allowed_tools) = &options.allowed_tools {
        let allowed_tools = allowed_tools
            .iter()
            .filter_map(|value| {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then(|| Value::String(trimmed.to_string()))
            })
            .collect::<Vec<_>>();
        if !allowed_tools.is_empty() {
            claude_code_options.insert("allowedTools".to_string(), Value::Array(allowed_tools));
        }
    }
    if let Some(max_turns) = options.max_turns {
        claude_code_options.insert("maxTurns".to_string(), json!(max_turns));
    }
    if claude_code_options.is_empty() {
        return None;
    }

    let mut meta = Map::new();
    meta.insert(
        "claudeCode".to_string(),
        json!({
            "options": Value::Object(claude_code_options),
        }),
    );
    Some(meta)
}
