    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_default_letter_spacing() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PERCENT", "value": 0.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        assert!(tree.get("letterSpacing").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_default_line_height() {
        let mut tree = json!({
            "name": "Text",
            "lineHeight": {"units": "PERCENT", "value": 100.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        assert!(tree.get("lineHeight").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_both_defaults() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PERCENT", "value": 0.0},
            "lineHeight": {"units": "PERCENT", "value": 100.0},
            "fontSize": 14.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        assert!(tree.get("letterSpacing").is_none());
        assert!(tree.get("lineHeight").is_none());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_preserve_non_default_letter_spacing() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PERCENT", "value": 5.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        // 应保留非默认字母间距
        assert!(tree.get("letterSpacing").is_some());
        assert_eq!(tree["letterSpacing"]["value"].as_f64(), Some(5.0));
    }

    #[test]
    fn test_preserve_non_default_line_height() {
        let mut tree = json!({
            "name": "Text",
            "lineHeight": {"units": "PERCENT", "value": 120.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        // 应保留非默认 lineHeight
        assert!(tree.get("lineHeight").is_some());
        assert_eq!(tree["lineHeight"]["value"].as_f64(), Some(120.0));
    }

    #[test]
    fn test_preserve_pixels_units() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {"units": "PIXELS", "value": 0.0},
            "lineHeight": {"units": "PIXELS", "value": 100.0},
            "fontSize": 16.0
        });

        remove_default_text_properties(&mut tree).unwrap();

        // 即使使用默认值，也应保留非 PERCENT 单位
        assert!(tree.get("letterSpacing").is_some());
        assert!(tree.get("lineHeight").is_some());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "letterSpacing": {"units": "PERCENT", "value": 0.0}
                },
                {
                    "name": "Text2",
                    "lineHeight": {"units": "PERCENT", "value": 100.0}
                }
            ]
        });

        remove_default_text_properties(&mut tree).unwrap();

        // 两个嵌套默认值都应该被删除
        assert!(tree["children"][0].get("letterSpacing").is_none());
        assert!(tree["children"][1].get("lineHeight").is_none());
    }

    #[test]
    fn test_no_text_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_text_properties(&mut tree).unwrap();

        // 没有文本属性的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "letterSpacing": {"units": "PERCENT", "value": 0.0},
                        "lineHeight": {"units": "PERCENT", "value": 100.0}
                    }
                ]
            }
        });

        remove_default_text_properties(&mut tree).unwrap();

        let text_node = &tree["document"]["children"][0];
        assert!(text_node.get("letterSpacing").is_none());
        assert!(text_node.get("lineHeight").is_none());
        assert_eq!(text_node["type"].as_str(), Some("TEXT"));
    }

    #[test]
    fn test_is_default_letter_spacing() {
        assert!(is_default_letter_spacing(&json!({
            "units": "PERCENT",
            "value": 0.0
        })));

        assert!(!is_default_letter_spacing(&json!({
            "units": "PERCENT",
            "value": 5.0
        })));

        assert!(!is_default_letter_spacing(&json!({
            "units": "PIXELS",
            "value": 0.0
        })));

        assert!(!is_default_letter_spacing(&json!(0.0)));
    }

    #[test]
    fn test_is_default_line_height() {
        assert!(is_default_line_height(&json!({
            "units": "PERCENT",
            "value": 100.0
        })));

        assert!(!is_default_line_height(&json!({
            "units": "PERCENT",
            "value": 120.0
        })));

        assert!(!is_default_line_height(&json!({
            "units": "PIXELS",
            "value": 100.0
        })));

        assert!(!is_default_line_height(&json!(100.0)));
    }
