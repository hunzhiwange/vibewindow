use super::{CreateInput, RemoveInput, ResetInput};

#[test]
fn operation_inputs_deserialize_aliases_and_defaults() {
    let create: CreateInput =
        serde_json::from_value(serde_json::json!({"name": "demo", "startCommand": "echo ok"}))
            .expect("create input");
    let remove: RemoveInput =
        serde_json::from_value(serde_json::json!({"directory": "/tmp/wt"})).expect("remove input");
    let reset: ResetInput =
        serde_json::from_value(serde_json::json!({"directory": "/tmp/wt", "baseRef": "main"}))
            .expect("reset input");

    assert_eq!(create.start_command.as_deref(), Some("echo ok"));
    assert!(!remove.force);
    assert_eq!(reset.base_ref.as_deref(), Some("main"));
}
