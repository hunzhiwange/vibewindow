//! # SOP 条件求值模块
//!
//! 本模块提供触发条件评估功能，用于在 SOP（标准作业程序）系统中判断事件载荷是否满足预设条件。
//!
//! ## 主要功能
//!
//! - 支持 JSON 路径表达式条件求值（如 `$.key.subkey > 85`）
//! - 支持直接数值比较条件（如 `> 0`，用于外设触发器）
//! - 提供安全的失败关闭（fail-closed）语义：当条件无法解析或值不可比较时返回 `false`
//!
//! ## 支持的操作符
//!
//! - `>=` 大于等于
//! - `<=` 小于等于
//! - `!=` 不等于
//! - `==` 等于
//! - `>` 大于
//! - `<` 小于
//!
//! ## 失败关闭行为
//!
//! 在以下情况下返回 `false`（失败关闭）：
//! - 载荷缺失或为空
//! - 条件字符串无法解析
//! - JSON 路径无法解析到值
//! - 提取的值与比较值类型不兼容

use serde_json::Value;

/// 根据事件载荷评估触发条件。
///
/// ## 条件语法
///
/// - JSON 路径比较：`$.key.subkey > 85`
/// - 直接数值比较：`> 0`（用于外设触发器）
///
/// ## 支持的操作符
///
/// `>=`, `<=`, `!=`, `>`, `<`, `==`
///
/// ## 参数
///
/// - `condition`: 条件字符串，如 `$.temperature > 30` 或 `> 0`
/// - `payload`: 可选的 JSON 格式载荷字符串，用于与条件进行比较
///
/// ## 返回值
///
/// - `true`: 条件为空（无条件匹配）或条件满足
/// - `false`: 载荷缺失、条件解析失败、JSON 路径无效或条件不满足
///
/// ## 示例
///
/// ```ignore
/// // JSON 路径条件
/// let result = evaluate_condition("$.value > 85", Some(r#"{"value": 90}"#));
/// assert!(result);
///
/// // 直接比较条件
/// let result = evaluate_condition("> 0", Some("42"));
/// assert!(result);
///
/// // 空条件始终匹配
/// let result = evaluate_condition("", None);
/// assert!(result);
/// ```
pub fn evaluate_condition(condition: &str, payload: Option<&str>) -> bool {
    let condition = condition.trim();
    if condition.is_empty() {
        return true; // 空条件 = 无条件匹配
    }

    let payload = match payload {
        Some(p) if !p.is_empty() => p,
        _ => return false, // 没有可评估的载荷
    };

    if let Some(rest) = condition.strip_prefix('$') {
        // JSON 路径条件：$.key.sub >= 85
        evaluate_json_path_condition(rest, payload)
    } else {
        // 直接比较：> 0
        evaluate_direct_condition(condition, payload)
    }
}

/// 根据 JSON 载荷评估 `$.path.to.field op value` 形式的条件。
///
/// ## 参数
///
/// - `path_and_op`: 去除 `$` 前缀后的路径和操作符字符串，如 `.key.sub >= 85`
/// - `payload`: JSON 格式的载荷字符串
///
/// ## 返回值
///
/// - `true`: JSON 路径解析成功且条件满足
/// - `false`: JSON 解析失败、路径无效或条件不满足
fn evaluate_json_path_condition(path_and_op: &str, payload: &str) -> bool {
    // 尝试解析 JSON 载荷
    let json: Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // 将输入拆分为（点分路径、操作符、比较值）
    let (dot_path, op, comparand) = match parse_path_op_value(path_and_op) {
        Some(t) => t,
        None => return false,
    };

    // 根据 JSON 路径提取值
    let extracted = resolve_json_path(&json, &dot_path);
    let extracted = match extracted {
        Some(v) => v,
        None => return false,
    };

    // 执行值比较
    compare_values(extracted, op, &comparand)
}

/// 直接对载荷进行 `op value` 形式的比较（载荷作为数值处理）。
///
/// ## 参数
///
/// - `condition`: 操作符和值字符串，如 `> 0`
/// - `payload`: 作为数值解析的载荷字符串
///
/// ## 返回值
///
/// - `true`: 载荷和比较值均可解析为数值且条件满足
/// - `false`: 解析失败或条件不满足
fn evaluate_direct_condition(condition: &str, payload: &str) -> bool {
    // 解析操作符和比较值
    let (op, comparand) = match parse_op_value(condition) {
        Some(t) => t,
        None => return false,
    };

    // 尝试将载荷解析为数值
    let payload_num: f64 = match payload.trim().parse() {
        Ok(n) => n,
        Err(_) => return false,
    };

    // 尝试将比较值解析为数值
    let comparand_num: f64 = match comparand.parse() {
        Ok(n) => n,
        Err(_) => return false,
    };

    // 应用数值比较操作
    apply_op_f64(payload_num, op, comparand_num)
}

// ═══════════════════════════════════════════════════════════════════════
// 解析辅助函数
// ═══════════════════════════════════════════════════════════════════════

/// 比较操作符列表，按最长优先顺序排列以避免前缀歧义。
///
/// 例如，将 `>=` 放在 `>` 之前，确保解析时优先匹配更长的操作符。
const OPERATORS: &[&str] = &[">=", "<=", "!=", "==", ">", "<"];

/// 解析 `.path.to.field op value` 形式的字符串。
///
/// ## 参数
///
/// - `input`: 去除 `$` 前缀后的输入字符串，如 `.value > 85` 或 `.data.temp >= 100`
///
/// ## 返回值
///
/// 返回元组 `(路径段列表, 操作符, 比较值)`，解析失败时返回 `None`。
///
/// ## 解析逻辑
///
/// 1. 查找操作符位置
/// 2. 分割路径部分和值部分
/// 3. 将路径按 `.` 分割为段列表（过滤空段）
fn parse_path_op_value(input: &str) -> Option<(Vec<&str>, Op, String)> {
    // 输入从 `$` 之后开始，例如 `.value > 85` 或 `.data.temp >= 100`
    // 查找操作符位置
    for &op_str in OPERATORS {
        if let Some(pos) = input.find(op_str) {
            let path_part = input[..pos].trim();
            let value_part = input[pos + op_str.len()..].trim();

            // 值部分不能为空
            if value_part.is_empty() {
                return None;
            }

            // 将操作符字符串转换为 Op 枚举
            let op = Op::from_str(op_str)?;

            // 将路径按点分割，过滤掉空字符串段
            let segments: Vec<&str> = path_part.split('.').filter(|s| !s.is_empty()).collect();

            // 路径段列表不能为空
            if segments.is_empty() {
                return None;
            }

            return Some((segments, op, value_part.to_string()));
        }
    }
    None
}

/// 解析 `op value` 形式的字符串。
///
/// ## 参数
///
/// - `input`: 操作符和值字符串，如 `> 0`
///
/// ## 返回值
///
/// 返回元组 `(操作符, 值)`，解析失败时返回 `None`。
fn parse_op_value(input: &str) -> Option<(Op, String)> {
    let input = input.trim();
    for &op_str in OPERATORS {
        if let Some(rest) = input.strip_prefix(op_str) {
            let value = rest.trim();
            // 值不能为空
            if value.is_empty() {
                return None;
            }
            let op = Op::from_str(op_str)?;
            return Some((op, value.to_string()));
        }
    }
    None
}

/// 根据点分路径段遍历 JSON 值。
///
/// ## 参数
///
/// - `value`: JSON 值的引用
/// - `segments`: 路径段切片，如 `["data", "temperature"]`
///
/// ## 返回值
///
/// 返回解析到的 JSON 值引用，路径无效时返回 `None`。
///
/// ## 遍历逻辑
///
/// - 对于对象：尝试按键名访问
/// - 对于数组：尝试将路径段解析为索引并访问
fn resolve_json_path<'a>(value: &'a Value, segments: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for &seg in segments {
        // 尝试作为对象键访问
        if let Some(next) = current.get(seg) {
            current = next;
            continue;
        }
        // 尝试作为数组索引访问
        if let Ok(idx) = seg.parse::<usize>() {
            if let Some(next) = current.get(idx) {
                current = next;
                continue;
            }
        }
        return None;
    }
    Some(current)
}

// ═══════════════════════════════════════════════════════════════════════
// 比较逻辑
// ═══════════════════════════════════════════════════════════════════════

/// 比较操作符枚举。
///
/// 定义了条件求值中支持的所有比较操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    /// 大于 (`>`)
    Gt,
    /// 小于 (`<`)
    Lt,
    /// 大于等于 (`>=`)
    Gte,
    /// 小于等于 (`<=`)
    Lte,
    /// 等于 (`==`)
    Eq,
    /// 不等于 (`!=`)
    Neq,
}

impl Op {
    /// 从字符串解析操作符。
    ///
    /// ## 参数
    ///
    /// - `s`: 操作符字符串，必须是 `>`, `<`, `>=`, `<=`, `==`, `!=` 之一
    ///
    /// ## 返回值
    ///
    /// 返回对应的 `Op` 枚举变体，无效字符串返回 `None`。
    fn from_str(s: &str) -> Option<Self> {
        match s {
            ">" => Some(Self::Gt),
            "<" => Some(Self::Lt),
            ">=" => Some(Self::Gte),
            "<=" => Some(Self::Lte),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Neq),
            _ => None,
        }
    }
}

/// 使用指定操作符比较 JSON 值与字符串比较值。
///
/// ## 参数
///
/// - `extracted`: 从载荷中提取的 JSON 值
/// - `op`: 比较操作符
/// - `comparand`: 比较值字符串
///
/// ## 返回值
///
/// 返回比较结果的布尔值。
///
/// ## 比较逻辑
///
/// 1. 优先尝试数值比较（将两边都转换为 f64）
/// 2. 数值比较失败时回退到字符串比较
/// 3. 字符串比较时会去除比较值两端的引号（如果存在）
fn compare_values(extracted: &Value, op: Op, comparand: &str) -> bool {
    // 优先尝试数值比较
    if let Some(lhs) = value_as_f64(extracted) {
        if let Ok(rhs) = comparand.parse::<f64>() {
            return apply_op_f64(lhs, op, rhs);
        }
    }

    // 回退到字符串比较
    let lhs = value_as_string(extracted);

    // 如果比较值两端有引号则去除
    let rhs = comparand.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(comparand);

    // 根据操作符执行字符串比较
    match op {
        Op::Eq => lhs == rhs,
        Op::Neq => lhs != rhs,
        Op::Gt => lhs.as_str() > rhs,
        Op::Lt => lhs.as_str() < rhs,
        Op::Gte => lhs.as_str() >= rhs,
        Op::Lte => lhs.as_str() <= rhs,
    }
}

/// 尝试将 JSON 值转换为 f64。
///
/// ## 参数
///
/// - `v`: JSON 值的引用
///
/// ## 返回值
///
/// - 对于数值类型：返回数值的 f64 表示
/// - 对于字符串类型：尝试解析为 f64
/// - 其他类型：返回 `None`
fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// 将 JSON 值转换为字符串。
///
/// ## 参数
///
/// - `v`: JSON 值的引用
///
/// ## 返回值
///
/// - 对于字符串类型：返回克隆的字符串
/// - 对于布尔类型：返回 "true" 或 "false"
/// - 对于 Null：返回空字符串
/// - 其他类型：返回 `to_string()` 结果（如数值的字符串表示）
fn value_as_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// 对两个 f64 值应用比较操作符。
///
/// ## 参数
///
/// - `lhs`: 左侧操作数
/// - `op`: 比较操作符
/// - `rhs`: 右侧操作数
///
/// ## 返回值
///
/// 返回比较结果的布尔值。
///
/// ## 相等性判断
///
/// 对于浮点数相等性判断，使用 `f64::EPSILON` 作为容差阈值，
/// 当两数差值绝对值小于 `EPSILON` 时认为相等。
fn apply_op_f64(lhs: f64, op: Op, rhs: f64) -> bool {
    match op {
        Op::Gt => lhs > rhs,
        Op::Lt => lhs < rhs,
        Op::Gte => lhs >= rhs,
        Op::Lte => lhs <= rhs,
        Op::Eq => (lhs - rhs).abs() < f64::EPSILON,
        Op::Neq => (lhs - rhs).abs() >= f64::EPSILON,
    }
}

#[cfg(test)]
#[path = "condition_tests.rs"]
mod tests;
