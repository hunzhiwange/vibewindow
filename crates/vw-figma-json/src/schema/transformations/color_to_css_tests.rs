    use super::*;
    use serde_json::json;

    #[test]
    fn test_float_to_byte() {
        assert_eq!(float_to_byte(0.0), 0);
        assert_eq!(float_to_byte(1.0), 255);
        assert_eq!(float_to_byte(0.5), 128);
        assert_eq!(float_to_byte(0.8725961446762085), 223); // From example
        assert_eq!(float_to_byte(0.06292760372161865), 16);
    }

    #[test]
    fn test_float_to_byte_clamping() {
        assert_eq!(float_to_byte(-0.5), 0); // Negative clamped to 0
        assert_eq!(float_to_byte(1.5), 255); // Over 1.0 clamped to 255
    }

    #[test]
    fn test_is_color_object() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5,
            "b": 0.5,
            "a": 1.0
        }))
        .unwrap();
        assert!(is_color_object(&color));

        let color_no_alpha = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.5,
            "g": 0.5,
            "b": 0.5
        }))
        .unwrap();
        assert!(is_color_object(&color_no_alpha));

        let not_color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "x": 10,
            "y": 20
        }))
        .unwrap();
        assert!(!is_color_object(&not_color));

        let incomplete_color =
            serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
                "r": 0.5,
                "g": 0.5
            }))
            .unwrap();
        assert!(!is_color_object(&incomplete_color));
    }

    #[test]
    fn test_convert_color_to_css_opaque() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.8725961446762085,
            "g": 0.06292760372161865,
            "b": 0.06292760372161865,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#df1010");
    }

    #[test]
    fn test_convert_color_to_css_transparent() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 1.0,
            "g": 0.0,
            "b": 0.0,
            "a": 0.5
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#ff000080");
    }

    #[test]
    fn test_convert_color_to_css_no_alpha() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.0,
            "g": 0.5,
            "b": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#0080ff");
    }

    #[test]
    fn test_convert_color_to_css_black() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 0.0,
            "g": 0.0,
            "b": 0.0,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#000000");
    }

    #[test]
    fn test_convert_color_to_css_white() {
        let color = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "r": 1.0,
            "g": 1.0,
            "b": 1.0,
            "a": 1.0
        }))
        .unwrap();

        let css = convert_color_to_css(&color).unwrap();
        assert_eq!(css, "#ffffff");
    }

    #[test]
    fn test_transform_simple_color() {
        let mut tree = json!({
            "name": "Rectangle",
            "color": {
                "r": 0.8725961446762085,
                "g": 0.06292760372161865,
                "b": 0.06292760372161865,
                "a": 1.0
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree.get("color").unwrap().as_str(), Some("#df1010"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_transform_multiple_colors() {
        let mut tree = json!({
            "backgroundColor": {
                "r": 1.0,
                "g": 0.0,
                "b": 0.0,
                "a": 1.0
            },
            "foregroundColor": {
                "r": 0.0,
                "g": 1.0,
                "b": 0.0,
                "a": 0.5
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree.get("backgroundColor").unwrap().as_str(), Some("#ff0000"));
        assert_eq!(tree.get("foregroundColor").unwrap().as_str(), Some("#00ff0080"));
    }

    #[test]
    fn test_transform_nested_colors() {
        let mut tree = json!({
            "name": "Root",
            "style": {
                "fill": {
                    "r": 1.0,
                    "g": 0.0,
                    "b": 0.0,
                    "a": 1.0
                },
                "stroke": {
                    "r": 0.0,
                    "g": 0.0,
                    "b": 1.0,
                    "a": 0.8
                }
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["style"]["fill"].as_str(), Some("#ff0000"));
        assert_eq!(tree["style"]["stroke"].as_str(), Some("#0000ffcc"));
    }

    #[test]
    fn test_transform_colors_in_array() {
        let mut tree = json!({
            "fills": [
                {
                    "type": "solid",
                    "color": {
                        "r": 1.0,
                        "g": 0.0,
                        "b": 0.0,
                        "a": 1.0
                    }
                },
                {
                    "type": "solid",
                    "color": {
                        "r": 0.0,
                        "g": 1.0,
                        "b": 0.0,
                        "a": 0.5
                    }
                }
            ]
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["fills"][0]["color"].as_str(), Some("#ff0000"));
        assert_eq!(tree["fills"][1]["color"].as_str(), Some("#00ff0080"));
    }

    #[test]
    fn test_transform_preserves_non_color_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "position": {
                "x": 10,
                "y": 20
            },
            "color": {
                "r": 1.0,
                "g": 0.0,
                "b": 0.0,
                "a": 1.0
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        // 颜色应该转换
        assert_eq!(tree.get("color").unwrap().as_str(), Some("#ff0000"));

        // 位置应保持不变
        assert_eq!(tree["position"]["x"].as_i64(), Some(10));
        assert_eq!(tree["position"]["y"].as_i64(), Some(20));

        // 名称应保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_transform_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "name": "Frame",
                        "fills": [
                            {
                                "type": "solid",
                                "color": {
                                    "r": 0.5,
                                    "g": 0.5,
                                    "b": 0.5,
                                    "a": 1.0
                                }
                            }
                        ]
                    }
                ]
            }
        });

        transform_colors_to_css(&mut tree).unwrap();

        assert_eq!(tree["document"]["children"][0]["fills"][0]["color"].as_str(), Some("#808080"));
    }
