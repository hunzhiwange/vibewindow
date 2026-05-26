//! 设计画布功能模块，维护当前文件对应的局部职责。

use serde_json::Value;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default, Clone)]
/// TextStyle 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub(super) struct TextStyle {
    pub(super) color: Option<String>,
    pub(super) font_size: Option<serde_json::Value>,
    pub(super) font_weight: Option<serde_json::Value>,
    pub(super) text_align: Option<String>,
    pub(super) font_style: Option<String>,
    pub(super) text_decoration: Option<String>,
    pub(super) line_height: Option<serde_json::Value>,
    pub(super) letter_spacing: Option<serde_json::Value>,
}

/// 执行 number_value 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn number_value(value: f32) -> Value {
    Value::Number(serde_json::Number::from_f64(value as f64).unwrap())
}

/// 根据设计文档或元素生成外部表示。
///
/// 返回生成后的内容；找不到指定元素时返回 `None`，避免导出不完整结果。
pub(super) fn generate_id() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::{SystemTime, UNIX_EPOCH};
    #[cfg(target_arch = "wasm32")]
    use web_time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let nanos = since_the_epoch.as_nanos();
    let mut buf = [0u8; 8];
    for i in 0..8 {
        buf[i] = ((nanos >> (i * 8)) & 0xFF) as u8;
    }
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
    buf[0] ^= (counter & 0xFF) as u8;
    buf.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 执行 numeric_value 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn numeric_value(value: &Option<Value>) -> Option<f32> {
    match value {
        Some(Value::Number(number)) => number.as_f64().map(|number| number as f32),
        Some(Value::String(string)) => string.parse::<f32>().ok(),
        _ => None,
    }
}

/// 解析外部输入并转换为内部设计模型。
///
/// 不支持或格式不完整的输入通过 `Option`/`Result` 显式表达。
pub(super) fn parse_measurement_string(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    trimmed.strip_suffix("px").map(str::trim).unwrap_or(trimmed).parse::<f64>().ok()
}

#[cfg(test)]
#[path = "shared_tests.rs"]
mod shared_tests;
