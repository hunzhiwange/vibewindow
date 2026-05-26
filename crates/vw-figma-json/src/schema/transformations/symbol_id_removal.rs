use crate::error::Result;
use serde_json::Value as JsonValue;

/// 删除仅包含 localID 和/或 sessionID 的 symbolID 字段。
///
/// 当 `symbolID` 对象仅包含
/// 字段 `localID` 和/或 `sessionID`。如果 `symbolID` 对象有
/// 除了这两个字段之外的任何其他字段，都会被保留。
///
/// # 示例
///
/// 已删除(仅限标准字段)：
/// ```json
/// {
///   "symbolID": {
///     "localID": 10596,
///     "sessionID": 4331
///   }
/// }
/// ```
///
/// 保留(有额外字段)：
/// ```json
/// {
///   "symbolID": {
///     "localID": 10596,
///     "sessionID": 4331,
///     "customField": "value"
///   }
/// }
/// ```
pub fn remove_symbol_id_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查此对象是否具有应删除的 "symbolID" 字段
            if let Some(symbol_id_value) = map.get("symbolID")
                && should_remove_symbol_id(symbol_id_value)
            {
                map.remove("symbolID");
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

/// 确定是否应删除 symbolID 字段。
///
/// 如果值是仅包含字段的对象，则返回 true
/// "localID" 和/或 "sessionID" 没有其他字段。
fn should_remove_symbol_id(value: &JsonValue) -> bool {
    if let JsonValue::Object(map) = value {
        // 检查所有键是否在允许的集合中
        for key in map.keys() {
            if key != "localID" && key != "sessionID" {
                // 发现一个不是 localID 或 sessionID 的字段
                return false;
            }
        }
        // 所有键都是 localID 或 sessionID (或为空)
        true
    } else {
        // 不是一个对象，不要删除
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_symbol_id_with_both_fields() {
        let mut input = json!({
            "name": "Navigation",
            "symbolID": {
                "localID": 10596,
                "sessionID": 4331
            },
            "size": {
                "x": 375.0,
                "y": 122.0
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Navigation",
            "size": {
                "x": 375.0,
                "y": 122.0
            }
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_remove_symbol_id_with_only_local_id() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "localID": 123
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Test"
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_remove_symbol_id_with_only_session_id() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "sessionID": 456
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Test"
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_keep_symbol_id_with_extra_fields() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "localID": 123,
                "sessionID": 456,
                "customField": "value"
            }
        });

        let expected = input.clone();
        remove_symbol_id_fields(&mut input).unwrap();

        assert_eq!(input, expected);
    }

    #[test]
    fn test_keep_symbol_id_with_different_field() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {
                "localID": 123,
                "customField": "value"
            }
        });

        let expected = input.clone();
        remove_symbol_id_fields(&mut input).unwrap();

        assert_eq!(input, expected);
    }

    #[test]
    fn test_remove_empty_symbol_id() {
        let mut input = json!({
            "name": "Test",
            "symbolID": {}
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "name": "Test"
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_nested_symbol_id_removal() {
        let mut input = json!({
            "children": [
                {
                    "name": "Child1",
                    "symbolID": {
                        "localID": 1,
                        "sessionID": 2
                    }
                },
                {
                    "name": "Child2",
                    "nested": {
                        "symbolID": {
                            "localID": 3,
                            "sessionID": 4
                        }
                    }
                }
            ]
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "children": [
                {
                    "name": "Child1"
                },
                {
                    "name": "Child2",
                    "nested": {}
                }
            ]
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_mixed_symbol_ids() {
        let mut input = json!({
            "nodes": [
                {
                    "name": "Remove",
                    "symbolID": {
                        "localID": 1,
                        "sessionID": 2
                    }
                },
                {
                    "name": "Keep",
                    "symbolID": {
                        "localID": 3,
                        "sessionID": 4,
                        "extra": "field"
                    }
                }
            ]
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "nodes": [
                {
                    "name": "Remove"
                },
                {
                    "name": "Keep",
                    "symbolID": {
                        "localID": 3,
                        "sessionID": 4,
                        "extra": "field"
                    }
                }
            ]
        });

        assert_eq!(input, expected);
    }

    #[test]
    fn test_symbol_id_not_object() {
        // 如果 symbolID 不是对象，则保留它(不应该发生，但要采取防御措施)
        let mut input = json!({
            "name": "Test",
            "symbolID": "string_value"
        });

        let expected = input.clone();
        remove_symbol_id_fields(&mut input).unwrap();

        assert_eq!(input, expected);
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut input = json!({
            "level1": {
                "symbolID": {
                    "localID": 1
                },
                "level2": {
                    "symbolID": {
                        "sessionID": 2
                    },
                    "level3": {
                        "symbolID": {
                            "localID": 3,
                            "sessionID": 4
                        }
                    }
                }
            }
        });

        remove_symbol_id_fields(&mut input).unwrap();

        let expected = json!({
            "level1": {
                "level2": {
                    "level3": {}
                }
            }
        });

        assert_eq!(input, expected);
    }
}
