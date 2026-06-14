use super::*;
use serde_json::json;

#[test]
fn test_remove_standalone_overridden_symbol_id() {
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": {
                    "localID": 7979,
                    "sessionID": 8184
                }
            },
            {
                "textData": {
                    "characters": "Roles"
                }
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    assert_eq!(overrides.len(), 1);
    assert!(overrides[0].get("textData").is_some());
}

#[test]
fn test_preserve_overridden_symbol_id_with_other_fields() {
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": {
                    "localID": 441,
                    "sessionID": 56
                },
                "visible": false
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    // 对象被保留，因为它有其他字段
    assert_eq!(overrides.len(), 1);
    assert!(overrides[0].get("overriddenSymbolID").is_some());
    assert_eq!(overrides[0].get("visible").unwrap().as_bool(), Some(false));
}

#[test]
fn test_remove_multiple_standalone_objects() {
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": {
                    "localID": 1,
                    "sessionID": 2
                }
            },
            {
                "textData": {
                    "characters": "Keep"
                }
            },
            {
                "overriddenSymbolID": {
                    "localID": 3,
                    "sessionID": 4
                }
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    assert_eq!(overrides.len(), 1);
    assert!(overrides[0].get("textData").is_some());
}

#[test]
fn test_all_standalone_removed() {
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": {
                    "localID": 1,
                    "sessionID": 2
                }
            },
            {
                "overriddenSymbolID": {
                    "localID": 3,
                    "sessionID": 4
                }
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    // 所有独立对象已删除，空数组
    assert_eq!(overrides.len(), 0);
}

#[test]
fn test_no_standalone_objects() {
    let mut tree = json!({
        "symbolOverrides": [
            {
                "textData": {
                    "characters": "One"
                }
            },
            {
                "visible": true
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    // 保留所有对象
    assert_eq!(overrides.len(), 2);
}

#[test]
fn test_nested_arrays() {
    let mut tree = json!({
        "parent": {
            "symbolOverrides": [
                {
                    "overriddenSymbolID": {
                        "localID": 123,
                        "sessionID": 456
                    }
                },
                {
                    "textData": {
                        "characters": "Text"
                    }
                }
            ]
        }
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree["parent"]["symbolOverrides"].as_array().unwrap();
    assert_eq!(overrides.len(), 1);
    assert!(overrides[0].get("textData").is_some());
}

#[test]
fn test_deeply_nested_structure() {
    let mut tree = json!({
        "document": {
            "children": [
                {
                    "symbolData": {
                        "symbolOverrides": [
                            {
                                "overriddenSymbolID": {
                                    "localID": 789,
                                    "sessionID": 12
                                }
                            },
                            {
                                "opacity": 0.5
                            }
                        ]
                    }
                }
            ]
        }
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides =
        tree["document"]["children"][0]["symbolData"]["symbolOverrides"].as_array().unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].get("opacity").unwrap().as_f64(), Some(0.5));
}

#[test]
fn test_overridden_symbol_id_with_extra_fields() {
    // 如果 overridedenSymbolID 对象具有 localID/sessionID 之外的额外字段，
    // 整个对象应该被保留
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": {
                    "localID": 123,
                    "sessionID": 456,
                    "extraField": "value"
                }
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    // 保留，因为 overriddenSymbolID 有额外的字段
    assert_eq!(overrides.len(), 1);
}

#[test]
fn test_overridden_symbol_id_missing_session_id() {
    // 如果 overriddenSymbolID 缺少 sessionID，则保留对象
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": {
                    "localID": 123
                }
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    // 由于格式错误而被保留
    assert_eq!(overrides.len(), 1);
}

#[test]
fn test_non_object_array_elements() {
    let mut tree = json!({
        "data": [1, 2, 3, "string", null, true]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let data = tree.get("data").unwrap().as_array().unwrap();
    // 应保留非对象元素
    assert_eq!(data.len(), 6);
}

#[test]
fn test_mixed_array_with_primitives() {
    let mut tree = json!({
        "mixed": [
            {
                "overriddenSymbolID": {
                    "localID": 1,
                    "sessionID": 2
                }
            },
            "string",
            42,
            {
                "name": "Keep"
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let mixed = tree.get("mixed").unwrap().as_array().unwrap();
    assert_eq!(mixed.len(), 3);
    assert_eq!(mixed[0].as_str(), Some("string"));
    assert_eq!(mixed[1].as_i64(), Some(42));
    assert!(mixed[2].get("name").is_some());
}

#[test]
fn test_real_world_example() {
    // 基于 archives/roles-members.json 中的实际数据
    let mut tree = json!({
        "symbolData": {
            "symbolOverrides": [
                {
                    "textData": {
                        "characters": "Roles"
                    }
                },
                {
                    "overriddenSymbolID": {
                        "localID": 7974,
                        "sessionID": 8184
                    }
                },
                {
                    "textData": {
                        "characters": "Members"
                    }
                },
                {
                    "overriddenSymbolID": {
                        "localID": 7979,
                        "sessionID": 8184
                    }
                },
                {
                    "textData": {
                        "characters": "Audit"
                    }
                },
                {
                    "visible": false
                },
                {
                    "overrideLevel": 1,
                    "textData": {
                        "characters": "Commands"
                    }
                }
            ]
        }
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree["symbolData"]["symbolOverrides"].as_array().unwrap();
    // 应该只删除 2 个独立的 overridedenSymbolID 对象
    assert_eq!(overrides.len(), 5);

    // 验证保存的对象
    assert!(overrides[0].get("textData").is_some());
    assert_eq!(overrides[0]["textData"]["characters"].as_str(), Some("Roles"));

    assert!(overrides[1].get("textData").is_some());
    assert_eq!(overrides[1]["textData"]["characters"].as_str(), Some("Members"));

    assert!(overrides[2].get("textData").is_some());
    assert_eq!(overrides[2]["textData"]["characters"].as_str(), Some("Audit"));

    assert_eq!(overrides[3].get("visible").unwrap().as_bool(), Some(false));

    assert!(overrides[4].get("overrideLevel").is_some());
    assert!(overrides[4].get("textData").is_some());
}

#[test]
fn test_overridden_symbol_id_non_object_value_is_preserved() {
    let mut tree = json!({
        "symbolOverrides": [
            {
                "overriddenSymbolID": "swapped-component"
            }
        ]
    });

    remove_overridden_symbol_id(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].get("overriddenSymbolID").unwrap().as_str(), Some("swapped-component"));
}
