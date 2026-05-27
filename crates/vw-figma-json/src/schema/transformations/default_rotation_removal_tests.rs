    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_default_rotation() {
        let mut tree = json!({
            "name": "Image",
            "rotation": 0.0,
            "scale": 0.5
        });

        remove_default_rotation(&mut tree).unwrap();

        assert!(tree.get("rotation").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
        assert_eq!(tree.get("scale").unwrap().as_f64(), Some(0.5));
    }

    #[test]
    fn test_preserve_non_zero_rotation() {
        let mut tree = json!({
            "name": "Image",
            "rotation": 45.0,
            "scale": 1.0
        });

        remove_default_rotation(&mut tree).unwrap();

        // 应保留非零旋转
        assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(45.0));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
    }

    #[test]
    fn test_preserve_negative_rotation() {
        let mut tree = json!({
            "name": "Image",
            "rotation": -30.0
        });

        remove_default_rotation(&mut tree).unwrap();

        // 应保留负旋转
        assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(-30.0));
    }

    #[test]
    fn test_preserve_various_rotations() {
        let rotations = vec![15.0, 30.0, 45.0, 90.0, 180.0, 270.0, -45.0, -90.0];

        for rotation_value in rotations {
            let mut tree = json!({
                "rotation": rotation_value
            });

            remove_default_rotation(&mut tree).unwrap();

            // 应保留所有非零旋转
            assert_eq!(tree.get("rotation").unwrap().as_f64(), Some(rotation_value));
        }
    }

    #[test]
    fn test_no_rotation() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_rotation(&mut tree).unwrap();

        // 没有旋转的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("rotation").is_none());
    }

    #[test]
    fn test_rotation_in_image_paint() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "IMAGE",
                    "rotation": 0.0,
                    "scale": 0.5,
                    "image": {
                        "filename": "test.png"
                    }
                }
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // 旋转 0.0 应删除
        assert!(tree["fillPaints"][0].get("rotation").is_none());
        assert_eq!(tree["fillPaints"][0]["scale"].as_f64(), Some(0.5));
    }

    #[test]
    fn test_rotation_in_transform() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "IMAGE",
                    "transform": {
                        "rotation": 0.0,
                        "x": 100.0,
                        "y": 200.0
                    }
                }
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // 旋转 0.0 应从变换中删除
        assert!(tree["fillPaints"][0]["transform"].get("rotation").is_none());
        assert_eq!(tree["fillPaints"][0]["transform"]["x"].as_f64(), Some(100.0));
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "rotation": 0.0
                },
                {
                    "name": "Child2",
                    "rotation": 15.0
                }
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // 旋转 0.0 已删除，15.0 保留
        assert!(tree["children"][0].get("rotation").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["rotation"].as_f64(), Some(15.0));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "RECTANGLE",
                        "rotation": 0.0,
                        "fillPaints": [
                            {
                                "type": "IMAGE",
                                "rotation": 0.0
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_rotation(&mut tree).unwrap();

        // 所有级别的旋转 0.0 都应该被删除
        let rect = &tree["document"]["children"][0];
        assert!(rect.get("rotation").is_none());
        assert!(rect["fillPaints"][0].get("rotation").is_none());
        assert_eq!(rect["type"].as_str(), Some("RECTANGLE"));
    }

    #[test]
    fn test_multiple_default_rotations() {
        let mut tree = json!({
            "children": [
                {"rotation": 0.0, "name": "A"},
                {"rotation": 0.0, "name": "B"},
                {"rotation": 0.0, "name": "C"}
            ]
        });

        remove_default_rotation(&mut tree).unwrap();

        // 所有旋转 0.0 应删除
        assert!(tree["children"][0].get("rotation").is_none());
        assert!(tree["children"][1].get("rotation").is_none());
        assert!(tree["children"][2].get("rotation").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_rotation_as_integer() {
        let mut tree = json!({
            "name": "Shape",
            "rotation": 0
        });

        remove_default_rotation(&mut tree).unwrap();

        // 整数 0 也应该被删除(因为 0 == 0.0)
        assert!(tree.get("rotation").is_none());
    }

    #[test]
    fn test_rotation_string_not_touched() {
        let mut tree = json!({
            "name": "Test",
            "rotation": "0.0"
        });

        remove_default_rotation(&mut tree).unwrap();

        // 字符串旋转不应被触及
        assert_eq!(tree.get("rotation").unwrap().as_str(), Some("0.0"));
    }
