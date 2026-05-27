use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 fontName 对象中删除空的 postscript 字段
///
/// 递归遍历JSON树并移除其中的 "postscript" 字段
/// "fontName" 为空字符串时对象。使用 postscript 字段
/// 指定 PostScript 字体名称，但为空时不提供任何信息
/// 并且可以安全地删除以减少 JSON 大小。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有空postscript字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_empty_font_postscript;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "fontName": {
///         "family": "Inter",
///         "style": "Regular",
///         "postscript": ""
///     }
/// });
/// remove_empty_font_postscript(&mut tree).unwrap();
/// // fontName 现在只有 "family" 和 "style" 字段
/// ```
pub fn remove_empty_font_postscript(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 fontName 对象中递归删除空的 postscript 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "fontName" 字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "fontName" {
                    // 这可能是带有 postscript 字段的 fontName 对象
                    if let Some(font_name) = map.get_mut(&key)
                        && let Some(font_obj) = font_name.as_object_mut()
                    {
                        // 检查 postscript 是否存在且为空
                        if let Some(postscript) = font_obj.get("postscript")
                            && let Some(s) = postscript.as_str()
                            && s.is_empty()
                        {
                            font_obj.remove("postscript");
                        }
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
#[path = "empty_font_postscript_removal_tests.rs"]
mod empty_font_postscript_removal_tests;
