use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从衍生文本数据对象中删除layoutSize字段
///
/// 递归遍历JSON树并移除其中的 "layoutSize" 字段
/// "derivedTextData" 对象。 layoutSize 是多余的，因为它通常
/// 与节点的 "size" 字段匹配，因此删除它会减少 JSON 大小而不用
/// 丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功从衍生文本数据中删除所有布局大小字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_derived_text_layout_size;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "layoutSize": {"x": 100.0, "y": 50.0},
///         "otherInfo": "preserved"
///     },
///     "size": {"x": 100.0, "y": 50.0}
/// });
/// remove_derived_text_layout_size(&mut tree).unwrap();
/// // derivedTextData 现在只有 "otherInfo"
/// ```
pub fn remove_derived_text_layout_size(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从衍生文本数据对象中递归删除layoutSize
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "derivedTextData" 字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // 这可能是具有layoutSize 的衍生文本数据对象
                    if let Some(derived_text_data) = map.get_mut(&key)
                        && let Some(data_obj) = derived_text_data.as_object_mut()
                    {
                        // 删除 layoutSize 字段
                        data_obj.remove("layoutSize");
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
#[path = "derived_text_layout_size_removal_tests.rs"]
mod derived_text_layout_size_removal_tests;
