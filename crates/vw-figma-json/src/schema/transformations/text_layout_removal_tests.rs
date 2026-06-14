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

#[test]
fn test_non_object_derived_text_data_is_preserved() {
    let mut tree = json!({
        "children": [
            {
                "name": "ArrayValue",
                "derivedTextData": [
                    {
                        "baselines": [{"lineY": 10.0}],
                        "layoutSize": {"x": 100.0, "y": 50.0}
                    }
                ]
            },
            {
                "name": "NullValue",
                "derivedTextData": null
            },
            {
                "name": "ObjectValue",
                "derivedTextData": {
                    "baselines": [{"lineY": 20.0}],
                    "layoutSize": {"x": 200.0, "y": 60.0}
                }
            }
        ]
    });

    remove_text_layout_fields(&mut tree).unwrap();

    assert!(tree["children"][0]["derivedTextData"].is_array());
    assert!(tree["children"][0]["derivedTextData"][0].get("baselines").is_some());
    assert!(tree["children"][1]["derivedTextData"].is_null());
    assert!(tree["children"][2]["derivedTextData"].get("baselines").is_none());
    assert!(tree["children"][2]["derivedTextData"].get("layoutSize").is_some());
}
