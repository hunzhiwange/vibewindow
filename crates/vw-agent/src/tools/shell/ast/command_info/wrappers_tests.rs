use super::*;
use crate::tools::shell::ast::CommandInfo;

fn info(name: &str, args: &[&str]) -> CommandInfo {
    CommandInfo {
        name: name.into(),
        args: args.iter().map(|arg| (*arg).into()).collect(),
        ..CommandInfo::default()
    }
}

#[test]
fn strip_wrappers_removes_env_and_timeout_layers() {
    let stripped =
        strip_wrappers(&info("env", &["RUST_LOG=debug", "timeout", "5", "git", "status"]));
    assert_eq!(stripped.name, "git");
    assert_eq!(stripped.args, vec!["status".to_string()]);
}

#[test]
fn wrapper_helpers_classify_env_assignments_and_nice_priorities() {
    assert!(is_env_assignment("KEY=value"));
    assert!(!is_env_assignment("=value"));
    assert!(is_nice_priority("-10"));
    assert!(!is_nice_priority("--flag"));
}
