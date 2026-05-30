use crate::blobs::parser::parse_blob;
use crate::error::Result;
use serde_json::Value as JsonValue;

/// 用解析的 blob 内容替换文档树中的 blob 引用
///
/// 递归遍历JSON树并替换以"Blob"结尾的字段
/// 及其解析内容。例如：
/// - `commandsBlob: 5` → `commands: ["M", 1.0, 2.0, ...]`
/// - `vectorNetworkBlob: 3` → `vectorNetwork: {vertices: [...], ...}`
///
/// blob索引用于在blobs数组中查找blob，然后
/// 根据字段名称解析(删除 "Blob" 后缀)。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
/// * `blobs` - 包含二进制数据的 blob 对象数组
///
/// # 返回值
/// * `Ok(())` - 成功替换所有 blob 引用
/// * `Err(FigError)` - 如果 blob 解析或访问失败
///
/// # 示例
/// ```no_run
/// use fig2json::blobs::substitute_blobs;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "commandsBlob": 0,
///     "children": []
/// });
/// let blobs = vec![json!({"bytes": vec![0u8]})];
/// substitute_blobs(&mut tree, &blobs).unwrap();
/// // 树现在有 "commands" 字段而不是 "commandsBlob"
/// ```
pub fn substitute_blobs(tree: &mut JsonValue, blobs: &[JsonValue]) -> Result<()> {
    substitute_blobs_recursive(tree, blobs)
}

/// 递归替换 JSON 值中的 blob 引用
fn substitute_blobs_recursive(value: &mut JsonValue, blobs: &[JsonValue]) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 收集要替换的 blob 字段(迭代时无法修改映射)
            let mut replacements = Vec::new();

            for (key, val) in map.iter() {
                if key.ends_with("Blob")
                    && let Some(index) = val.as_u64()
                {
                    let index = index as usize;
                    if index < blobs.len() {
                        // 解析 blob
                        let blob_type = &key[..key.len() - 4]; // Remove "Blob" suffix
                        if let Some(parsed) = parse_blob(blob_type, &blobs[index])? {
                            replacements.push((key.clone(), blob_type.to_string(), parsed));
                        }
                    }
                }
            }

            // 应用替换
            for (old_key, new_key, new_value) in replacements {
                map.remove(&old_key);
                map.insert(new_key, new_value);
            }

            // 递归到所有值
            for val in map.values_mut() {
                substitute_blobs_recursive(val, blobs)?;
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                substitute_blobs_recursive(val, blobs)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "substitution_tests.rs"]
mod substitution_tests;
