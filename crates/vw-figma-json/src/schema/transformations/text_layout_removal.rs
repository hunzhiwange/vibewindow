use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从衍生文本数据对象中删除详细的文本布局数据
///
/// 递归遍历 JSON 树并从中删除详细的布局字段
/// "derivedTextData" 对象：
/// - "baselines" - 精确的线基线定位
/// - "logicalIndexToCharacterOffsetMap" - 角色位置图
/// - "fontMetaData" - 字体摘要和元数据数组
/// - "derivedLines" - 线路方向性信息
/// - "truncatedHeight" - 截断高度值
/// - "truncationStartIndex" - 截断起始索引
///
/// 这些字段包含精确的文本布局数据，这些数据不需要
/// 基本 HTML/CSS 文本渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有文本布局字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_text_layout_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "baselines": [{"lineY": 10.0, "width": 100.0}],
///         "logicalIndexToCharacterOffsetMap": [0.0, 10.0, 20.0],
///         "fontMetaData": [{"fontDigest": [1, 2, 3]}],
///         "layoutSize": {"x": 100.0, "y": 50.0}
///     }
/// });
/// remove_text_layout_fields(&mut tree).unwrap();
/// // derivedTextData 现在只有 "layoutSize"
/// ```
pub fn remove_text_layout_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除文本布局字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "derivedTextData" 字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // 该字段可能包含要删除的布局数据
                    if let Some(derived_text_data) = map.get_mut(&key)
                        && let Some(obj) = derived_text_data.as_object_mut()
                    {
                        // 删除所有详细布局字段
                        obj.remove("baselines");
                        obj.remove("logicalIndexToCharacterOffsetMap");
                        obj.remove("fontMetaData");
                        obj.remove("derivedLines");
                        obj.remove("truncatedHeight");
                        obj.remove("truncationStartIndex");
                    }
                }

                // 递归到该值，不管
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
    fn test_remove_baselines() {
        let mut tree = json!({
            "derivedTextData": {
                "baselines": [
                    {
                        "endCharacter": 5,
                        "firstCharacter": 0,
                        "lineAscent": 124.0,
                        "lineHeight": 155.0,
                        "lineY": 1.3871626833861228e-6,
                        "position": {"x": 0.0, "y": 124.04545593261719},
                        "width": 306.375
                    }
                ],
                "layoutSize": {"x": 307.0, "y": 155.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("baselines").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_character_offset_map() {
        let mut tree = json!({
            "derivedTextData": {
                "logicalIndexToCharacterOffsetMap": [0.0, 94.75, 169.25, 199.625, 230.0],
                "layoutSize": {"x": 307.0, "y": 155.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("logicalIndexToCharacterOffsetMap").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_font_metadata() {
        let mut tree = json!({
            "derivedTextData": {
                "fontMetaData": [
                    {
                        "fontDigest": [212, 131, 226, 199],
                        "fontLineHeight": 1.2102272510528564,
                        "fontStyle": {"__enum__": "FontStyle", "value": "NORMAL"},
                        "fontWeight": 400,
                        "key": {
                            "family": "Inter",
                            "postscript": "",
                            "style": "Regular"
                        }
                    }
                ],
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("fontMetaData").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_derived_lines() {
        let mut tree = json!({
            "derivedTextData": {
                "derivedLines": [
                    {
                        "directionality": {"__enum__": "Directionality", "value": "LTR"}
                    }
                ],
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("derivedLines").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_truncation_fields() {
        let mut tree = json!({
            "derivedTextData": {
                "truncatedHeight": 100.0,
                "truncationStartIndex": 42,
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("truncatedHeight").is_none());
        assert!(derived_text_data.get("truncationStartIndex").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_remove_all_layout_fields() {
        let mut tree = json!({
            "derivedTextData": {
                "baselines": [{"lineY": 10.0}],
                "logicalIndexToCharacterOffsetMap": [0.0, 10.0],
                "fontMetaData": [{"fontDigest": [1, 2, 3]}],
                "derivedLines": [{"directionality": {"__enum__": "Directionality", "value": "LTR"}}],
                "truncatedHeight": -1.0,
                "truncationStartIndex": -1,
                "layoutSize": {"x": 100.0, "y": 50.0}
            }
        });

        remove_text_layout_fields(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("baselines").is_none());
        assert!(derived_text_data.get("logicalIndexToCharacterOffsetMap").is_none());
        assert!(derived_text_data.get("fontMetaData").is_none());
        assert!(derived_text_data.get("derivedLines").is_none());
        assert!(derived_text_data.get("truncatedHeight").is_none());
        assert!(derived_text_data.get("truncationStartIndex").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_preserve_other_derived_text_data_fields() {
        let mut tree = json!({
            "name": "TextNode",
            "derivedTextData": {
                "baselines": [{"lineY": 10.0}],
                "layoutSize": {"x": 100.0, "y": 50.0},
                "customField": "preserved"
            },
            "visible": true
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // 检查非布局字段是否被保留
        assert_eq!(tree.get("name").unwrap().as_str(), Some("TextNode"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));

        // 检查衍生文本数据是否保留非布局字段
        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("baselines").is_none());
        assert!(derived_text_data.get("layoutSize").is_some());
        assert_eq!(derived_text_data.get("customField").unwrap().as_str(), Some("preserved"));
    }

    #[test]
    fn test_nested_derived_text_data() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "derivedTextData": {
                        "baselines": [{"lineY": 10.0}],
                        "layoutSize": {"x": 100.0, "y": 50.0}
                    }
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "derivedTextData": {
                                "fontMetaData": [{"fontDigest": [1, 2, 3]}],
                                "layoutSize": {"x": 200.0, "y": 100.0}
                            }
                        }
                    ]
                }
            ]
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // 检查第一个嵌套的derivedTextData
        let child1_data = &tree["children"][0]["derivedTextData"];
        assert!(child1_data.get("baselines").is_none());
        assert!(child1_data.get("layoutSize").is_some());

        // 检查深度嵌套的derivedTextData
        let deep_child_data = &tree["children"][1]["children"][0]["derivedTextData"];
        assert!(deep_child_data.get("fontMetaData").is_none());
        assert!(deep_child_data.get("layoutSize").is_some());
    }

    #[test]
    fn test_no_derived_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // 没有衍生文本数据的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert!(tree.get("derivedTextData").is_none());
    }

    #[test]
    fn test_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {}
        });

        remove_text_layout_fields(&mut tree).unwrap();

        // 空的 derivedTextData 应保持为空
        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert_eq!(derived_text_data.as_object().unwrap().len(), 0);
    }
}
