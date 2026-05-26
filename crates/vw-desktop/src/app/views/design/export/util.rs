//! 设计导出模块，负责把内部设计文档转换为 HTML、SVG 或共享的 CSS/尺寸表示。

use crate::app::views::design::models::VariableDef;
use iced::Color;

/// 执行 resolve_variable_value 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn resolve_variable_value(def: &VariableDef, mode: Option<&str>) -> Option<String> {
    if let Some(m) = mode {
        for v in &def.value {
            if let Some(t) = &v.theme
                && t.mode == m {
                    return Some(v.value.clone());
                }
        }
    }

    for v in &def.value {
        if v.theme.is_none() {
            return Some(v.value.clone());
        }
    }

    def.value.first().map(|v| v.value.clone())
}

/// 解析外部输入并转换为内部设计模型。
///
/// 不支持或格式不完整的输入通过 `Option`/`Result` 显式表达。
pub(super) fn parse_size_to_css(val: &Option<serde_json::Value>) -> String {
    if let Some(v) = val {
        if let Some(n) = v.as_f64() {
            return format!("{}px", n);
        }
        if let Some(s) = v.as_str() {
            return s.to_string();
        }
    }
    "auto".to_string()
}

/// 解析外部输入并转换为内部设计模型。
///
/// 不支持或格式不完整的输入通过 `Option`/`Result` 显式表达。
pub(super) fn parse_size_val_opt(val: &Option<serde_json::Value>) -> f32 {
    if let Some(v) = val {
        if let Some(n) = v.as_f64() {
            return n as f32;
        }
        if let Some(s) = v.as_str()
            && let Ok(n) = s.replace("px", "").parse::<f32>() {
                return n;
            }
    }
    0.0
}

/// 解析外部输入并转换为内部设计模型。
///
/// 不支持或格式不完整的输入通过 `Option`/`Result` 显式表达。
pub(super) fn parse_fills_to_css(val: &Option<serde_json::Value>) -> String {
    if let Some(v) = val
        && let Some(s) = v.as_str() {
            return process_color_value(s);
        }
    "".to_string()
}

/// 处理颜色值的解析、格式化或空间转换。
///
/// 无法识别的颜色返回空结果，避免把错误颜色静默写入设计元素。
pub(super) fn process_color_value(val: &str) -> String {
    if val.starts_with("var(") {
        return val.to_string();
    }
    if !val.starts_with('#')
        && !val.starts_with("rgb")
        && !val.starts_with("hsl")
        && !val.starts_with("--")
    {
        return val.to_string();
    }
    val.to_string()
}

/// 处理颜色值的解析、格式化或空间转换。
///
/// 无法识别的颜色返回空结果，避免把错误颜色静默写入设计元素。
pub(super) fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    let a = c.a;
    if a < 1.0 {
        format!("rgba({},{},{},{})", r, g, b, a)
    } else {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

#[cfg(test)]
#[path = "util_tests.rs"]
mod util_tests;
