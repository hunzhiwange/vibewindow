    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_text_property_object() {
        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PERCENT",
            "value": -1.0
        }))
        .unwrap();
        assert!(is_text_property_object(&obj));

        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PIXELS",
            "value": 20.0
        }))
        .unwrap();
        assert!(is_text_property_object(&obj));

        let not_text_prop = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "x": 10,
            "y": 20
        }))
        .unwrap();
        assert!(!is_text_property_object(&not_text_prop));
    }

    #[test]
    fn test_convert_percent_to_css_string() {
        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PERCENT",
            "value": -1.0
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), Some("-1%".to_string()));

        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PERCENT",
            "value": 100.0
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), Some("100%".to_string()));

        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PERCENT",
            "value": 150.5
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), Some("150.5%".to_string()));
    }

    #[test]
    fn test_convert_pixels_to_css_string() {
        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PIXELS",
            "value": 20.0
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), Some("20px".to_string()));

        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PIXELS",
            "value": 16.5
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), Some("16.5px".to_string()));

        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "PIXELS",
            "value": 0.0
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), Some("0px".to_string()));
    }

    #[test]
    fn test_convert_unknown_units_returns_none() {
        let obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "units": "UNKNOWN",
            "value": 10.0
        }))
        .unwrap();
        assert_eq!(convert_to_css_string(&obj), None);
    }

    #[test]
    fn test_simplify_letter_spacing() {
        let mut tree = json!({
            "name": "Text",
            "fontSize": 14.0,
            "letterSpacing": {
                "units": "PERCENT",
                "value": -1.0
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("-1%"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_simplify_line_height() {
        let mut tree = json!({
            "name": "Text",
            "fontSize": 14.0,
            "lineHeight": {
                "units": "PIXELS",
                "value": 20.0
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("lineHeight").unwrap().as_str(), Some("20px"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_simplify_both_properties() {
        let mut tree = json!({
            "name": "Text",
            "fontSize": 14.0,
            "letterSpacing": {
                "units": "PERCENT",
                "value": -1.0
            },
            "lineHeight": {
                "units": "PIXELS",
                "value": 20.0
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("-1%"));
        assert_eq!(tree.get("lineHeight").unwrap().as_str(), Some("20px"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_simplify_nested_text_properties() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "letterSpacing": {
                        "units": "PERCENT",
                        "value": -1.0
                    }
                },
                {
                    "name": "Child2",
                    "lineHeight": {
                        "units": "PIXELS",
                        "value": 16.0
                    }
                },
                {
                    "name": "Child3",
                    "letterSpacing": {
                        "units": "PERCENT",
                        "value": 0.0
                    },
                    "lineHeight": {
                        "units": "PIXELS",
                        "value": 24.0
                    }
                }
            ]
        });

        simplify_text_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert_eq!(children[0].get("letterSpacing").unwrap().as_str(), Some("-1%"));
        assert_eq!(children[1].get("lineHeight").unwrap().as_str(), Some("16px"));
        assert_eq!(children[2].get("letterSpacing").unwrap().as_str(), Some("0%"));
        assert_eq!(children[2].get("lineHeight").unwrap().as_str(), Some("24px"));
    }

    #[test]
    fn test_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Level1",
                    "lineHeight": {
                        "units": "PIXELS",
                        "value": 20.0
                    },
                    "children": [
                        {
                            "name": "Level2",
                            "letterSpacing": {
                                "units": "PERCENT",
                                "value": -1.0
                            },
                            "lineHeight": {
                                "units": "PIXELS",
                                "value": 16.0
                            }
                        }
                    ]
                }
            ]
        });

        simplify_text_properties(&mut tree).unwrap();

        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert_eq!(level1.get("lineHeight").unwrap().as_str(), Some("20px"));

        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert_eq!(level2.get("letterSpacing").unwrap().as_str(), Some("-1%"));
        assert_eq!(level2.get("lineHeight").unwrap().as_str(), Some("16px"));
    }

    #[test]
    fn test_handles_missing_properties() {
        let mut tree = json!({
            "name": "Text",
            "fontSize": 14.0
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
    }

    #[test]
    fn test_preserves_non_matching_structures() {
        let mut tree = json!({
            "name": "Text",
            "fontSize": 14.0,
            "letterSpacing": "already-simple",
            "lineHeight": 1.5,
            "otherProperty": {
                "units": "METERS",
                "value": 100.0
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        // letterSpacing and lineHeight should be unchanged (not matching structure)
        assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("already-simple"));
        assert_eq!(tree.get("lineHeight").unwrap().as_f64(), Some(1.5));

        // otherProperty 应保持不变(不是 letterSpacing 或 lineHeight)
        assert!(tree.get("otherProperty").unwrap().is_object());
    }

    #[test]
    fn test_handles_float_values() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {
                "units": "PERCENT",
                "value": -0.5
            },
            "lineHeight": {
                "units": "PIXELS",
                "value": 18.75
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("-0.5%"));
        assert_eq!(tree.get("lineHeight").unwrap().as_str(), Some("18.75px"));
    }

    #[test]
    fn test_handles_integer_values() {
        let mut tree = json!({
            "name": "Text",
            "letterSpacing": {
                "units": "PERCENT",
                "value": 100.0
            },
            "lineHeight": {
                "units": "PIXELS",
                "value": 24.0
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("100%"));
        assert_eq!(tree.get("lineHeight").unwrap().as_str(), Some("24px"));
    }

    #[test]
    fn test_real_world_example() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "color": "#ffffff"
                }
            ],
            "fontName": {
                "family": "Inter",
                "style": "Medium"
            },
            "fontSize": 14.0,
            "letterSpacing": {
                "units": "PERCENT",
                "value": -1.0
            },
            "lineHeight": {
                "units": "PIXELS",
                "value": 20.0
            },
            "name": "Members without roles",
            "size": {
                "x": 203.0,
                "y": 20.0
            }
        });

        simplify_text_properties(&mut tree).unwrap();

        assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("-1%"));
        assert_eq!(tree.get("lineHeight").unwrap().as_str(), Some("20px"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Members without roles"));
        assert!(tree.get("fontName").is_some());
        assert!(tree.get("size").is_some());
    }
