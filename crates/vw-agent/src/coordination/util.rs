//! 协调模块工具函数
//!
//! 本模块提供了协调系统（coordination）中使用的通用工具函数，主要用于：
//! - 字段验证（非空检查）
//! - 字符串规范化处理
//! - 键路径解析（特别是委托上下文关联ID的提取）
//!
//! 这些工具函数为协调模块的其他组件提供基础的验证和解析能力。

use crate::app::agent::coordination::errors::CoordinationError;

/// 验证字符串字段非空
///
/// 检查给定的字符串在去除首尾空白字符后是否为空。
/// 如果为空，返回包含字段名的错误；否则返回成功。
///
/// # 参数
///
/// - `value`: 待验证的字符串值
/// - `field`: 字段名称（静态字符串），用于错误信息中标识具体字段
///
/// # 返回值
///
/// - `Ok(())`: 字段非空，验证通过
/// - `Err(CoordinationError::EmptyField)`: 字段为空，验证失败
///
/// # 示例
///
/// ```ignore
/// use crate::app::agent::coordination::util::require_non_empty;
///
/// // 验证通过
/// require_non_empty("hello", "name")?;  // Ok(())
///
/// // 验证失败（空字符串）
/// require_non_empty("", "name")?;       // Err(EmptyField { field: "name" })
///
/// // 验证失败（仅空白字符）
/// require_non_empty("  ", "name")?;     // Err(EmptyField { field: "name" })
/// ```
pub(crate) fn require_non_empty(value: &str, field: &'static str) -> Result<(), CoordinationError> {
    if value.trim().is_empty() {
        return Err(CoordinationError::EmptyField { field });
    }
    Ok(())
}

/// 规范化并过滤非空字符串
///
/// 对可选字符串进行规范化处理：去除首尾空白字符，并过滤掉空字符串。
/// 如果输入为 None、空字符串或仅包含空白字符，返回 None。
///
/// # 参数
///
/// - `value`: 可选的字符串引用
///
/// # 返回值
///
/// - `Some(&str)`: 规范化后的非空字符串
/// - `None`: 输入为 None 或规范化后为空
///
/// # 示例
///
/// ```ignore
/// use crate::app::agent::coordination::util::normalized_non_empty;
///
/// assert_eq!(normalized_non_empty(Some("  hello  ")), Some("hello"));
/// assert_eq!(normalized_non_empty(Some("  ")), None);
/// assert_eq!(normalized_non_empty(Some("")), None);
/// assert_eq!(normalized_non_empty(None), None);
/// ```
pub(crate) fn normalized_non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// 从键路径中解析委托上下文关联ID
///
/// 从格式为 `delegate/<correlation>/<tail>` 的键路径中提取中间的关联ID部分。
/// 该函数用于解析委托上下文的存储键，以支持委托任务的状态跟踪和关联。
///
/// # 键路径格式
///
/// 期望的键路径格式：`delegate/<correlation_id>/<state_or_other_info>`
/// - 第一部分：命名空间，必须是 "delegate"
/// - 第二部分：关联ID（correlation ID），不能为空
/// - 第三部分：尾部信息（如 state、metadata 等），至少需要一个非空部分
///
/// # 参数
///
/// - `key`: 键路径字符串引用
///
/// # 返回值
///
/// - `Some(&str)`: 成功解析出的关联ID
/// - `None`: 键路径格式不符合要求（命名空间不匹配、部分为空等）
///
/// # 示例
///
/// ```ignore
/// use crate::app::agent::coordination::util::parse_delegate_context_correlation_from_key;
///
/// // 有效键路径
/// assert_eq!(
///     parse_delegate_context_correlation_from_key("delegate/abc123/state"),
///     Some("abc123")
/// );
/// assert_eq!(
///     parse_delegate_context_correlation_from_key("delegate/task-456/metadata/version"),
///     Some("task-456")
/// );
///
/// // 无效键路径
/// assert_eq!(parse_delegate_context_correlation_from_key("other/abc123/state"), None);
/// assert_eq!(parse_delegate_context_correlation_from_key("delegate//state"), None);
/// assert_eq!(parse_delegate_context_correlation_from_key("delegate/abc123/"), None);
/// assert_eq!(parse_delegate_context_correlation_from_key("delegate/abc123"), None);
/// ```
pub(crate) fn parse_delegate_context_correlation_from_key(key: &str) -> Option<&str> {
    let mut parts = key.splitn(3, '/');
    let namespace = parts.next()?;
    if namespace != "delegate" {
        return None;
    }
    let correlation = parts.next()?.trim();
    if correlation.is_empty() {
        return None;
    }
    // 要求至少有一个尾部段（例如：delegate/<correlation>/state）
    let tail = parts.next()?.trim();
    if tail.is_empty() {
        return None;
    }
    Some(correlation)
}
