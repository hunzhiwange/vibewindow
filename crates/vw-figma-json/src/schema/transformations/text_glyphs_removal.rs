use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文本对象中删除字形矢量数据
///
/// 递归遍历 JSON 树并从中删除 "glyphs" 数组
/// "derivedTextData" 对象。文本字形矢量路径(M、L、Q、Z 命令)
/// 在 JSON 输出中不需要，因此删除它们会显著减少文件大小。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有文本字形
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_text_glyphs;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "glyphs": [
///             {"advance": 0.74, "commands": ["Z", "M"]}
///         ],
///         "layoutInfo": "preserved"
///     }
/// });
/// remove_text_glyphs(&mut tree).unwrap();
/// // 树现在有 "derivedTextData": {"layoutInfo": "preserved"}
/// ```
pub fn remove_text_glyphs(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除文本字形
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "derivedTextData" 字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // 该字段可能包含要删除的字形
                    if let Some(derived_text_data) = map.get_mut(&key)
                        && let Some(obj) = derived_text_data.as_object_mut()
                    {
                        // 删除 "glyphs" 字段(如果存在)
                        obj.remove("glyphs");
                    }
                }

                // 递归到该值，不管
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "text_glyphs_removal_tests.rs"]
mod text_glyphs_removal_tests;
