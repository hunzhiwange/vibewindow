    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_empty_postscript() {
        let mut tree = json!({
            "name": "Text",
            "fontName": {
                "family": "Inter",
                "style": "Regular",
                "postscript": ""
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = tree.get("fontName").unwrap();
        assert!(font_name.get("postscript").is_none());
        assert_eq!(font_name.get("family").unwrap().as_str(), Some("Inter"));
        assert_eq!(font_name.get("style").unwrap().as_str(), Some("Regular"));
    }

    #[test]
    fn test_preserve_non_empty_postscript() {
        let mut tree = json!({
            "name": "Text",
            "fontName": {
                "family": "Helvetica",
                "style": "Bold",
                "postscript": "Helvetica-Bold"
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = tree.get("fontName").unwrap();
        // 应保留非空附言
        assert_eq!(font_name.get("postscript").unwrap().as_str(), Some("Helvetica-Bold"));
        assert_eq!(font_name.get("family").unwrap().as_str(), Some("Helvetica"));
        assert_eq!(font_name.get("style").unwrap().as_str(), Some("Bold"));
    }

    #[test]
    fn test_no_postscript_field() {
        let mut tree = json!({
            "name": "Text",
            "fontName": {
                "family": "Arial",
                "style": "Regular"
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = tree.get("fontName").unwrap();
        // 不带postscript的字体名称应保持不变
        assert!(font_name.get("postscript").is_none());
        assert_eq!(font_name.get("family").unwrap().as_str(), Some("Arial"));
        assert_eq!(font_name.get("style").unwrap().as_str(), Some("Regular"));
    }

    #[test]
    fn test_no_font_name() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // 没有 fontName 的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("fontName").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "fontName": {
                        "family": "Inter",
                        "postscript": ""
                    }
                },
                {
                    "name": "Text2",
                    "fontName": {
                        "family": "Roboto",
                        "postscript": ""
                    }
                }
            ]
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // 两个空附言都应该删除
        assert!(tree["children"][0]["fontName"].get("postscript").is_none());
        assert_eq!(tree["children"][0]["fontName"]["family"].as_str(), Some("Inter"));
        assert!(tree["children"][1]["fontName"].get("postscript").is_none());
        assert_eq!(tree["children"][1]["fontName"]["family"].as_str(), Some("Roboto"));
    }

    #[test]
    fn test_mixed_empty_and_non_empty() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "fontName": {
                        "family": "Inter",
                        "postscript": ""
                    }
                },
                {
                    "name": "Text2",
                    "fontName": {
                        "family": "Helvetica",
                        "postscript": "Helvetica-Bold"
                    }
                }
            ]
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // 空postscript删除，非空保留
        assert!(tree["children"][0]["fontName"].get("postscript").is_none());
        assert_eq!(tree["children"][1]["fontName"]["postscript"].as_str(), Some("Helvetica-Bold"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "fontName": {
                            "family": "Times",
                            "style": "Italic",
                            "postscript": ""
                        }
                    }
                ]
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        let font_name = &tree["document"]["children"][0]["fontName"];
        assert!(font_name.get("postscript").is_none());
        assert_eq!(font_name["family"].as_str(), Some("Times"));
        assert_eq!(font_name["style"].as_str(), Some("Italic"));
    }

    #[test]
    fn test_postscript_non_string() {
        let mut tree = json!({
            "fontName": {
                "family": "Test",
                "postscript": 123
            }
        });

        remove_empty_font_postscript(&mut tree).unwrap();

        // 应保留非字符串postscript
        let font_name = tree.get("fontName").unwrap();
        assert!(font_name.get("postscript").is_some());
        assert_eq!(font_name.get("postscript").unwrap().as_i64(), Some(123));
    }
