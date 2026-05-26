use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当旋转字段具有默认值 0.0 时删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "rotation" 字段
/// 值 0.0。由于 0.0 是 Figma 中的默认旋转(不旋转)
/// 和 CSS，省略它会减少输出大小而不丢失信息。
///
/// 这通常出现在图像绘制变换和其他变换上下文中。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认旋转字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_rotation;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "image": {
///         "rotation": 0.0,
///         "scale": 0.5
///     }
/// });
/// remove_default_rotation(&mut tree).unwrap();
/// // 图像现在只有 "scale" 字段
/// ```
pub fn remove_default_rotation(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认旋转字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查旋转是否存在且为0.0
            if let Some(rotation) = map.get("rotation")
                && let Some(n) = rotation.as_f64()
            {
                // 使用 epsilon 比较浮点
                if n.abs() < f64::EPSILON {
                    map.remove("rotation");
                }
            }

            // 递归到所有剩余值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
}
