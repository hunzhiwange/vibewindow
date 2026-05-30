//! Figma 导入模块，负责把 Figma JSON 中的节点、几何、样式和辅助字段转换为设计模型。

use serde_json::{Map, Value};

use super::shared::parse_measurement_string;

/// 执行 has_children 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn has_children(object: &Map<String, Value>) -> bool {
    object.get("children").and_then(Value::as_array).is_some_and(|children| !children.is_empty())
}

/// 执行 has_styled_frame_characteristics 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn has_styled_frame_characteristics(object: &Map<String, Value>) -> bool {
    object.get("size").is_some()
        || object.get("backgroundColor").is_some()
        || object.get("fillPaints").is_some()
        || object.get("fills").is_some()
        || object.get("effects").is_some()
}

/// 执行 first_visible_paint 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn first_visible_paint(value: &Value) -> Option<&Map<String, Value>> {
    value.as_array()?.iter().find_map(|paint| {
        let object = paint.as_object()?;
        if object.get("visible").and_then(Value::as_bool) == Some(false) {
            None
        } else {
            Some(object)
        }
    })
}

/// 执行 read_clip_value 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn read_clip_value(object: &Map<String, Value>) -> Option<bool> {
    object
        .get("clipsContent")
        .or_else(|| object.get("clip"))
        .and_then(Value::as_bool)
        .or_else(|| object.get("frameMaskDisabled").and_then(Value::as_bool).map(|value| !value))
}

/// 执行 read_node_type 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn read_node_type(object: &Map<String, Value>) -> Option<&str> {
    object.get("type").and_then(value_to_str)
}

/// 执行 value_to_str 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn value_to_str(value: &Value) -> Option<&str> {
    value.as_str().or_else(|| {
        value.as_object().and_then(|object| object.get("value")).and_then(Value::as_str)
    })
}

/// 执行 read_guid_key_from_object 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn read_guid_key_from_object(object: &Map<String, Value>, key: &str) -> Option<String> {
    object.get(key).and_then(read_guid_key_from_value)
}

/// 执行 read_guid_key_from_value 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn read_guid_key_from_value(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let session_id = object.get("sessionID")?.as_u64()?;
    let local_id = object.get("localID")?.as_u64()?;
    Some(format!("{session_id}:{local_id}"))
}

/// 处理颜色值的解析、格式化或空间转换。
///
/// 无法识别的颜色返回空结果，避免把错误颜色静默写入设计元素。
pub(super) fn normalized_color_channel(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// 执行 clone_number_or_string 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn clone_number_or_string(value: Option<&Value>) -> Option<Value> {
    match value? {
        Value::Number(number) => Some(Value::Number(number.clone())),
        Value::String(string) => Some(Value::String(string.clone())),
        _ => None,
    }
}

/// 执行 value_to_f64 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(string) => parse_measurement_string(string),
        Value::Object(object) => object.get("value").and_then(value_to_f64),
        _ => None,
    }
}

/// 执行 value_to_f32 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn value_to_f32(value: &Value) -> Option<f32> {
    value_to_f64(value).map(|number| number as f32)
}

/// 执行 is_image_fill 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn is_image_fill(fill: &Value) -> bool {
    match fill {
        Value::Object(object) => object.get("type").and_then(Value::as_str) == Some("image"),
        Value::Array(items) => items.first().is_some_and(is_image_fill),
        _ => false,
    }
}

#[cfg(test)]
#[path = "figma_support_tests.rs"]
mod figma_support_tests;
