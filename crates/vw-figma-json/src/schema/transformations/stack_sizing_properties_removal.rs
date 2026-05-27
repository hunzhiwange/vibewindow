use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 特定的堆栈大小属性。
///
/// 此转换删除以下字段：
/// - `stackCounterSizing`：控制 Figma 自动布局中节点如何沿反轴调整大小
/// - `stackPrimarySizing`：控制 Figma 自动布局中节点如何沿主轴调整大小
///
/// 这些属性特定于 Figma 的自动布局尺寸系统，HTML/CSS 渲染并不直接需要，
/// 因为 CSS 使用不同的机制(flexbox、网格等)
/// 用于调整大小行为。
///
/// 常见值包括：
/// - `RESIZE_TO_FIT_WITH_IMPLICIT_SIZE`
/// - `FIXED`
/// - `AUTO`
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_stack_sizing_properties;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "stackCounterSizing": "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE",
///     "stackPrimarySizing": "FIXED",
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_stack_sizing_properties(&mut tree).unwrap();
///
/// assert!(tree.get("stackCounterSizing").is_none());
/// assert!(tree.get("stackPrimarySizing").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_stack_sizing_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除堆栈大小属性
            map.remove("stackCounterSizing");
            map.remove("stackPrimarySizing");

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
#[path = "stack_sizing_properties_removal_tests.rs"]
mod stack_sizing_properties_removal_tests;
