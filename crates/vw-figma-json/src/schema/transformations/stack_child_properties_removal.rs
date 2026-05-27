use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 特定的堆栈子属性。
///
/// 此转换删除以下字段：
/// - `stackChildAlignSelf`：控制单个子项如何在其父项的自动布局中对齐
/// - `stackChildPrimaryGrow`：控制子项是否增长以填充主轴中的可用空间
///
/// 这些属性特定于 Figma 的自动布局子配置，HTML/CSS 渲染并不直接需要。
/// CSS 使用不同的机制(flexbox `align-self`、`flex-grow` 等)
/// 类似的行为，但映射并不总是 1:1 并且这些 Figma 特定的值可能不会
/// 直接翻译。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_stack_child_properties;
///
/// let mut tree = json!({
///     "name": "Button",
///     "stackChildAlignSelf": "STRETCH",
///     "stackChildPrimaryGrow": 1.0,
///     "size": {"x": 100.0, "y": 48.0}
/// });
///
/// remove_stack_child_properties(&mut tree).unwrap();
///
/// assert!(tree.get("stackChildAlignSelf").is_none());
/// assert!(tree.get("stackChildPrimaryGrow").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_stack_child_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除堆栈子属性
            map.remove("stackChildAlignSelf");
            map.remove("stackChildPrimaryGrow");

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
#[path = "stack_child_properties_removal_tests.rs"]
mod stack_child_properties_removal_tests;
