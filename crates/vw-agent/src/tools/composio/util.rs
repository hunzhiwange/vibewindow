//! Composio 工具集的实用函数模块
//!
//! 本模块提供了一系列用于处理 Composio API 集成的辅助函数，包括：
//! - URL 安全性校验（强制 HTTPS）
//! - 实体 ID、工具名称、应用名称的标准化处理
//! - 缓存键的构建与标准化
//! - V3 API 响应到内部数据结构的映射
//! - HTTP 错误响应的提取与净化
//! - JSON Schema 的格式化提示生成
//!
//! 这些函数主要用于确保数据传输安全、统一命名格式、
//! 以及为 LLM 提供友好的参数提示信息。

use crate::app::agent::tools::composio::types::{ComposioAction, ComposioV3Tool};

/// 确保 URL 使用 HTTPS 协议
///
/// 出于安全考虑，拒绝向非 HTTPS 的 URL 传输敏感数据。
/// 这是防止中间人攻击和数据泄露的基本安全措施。
///
/// # 参数
///
/// * `url` - 待校验的 URL 字符串
///
/// # 返回值
///
/// * `Ok(())` - URL 以 "https://" 开头，校验通过
/// * `Err` - URL 不是 HTTPS 协议，返回错误信息
///
/// # 示例
///
/// ```ignore
/// ensure_https("https://api.example.com")?; // 通过
/// ensure_https("http://api.example.com")?;  // 失败
/// ```
pub(crate) fn ensure_https(url: &str) -> anyhow::Result<()> {
    if !url.starts_with("https://") {
        anyhow::bail!(
            "Refusing to transmit sensitive data over non-HTTPS URL: URL scheme must be https"
        );
    }
    Ok(())
}

/// 标准化实体 ID
///
/// 对实体 ID 进行修剪处理，如果结果为空则返回 "default"。
/// 这确保了实体 ID 在整个系统中具有一致的格式。
///
/// # 参数
///
/// * `entity_id` - 原始实体 ID 字符串
///
/// # 返回值
///
/// 修剪后的实体 ID，若为空则返回 "default"
///
/// # 示例
///
/// ```ignore
/// assert_eq!(normalize_entity_id("  user123  "), "user123");
/// assert_eq!(normalize_entity_id("   "), "default");
/// assert_eq!(normalize_entity_id(""), "default");
/// ```
pub(crate) fn normalize_entity_id(entity_id: &str) -> String {
    let trimmed = entity_id.trim();
    if trimmed.is_empty() { "default".to_string() } else { trimmed.to_string() }
}

/// 标准化工具 slug
///
/// 将工具名称转换为标准格式：
/// - 修剪首尾空白
/// - 将下划线替换为连字符
/// - 转换为小写
///
/// # 参数
///
/// * `action_name` - 原始操作/工具名称
///
/// # 返回值
///
/// 标准化后的工具 slug 字符串
///
/// # 示例
///
/// ```ignore
/// assert_eq!(normalize_tool_slug("GITHUB_GET_REPO"), "github-get-repo");
/// assert_eq!(normalize_tool_slug("slack_send_message"), "slack-send-message");
/// ```
pub(crate) fn normalize_tool_slug(action_name: &str) -> String {
    action_name.trim().replace('_', "-").to_ascii_lowercase()
}

/// 构建工具 slug 候选列表
///
/// 为给定的操作名称生成多个可能的 slug 变体，
/// 以便在查找工具时进行模糊匹配。
///
/// 生成策略包括：
/// 1. 原始名称（保持 API 返回的精确工具 ID）
/// 2. 标准化后的 slug（下划线转连字符，小写）
/// 3. 全小写版本
/// 4. 使用下划线的小写版本
/// 5. 使用连字符的小写版本
/// 6. 全大写版本及其变体
///
/// # 参数
///
/// * `action_name` - 原始操作名称
///
/// # 返回值
///
/// 去重后的候选 slug 列表，保持优先级顺序
///
/// # 示例
///
/// ```ignore
/// let candidates = build_tool_slug_candidates("GITHUB_GET_REPO");
/// // 返回包含多种变体的列表
/// ```
pub(crate) fn build_tool_slug_candidates(action_name: &str) -> Vec<String> {
    let trimmed = action_name.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();

    // 内部闭包：添加候选值，确保不重复
    let mut push_candidate = |candidate: String| {
        if !candidate.is_empty() && !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    };

    // 优先保留原始 slug/名称，以便 execute() 函数能够优先匹配
    // Composio 列表 API 返回的精确工具 ID，然后再尝试标准化变体
    push_candidate(trimmed.to_string());
    push_candidate(normalize_tool_slug(trimmed));

    let lower = trimmed.to_ascii_lowercase();
    push_candidate(lower.clone());

    let underscore_lower = lower.replace('-', "_");
    push_candidate(underscore_lower);

    let hyphen_lower = lower.replace('_', "-");
    push_candidate(hyphen_lower);

    let upper = trimmed.to_ascii_uppercase();
    push_candidate(upper.clone());
    push_candidate(upper.replace('-', "_"));
    push_candidate(upper.replace('_', "-"));

    candidates
}

/// 标准化应用 slug
///
/// 将应用名称转换为标准格式：
/// - 修剪首尾空白
/// - 将下划线替换为连字符
/// - 转换为小写
/// - 移除空的部分（连续连字符）
///
/// # 参数
///
/// * `app_name` - 原始应用名称
///
/// # 返回值
///
/// 标准化后的应用 slug 字符串
///
/// # 示例
///
/// ```ignore
/// assert_eq!(normalize_app_slug("GitHub_App"), "github-app");
/// assert_eq!(normalize_app_slug("--slack--"), "slack");
/// ```
pub(crate) fn normalize_app_slug(app_name: &str) -> String {
    app_name
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// 从操作名称推断应用 slug
///
/// 根据操作名称的命名模式（通常为 "APP_ACTION" 格式）
/// 提取并推断所属应用的 slug。
///
/// # 参数
///
/// * `action_name` - 操作名称，通常包含应用前缀
///
/// # 返回值
///
/// 推断出的应用 slug，若无法推断则返回 None
///
/// # 示例
///
/// ```ignore
/// assert_eq!(infer_app_slug_from_action_name("github-get-repo"), Some("github".to_string()));
/// assert_eq!(infer_app_slug_from_action_name("slack_send_message"), Some("slack".to_string()));
/// assert_eq!(infer_app_slug_from_action_name("unknownaction"), None);
/// ```
pub(crate) fn infer_app_slug_from_action_name(action_name: &str) -> Option<String> {
    let trimmed = action_name.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 尝试从连字符或下划线分隔的名称中提取第一部分作为应用名
    let raw = if trimmed.contains('-') {
        trimmed.split('-').next()
    } else if trimmed.contains('_') {
        trimmed.split('_').next()
    } else {
        None
    }?;

    let app = normalize_app_slug(raw);
    (!app.is_empty()).then_some(app)
}

/// 构建已连接账户的缓存键
///
/// 根据实体 ID 和应用名称生成唯一的缓存键，
/// 用于缓存已连接账户的查询结果。
///
/// # 参数
///
/// * `app_name` - 应用名称
/// * `entity_id` - 实体 ID
///
/// # 返回值
///
/// 格式为 "{entity_id}:{app_slug}" 的缓存键
///
/// # 示例
///
/// ```ignore
/// let key = connected_account_cache_key("GitHub", "user123");
/// assert_eq!(key, "user123:github");
/// ```
pub(crate) fn connected_account_cache_key(app_name: &str, entity_id: &str) -> String {
    format!("{}:{}", normalize_entity_id(entity_id), normalize_app_slug(app_name))
}

/// 标准化操作缓存键
///
/// 将别名转换为标准格式的缓存键。
///
/// # 参数
///
/// * `alias` - 操作别名
///
/// # 返回值
///
/// 标准化后的缓存键，若输入为空则返回 None
///
/// # 示例
///
/// ```ignore
/// assert_eq!(normalize_action_cache_key("GITHUB_GET_REPO"), Some("github-get-repo".to_string()));
/// assert_eq!(normalize_action_cache_key(""), None);
/// ```
pub(crate) fn normalize_action_cache_key(alias: &str) -> Option<String> {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(
        trimmed
            .to_ascii_lowercase()
            .replace('_', "-")
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-"),
    )
}

/// 构建已连接账户的提示信息
///
/// 当缺少 connected_account_id 时，生成提示用户如何获取该 ID 的帮助信息。
/// 如果已经提供了 connected_account_ref，则返回空字符串（无需提示）。
///
/// # 参数
///
/// * `app_hint` - 可选的应用名称提示
/// * `entity_id` - 可选的实体 ID
/// * `connected_account_ref` - 可选的已连接账户引用
///
/// # 返回值
///
/// 提示信息字符串，包含使用 list_accounts 操作获取 connected_account_id 的指导
///
/// # 示例
///
/// ```ignore
/// let hint = build_connected_account_hint(Some("github"), Some("user123"), None);
/// // 返回包含具体应用和实体 ID 的提示信息
/// ```
pub(crate) fn build_connected_account_hint(
    app_hint: Option<&str>,
    entity_id: Option<&str>,
    connected_account_ref: Option<&str>,
) -> String {
    // 如果已提供 connected_account_ref，无需提示
    if connected_account_ref.is_some() {
        return String::new();
    }

    // 必须有 entity_id 才能提供有意义的提示
    let Some(entity) = entity_id else {
        return String::new();
    };

    // 根据是否有应用提示，生成不同格式的提示信息
    if let Some(app) = app_hint {
        format!(
            " Hint: use action='list_accounts' with app='{app}' and entity_id='{entity}' to retrieve connected_account_id."
        )
    } else {
        format!(
            " Hint: use action='list_accounts' with entity_id='{entity}' to retrieve connected_account_id."
        )
    }
}

/// 将 V3 API 工具列表映射为内部操作列表
///
/// 将 Composio V3 API 返回的工具数据转换为系统内部使用的 ComposioAction 结构。
/// 处理字段缺失的情况，通过备选字段进行填充。
///
/// # 参数
///
/// * `items` - V3 API 返回的工具列表
///
/// # 返回值
///
/// 转换后的 ComposioAction 列表，跳过无效条目
///
/// # 映射规则
///
/// - name: 优先使用 slug，其次使用 name
/// - app_name: 优先使用 toolkit.slug，其次 toolkit.name，最后使用 app_name
/// - description: 优先使用 description，其次使用 name
/// - enabled: 默认设为 true
pub(crate) fn map_v3_tools_to_actions(items: Vec<ComposioV3Tool>) -> Vec<ComposioAction> {
    items
        .into_iter()
        .filter_map(|item| {
            // name 字段：优先使用 slug，否则使用 name
            let name = item.slug.or(item.name.clone())?;

            // app_name 字段：从 toolkit 或直接字段获取
            let app_name = item
                .toolkit
                .as_ref()
                .and_then(|toolkit| toolkit.slug.clone().or(toolkit.name.clone()))
                .or(item.app_name);

            // description 字段：优先使用 description，否则使用 name
            let description = item.description.or(item.name);

            Some(ComposioAction {
                name,
                app_name,
                description,
                enabled: true,
                input_parameters: item.input_parameters,
            })
        })
        .collect()
}

/// 从响应结果中提取重定向 URL
///
/// 尝试从多种可能的字段位置提取重定向 URL：
/// - 顶层 "redirect_url" 字段（snake_case）
/// - 顶层 "redirectUrl" 字段（camelCase）
/// - 嵌套在 "data" 对象中的 "redirect_url"
///
/// # 参数
///
/// * `result` - JSON 响应值
///
/// # 返回值
///
/// 提取到的重定向 URL，若不存在则返回 None
pub(crate) fn extract_redirect_url(result: &serde_json::Value) -> Option<String> {
    result
        .get("redirect_url")
        .and_then(|v| v.as_str())
        .or_else(|| result.get("redirectUrl").and_then(|v| v.as_str()))
        .or_else(|| result.get("data").and_then(|v| v.get("redirect_url")).and_then(|v| v.as_str()))
        .map(ToString::to_string)
}

/// 从响应结果中提取已连接账户 ID
///
/// 尝试从多种可能的字段位置提取已连接账户 ID：
/// - 顶层 "connected_account_id" 字段（snake_case）
/// - 顶层 "connectedAccountId" 字段（camelCase）
/// - 嵌套在 "data" 对象中的两种格式
///
/// # 参数
///
/// * `result` - JSON 响应值
///
/// # 返回值
///
/// 提取到的已连接账户 ID，若不存在则返回 None
pub(crate) fn extract_connected_account_id(result: &serde_json::Value) -> Option<String> {
    result
        .get("connected_account_id")
        .and_then(|v| v.as_str())
        .or_else(|| result.get("connectedAccountId").and_then(|v| v.as_str()))
        .or_else(|| {
            result.get("data").and_then(|v| v.get("connected_account_id")).and_then(|v| v.as_str())
        })
        .or_else(|| {
            result.get("data").and_then(|v| v.get("connectedAccountId")).and_then(|v| v.as_str())
        })
        .map(ToString::to_string)
}

/// 构建响应错误信息
///
/// 异步处理 HTTP 响应，提取状态码和响应体，
/// 并尝试从响应体中解析 API 错误消息。
///
/// # 参数
///
/// * `resp` - HTTP 响应对象
///
/// # 返回值
///
/// 格式化的错误信息字符串，格式为 "HTTP {状态码}: {错误消息}" 或仅 "HTTP {状态码}"
pub(crate) async fn response_error(resp: reqwest::Response) -> String {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    // 如果响应体为空，仅返回 HTTP 状态码
    if body.trim().is_empty() {
        return format!("HTTP {}", status.as_u16());
    }

    // 尝试从响应体中提取 API 错误消息
    if let Some(api_error) = extract_api_error_message(&body) {
        return format!("HTTP {}: {}", status.as_u16(), sanitize_error_message(&api_error));
    }

    format!("HTTP {}", status.as_u16())
}

/// 净化错误消息
///
/// 对错误消息进行安全处理：
/// - 将换行符替换为空格（防止日志污染）
/// - 对敏感字段进行脱敏处理
/// - 截断过长的消息
///
/// # 参数
///
/// * `message` - 原始错误消息
///
/// # 返回值
///
/// 净化后的错误消息
///
/// # 脱敏字段
///
/// 以下字段会被替换为 "[redacted]"：
/// - connected_account_id / connectedAccountId
/// - entity_id / entityId
/// - user_id / userId
pub(crate) fn sanitize_error_message(message: &str) -> String {
    // 将换行符替换为空格，保持消息在一行内
    let mut sanitized = message.replace('\n', " ");

    // 对敏感标识符进行脱敏处理，防止泄露
    for marker in
        ["connected_account_id", "connectedAccountId", "entity_id", "entityId", "user_id", "userId"]
    {
        sanitized = sanitized.replace(marker, "[redacted]");
    }

    // 限制消息长度，防止日志膨胀
    let max_chars = 240;
    if sanitized.chars().count() <= max_chars {
        sanitized
    } else {
        // 确保在 UTF-8 字符边界处截断，避免 panic
        let mut end = max_chars;
        while end > 0 && !sanitized.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &sanitized[..end])
    }
}

/// 从响应体中提取 API 错误消息
///
/// 尝试从 JSON 响应体中解析错误消息，
/// 支持两种常见的错误格式：
/// - `{"error": {"message": "..."}}`
/// - `{"message": "..."}`
///
/// # 参数
///
/// * `body` - JSON 响应体字符串
///
/// # 返回值
///
/// 提取到的错误消息，若解析失败或不存在则返回 None
pub(crate) fn extract_api_error_message(body: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
    parsed
        .get("error")
        .and_then(|v| v.get("message"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .or_else(|| parsed.get("message").and_then(|v| v.as_str()).map(ToString::to_string))
}

/// 从 input_parameters JSON Schema 构建紧凑的参数提示字符串
///
/// 用于 list 命令输出，让 LLM 能够看到每个操作期望的参数键名，
/// 而无需展示完整的 schema 定义。必填参数会标记星号。
///
/// # 参数
///
/// * `schema` - 可选的 JSON Schema 值，包含 input_parameters
///
/// # 返回值
///
/// 格式为 " [params: key1*, key2, key3*]" 的提示字符串，
/// 其中带星号的为必填参数
///
/// # 示例
///
/// ```ignore
/// let hint = format_input_params_hint(Some(&schema));
/// // 可能返回: " [params: repo*, owner*, branch]"
/// ```
pub(crate) fn format_input_params_hint(schema: Option<&serde_json::Value>) -> String {
    // 提取 properties 对象
    let props = schema.and_then(|v| v.get("properties")).and_then(|v| v.as_object());

    // 提取 required 数组
    let required: Vec<&str> = schema
        .and_then(|v| v.get("required"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let Some(props) = props else {
        return String::new();
    };
    if props.is_empty() {
        return String::new();
    }

    // 构建参数列表，必填参数标记星号
    let keys: Vec<String> = props
        .keys()
        .map(|k| if required.contains(&k.as_str()) { format!("{k}*") } else { k.clone() })
        .collect();
    format!(" [params: {}]", keys.join(", "))
}

/// 从完整的工具 schema 响应构建人类可读的 schema 提示
///
/// 用于 execute 命令的错误消息中，让 LLM 能够看到期望的参数名称和类型，
/// 以便在下一次尝试时自我修正。
///
/// # 参数
///
/// * `schema` - 完整的工具 schema JSON 值
///
/// # 返回值
///
/// 格式化的参数说明字符串，包含参数名、类型、是否必填和描述
///
/// # 输出格式
///
/// ```text
///
/// Expected input parameters:
///   repo: string (required) - 仓库名称
///   owner: string (required) - 所有者
///   branch: string - 分支名称
/// ```
pub(crate) fn format_schema_hint(schema: &serde_json::Value) -> Option<String> {
    // 提取 input_parameters 对象
    let input_params = schema.get("input_parameters")?;
    let props = input_params.get("properties")?.as_object()?;
    if props.is_empty() {
        return None;
    }

    // 提取必填参数列表
    let required: Vec<&str> = input_params
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut lines = Vec::new();
    for (key, spec) in props {
        // 获取参数类型，默认为 "any"
        let type_str = spec.get("type").and_then(|v| v.as_str()).unwrap_or("any");

        // 获取参数描述
        let desc = spec.get("description").and_then(|v| v.as_str()).unwrap_or("");

        // 标记是否必填
        let req = if required.contains(&key.as_str()) { " (required)" } else { "" };

        // 处理描述后缀，长描述需要截断
        let desc_suffix = if desc.is_empty() {
            String::new()
        } else {
            // 截断长描述以保持提示简洁
            // 使用字符边界避免在多字节 UTF-8 字符中间截断导致 panic
            let short = if desc.len() > 80 {
                let end = crate::app::agent::util::floor_utf8_char_boundary(desc, 77);
                format!("{}...", &desc[..end])
            } else {
                desc.to_string()
            };
            format!(" - {short}")
        };
        lines.push(format!("  {key}: {type_str}{req}{desc_suffix}"));
    }

    Some(format!("\n\nExpected input parameters:\n{}", lines.join("\n")))
}
#[cfg(test)]
#[path = "util_tests.rs"]
mod util_tests;
