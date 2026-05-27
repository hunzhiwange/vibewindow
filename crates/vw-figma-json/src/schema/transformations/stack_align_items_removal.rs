use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 特定的堆栈对齐属性。
///
/// 此转换删除以下字段：
/// - `stackCounterAlignItems`：控制项目沿横轴(垂直于堆叠方向)的对齐方式
/// - `stackPrimaryAlignItems`：控制项目沿主轴(平行于堆叠方向)的对齐/分布
///
/// 这些属性特定于 Figma 的自动布局配置，HTML/CSS 渲染并不直接需要。
/// CSS 使用不同的机制(flexbox `align-items`、`justify-content` 等)
/// 具有类似的行为，但 Figma 特定值不会按 1:1 转换。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_stack_align_items;
///
/// let mut tree = json!({
///     "name": "Row",
///     "stackMode": "HORIZONTAL",
///     "stackCounterAlignItems": "CENTER",
///     "stackPrimaryAlignItems": "SPACE_BETWEEN",
///     "size": {"x": 327.0, "y": 40.0}
/// });
///
/// remove_stack_align_items(&mut tree).unwrap();
///
/// assert!(tree.get("stackCounterAlignItems").is_none());
/// assert!(tree.get("stackPrimaryAlignItems").is_none());
/// assert!(tree.get("stackMode").is_some());
/// ```
pub fn remove_stack_align_items(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除堆栈对齐属性
            map.remove("stackCounterAlignItems");
            map.remove("stackPrimaryAlignItems");

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
#[path = "stack_align_items_removal_tests.rs"]
mod stack_align_items_removal_tests;
