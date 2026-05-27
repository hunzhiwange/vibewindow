use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 的角平滑属性。
///
/// 此转换删除了 `cornerSmoothing` 字段，该字段控制
/// Figma 中圆角的平滑度。此功能创建 "iOS-style"
/// 圆角比标准圆弧更平滑。
///
/// 标准 CSS `border-radius` 只支持圆弧，没有
/// 相当于 Figma 的角平滑功能。因此，该属性
/// 对于 HTML/CSS 渲染没有用，可以安全删除。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_corner_smoothing;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "cornerRadius": 12.0,
///     "cornerSmoothing": 0.6000000238418579,
///     "type": "ROUNDED_RECTANGLE"
/// });
///
/// remove_corner_smoothing(&mut tree).unwrap();
///
/// assert!(tree.get("cornerSmoothing").is_none());
/// assert!(tree.get("cornerRadius").is_some());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_corner_smoothing(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除角平滑属性
            map.remove("cornerSmoothing");

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
#[path = "corner_smoothing_removal_tests.rs"]
mod corner_smoothing_removal_tests;
