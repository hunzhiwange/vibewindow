    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_background_enabled() {
        let mut tree = json!({
            "name": "Canvas",
            "backgroundColor": "#ffffff",
            "backgroundEnabled": true
        });

        remove_background_properties(&mut tree).unwrap();

        assert!(tree.get("backgroundEnabled").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Canvas"));
        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#ffffff"));
    }

    #[test]
    fn test_remove_background_opacity() {
        let mut tree = json!({
            "name": "Canvas",
            "backgroundColor": "#f5f5f5",
            "backgroundOpacity": 1.0
        });

        remove_background_properties(&mut tree).unwrap();

        assert!(tree.get("backgroundOpacity").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Canvas"));
        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#f5f5f5"));
    }

    #[test]
    fn test_remove_both_background_properties() {
        let mut tree = json!({
            "name": "Canvas",
            "backgroundColor": "#e0e0e0",
            "backgroundEnabled": true,
            "backgroundOpacity": 0.8
        });

        remove_background_properties(&mut tree).unwrap();

        assert!(tree.get("backgroundEnabled").is_none());
        assert!(tree.get("backgroundOpacity").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Canvas"));
        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#e0e0e0"));
    }

    #[test]
    fn test_preserve_background_color() {
        let mut tree = json!({
            "name": "Canvas",
            "backgroundColor": "#ff0000",
            "backgroundEnabled": true,
            "backgroundOpacity": 1.0,
            "visible": true
        });

        remove_background_properties(&mut tree).unwrap();

        // 背景颜色应保留
        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#ff0000"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert!(tree.get("backgroundEnabled").is_none());
        assert!(tree.get("backgroundOpacity").is_none());
    }

    #[test]
    fn test_no_background_properties() {
        let mut tree = json!({
            "name": "Frame",
            "width": 100,
            "height": 200
        });

        remove_background_properties(&mut tree).unwrap();

        // 没有背景属性的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("backgroundEnabled").is_none());
        assert!(tree.get("backgroundOpacity").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Canvas1",
                    "backgroundEnabled": true,
                    "backgroundColor": "#fff"
                },
                {
                    "name": "Canvas2",
                    "backgroundOpacity": 0.5,
                    "backgroundColor": "#000"
                }
            ]
        });

        remove_background_properties(&mut tree).unwrap();

        // 两个嵌套的背景属性都应该被删除
        assert!(tree["children"][0].get("backgroundEnabled").is_none());
        assert_eq!(tree["children"][0]["backgroundColor"].as_str(), Some("#fff"));

        assert!(tree["children"][1].get("backgroundOpacity").is_none());
        assert_eq!(tree["children"][1]["backgroundColor"].as_str(), Some("#000"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "CANVAS",
                        "backgroundColor": "#f0f0f0",
                        "backgroundEnabled": true,
                        "backgroundOpacity": 1.0
                    }
                ]
            }
        });

        remove_background_properties(&mut tree).unwrap();

        let canvas = &tree["document"]["children"][0];
        assert!(canvas.get("backgroundEnabled").is_none());
        assert!(canvas.get("backgroundOpacity").is_none());
        assert_eq!(canvas["backgroundColor"].as_str(), Some("#f0f0f0"));
        assert_eq!(canvas["type"].as_str(), Some("CANVAS"));
    }

    #[test]
    fn test_background_disabled() {
        let mut tree = json!({
            "name": "Canvas",
            "backgroundColor": "#ffffff",
            "backgroundEnabled": false,
            "backgroundOpacity": 0.5
        });

        remove_background_properties(&mut tree).unwrap();

        // 无论值如何，都应删除
        assert!(tree.get("backgroundEnabled").is_none());
        assert!(tree.get("backgroundOpacity").is_none());
        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#ffffff"));
    }
