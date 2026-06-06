use super::VariablePool;
use serde_json::{Value, json};

#[test]
fn get_selector_reads_nested_object_fields() {
    let mut pool = VariablePool::default();
    pool.insert_node_output(
        "code",
        "payload",
        json!({
            "order": {
                "id": "A-100"
            }
        }),
    );

    let selector = ["code", "payload", "order", "id"].map(str::to_string);

    assert_eq!(pool.get_selector(&selector), Some(&Value::String("A-100".to_string())));
}

#[test]
fn get_selector_reads_nested_array_items() {
    let mut pool = VariablePool::default();
    pool.insert_node_output(
        "code",
        "payload",
        json!({
            "items": [
                { "name": "first" },
                { "name": "second" }
            ]
        }),
    );

    let selector = ["code", "payload", "items[1]", "name"].map(str::to_string);

    assert_eq!(pool.get_selector(&selector), Some(&Value::String("second".to_string())));
}

#[test]
fn get_selector_reads_dotted_path_parts() {
    let mut pool = VariablePool::default();
    pool.insert_node_output(
        "start",
        "profile",
        json!({
            "addresses": [
                { "city": "Hangzhou" }
            ]
        }),
    );

    let selector = ["start", "profile", "addresses[0].city"].map(str::to_string);

    assert_eq!(pool.get_selector(&selector), Some(&Value::String("Hangzhou".to_string())));
}
