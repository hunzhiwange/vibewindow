//! Figma 导入模块，负责把 Figma JSON 中的节点、几何、样式和辅助字段转换为设计模型。

use crate::app::views::design::models::Stroke;
use serde_json::{Map, Value, json};

use super::figma_support::{
    clone_number_or_string, normalized_color_channel, value_to_f64, value_to_str,
};

/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub(super) fn map_figma_stroke(
    object: &Map<String, Value>,
    value: Option<&Value>,
) -> Option<Stroke> {
    let align =
        object.get("strokeAlign").and_then(value_to_str).map(|value| value.to_ascii_lowercase());
    let thickness = clone_number_or_string(object.get("strokeWeight"));
    let fill = value.and_then(first_color_string_from_value);

    let has_visible_thickness =
        thickness.as_ref().and_then(value_to_f64).is_some_and(|value| value > 0.0);

    if fill.is_none() || !has_visible_thickness {
        if align.is_none() && fill.is_none() {
            return None;
        }
        if !has_visible_thickness {
            return None;
        }
    }

    if align.is_none() && thickness.is_none() && fill.is_none() {
        None
    } else {
        Some(Stroke { align, thickness, fill })
    }
}

/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub(super) fn map_figma_effects(value: Option<&Value>) -> Option<Value> {
    let effects = value?.as_array()?;
    let mapped: Vec<Value> = effects.iter().filter_map(map_figma_effect).collect();
    match mapped.len() {
        0 => None,
        1 => mapped.into_iter().next(),
        _ => Some(Value::Array(mapped)),
    }
}

fn map_figma_effect(effect: &Value) -> Option<Value> {
    let object = effect.as_object()?;
    if object.get("visible").and_then(Value::as_bool) == Some(false) {
        return None;
    }

    let effect_type = object
        .get("type")
        .and_then(value_to_str)
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "shadow".to_string());

    let mapped_type = match effect_type.as_str() {
        "drop_shadow" | "shadow" => "shadow",
        "inner_shadow" => "shadow",
        "layer_blur" => "layer_blur",
        "background_blur" => "background_blur",
        _ => return None,
    };

    let mut mapped = Map::new();
    mapped.insert("type".to_string(), Value::String(mapped_type.to_string()));
    if effect_type == "drop_shadow" || effect_type == "shadow" {
        mapped.insert("shadowType".to_string(), Value::String("outer".to_string()));
    }
    if effect_type == "inner_shadow" {
        mapped.insert("shadowType".to_string(), Value::String("inner".to_string()));
    }
    if let Some(color) = object.get("color").and_then(color_from_value) {
        mapped.insert("color".to_string(), Value::String(color));
    }
    if let Some(offset) = object.get("offset").and_then(Value::as_object) {
        let mut mapped_offset = Map::new();
        if let Some(x) = offset.get("x").and_then(value_to_f64) {
            mapped_offset.insert("x".to_string(), json!(x));
        }
        if let Some(y) = offset.get("y").and_then(value_to_f64) {
            mapped_offset.insert("y".to_string(), json!(y));
        }
        if !mapped_offset.is_empty() {
            mapped.insert("offset".to_string(), Value::Object(mapped_offset));
        }
    }
    if let Some(blur) = object
        .get("blur")
        .and_then(value_to_f64)
        .or_else(|| object.get("radius").and_then(value_to_f64))
    {
        mapped.insert("blur".to_string(), json!(blur));
    }
    if let Some(radius) = object.get("radius").and_then(value_to_f64) {
        mapped.insert("radius".to_string(), json!(radius));
    }
    if let Some(spread) = object.get("spread").and_then(value_to_f64) {
        mapped.insert("spread".to_string(), json!(spread));
    }
    Some(Value::Object(mapped))
}

/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub(super) fn map_figma_paints(value: Option<&Value>) -> Option<Value> {
    let paints = value?.as_array()?;
    let mapped: Vec<Value> = paints.iter().filter_map(map_figma_paint).collect();
    match mapped.len() {
        0 => None,
        1 => mapped.into_iter().next(),
        _ => Some(Value::Array(mapped)),
    }
}

fn map_figma_paint(paint: &Value) -> Option<Value> {
    let object = paint.as_object()?;
    if object.get("visible").and_then(Value::as_bool) == Some(false) {
        return None;
    }

    let paint_type = object.get("type").and_then(Value::as_str);
    let paint_type = paint_type.or_else(|| object.get("type").and_then(value_to_str));

    if paint_type == Some("IMAGE") || object.get("image").is_some() {
        let url = object
            .get("image")
            .and_then(Value::as_object)
            .and_then(|image| image.get("filename"))
            .and_then(Value::as_str)?;
        return Some(json!({
            "type": "image",
            "url": url,
            "enabled": true,
            "mode": object.get("scaleMode").and_then(value_to_str).unwrap_or("fill").to_ascii_lowercase()
        }));
    }

    if paint_type.is_some_and(|kind| kind.starts_with("GRADIENT_"))
        || object.get("gradientStops").is_some()
    {
        let gradient_type = paint_type
            .map(|value| value.trim_start_matches("GRADIENT_").to_ascii_lowercase())
            .unwrap_or_else(|| "linear".to_string());
        let colors = object
            .get("gradientStops")
            .and_then(Value::as_array)
            .map(|stops| {
                stops
                    .iter()
                    .filter_map(|stop| {
                        let stop_object = stop.as_object()?;
                        let color = stop_object
                            .get("color")
                            .and_then(color_from_value_with_optional_opacity)?;
                        let position =
                            stop_object.get("position").and_then(Value::as_f64).unwrap_or(0.0);
                        Some(json!({ "color": color, "position": position }))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        return Some(json!({
            "type": "gradient",
            "gradientType": gradient_type,
            "enabled": true,
            "rotation": 0.0,
            "colors": colors
        }));
    }

    color_from_paint_object(object).map(Value::String)
}

/// 处理颜色值的解析、格式化或空间转换。
///
/// 无法识别的颜色返回空结果，避免把错误颜色静默写入设计元素。
pub(super) fn first_color_string(value: Option<&Value>) -> Option<String> {
    first_color_string_from_value(value?)
}

fn first_color_string_from_value(value: &Value) -> Option<String> {
    value.as_array()?.iter().find_map(|paint| {
        let object = paint.as_object()?;
        if object.get("visible").and_then(Value::as_bool) == Some(false) {
            return None;
        }
        color_from_paint_object(object)
    })
}

fn color_from_paint_object(object: &Map<String, Value>) -> Option<String> {
    color_from_value_with_opacity(
        object.get("color")?,
        object.get("opacity").and_then(Value::as_f64),
    )
}

fn color_from_value_with_optional_opacity(value: &Value) -> Option<String> {
    let opacity =
        value.as_object().and_then(|object| object.get("opacity")).and_then(Value::as_f64);
    color_from_value_with_opacity(value, opacity)
}

fn color_from_value_with_opacity(value: &Value, opacity: Option<f64>) -> Option<String> {
    match value {
        Value::String(color) => apply_alpha_to_hex_color(color, opacity),
        Value::Object(object) => {
            if let Some(color) = object.get("color").and_then(Value::as_str) {
                return apply_alpha_to_hex_color(color, opacity);
            }

            let r = object.get("r").and_then(Value::as_f64)?;
            let g = object.get("g").and_then(Value::as_f64)?;
            let b = object.get("b").and_then(Value::as_f64)?;
            let alpha =
                opacity.unwrap_or_else(|| object.get("a").and_then(Value::as_f64).unwrap_or(1.0));

            Some(format!(
                "#{:02x}{:02x}{:02x}{}",
                normalized_color_channel(r),
                normalized_color_channel(g),
                normalized_color_channel(b),
                opacity_to_hex(alpha)
            ))
        }
        _ => None,
    }
}

/// 处理颜色值的解析、格式化或空间转换。
///
/// 无法识别的颜色返回空结果，避免把错误颜色静默写入设计元素。
pub(super) fn color_from_value(value: &Value) -> Option<String> {
    color_from_value_with_optional_opacity(value)
}

fn apply_alpha_to_hex_color(color: &str, opacity: Option<f64>) -> Option<String> {
    let hex = color.trim().strip_prefix('#')?;
    match hex.len() {
        8 => Some(format!("#{}", hex.to_ascii_lowercase())),
        6 => {
            Some(format!("#{}{}", hex.to_ascii_lowercase(), opacity_to_hex(opacity.unwrap_or(1.0))))
        }
        _ => None,
    }
}

fn opacity_to_hex(opacity: f64) -> String {
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("{alpha:02x}")
}

#[cfg(test)]
#[path = "figma_style_tests.rs"]
mod figma_style_tests;
