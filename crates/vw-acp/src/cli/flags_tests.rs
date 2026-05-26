//! CLI 标志解析与默认值处理的单元测试。

use std::collections::HashMap;

use crate::cli::flags::{GlobalFlagOptions, resolve_global_flags};
use crate::config::ResolvedAcpxConfig;
use crate::types::{AuthPolicy, NonInteractivePermissionPolicy, OutputFormat, PermissionMode};

fn make_config(ttl_ms: u64) -> ResolvedAcpxConfig {
    ResolvedAcpxConfig {
        default_agent: "codex".to_string(),
        default_permissions: PermissionMode::ApproveReads,
        non_interactive_permissions: NonInteractivePermissionPolicy::Deny,
        auth_policy: AuthPolicy::Skip,
        ttl_ms,
        timeout_ms: None,
        queue_max_depth: 16,
        format: OutputFormat::Text,
        agents: HashMap::new(),
        auth: HashMap::new(),
        disable_exec: false,
        mcp_servers: Vec::new(),
        global_path: "/tmp/global.json".to_string(),
        project_path: "/tmp/project.json".to_string(),
        has_global_config: false,
        has_project_config: false,
    }
}

#[test]
fn resolve_global_flags_falls_back_to_five_minute_queue_owner_ttl() {
    let flags = resolve_global_flags(&GlobalFlagOptions::default(), &make_config(0))
        .expect("resolve flags");

    assert_eq!(flags.ttl, 300_000);
}
