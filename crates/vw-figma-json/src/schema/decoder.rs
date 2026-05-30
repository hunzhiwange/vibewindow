use crate::error::{FigError, Result};
use kiwi_schema::{Schema, Value};
use serde_json::Value as JsonValue;

/// 将 .fig 文件数据解码为 JSON
///
/// 获取解压缩的模式和数据块并使用 Kiwi schema格式对其进行解码。
///
/// # 参数
/// * `schema_bytes` - 解压缩的schema 块(块 0)
/// * `data_bytes` - 解压缩的数据块(块1)
///
/// # 返回值
/// * `Ok(JsonValue)` - 解码后的 JSON 数据
/// * `Err(FigError)` - 如果解码失败
///
/// # 示例
/// ```no_run
/// use fig2json::schema::decode_fig_to_json;
///
/// let schema_bytes = vec![/* decompressed schema */];
/// let data_bytes = vec![/* decompressed data */];
/// let json = decode_fig_to_json(&schema_bytes, &data_bytes).unwrap();
/// ```
pub fn decode_fig_to_json(schema_bytes: &[u8], data_bytes: &[u8]) -> Result<JsonValue> {
    // 1. 解码二进制 schema
    let schema = Schema::decode(schema_bytes)
        .map_err(|_| FigError::ZipError("Failed to decode Kiwi binary schema".to_string()))?;

    // 2.查找根消息类型
    // 在 Figma .fig 文件中，根消息名为 "Message" 并包含 nodeChanges 和 blob
    let root_type_id = schema
        .defs
        .iter()
        .find(|def| {
            def.name == "Message"
                && def.fields.iter().any(|f| f.name == "nodeChanges")
                && def.fields.iter().any(|f| f.name == "blobs")
        })
        .map(|def| def.index)
        .ok_or_else(|| {
            FigError::ZipError("No root Message definition found in schema".to_string())
        })?;

    // 3. 解码消息数据
    let value = Value::decode(&schema, root_type_id, data_bytes)
        .map_err(|_| FigError::ZipError("Failed to decode message data".to_string()))?;

    // 4. 将 Kiwi 值转换为 JSON
    let json = kiwi_value_to_json(&value);

    Ok(json)
}

/// 将 Kiwi 值转换为 serde_json 值
fn kiwi_value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Byte(n) => JsonValue::Number((*n).into()),
        Value::Int(n) => JsonValue::Number((*n).into()),
        Value::UInt(n) => JsonValue::Number((*n).into()),
        Value::Float(f) => {
            // 处理特殊浮点值
            if f.is_nan() || f.is_infinite() {
                JsonValue::Null
            } else {
                JsonValue::Number(
                    serde_json::Number::from_f64(*f as f64)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )
            }
        }
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Int64(n) => JsonValue::Number((*n).into()),
        Value::UInt64(n) => JsonValue::Number((*n).into()),
        Value::Array(arr) => JsonValue::Array(arr.iter().map(kiwi_value_to_json).collect()),
        Value::Enum(enum_name, variant_name) => {
            // 将枚举表示为具有type 字段的对象
            let mut map = serde_json::Map::new();
            map.insert("__enum__".to_string(), JsonValue::String(enum_name.to_string()));
            map.insert("value".to_string(), JsonValue::String(variant_name.to_string()));
            JsonValue::Object(map)
        }
        Value::Object(_type_name, fields) => {
            // 转换为 JSON 对象
            let mut map = serde_json::Map::new();

            for (field_name, field_value) in fields {
                map.insert(field_name.to_string(), kiwi_value_to_json(field_value));
            }

            JsonValue::Object(map)
        }
    }
}

#[cfg(test)]
#[path = "decoder_tests.rs"]
mod decoder_tests;
