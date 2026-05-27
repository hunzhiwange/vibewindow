    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_text_bidi_version() {
        let mut tree = json!({
            "name": "Text",
            "textBidiVersion": 1,
            "fontSize": 16.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textBidiVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_text_layout_versions() {
        let mut tree = json!({
            "name": "Text",
            "textExplicitLayoutVersion": 1,
            "textUserLayoutVersion": 5,
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textExplicitLayoutVersion").is_none());
        assert!(tree.get("textUserLayoutVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_font_variants() {
        let mut tree = json!({
            "name": "Text",
            "fontVariantCommonLigatures": true,
            "fontVariantContextualLigatures": true,
            "fontVariantNumericFigure": "LINING",
            "fontVariantNumericSpacing": "PROPORTIONAL",
            "fontVariations": [],
            "fontVersion": "1.0",
            "fontSize": 14.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("fontVariantCommonLigatures").is_none());
        assert!(tree.get("fontVariantContextualLigatures").is_none());
        assert!(tree.get("fontVariantNumericFigure").is_none());
        assert!(tree.get("fontVariantNumericSpacing").is_none());
        assert!(tree.get("fontVariations").is_none());
        assert!(tree.get("fontVersion").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_remove_emoji_image_set() {
        let mut tree = json!({
            "name": "Text",
            "emojiImageSet": {
                "__enum__": "EmojiImageSet",
                "value": "APPLE"
            },
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("emojiImageSet").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_auto_rename_and_tracking() {
        let mut tree = json!({
            "name": "Text",
            "autoRename": true,
            "textTracking": 0.0,
            "fontSize": 12.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("autoRename").is_none());
        assert!(tree.get("textTracking").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(12.0));
    }

    #[test]
    fn test_remove_text_decoration_skip_ink() {
        let mut tree = json!({
            "name": "Text",
            "textDecorationSkipInk": true,
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textDecorationSkipInk").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_all_text_metadata() {
        let mut tree = json!({
            "name": "ComplexText",
            "textBidiVersion": 1,
            "textExplicitLayoutVersion": 1,
            "textUserLayoutVersion": 5,
            "textDecorationSkipInk": true,
            "fontVariantCommonLigatures": true,
            "fontVariantContextualLigatures": true,
            "fontVariations": [],
            "fontVersion": "",
            "emojiImageSet": {"__enum__": "EmojiImageSet", "value": "APPLE"},
            "autoRename": true,
            "textTracking": 0.0,
            "fontSize": 128.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // 删除所有元数据字段
        assert!(tree.get("textBidiVersion").is_none());
        assert!(tree.get("textExplicitLayoutVersion").is_none());
        assert!(tree.get("textUserLayoutVersion").is_none());
        assert!(tree.get("textDecorationSkipInk").is_none());
        assert!(tree.get("fontVariantCommonLigatures").is_none());
        assert!(tree.get("fontVariantContextualLigatures").is_none());
        assert!(tree.get("fontVariations").is_none());
        assert!(tree.get("fontVersion").is_none());
        assert!(tree.get("emojiImageSet").is_none());
        assert!(tree.get("autoRename").is_none());
        assert!(tree.get("textTracking").is_none());

        // 保留其他字段
        assert_eq!(tree.get("name").unwrap().as_str(), Some("ComplexText"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(128.0));
    }

    #[test]
    fn test_nested_text_metadata() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "textBidiVersion": 1,
                    "autoRename": true
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "textUserLayoutVersion": 5,
                            "textTracking": 0.0
                        }
                    ]
                }
            ]
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // 检查第一个嵌套文本
        assert!(tree["children"][0].get("textBidiVersion").is_none());
        assert!(tree["children"][0].get("autoRename").is_none());
        assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

        // 检查深度嵌套的文本
        let deep_child = &tree["children"][1]["children"][0];
        assert!(deep_child.get("textUserLayoutVersion").is_none());
        assert!(deep_child.get("textTracking").is_none());
        assert_eq!(deep_child.get("name").unwrap().as_str(), Some("DeepChild"));
    }

    #[test]
    fn test_no_text_metadata() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // 没有文本元数据的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserves_important_text_fields() {
        let mut tree = json!({
            "name": "Hello",
            "autoRename": true,
            "textTracking": 0.0,
            "fontName": {
                "family": "Inter",
                "style": "Regular"
            },
            "fontSize": 128.0,
            "textData": {
                "characters": "Hello"
            }
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        // 元数据已删除
        assert!(tree.get("autoRename").is_none());
        assert!(tree.get("textTracking").is_none());

        // 保留重要字段
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Hello"));
        assert!(tree.get("fontName").is_some());
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(128.0));
        assert!(tree.get("textData").is_some());
        assert_eq!(tree["textData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_remove_text_align_vertical() {
        let mut tree = json!({
            "name": "Text",
            "textAlignVertical": "TOP",
            "fontSize": 16.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textAlignVertical").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_remove_text_auto_resize() {
        let mut tree = json!({
            "name": "Text",
            "textAutoResize": "WIDTH_AND_HEIGHT",
            "fontSize": 14.0
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("textAutoResize").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_remove_font_variant_numeric_properties() {
        let mut tree = json!({
            "name": "Members without roles",
            "fontVariantNumericFigure": "LINING",
            "fontVariantNumericSpacing": "PROPORTIONAL",
            "fontSize": 14.0,
            "fontName": {
                "family": "Inter",
                "style": "Medium"
            }
        });

        remove_text_metadata_fields(&mut tree).unwrap();

        assert!(tree.get("fontVariantNumericFigure").is_none());
        assert!(tree.get("fontVariantNumericSpacing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Members without roles"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
        assert!(tree.get("fontName").is_some());
    }
