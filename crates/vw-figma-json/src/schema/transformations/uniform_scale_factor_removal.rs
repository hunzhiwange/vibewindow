use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当其默认值为 1.0 时，删除uniformScaleFactor字段
///
/// 递归遍历 JSON 树并删除具有 "uniformScaleFactor" 字段
/// 值 1.0。这是默认比例因子，因此省略它会减少输出大小
/// 而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认的uniformScaleFactor字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_uniform_scale_factor;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "uniformScaleFactor": 1.0,
///     "width": 100
/// });
/// remove_default_uniform_scale_factor(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "width" 字段
/// ```
pub fn remove_default_uniform_scale_factor(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认的uniformScaleFactor字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查uniformScaleFactor是否存在并且是否为1.0
            if let Some(scale_factor) = map.get("uniformScaleFactor")
                && let Some(num) = scale_factor.as_f64()
            {
                // 如果恰好为 1.0，则删除
                if (num - 1.0).abs() < f64::EPSILON {
                    map.remove("uniformScaleFactor");
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
    fn test_remove_default_uniform_scale_factor() {
        let mut tree = json!({
            "name": "Shape",
            "uniformScaleFactor": 1.0,
            "width": 100
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        assert!(tree.get("uniformScaleFactor").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    }

    #[test]
    fn test_preserve_non_default_scale_factor() {
        let mut tree = json!({
            "name": "Shape",
            "uniformScaleFactor": 2.5,
            "width": 100
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 应保留非默认比例因子
        assert_eq!(tree.get("uniformScaleFactor").unwrap().as_f64(), Some(2.5));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    }

    #[test]
    fn test_preserve_various_scale_factors() {
        let scale_factors = vec![0.5, 1.5, 2.0, 0.1, 10.0];

        for factor in scale_factors {
            let mut tree = json!({
                "uniformScaleFactor": factor
            });

            remove_default_uniform_scale_factor(&mut tree).unwrap();

            // 应保留所有非默认比例因子
            assert_eq!(tree.get("uniformScaleFactor").unwrap().as_f64(), Some(factor));
        }
    }

    #[test]
    fn test_no_scale_factor() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 没有uniformScaleFactor的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("uniformScaleFactor").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "uniformScaleFactor": 1.0
                },
                {
                    "name": "Child2",
                    "uniformScaleFactor": 2.0
                }
            ]
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 删除默认值 (1.0)，保留 2.0
        assert!(tree["children"][0].get("uniformScaleFactor").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["uniformScaleFactor"].as_f64(), Some(2.0));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "uniformScaleFactor": 1.0,
                        "layers": [
                            {
                                "type": "SHAPE",
                                "uniformScaleFactor": 1.0
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 应在所有级别删除所有默认比例因子
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("uniformScaleFactor").is_none());
        assert!(frame["layers"][0].get("uniformScaleFactor").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_multiple_default_scale_factors() {
        let mut tree = json!({
            "children": [
                {"uniformScaleFactor": 1.0, "name": "A"},
                {"uniformScaleFactor": 1.0, "name": "B"},
                {"uniformScaleFactor": 1.0, "name": "C"}
            ]
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 应删除所有默认比例因子
        assert!(tree["children"][0].get("uniformScaleFactor").is_none());
        assert!(tree["children"][1].get("uniformScaleFactor").is_none());
        assert!(tree["children"][2].get("uniformScaleFactor").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_zero_scale_factor() {
        let mut tree = json!({
            "uniformScaleFactor": 0.0
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 零不是默认值，因此应该保留
        assert_eq!(tree.get("uniformScaleFactor").unwrap().as_f64(), Some(0.0));
    }

    #[test]
    fn test_integer_one_vs_float_one() {
        // JSON不区分1和1.0，两者都被视为数字
        let mut tree = json!({
            "uniformScaleFactor": 1
        });

        remove_default_uniform_scale_factor(&mut tree).unwrap();

        // 整数 1 也应该被删除(它与 1.0 相同)
        assert!(tree.get("uniformScaleFactor").is_none());
    }
}
