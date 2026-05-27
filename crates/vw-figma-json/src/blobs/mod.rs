pub mod parser;
pub mod substitution;

use crate::error::Result;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value as JsonValue;

// 重新导出常用项
pub use parser::{parse_blob, parse_commands, parse_vector_network};
pub use substitution::substitute_blobs;

/// 通过将二进制数据编码为 Base64 来处理 blob 数组
///
/// 从解码的 Kiwi 数据中获取 blobs 数组并转换任何二进制
/// 字节数组转换为 Base64 编码字符串以实现 JSON 兼容性。
///
/// # 参数
/// * `blobs` - 来自解码的 Kiwi 数据的 blob 对象数组
///
/// # 返回值
/// * `Ok(JsonValue)` - 具有 Base64 编码数据的已处理 blob 数组
/// * `Err(FigError)` - 如果 blob 处理失败
///
/// # 示例
/// ```no_run
/// use fig2json::blobs::process_blobs;
/// use serde_json::json;
///
/// let blobs = vec![/* blob objects */];
/// let processed = process_blobs(blobs).unwrap();
/// ```
pub fn process_blobs(blobs: Vec<JsonValue>) -> Result<JsonValue> {
    let mut processed_blobs = Vec::new();

    for blob in blobs {
        let mut processed_blob = blob.clone();

        // 如果 blob 有一个带有数组的 bytes 字段，则将其编码为 base64
        if let Some(obj) = processed_blob.as_object_mut()
            && let Some(bytes_value) = obj.get("bytes")
            && let Some(bytes_array) = bytes_value.as_array()
        {
            // 将 JSON 数字数组转换为字节向量
            let bytes: Vec<u8> =
                bytes_array.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect();

            // 编码为base64
            let base64_string = general_purpose::STANDARD.encode(&bytes);

            // 将字节数组替换为 Base64 字符串
            obj.insert("bytes".to_string(), JsonValue::String(base64_string));
        }

        processed_blobs.push(processed_blob);
    }

    Ok(JsonValue::Array(processed_blobs))
}

#[cfg(test)]
mod blobs_tests;
