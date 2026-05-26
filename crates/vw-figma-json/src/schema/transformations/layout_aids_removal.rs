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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_removes_empty_guides() {
        let mut tree = json!({
            "name": "Frame",
            "guides": [],
            "type": "FRAME"
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("guides").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    }

    #[test]
    fn test_removes_guides_with_data() {
        let mut tree = json!({
            "name": "Frame",
            "guides": [
                {"axis": "X", "offset": 100.0},
                {"axis": "Y", "offset": 200.0}
            ],
            "type": "FRAME"
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("guides").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_removes_empty_layout_grids() {
        let mut tree = json!({
            "name": "Frame",
            "layoutGrids": [],
            "type": "FRAME"
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("layoutGrids").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_removes_layout_grids_with_complex_data() {
        let mut tree = json!({
            "name": "Frame",
            "layoutGrids": [
                {
                    "axis": "X",
                    "color": "#ff00001a",
                    "gutterSize": 20.0,
                    "numSections": 5,
                    "offset": 24.0,
                    "pattern": "STRIPES",
                    "sectionSize": 10.0,
                    "type": "STRETCH"
                }
            ],
            "type": "FRAME"
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("layoutGrids").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_removes_both_guides_and_layout_grids() {
        let mut tree = json!({
            "name": "Frame",
            "guides": [{"axis": "X", "offset": 50.0}],
            "layoutGrids": [{"pattern": "COLUMNS", "numSections": 12}],
            "type": "FRAME"
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("guides").is_none());
        assert!(tree.get("layoutGrids").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "guides": [],
            "children": [
                {
                    "name": "Child1",
                    "layoutGrids": [{"pattern": "GRID"}]
                },
                {
                    "name": "Child2",
                    "guides": [{"axis": "Y"}],
                    "layoutGrids": []
                }
            ]
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("guides").is_none());
        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("layoutGrids").is_none());
        assert!(children[1].get("guides").is_none());
        assert!(children[1].get("layoutGrids").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "layoutGrids": [{"type": "ROWS"}],
            "children": [
                {
                    "name": "Level1",
                    "guides": [{"axis": "X"}],
                    "children": [
                        {
                            "name": "Level2",
                            "guides": [],
                            "layoutGrids": [{"pattern": "COLUMNS"}]
                        }
                    ]
                }
            ]
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("layoutGrids").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("guides").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("guides").is_none());
        assert!(level2.get("layoutGrids").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_layout_aids(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("type").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_layout_aids(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "guides": [],
            "layoutGrids": [{"pattern": "COLUMNS"}],
            "stackMode": "HORIZONTAL",
            "size": {"x": 100.0, "y": 100.0},
            "children": []
        });

        remove_layout_aids(&mut tree).unwrap();

        assert!(tree.get("guides").is_none());
        assert!(tree.get("layoutGrids").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("children").is_some());
    }
}
