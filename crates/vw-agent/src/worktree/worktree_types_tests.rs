use super::{CreateInput, Error, Info, RemoveInput, ResetInput};

#[test]
fn error_display_uses_inner_message() {
    assert_eq!(Error::NotGit("no git".into()).to_string(), "no git");
    assert_eq!(Error::MissingProject("missing".into()).to_string(), "missing");
    assert_eq!(Error::Invalid("bad".into()).to_string(), "bad");
}

#[test]
fn worktree_types_roundtrip_json_fields() {
    let info = Info { name: "n".into(), branch: "b".into(), directory: "d".into() };
    assert_eq!(serde_json::to_value(&info).expect("json")["directory"], "d");

    let create: CreateInput =
        serde_json::from_value(serde_json::json!({"startCommand": "echo"})).expect("create");
    let remove: RemoveInput =
        serde_json::from_value(serde_json::json!({"directory": "d"})).expect("remove");
    let reset: ResetInput =
        serde_json::from_value(serde_json::json!({"directory": "d", "baseRef": null}))
            .expect("reset");

    assert_eq!(create.start_command.as_deref(), Some("echo"));
    assert!(!remove.force);
    assert_eq!(reset.base_ref, None);
}
