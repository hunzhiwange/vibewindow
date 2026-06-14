use super::*;
use serde_json::json;

#[test]
fn test_remove_guid_path_simple() {
    let mut tree = json!({
        "name": "Override",
        "guidPath": {
            "guids": [
                {
                    "localID": 123,
                    "sessionID": 456
                }
            ]
        },
        "visible": false
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree.get("guidPath").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Override"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(false));
}

#[test]
fn test_remove_guid_path_multiple_guids() {
    let mut tree = json!({
        "guidPath": {
            "guids": [
                {"localID": 1, "sessionID": 1},
                {"localID": 2, "sessionID": 1},
                {"localID": 3, "sessionID": 1}
            ]
        },
        "opacity": 0.5
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree.get("guidPath").is_none());
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.5));
}

#[test]
fn test_remove_guid_path_nested() {
    let mut tree = json!({
        "overrides": [
            {
                "guidPath": {
                    "guids": [{"localID": 1, "sessionID": 1}]
                },
                "visible": true
            },
            {
                "guidPath": {
                    "guids": [{"localID": 2, "sessionID": 1}]
                },
                "opacity": 0.8
            }
        ]
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree["overrides"][0].get("guidPath").is_none());
    assert!(tree["overrides"][1].get("guidPath").is_none());
    assert_eq!(tree["overrides"][0]["visible"].as_bool(), Some(true));
    assert_eq!(tree["overrides"][1]["opacity"].as_f64(), Some(0.8));
}

#[test]
fn test_remove_guid_path_deeply_nested() {
    let mut tree = json!({
        "symbolData": {
            "symbolOverrides": [
                {
                    "guidPath": {
                        "guids": [
                            {"localID": 1, "sessionID": 1},
                            {"localID": 2, "sessionID": 1}
                        ]
                    },
                    "properties": {
                        "nested": {
                            "guidPath": {
                                "guids": [{"localID": 3, "sessionID": 1}]
                            }
                        }
                    }
                }
            ]
        }
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree["symbolData"]["symbolOverrides"][0].get("guidPath").is_none());
    assert!(
        tree["symbolData"]["symbolOverrides"][0]["properties"]["nested"].get("guidPath").is_none()
    );
}

#[test]
fn test_remove_guid_path_missing() {
    let mut tree = json!({
        "name": "Node",
        "visible": true,
        "opacity": 1.0
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree.get("guidPath").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_guid_path_preserves_other_fields() {
    let mut tree = json!({
        "name": "Override",
        "guidPath": {
            "guids": [{"localID": 5, "sessionID": 2}]
        },
        "overriddenSymbolID": {
            "localID": 100,
            "sessionID": 50
        },
        "visible": false,
        "opacity": 0.7
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree.get("guidPath").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Override"));
    assert_eq!(tree["overriddenSymbolID"]["localID"].as_i64(), Some(100));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(false));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.7));
}

#[test]
fn test_remove_guid_path_in_derived_symbol_data() {
    let mut tree = json!({
        "derivedSymbolData": [
            {
                "guidPath": {
                    "guids": [{"localID": 1, "sessionID": 1}]
                },
                "size": {"x": 100.0, "y": 50.0}
            },
            {
                "guidPath": {
                    "guids": [
                        {"localID": 2, "sessionID": 1},
                        {"localID": 3, "sessionID": 1}
                    ]
                },
                "transform": {"x": 10.0, "y": 20.0}
            }
        ]
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree["derivedSymbolData"][0].get("guidPath").is_none());
    assert!(tree["derivedSymbolData"][1].get("guidPath").is_none());
    assert_eq!(tree["derivedSymbolData"][0]["size"]["x"].as_f64(), Some(100.0));
    assert_eq!(tree["derivedSymbolData"][1]["transform"]["x"].as_f64(), Some(10.0));
}

#[test]
fn test_remove_guid_path_empty_guids_array() {
    let mut tree = json!({
        "guidPath": {
            "guids": []
        },
        "name": "Empty"
    });

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree.get("guidPath").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Empty"));
}

#[test]
fn test_remove_guid_path_empty_object() {
    let mut tree = json!({});

    remove_guid_paths(&mut tree).unwrap();

    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_remove_guid_path_primitives() {
    let mut tree = json!(42);

    remove_guid_paths(&mut tree).unwrap();

    assert_eq!(tree.as_i64(), Some(42));
}

#[test]
fn test_remove_guid_path_mixed_array_primitives() {
    let mut tree = json!([
        {"guidPath": {"guids": []}, "name": "Override"},
        true,
        "plain"
    ]);

    remove_guid_paths(&mut tree).unwrap();

    assert!(tree[0].get("guidPath").is_none());
    assert_eq!(tree[0]["name"].as_str(), Some("Override"));
    assert_eq!(tree[1].as_bool(), Some(true));
    assert_eq!(tree[2].as_str(), Some("plain"));
}
