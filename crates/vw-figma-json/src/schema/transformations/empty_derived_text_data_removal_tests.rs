    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {},
            "fontSize": 16.0
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        assert!(tree.get("derivedTextData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_preserve_non_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "fontFamily": "Arial",
                "fontSize": 12.0
            }
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 应保留非空的derivedTextData
        assert!(tree.get("derivedTextData").is_some());
        let derived = tree.get("derivedTextData").unwrap();
        assert_eq!(derived.get("fontFamily").unwrap().as_str(), Some("Arial"));
        assert_eq!(derived.get("fontSize").unwrap().as_f64(), Some(12.0));
    }

    #[test]
    fn test_preserve_derived_text_data_with_one_field() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "characters": "Hello"
            }
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 衍生文本数据，即使只有一个字段也应该被保留
        assert!(tree.get("derivedTextData").is_some());
        assert_eq!(tree["derivedTextData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_no_derived_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 没有衍生文本数据的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("derivedTextData").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "derivedTextData": {}
                },
                {
                    "name": "Text2",
                    "derivedTextData": {
                        "info": "data"
                    }
                }
            ]
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 删除了空的derivedTextData
        assert!(tree["children"][0].get("derivedTextData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Text1"));

        // 保留非空的derivedTextData
        assert!(tree["children"][1].get("derivedTextData").is_some());
        assert_eq!(tree["children"][1]["derivedTextData"]["info"].as_str(), Some("data"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "derivedTextData": {},
                        "name": "Text"
                    }
                ]
            }
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        let text_node = &tree["document"]["children"][0];
        assert!(text_node.get("derivedTextData").is_none());
        assert_eq!(text_node["type"].as_str(), Some("TEXT"));
        assert_eq!(text_node["name"].as_str(), Some("Text"));
    }

    #[test]
    fn test_multiple_empty_derived_text_data() {
        let mut tree = json!({
            "children": [
                {"derivedTextData": {}, "name": "A"},
                {"derivedTextData": {}, "name": "B"},
                {"derivedTextData": {}, "name": "C"}
            ]
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 所有空的derivedTextData应被删除
        assert!(tree["children"][0].get("derivedTextData").is_none());
        assert!(tree["children"][1].get("derivedTextData").is_none());
        assert!(tree["children"][2].get("derivedTextData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_derived_text_data_in_arrays() {
        let mut tree = json!({
            "textNodes": [
                {
                    "derivedTextData": {}
                },
                {
                    "derivedTextData": {
                        "layoutSize": {"x": 100.0, "y": 50.0}
                    }
                }
            ]
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        assert!(tree["textNodes"][0].get("derivedTextData").is_none());
        assert!(tree["textNodes"][1].get("derivedTextData").is_some());
    }

    #[test]
    fn test_derived_text_data_not_object() {
        let mut tree = json!({
            "name": "Test",
            "derivedTextData": "not an object"
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 应保留非对象派生文本数据
        assert_eq!(tree.get("derivedTextData").unwrap().as_str(), Some("not an object"));
    }

    #[test]
    fn test_preserve_other_empty_objects() {
        let mut tree = json!({
            "name": "Test",
            "derivedTextData": {},
            "otherEmptyObject": {},
            "metadata": {}
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 仅应删除衍生文本数据
        assert!(tree.get("derivedTextData").is_none());
        assert!(tree.get("otherEmptyObject").is_some());
        assert!(tree.get("metadata").is_some());
    }
