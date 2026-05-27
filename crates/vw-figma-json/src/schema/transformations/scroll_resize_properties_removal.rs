use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 特定的滚动和调整大小属性。
///
/// 此转换删除以下字段：
/// - `scrollBehavior`：控制当父级滚动时框架是否保持固定
/// - `resizeToFit`：控制框架是否自动调整大小以适应其内容
///
/// 这些属性特定于 Figma 的框架行为，HTML/CSS 渲染并不直接需要。
/// CSS 使用不同的机制，例如 `position: fixed`，
/// `position: sticky`，并使用 flexbox/grid 自动调整大小。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_scroll_resize_properties;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
///     "resizeToFit": true,
///     "type": "FRAME"
/// });
///
/// remove_scroll_resize_properties(&mut tree).unwrap();
///
/// assert!(tree.get("scrollBehavior").is_none());
/// assert!(tree.get("resizeToFit").is_none());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_scroll_resize_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除滚动和调整大小属性
            map.remove("scrollBehavior");
            map.remove("resizeToFit");

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
#[path = "scroll_resize_properties_removal_tests.rs"]
mod scroll_resize_properties_removal_tests;
