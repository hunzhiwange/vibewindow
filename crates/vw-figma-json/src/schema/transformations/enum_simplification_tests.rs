    use super::*;
    use serde_json::json;

    #[test]
    fn test_simplify_node_type() {
        let mut tree = json!({
            "name": "Frame",
            "type": {
                "__enum__": "NodeType",
                "value": "FRAME"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_simplify_blend_mode() {
        let mut tree = json!({
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("NORMAL"));
    }

    #[test]
    fn test_simplify_paint_type() {
        let mut tree = json!({
            "type": {
                "__enum__": "PaintType",
                "value": "SOLID"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("type").unwrap().as_str(), Some("SOLID"));
    }

    #[test]
    fn test_simplify_multiple_enums() {
        let mut tree = json!({
            "type": {
                "__enum__": "NodeType",
                "value": "ROUNDED_RECTANGLE"
            },
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            },
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "INSIDE"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("type").unwrap().as_str(), Some("ROUNDED_RECTANGLE"));
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("NORMAL"));
        assert_eq!(tree.get("strokeAlign").unwrap().as_str(), Some("INSIDE"));
    }

    #[test]
    fn test_simplify_nested_enums() {
        let mut tree = json!({
            "name": "Root",
            "type": {
                "__enum__": "NodeType",
                "value": "DOCUMENT"
            },
            "children": [
                {
                    "name": "Child1",
                    "type": {
                        "__enum__": "NodeType",
                        "value": "FRAME"
                    }
                },
                {
                    "name": "Child2",
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    }
                }
            ]
        });

        simplify_enums(&mut tree).unwrap();

        // 根枚举简化
        assert_eq!(tree.get("type").unwrap().as_str(), Some("DOCUMENT"));

        // 儿童枚举简化
        assert_eq!(tree["children"][0]["type"].as_str(), Some("FRAME"));
        assert_eq!(tree["children"][1]["phase"].as_str(), Some("CREATED"));
    }

    #[test]
    fn test_simplify_deeply_nested_enums() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "fillPaints": [
                            {
                                "type": {
                                    "__enum__": "PaintType",
                                    "value": "IMAGE"
                                },
                                "blendMode": {
                                    "__enum__": "BlendMode",
                                    "value": "NORMAL"
                                }
                            }
                        ]
                    }
                ]
            }
        });

        simplify_enums(&mut tree).unwrap();

        // 深度嵌套枚举的简化
        let paint = &tree["document"]["children"][0]["fillPaints"][0];
        assert_eq!(paint["type"].as_str(), Some("IMAGE"));
        assert_eq!(paint["blendMode"].as_str(), Some("NORMAL"));
    }

    #[test]
    fn test_preserve_non_enum_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "transform": {
                "x": 100.0,
                "y": 200.0,
                "rotation": 0.0
            },
            "type": {
                "__enum__": "NodeType",
                "value": "FRAME"
            }
        });

        simplify_enums(&mut tree).unwrap();

        // 枚举简化
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));

        // 保留非枚举对象
        assert_eq!(tree["transform"]["x"].as_f64(), Some(100.0));
        assert_eq!(tree["transform"]["y"].as_f64(), Some(200.0));
        assert_eq!(tree["transform"]["rotation"].as_f64(), Some(0.0));
    }

    #[test]
    fn test_no_enums() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        simplify_enums(&mut tree).unwrap();

        // 没有枚举的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_different_enum_types() {
        let mut tree = json!({
            "textAlignVertical": {
                "__enum__": "TextAlignVertical",
                "value": "TOP"
            },
            "textAutoResize": {
                "__enum__": "TextAutoResize",
                "value": "WIDTH_AND_HEIGHT"
            },
            "lineType": {
                "__enum__": "LineType",
                "value": "PLAIN"
            },
            "fontStyle": {
                "__enum__": "FontStyle",
                "value": "NORMAL"
            }
        });

        simplify_enums(&mut tree).unwrap();

        // 所有枚举类型均已简化
        assert_eq!(tree.get("textAlignVertical").unwrap().as_str(), Some("TOP"));
        assert_eq!(tree.get("textAutoResize").unwrap().as_str(), Some("WIDTH_AND_HEIGHT"));
        assert_eq!(tree.get("lineType").unwrap().as_str(), Some("PLAIN"));
        assert_eq!(tree.get("fontStyle").unwrap().as_str(), Some("NORMAL"));
    }

    #[test]
    fn test_enum_in_array() {
        let mut tree = json!({
            "paints": [
                {
                    "type": {
                        "__enum__": "PaintType",
                        "value": "SOLID"
                    }
                },
                {
                    "type": {
                        "__enum__": "PaintType",
                        "value": "IMAGE"
                    }
                }
            ]
        });

        simplify_enums(&mut tree).unwrap();

        // 简化数组中的所有枚举
        assert_eq!(tree["paints"][0]["type"].as_str(), Some("SOLID"));
        assert_eq!(tree["paints"][1]["type"].as_str(), Some("IMAGE"));
    }

    #[test]
    fn test_is_enum_object() {
        let enum_obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "__enum__": "BlendMode",
            "value": "NORMAL"
        }))
        .unwrap();
        assert!(is_enum_object(&enum_obj));

        let not_enum = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "x": 10,
            "y": 20
        }))
        .unwrap();
        assert!(!is_enum_object(&not_enum));

        let incomplete = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "__enum__": "BlendMode"
        }))
        .unwrap();
        assert!(!is_enum_object(&incomplete));
    }

    #[test]
    fn test_extract_enum_value() {
        let enum_obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "__enum__": "BlendMode",
            "value": "NORMAL"
        }))
        .unwrap();

        let value = extract_enum_value(&enum_obj).unwrap();
        assert_eq!(value, "NORMAL");
    }
