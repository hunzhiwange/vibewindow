//! 设计画布布局模块。
//!
//! 该模块负责解析和计算画布节点布局，帮助渲染层获得稳定的几何信息。

use crate::app::views::design::canvas::parse::resolve_variable;
use crate::app::views::design::canvas::types::{AlignMode, LayoutDirection, Padding};
use crate::app::views::design::models::VariableDef;
use std::collections::HashMap;

/// 公开的 parse_padding 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn parse_padding(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Padding {
    match v {
        Some(serde_json::Value::Number(n)) => {
            let val = n.as_f64().map(|f| f as f32).unwrap_or(0.0);
            Padding { top: val, right: val, bottom: val, left: val }
        }
        Some(serde_json::Value::Array(arr)) => {
            let parse_item = |item: &serde_json::Value| -> Option<f32> {
                let parse_str = |s: &str| -> Option<f32> {
                    s.trim().trim_end_matches("px").trim().parse::<f32>().ok()
                };
                match item {
                    serde_json::Value::Number(n) => n.as_f64().map(|f| f as f32),
                    serde_json::Value::String(s) => {
                        let mut resolved = s.clone();
                        while resolved.starts_with("$-") {
                            let var_name = resolved.strip_prefix("$").unwrap_or(&resolved);
                            if let Some(val_str) = resolve_variable(var_name, variables, theme_mode)
                            {
                                resolved = val_str.clone();
                            } else {
                                break;
                            }
                        }
                        parse_str(&resolved)
                    }
                    _ => None,
                }
            };

            let parts: Vec<f32> = arr.iter().filter_map(parse_item).collect();
            match parts.len() {
                1 => Padding { top: parts[0], right: parts[0], bottom: parts[0], left: parts[0] },
                2 => Padding { top: parts[0], right: parts[1], bottom: parts[0], left: parts[1] },
                3 => Padding { top: parts[0], right: parts[1], bottom: parts[2], left: parts[1] },
                4 => Padding { top: parts[0], right: parts[1], bottom: parts[2], left: parts[3] },
                _ => Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 },
            }
        }
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_padding(
                        &Some(serde_json::Value::String(val_str.clone())),
                        variables,
                        theme_mode,
                    );
                }
            }

            let parts: Vec<f32> = s.split_whitespace().filter_map(|p| p.parse().ok()).collect();
            match parts.len() {
                1 => Padding { top: parts[0], right: parts[0], bottom: parts[0], left: parts[0] },
                2 => Padding { top: parts[0], right: parts[1], bottom: parts[0], left: parts[1] },
                3 => Padding { top: parts[0], right: parts[1], bottom: parts[2], left: parts[1] },
                4 => Padding { top: parts[0], right: parts[1], bottom: parts[2], left: parts[3] },
                _ => Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 },
            }
        }
        _ => Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 },
    }
}

/// 公开的 parse_layout 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn parse_layout(v: &Option<String>) -> Option<LayoutDirection> {
    match v.as_deref() {
        Some("horizontal") | Some("row") => Some(LayoutDirection::Horizontal),
        Some("vertical") | Some("column") => Some(LayoutDirection::Vertical),
        _ => None,
    }
}

/// 公开的 parse_align_mode 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn parse_align_mode(v: &Option<String>) -> Option<AlignMode> {
    match v.as_deref().map(|s| s.to_lowercase().replace('_', "-")) {
        Some(s) if s == "start" || s == "flex-start" => Some(AlignMode::Start),
        Some(s) if s == "end" || s == "flex-end" => Some(AlignMode::End),
        Some(s) if s == "center" => Some(AlignMode::Center),
        Some(s) if s == "space-between" => Some(AlignMode::SpaceBetween),
        Some(s) if s == "space-around" => Some(AlignMode::SpaceAround),
        Some(s) if s == "space-evenly" => Some(AlignMode::SpaceEvenly),
        Some(s) if s == "stretch" => Some(AlignMode::Stretch),
        _ => None,
    }
}

/// 公开的 parse_gap 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn parse_gap(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> f32 {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f as f32).unwrap_or(0.0),
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_gap(
                        &Some(serde_json::Value::String(val_str.clone())),
                        variables,
                        theme_mode,
                    );
                }
            }
            s.parse::<f32>().unwrap_or(0.0)
        }
        _ => 0.0,
    }
}

#[cfg(test)]
#[path = "parse_tests.rs"]
mod parse_tests;
