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
fn args_deserializes_required_id() {
    let args: Args = serde_json::from_value(json!({"id": "team-a"})).expect("valid args");

    assert_eq!(args.id, "team-a");
}

#[test]
fn schema_marks_delete_as_destructive() {
    let tool = TeamDeleteTool::new(test_ipc_db(), Arc::new(SecurityPolicy::default()));

    assert_eq!(tool.parameters_schema()["required"], json!(["id"]));
    assert!(tool.spec().destructive);
}
