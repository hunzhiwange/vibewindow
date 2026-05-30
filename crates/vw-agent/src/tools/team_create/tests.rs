use super::*;
use crate::app::agent::config::AgentsIpcConfig;
use crate::app::agent::tools::traits::Tool;

fn test_ipc_db() -> Arc<super::super::agents_ipc::IpcDb> {
    let root = tempfile::tempdir().expect("temp dir");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let config = AgentsIpcConfig {
        enabled: true,
        db_path: root.path().join("agents.db").to_string_lossy().to_string(),
        staleness_secs: 300,
    };
    Arc::new(super::super::agents_ipc::IpcDb::open(&workspace, &config).expect("ipc db"))
}

#[test]
fn args_deserializes_required_fields() {
    let args: Args =
        serde_json::from_value(json!({"id": "team-a", "members": ["a", "b"]})).expect("valid args");

    assert_eq!(args.id, "team-a");
    assert_eq!(args.members, vec!["a", "b"]);
}

#[test]
fn schema_requires_id_and_members() {
    let tool = TeamCreateTool::new(test_ipc_db(), Arc::new(SecurityPolicy::default()));

    let schema = tool.parameters_schema();
    assert_eq!(schema["required"], json!(["id", "members"]));
    assert_eq!(tool.spec().strict, true);
}
