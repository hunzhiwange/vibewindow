//! 配置加载与解析逻辑的单元测试。

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::load_resolved_config_from_paths;

#[tokio::test]
async fn load_resolved_config_defaults_queue_owner_ttl_to_five_minutes() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("vw-acp-config-tests-{unique}"));
    fs::create_dir_all(&temp).expect("temp dir");
    let global_path = temp.join("global.json");
    let project_path = temp.join("project.json");

    let config = load_resolved_config_from_paths(&global_path, &project_path)
        .await
        .expect("load default config");

    assert_eq!(config.ttl_ms, 300_000);

    let _ = fs::remove_dir_all(temp);
}
