//! 验证 Codex ACP 兼容识别逻辑。
//!
//! 这些测试覆盖二进制名称、包装脚本参数和命令行片段的匹配边界，确保
//! `vw-acp` 只在明确指向 Codex/`codex-acp` 时启用兼容路径。

use vw_acp::{is_codex_acp_command, is_codex_invocation};

/// 确认 ACP 命令检测既支持直接二进制，也支持通过 Node 等包装器传入的脚本路径。
#[test]
fn is_codex_acp_command_matches_binary_name_or_args() {
    assert!(is_codex_acp_command("/usr/local/bin/codex-acp", &[]));
    assert!(is_codex_acp_command("node", &["/tmp/wrapper/codex-acp.js".to_string()]));
    assert!(!is_codex_acp_command("/usr/local/bin/vwacp", &["serve".to_string()]));
}

/// 确认 Codex 调用检测接受代理名称或独立命令 token，但不会误匹配后缀相似的命令。
#[test]
fn is_codex_invocation_matches_agent_name_or_command_token() {
    assert!(is_codex_invocation("codex", "anything"));
    assert!(is_codex_invocation("other", "npx codex-acp --stdio"));
    assert!(!is_codex_invocation("other", "npx codex-acp-wrapper --stdio"));
}
