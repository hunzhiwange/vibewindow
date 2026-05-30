use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当存在通用cornerRadius 时，删除冗余的各个角半径属性。
///
/// 当 `cornerRadius` 存在时，此转换会删除以下字段：
/// - `rectangleTopLeftCornerRadius`
/// - `rectangleTopRightCornerRadius`
/// - `rectangleBottomLeftCornerRadius`
/// - `rectangleBottomRightCornerRadius`
///
/// 在 Figma 中，当所有角的半径相同时，一般的 `cornerRadius` 和
/// 可能存在单独的角属性。对于 HTML/CSS 渲染，一般
/// `cornerRadius` 就足够了，可以映射到 `border-radius`。在这种情况下，
/// 单独的属性是多余的。
///
/// 如果 `cornerRadius` 不存在，则保留各个角属性
/// 因为它们可能代表每个角的不同半径。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_redundant_corner_radii;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "cornerRadius": 12.0,
///     "rectangleTopLeftCornerRadius": 12.0,
///     "rectangleTopRightCornerRadius": 12.0,
///     "rectangleBottomLeftCornerRadius": 12.0,
///     "rectangleBottomRightCornerRadius": 12.0
/// });
///
/// remove_redundant_corner_radii(&mut tree).unwrap();
///
/// assert!(tree.get("cornerRadius").is_some());
/// assert!(tree.get("rectangleTopLeftCornerRadius").is_none());
/// assert!(tree.get("rectangleTopRightCornerRadius").is_none());
/// assert!(tree.get("rectangleBottomLeftCornerRadius").is_none());
/// assert!(tree.get("rectangleBottomRightCornerRadius").is_none());
/// ```
pub fn remove_redundant_corner_radii(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 如果cornerRadius存在，则仅删除单个角半径
            if map.contains_key("cornerRadius") {
                map.remove("rectangleTopLeftCornerRadius");
                map.remove("rectangleTopRightCornerRadius");
                map.remove("rectangleBottomLeftCornerRadius");
                map.remove("rectangleBottomRightCornerRadius");
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
#[path = "redundant_corner_radii_removal_tests.rs"]
mod redundant_corner_radii_removal_tests;
