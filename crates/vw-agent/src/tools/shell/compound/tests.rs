//! 复合命令权限测试，覆盖串联、管道和路径限制对审批结果的影响。

use std::path::PathBuf;

use crate::security::AutonomyLevel;
use crate::tools::shell::ast::parse_command;
use crate::tools::shell::compound::{CompoundAnalysisResult, CompoundCommandAnalyzer};
use crate::tools::shell::permissions::{PermissionContext, RuleEngine};

fn context() -> PermissionContext {
    PermissionContext {
        autonomy: AutonomyLevel::Supervised,
        in_sandbox: false,
        mode: Default::default(),
        approved: false,
        workspace_dir: PathBuf::from("/workspace"),
        allowed_roots: Vec::new(),
    }
}

#[test]
fn splits_and_allows_simple_pipeline_segments() {
    let engine = RuleEngine::new(false).with_legacy_allowlist(vec!["echo".into(), "grep".into()]);
    let parsed = parse_command("echo hello | grep h");

    let result = CompoundCommandAnalyzer::analyze(&parsed, &engine, &context());

    match result {
        CompoundAnalysisResult::Allowed { segments, .. } => {
            assert_eq!(segments.len(), 2);
            assert_eq!(segments[0].segment.command_name(), Some("echo"));
            assert_eq!(segments[1].segment.command_name(), Some("grep"));
        }
        other => panic!("expected allowed result, got {other:?}"),
    }
}

#[test]
fn blocks_background_compound_commands() {
    let engine = RuleEngine::new(false).with_legacy_allowlist(vec!["echo".into()]);
    let parsed = parse_command("echo hello &");

    let result = CompoundCommandAnalyzer::analyze(&parsed, &engine, &context());

    match result {
        CompoundAnalysisResult::Blocked { reason, .. } => {
            assert!(reason.contains("background"));
        }
        other => panic!("expected blocked result, got {other:?}"),
    }
}

#[test]
fn blocks_multiple_cd_segments() {
    let engine = RuleEngine::new(false).with_legacy_allowlist(vec!["cd".into(), "pwd".into()]);
    let parsed = parse_command("cd a && cd b && pwd");

    let result = CompoundCommandAnalyzer::analyze(&parsed, &engine, &context());

    match result {
        CompoundAnalysisResult::Blocked { reason, .. } => {
            assert!(reason.contains("Multiple cd commands"));
        }
        other => panic!("expected blocked result, got {other:?}"),
    }
}

#[test]
fn blocks_cd_then_git_outside_workspace() {
    let engine =
        RuleEngine::new(false).with_legacy_allowlist(vec!["cd".into(), "git".into(), "pwd".into()]);
    let parsed = parse_command("cd /etc && git status");

    let result = CompoundCommandAnalyzer::analyze(&parsed, &engine, &context());

    match result {
        CompoundAnalysisResult::Blocked { reason, .. } => {
            assert!(reason.contains("may escape workspace"));
        }
        other => panic!("expected blocked result, got {other:?}"),
    }
}
