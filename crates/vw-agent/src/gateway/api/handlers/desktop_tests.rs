use serde_json::json;

use super::merge_preferences_patch;

#[test]
fn merge_preferences_patch_initializes_empty_preferences() {
    let mut current = serde_json::Value::Null;

    merge_preferences_patch(
        &mut current,
        &json!({
            "model": "openai/gpt-5.1-codex-max",
            "auto_model": false
        }),
    );

    assert_eq!(
        current,
        json!({
            "model": "openai/gpt-5.1-codex-max",
            "auto_model": false
        })
    );
}

#[test]
fn merge_preferences_patch_null_removes_existing_key() {
    let mut current = json!({
        "model": "openai/gpt-5.1-codex-max",
        "auto_model": false
    });

    merge_preferences_patch(&mut current, &json!({ "model": null }));

    assert_eq!(current, json!({ "auto_model": false }));
}
