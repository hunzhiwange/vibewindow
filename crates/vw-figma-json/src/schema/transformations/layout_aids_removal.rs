use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除设计时布局辅助工具。
///
/// 此转换删除以下字段：
/// - `guides`：设计期间用于对齐的视觉指南
/// - `layoutGrids`：设计期间使用的网格/柱系统
///
/// 这些属性是 Figma 中用于对齐和布局的设计时辅助工具
/// 规划。 HTML/CSS 渲染不需要它们，因为它们不会影响
/// 实际渲染输出。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_layout_aids;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "guides": [],
///     "layoutGrids": [
///         {
///             "pattern": "COLUMNS",
///             "numSections": 12,
///             "gutterSize": 20.0
///         }
///     ],
///     "type": "FRAME"
/// });
///
/// remove_layout_aids(&mut tree).unwrap();
///
/// assert!(tree.get("guides").is_none());
/// assert!(tree.get("layoutGrids").is_none());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_layout_aids(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除布局辅助属性
            map.remove("guides");
            map.remove("layoutGrids");

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
#[path = "layout_aids_removal_tests.rs"]
mod layout_aids_removal_tests;
