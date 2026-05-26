use super::codex_compat::*;

#[test]
fn command_detection_accepts_codex_acp_basename_with_platform_suffixes() {
    assert!(is_codex_acp_command("/usr/local/bin/codex-acp", &[]));
    assert!(is_codex_acp_command("C:\\tools\\codex-acp.exe", &[]));
    assert!(is_codex_acp_command("codex-acp.cmd", &[]));
}

#[test]
fn command_detection_scans_arguments_for_adapter_token() {
    assert!(is_codex_acp_command("npx", &["@openai/codex-acp".to_string()]));
    assert!(!is_codex_acp_command("codex", &["acp".to_string()]));
}

#[test]
fn invocation_detection_requires_token_boundaries() {
    assert!(is_codex_invocation("codex", "anything"));
    assert!(is_codex_invocation("custom", "npx @openai/codex-acp"));
    assert!(!is_codex_invocation("custom", "my-codex-acp-wrapper"));
}
