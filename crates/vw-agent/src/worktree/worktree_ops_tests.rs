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
    assert_eq!(create.name.as_deref(), Some("demo"));
    assert!(!remove.force);
    assert_eq!(reset.base_ref.as_deref(), Some("main"));
}

#[test]
fn operation_inputs_serialize_camel_case_fields() {
    let create = serde_json::to_value(CreateInput {
        name: Some("demo".to_string()),
        start_command: Some("npm test".to_string()),
    })
    .expect("create json");
    let reset = serde_json::to_value(ResetInput {
        directory: "/tmp/wt".to_string(),
        base_ref: Some("main".to_string()),
    })
    .expect("reset json");
    let remove =
        serde_json::to_value(RemoveInput { directory: "/tmp/wt".to_string(), force: true })
            .expect("remove json");

    assert_eq!(create["startCommand"], "npm test");
    assert_eq!(reset["baseRef"], "main");
    assert_eq!(remove["force"], true);
}
