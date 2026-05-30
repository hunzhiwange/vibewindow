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
