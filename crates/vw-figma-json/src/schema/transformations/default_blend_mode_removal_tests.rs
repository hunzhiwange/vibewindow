    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_normal_blend_mode() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": "NORMAL",
            "opacity": 1.0
        });

        remove_default_blend_mode(&mut tree).unwrap();

        assert!(tree.get("blendMode").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_non_normal_blend_mode() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": "MULTIPLY",
            "opacity": 0.8
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 应保留非正常混合模式
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("MULTIPLY"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.8));
    }

    #[test]
    fn test_preserve_other_blend_modes() {
        let modes = vec!["MULTIPLY", "SCREEN", "OVERLAY", "DARKEN", "LIGHTEN"];

        for mode in modes {
            let mut tree = json!({
                "blendMode": mode
            });

            remove_default_blend_mode(&mut tree).unwrap();

            // 应保留所有非正常混合模式
            assert_eq!(tree.get("blendMode").unwrap().as_str(), Some(mode));
        }
    }

    #[test]
    fn test_no_blend_mode() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 没有 BlendMode 的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("blendMode").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "blendMode": "NORMAL"
                },
                {
                    "name": "Child2",
                    "blendMode": "MULTIPLY"
                }
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // NORMAL 已删除，MULTIPLY 已保留
        assert!(tree["children"][0].get("blendMode").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["blendMode"].as_str(), Some("MULTIPLY"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_blend_mode_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "blendMode": "NORMAL",
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "blendMode": "MULTIPLY",
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // NORMAL 从第一个paint中移除
        assert!(tree["fillPaints"][0].get("blendMode").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // 相乘保留在第二个paint中
        assert_eq!(tree["fillPaints"][1]["blendMode"].as_str(), Some("MULTIPLY"));
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "blendMode": "NORMAL",
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "blendMode": "NORMAL"
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 所有级别的所有 NORMAL 混合模式都应删除
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("blendMode").is_none());
        assert!(frame["fillPaints"][0].get("blendMode").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_blend_mode_enum_object_not_touched() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            }
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 不应触及枚举对象(这在 enum_simplification 之后运行)
        // 所以这应该保留原样
        assert!(tree.get("blendMode").is_some());
        let blend_mode = tree.get("blendMode").unwrap();
        assert!(blend_mode.is_object());
    }

    #[test]
    fn test_case_sensitive() {
        let mut tree = json!({
            "blendMode": "normal"
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 小写 "normal" 不应被删除(仅 "NORMAL")
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("normal"));
    }

    #[test]
    fn test_multiple_normal_blend_modes() {
        let mut tree = json!({
            "children": [
                {"blendMode": "NORMAL", "name": "A"},
                {"blendMode": "NORMAL", "name": "B"},
                {"blendMode": "NORMAL", "name": "C"}
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 所有正常混合模式应被删除
        assert!(tree["children"][0].get("blendMode").is_none());
        assert!(tree["children"][1].get("blendMode").is_none());
        assert!(tree["children"][2].get("blendMode").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }
