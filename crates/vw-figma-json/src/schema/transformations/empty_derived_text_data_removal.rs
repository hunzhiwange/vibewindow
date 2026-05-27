use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当衍生文本数据字段为空对象时删除它
///
/// 递归遍历 JSON 树并删除 "derivedTextData" 字段
/// 是空对象 ({})。空的derivedTextData不提供任何有用的信息
/// 用于 HTML/CSS 渲染，因此删除它会减少 JSON 大小。
///
/// 非空的derivedTextData对象被保留，以防它们包含有用的数据。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有空的derivedTextData字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_empty_derived_text_data;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "derivedTextData": {},
///     "fontSize": 16.0
/// });
/// remove_empty_derived_text_data(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "fontSize" 字段
/// ```
pub fn remove_empty_derived_text_data(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除空的derivedTextData字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查衍生文本数据是否存在并且是一个空对象
            if let Some(derived_text_data) = map.get("derivedTextData")
                && let Some(obj) = derived_text_data.as_object()
                && obj.is_empty()
            {
                map.remove("derivedTextData");
            }

            // 递归到所有剩余值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
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
#[path = "empty_derived_text_data_removal_tests.rs"]
mod empty_derived_text_data_removal_tests;
