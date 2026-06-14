use super::{Error, State, deduplicate_plugins, merge_json_value, plugin_name};
use serde_json::json;

#[test]
fn plugin_name_normalizes_versions_and_file_urls() {
    assert_eq!(plugin_name("tool@1.2.3"), "tool");
    assert_eq!(plugin_name("@scope/tool@1.2.3"), "@scope/tool");
    assert_eq!(plugin_name("@scope/tool"), "@scope/tool");
    assert_eq!(plugin_name("file:///tmp/my-plugin.wasm"), "my-plugin");
    assert_eq!(plugin_name("file:///tmp/plugin"), "plugin");
    assert_eq!(plugin_name("plain"), "plain");
}

#[test]
fn deduplicate_plugins_keeps_last_spec_for_each_name() {
    let plugins = vec!["a@1".to_string(), "b@1".to_string(), "a@2".to_string()];

    assert_eq!(deduplicate_plugins(plugins), vec!["b@1".to_string(), "a@2".to_string()]);
}

#[test]
fn deduplicate_plugins_normalizes_file_specs_by_stem() {
    let plugins = vec![
        "file:///tmp/one.wasm".to_string(),
        "file:///tmp/two.wasm".to_string(),
        "file:///opt/one.wasm".to_string(),
    ];

    assert_eq!(
        deduplicate_plugins(plugins),
        vec!["file:///tmp/two.wasm".to_string(), "file:///opt/one.wasm".to_string()]
    );
}

#[test]
fn merge_json_value_recurses_and_removes_nulls() {
    let mut target = json!({"a": 1, "nested": {"keep": true, "drop": 1}});

    merge_json_value(&mut target, json!({"nested": {"drop": null, "add": 2}, "a": 3}));

    assert_eq!(target, json!({"a": 3, "nested": {"keep": true, "add": 2}}));
}

#[test]
fn merge_json_value_replaces_non_objects_and_inserts_new_values() {
    let mut target = json!({"nested": 1});

    merge_json_value(&mut target, json!({"nested": {"now": true}, "new": [1, 2]}));

    assert_eq!(target, json!({"nested": {"now": true}, "new": [1, 2]}));

    merge_json_value(&mut target, json!(false));

    assert_eq!(target, json!(false));
}

#[test]
fn error_display_and_state_default_are_plain() {
    assert_eq!(Error::Invalid("bad config".to_string()).to_string(), "bad config");
    assert!(State::default().directories.is_empty());
}
