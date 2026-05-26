use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除阶段字段
///
/// 递归遍历 JSON 树并删除所有 "phase" 字段。
/// 这些字段包含 Figma 内部状态(通常为 {"__enum__": "NodePhase", "value": "CREATED"})
/// HTML/CSS 渲染不需要。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有相字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_phase_fields;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "phase": {
///         "__enum__": "NodePhase",
///         "value": "CREATED"
///     },
///     "visible": true
/// });
/// remove_phase_fields(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_phase_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除阶段字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除 "phase" 字段(如果存在)
            map.remove("phase");

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
    fn test_remove_phase_simple() {
        let mut tree = json!({
            "name": "Rectangle",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "visible": true
        });

        remove_phase_fields(&mut tree).unwrap();

        assert!(tree.get("phase").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_phase_nested() {
        let mut tree = json!({
            "name": "Root",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "children": [
                {
                    "name": "Child1",
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    }
                },
                {
                    "name": "Child2",
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "DELETED"
                    }
                }
            ]
        });

        remove_phase_fields(&mut tree).unwrap();

        // 根相应该被删除
        assert!(tree.get("phase").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));

        // 儿童阶段应该被删除
        assert!(tree["children"][0].get("phase").is_none());
        assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

        assert!(tree["children"][1].get("phase").is_none());
        assert_eq!(tree["children"][1].get("name").unwrap().as_str(), Some("Child2"));
    }

    #[test]
    fn test_remove_phase_deeply_nested() {
        let mut tree = json!({
            "document": {
                "phase": {
                    "__enum__": "NodePhase",
                    "value": "CREATED"
                },
                "children": [
                    {
                        "phase": {
                            "__enum__": "NodePhase",
                            "value": "CREATED"
                        },
                        "children": [
                            {
                                "phase": {
                                    "__enum__": "NodePhase",
                                    "value": "CREATED"
                                },
                                "name": "DeepChild"
                            }
                        ]
                    }
                ]
            }
        });

        remove_phase_fields(&mut tree).unwrap();

        // 所有级别的所有阶段都应删除
        assert!(tree["document"].get("phase").is_none());
        assert!(tree["document"]["children"][0].get("phase").is_none());
        assert!(tree["document"]["children"][0]["children"][0].get("phase").is_none());

        // 其他字段应保留
        assert_eq!(
            tree["document"]["children"][0]["children"][0].get("name").unwrap().as_str(),
            Some("DeepChild")
        );
    }

    #[test]
    fn test_remove_phase_missing() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "x": 10,
            "y": 20
        });

        remove_phase_fields(&mut tree).unwrap();

        // 没有阶段的树应该保持不变
        assert!(tree.get("phase").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_remove_phase_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "type": "FRAME",
            "opacity": 1.0,
            "visible": true,
            "x": 100,
            "y": 200
        });

        remove_phase_fields(&mut tree).unwrap();

        // 只应删除相
        assert!(tree.get("phase").is_none());

        // 保留所有其他字段
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
        assert_eq!(tree.get("x").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("y").unwrap().as_i64(), Some(200));
    }

    #[test]
    fn test_remove_phase_in_arrays() {
        let mut tree = json!({
            "items": [
                {
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    },
                    "name": "Item1"
                },
                {
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    },
                    "name": "Item2"
                }
            ]
        });

        remove_phase_fields(&mut tree).unwrap();

        // 阵列中的所有相都应被删除
        assert!(tree["items"][0].get("phase").is_none());
        assert_eq!(tree["items"][0].get("name").unwrap().as_str(), Some("Item1"));

        assert!(tree["items"][1].get("phase").is_none());
        assert_eq!(tree["items"][1].get("name").unwrap().as_str(), Some("Item2"));
    }

    #[test]
    fn test_remove_phase_mixed_objects() {
        let mut tree = json!({
            "name": "Root",
            "phase": {
                "__enum__": "NodePhase",
                "value": "CREATED"
            },
            "properties": {
                "width": 100,
                "height": 200
            },
            "children": [
                {
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    },
                    "name": "Child"
                }
            ]
        });

        remove_phase_fields(&mut tree).unwrap();

        // 根相已移除
        assert!(tree.get("phase").is_none());

        // 属性对象不变(无相)
        assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
        assert_eq!(tree["properties"]["height"].as_i64(), Some(200));

        // 儿童阶段已删除
        assert!(tree["children"][0].get("phase").is_none());
        assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child"));
    }

    #[test]
    fn test_remove_phase_empty_object() {
        let mut tree = json!({});

        remove_phase_fields(&mut tree).unwrap();

        // 空对象应保持为空
        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_remove_phase_primitives() {
        let mut tree = json!(true);

        remove_phase_fields(&mut tree).unwrap();

        // 原始值应保持不变
        assert_eq!(tree.as_bool(), Some(true));
    }
}
