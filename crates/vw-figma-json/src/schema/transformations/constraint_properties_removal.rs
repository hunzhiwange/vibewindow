use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 特定的约束属性。
///
/// 此转换删除以下字段：
/// - `horizontalConstraint`：控制节点在 Figma 的自动布局中水平调整大小的方式
/// - `verticalConstraint`：控制节点在 Figma 的自动布局中垂直调整大小的方式
///
/// 这些属性特定于 Figma 的约束系统，并不直接
/// ，因为 CSS 使用不同的机制(flexbox、网格等)
/// 用于响应式布局行为。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_constraint_properties;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "horizontalConstraint": "CENTER",
///     "verticalConstraint": "SCALE",
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_constraint_properties(&mut tree).unwrap();
///
/// assert!(tree.get("horizontalConstraint").is_none());
/// assert!(tree.get("verticalConstraint").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_constraint_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除约束属性
            map.remove("horizontalConstraint");
            map.remove("verticalConstraint");

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
#[path = "constraint_properties_removal_tests.rs"]
mod constraint_properties_removal_tests;
