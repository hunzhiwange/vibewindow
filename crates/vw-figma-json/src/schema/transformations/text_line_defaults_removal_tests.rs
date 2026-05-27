    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_all_defaults_removes_lines_array() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 整个行数组应该被删除
        assert!(tree["textData"].get("lines").is_none());
        assert_eq!(tree["textData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_preserve_non_default_indentation_level() {
        let mut tree = json!({
            "textData": {
                "characters": "Indented text",
                "lines": [
                    {
                        "indentationLevel": 2,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在，因为 indentationLevel 不是默认的
        assert!(tree["textData"].get("lines").is_some());
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(2));
        // 所有其他默认值应删除
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_preserve_list_item() {
        let mut tree = json!({
            "textData": {
                "characters": "• List item",
                "lines": [
                    {
                        "indentationLevel": 1,
                        "isFirstLineOfList": true,
                        "lineType": "UNORDERED_LIST",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(1));
        assert_eq!(lines[0]["isFirstLineOfList"].as_bool(), Some(true));
        assert_eq!(lines[0]["lineType"].as_str(), Some("UNORDERED_LIST"));
        // 应删除默认值
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_preserve_non_zero_style_id() {
        let mut tree = json!({
            "textData": {
                "characters": "Styled text",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 5
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines 数组应该仍然存在，因为 styleId 不是默认的
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["styleId"].as_i64(), Some(5));
        // 所有其他默认值应删除
        assert!(lines[0].get("indentationLevel").is_none());
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
    }

    #[test]
    fn test_multiple_lines_mixed() {
        let mut tree = json!({
            "textData": {
                "characters": "Multi-line text",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 1,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在，因为第二行有非默认缩进
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 2);
        // 第一行应为空(均为默认值)
        assert!(lines[0].as_object().unwrap().is_empty());
        // 第二行应该只有 indentationLevel
        assert_eq!(lines[1]["indentationLevel"].as_i64(), Some(1));
        assert!(lines[1].get("isFirstLineOfList").is_none());
    }

    #[test]
    fn test_nested_text_data() {
        let mut tree = json!({
            "children": [
                {
                    "textData": {
                        "characters": "First",
                        "lines": [
                            {
                                "indentationLevel": 0,
                                "isFirstLineOfList": false,
                                "lineType": "PLAIN",
                                "listStartOffset": 0,
                                "sourceDirectionality": "AUTO",
                                "styleId": 0
                            }
                        ]
                    }
                },
                {
                    "textData": {
                        "characters": "Second",
                        "lines": [
                            {
                                "indentationLevel": 1,
                                "isFirstLineOfList": false,
                                "lineType": "PLAIN",
                                "listStartOffset": 0,
                                "sourceDirectionality": "AUTO",
                                "styleId": 0
                            }
                        ]
                    }
                }
            ]
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 第一个子节点应该删除行数组(所有默认值)
        assert!(tree["children"][0]["textData"].get("lines").is_none());

        // 第二个子节点应该仍然有行数组(缩进不是默认的)
        assert!(tree["children"][1]["textData"].get("lines").is_some());
        let lines = tree["children"][1]["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(1));
    }

    #[test]
    fn test_text_value_lines() {
        let mut tree = json!({
            "textValue": {
                "characters": "Hello",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 也适用于 textValue.lines
        assert!(tree["textValue"].get("lines").is_none());
    }

    #[test]
    fn test_no_lines_field() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello"
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 没有行的树应该保持不变
        assert_eq!(tree["textData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_preserve_non_default_list_start_offset() {
        let mut tree = json!({
            "textData": {
                "characters": "Numbered list",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 5,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines[0]["listStartOffset"].as_i64(), Some(5));
        // 删除了其他默认值
        assert!(lines[0].get("indentationLevel").is_none());
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "textData": {
                            "characters": "Deep text",
                            "lines": [
                                {
                                    "indentationLevel": 0,
                                    "isFirstLineOfList": false,
                                    "lineType": "PLAIN",
                                    "listStartOffset": 0,
                                    "sourceDirectionality": "AUTO",
                                    "styleId": 0
                                }
                            ]
                        }
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        let text_data = &tree["document"]["children"][0]["textData"];
        assert!(text_data.get("lines").is_none());
        assert_eq!(text_data["characters"].as_str(), Some("Deep text"));
    }

    #[test]
    fn test_empty_lines_array() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello",
                "lines": []
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 应该保留空行数组(不是我们关心的)
        assert!(tree["textData"].get("lines").is_some());
        assert_eq!(tree["textData"]["lines"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_multiple_all_default_lines() {
        let mut tree = json!({
            "textData": {
                "characters": "Multi-line",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 所有行均为默认行，因此应删除整个行数组
        assert!(tree["textData"].get("lines").is_none());
    }
