//! 验证 ACP 配置加载、合并和初始化行为。
//!
//! 测试重点是全局配置与项目配置的覆盖顺序、用户输入的规范化，以及默认配置
//! 文件的幂等创建。这里使用真实临时文件而不是 mock，避免遗漏 JSON 读写和
//! 路径处理上的集成问题。

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_client_protocol::{McpServer, McpServerHttp};
use serde_json::{Value, json};
use tokio::fs;
use vw_acp::{
    AuthPolicy, NonInteractivePermissionPolicy, OutputFormat, PermissionMode,
    init_global_config_file_at, load_resolved_config_from_paths, to_config_display,
};

/// 构造带进程号和纳秒时间戳的临时路径，降低并发测试之间互相覆盖的风险。
fn temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("vw-acp-{name}-{}-{unique}", std::process::id()))
}

/// 验证项目配置按预期覆盖全局配置，同时保留可合并的 agent/auth 条目。
#[tokio::test]
async fn load_resolved_config_merges_global_and_project_values() {
    let root = temp_path("config-merge");
    fs::create_dir_all(&root).await.expect("temp directory should be created");
    let global_path = root.join("global.json");
    let project_path = root.join("project.json");

    fs::write(
        &global_path,
        json!({
            "defaultAgent": " Claude ",
            "defaultPermissions": "deny-all",
            "nonInteractivePermissions": "fail",
            "authPolicy": "fail",
            "ttl": 45,
            "timeout": 30,
            "queueMaxDepth": 7,
            "format": "json",
            "agents": {
                " Claude ": { "command": "claude-acp" }
            },
            "auth": {
                "anthropic": "global-token"
            },
            "disableExec": true,
            "mcpServers": [
                {
                    "type": "http",
                    "name": "global",
                    "url": "https://global.example/mcp"
                }
            ]
        })
        .to_string(),
    )
    .await
    .expect("global config should be written");

    // 项目配置中的 null timeout 用来确认显式清空能覆盖全局 timeout。
    fs::write(
        &project_path,
        json!({
            "defaultPermissions": "approve-reads",
            "ttl": 10,
            "timeout": null,
            "queueMaxDepth": 3,
            "format": "quiet",
            "agents": {
                " codex ": { "command": "codex-acp" }
            },
            "auth": {
                "github": "project-token"
            },
            "disableExec": false,
            "mcpServers": [
                {
                    "type": "http",
                    "name": "project",
                    "url": "https://project.example/mcp"
                }
            ]
        })
        .to_string(),
    )
    .await
    .expect("project config should be written");

    let resolved = load_resolved_config_from_paths(&global_path, &project_path)
        .await
        .expect("config should load");

    assert_eq!(resolved.default_agent, "claude");
    assert_eq!(resolved.default_permissions, PermissionMode::ApproveReads);
    assert_eq!(resolved.non_interactive_permissions, NonInteractivePermissionPolicy::Fail);
    assert_eq!(resolved.auth_policy, AuthPolicy::Fail);
    assert_eq!(resolved.ttl_ms, 10_000);
    assert_eq!(resolved.timeout_ms, None);
    assert_eq!(resolved.queue_max_depth, 3);
    assert_eq!(resolved.format, OutputFormat::Quiet);
    assert_eq!(resolved.agents.get("claude").map(|spec| spec.command.as_str()), Some("claude-acp"));
    assert_eq!(resolved.agents.get("codex").map(|spec| spec.command.as_str()), Some("codex-acp"));
    assert_eq!(resolved.auth.get("anthropic"), Some(&"global-token".to_string()));
    assert_eq!(resolved.auth.get("github"), Some(&"project-token".to_string()));
    assert!(!resolved.disable_exec);
    assert!(resolved.has_global_config);
    assert!(resolved.has_project_config);
    assert_eq!(
        resolved.mcp_servers,
        vec![McpServer::Http(McpServerHttp::new("project", "https://project.example/mcp"))]
    );

    let display = to_config_display(&resolved);
    // 展示层需要隐藏内部毫秒单位，并稳定输出已发现的认证方式。
    assert_eq!(display.ttl, 10);
    assert_eq!(display.timeout, None);
    assert_eq!(display.auth_methods, vec!["anthropic".to_string(), "github".to_string()]);
    assert_eq!(display.agents.get("codex").expect("codex agent should exist").command, "codex-acp");

    let _ = fs::remove_dir_all(&root).await;
}

/// 验证默认全局配置只在首次调用时创建，重复初始化不会覆盖用户已有文件。
#[tokio::test]
async fn init_global_config_file_at_creates_default_payload_once() {
    let root = temp_path("config-init");
    let config_path = root.join("config.json");

    let first =
        init_global_config_file_at(&config_path).await.expect("config file should initialize");
    let second =
        init_global_config_file_at(&config_path).await.expect("second init should succeed");

    assert!(first.created);
    assert!(!second.created);
    assert_eq!(first.path, config_path.to_string_lossy());
    assert_eq!(second.path, config_path.to_string_lossy());

    let payload: Value = serde_json::from_str(
        &fs::read_to_string(&config_path).await.expect("config file should be readable"),
    )
    .expect("config file should contain json");

    assert_eq!(payload.get("defaultPermissions"), Some(&json!("approve-all")));
    assert_eq!(payload.get("queueMaxDepth"), Some(&json!(16)));
    assert_eq!(payload.get("timeout"), Some(&Value::Null));

    let _ = fs::remove_dir_all(&root).await;
}
