use clap::{Arg, ArgAction, Command};
use vw_acp::cli::flags::GlobalFlagOptions;
use vw_acp::{AuthPolicy, NonInteractivePermissionPolicy, OutputFormat};

use super::extract_global_flags;

fn command() -> Command {
    Command::new("vwacp")
        .arg(Arg::new("agent").long("agent").global(true))
        .arg(Arg::new("cwd").long("cwd").global(true))
        .arg(Arg::new("auth-policy").long("auth-policy").global(true))
        .arg(
            Arg::new("non-interactive-permissions")
                .long("non-interactive-permissions")
                .global(true),
        )
        .arg(Arg::new("json-strict").long("json-strict").global(true).action(ArgAction::SetTrue))
        .arg(
            Arg::new("suppress-reads")
                .long("suppress-reads")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("timeout").long("timeout").global(true))
        .arg(Arg::new("ttl").long("ttl").global(true))
        .arg(Arg::new("verbose").long("verbose").global(true).action(ArgAction::SetTrue))
        .arg(Arg::new("format").long("format").global(true))
        .arg(Arg::new("model").long("model").global(true))
        .arg(Arg::new("allowed-tools").long("allowed-tools").global(true))
        .arg(Arg::new("max-turns").long("max-turns").global(true))
        .arg(Arg::new("prompt-retries").long("prompt-retries").global(true))
        .arg(Arg::new("approve-all").long("approve-all").global(true).action(ArgAction::SetTrue))
        .arg(
            Arg::new("approve-reads").long("approve-reads").global(true).action(ArgAction::SetTrue),
        )
        .arg(Arg::new("deny-all").long("deny-all").global(true).action(ArgAction::SetTrue))
}

#[test]
fn extract_global_flags_parses_supported_values() {
    let matches = command().get_matches_from([
        "vwacp",
        "--agent",
        "codex",
        "--cwd",
        "/tmp/project",
        "--auth-policy",
        "fail",
        "--non-interactive-permissions",
        "fail",
        "--json-strict",
        "--suppress-reads",
        "--timeout",
        "30",
        "--ttl",
        "45",
        "--verbose",
        "--format",
        "json",
        "--model",
        "gpt",
        "--allowed-tools",
        "Read, Write",
        "--max-turns",
        "4",
        "--prompt-retries",
        "2",
        "--approve-reads",
    ]);

    let flags = extract_global_flags(&matches);

    assert_eq!(
        flags,
        GlobalFlagOptions {
            agent: Some("codex".to_string()),
            cwd: Some("/tmp/project".to_string()),
            auth_policy: Some(AuthPolicy::Fail),
            non_interactive_permissions: Some(NonInteractivePermissionPolicy::Fail),
            json_strict: true,
            suppress_reads: true,
            timeout: Some(30_000),
            ttl: Some(45_000),
            verbose: true,
            format: Some(OutputFormat::Json),
            model: Some("gpt".to_string()),
            allowed_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            max_turns: Some(4),
            prompt_retries: Some(2),
            approve_all: false,
            approve_reads: true,
            deny_all: false,
        }
    );
}

#[test]
fn extract_global_flags_ignores_invalid_optional_values() {
    let matches = command().get_matches_from([
        "vwacp",
        "--auth-policy",
        "unknown",
        "--format",
        "xml",
        "--timeout",
        "not-a-number",
        "--allowed-tools",
        " ",
        "--max-turns",
        "0",
    ]);

    let flags = extract_global_flags(&matches);

    assert_eq!(flags.auth_policy, None);
    assert_eq!(flags.format, None);
    assert_eq!(flags.timeout, None);
    assert_eq!(flags.allowed_tools, None);
    assert_eq!(flags.max_turns, None);
}
