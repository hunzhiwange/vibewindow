use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除多余的填充属性。
///
/// 当更通用的填充时，此转换会删除重复的填充字段
/// 属性已存在：
/// - 当 `stackHorizontalPadding` 存在时删除 `stackPaddingRight`
/// - 当 `stackVerticalPadding` 存在时删除 `stackPaddingBottom`
///
/// 在 Figma 的自动布局系统中，填充可以通过特定的方式指定
/// 边值(paddingRight、paddingBottom)或基于轴的值
/// (水平Padding、垂直Padding)。当两者都存在时，具体值
/// 是多余的，可以删除以减少 JSON 大小。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_redundant_padding;
///
/// let mut tree = json!({
///     "name": "Button",
///     "stackHorizontalPadding": 20.0,
///     "stackPaddingRight": 20.0,  // redundant
///     "stackVerticalPadding": 14.0,
///     "stackPaddingBottom": 14.0  // redundant
/// });
///
/// remove_redundant_padding(&mut tree).unwrap();
///
/// assert!(tree.get("stackHorizontalPadding").is_some());
/// assert!(tree.get("stackPaddingRight").is_none());
/// assert!(tree.get("stackVerticalPadding").is_some());
/// assert!(tree.get("stackPaddingBottom").is_none());
/// ```
pub fn remove_redundant_padding(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 如果 stackHorizontalPadding 存在，则删除 stackPaddingRight
            if map.contains_key("stackHorizontalPadding") {
                map.remove("stackPaddingRight");
            }

            // 如果 stackVerticalPadding 存在，则删除 stackPaddingBottom
            if map.contains_key("stackVerticalPadding") {
                map.remove("stackPaddingBottom");
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
#[path = "redundant_padding_removal_tests.rs"]
mod redundant_padding_removal_tests;
