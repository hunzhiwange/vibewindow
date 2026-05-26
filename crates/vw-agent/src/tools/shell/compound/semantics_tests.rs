//! 退出语义测试，覆盖常见命令的非零退出码是否应被视为错误。

use crate::tools::shell::ast::parse_command;

use super::{ExitInterpretation, ExitSemantics};

#[test]
fn grep_exit_one_is_not_error() {
    let parsed = parse_command("grep needle file.txt");
    let semantics = ExitSemantics::for_parsed_command(&parsed);

    assert_eq!(semantics.interpret(Some(1)), ExitInterpretation::NoMatches);
    assert!(!semantics.interpret(Some(1)).is_error_for_llm());
}

#[test]
fn diff_exit_one_reports_differences() {
    let semantics = ExitSemantics::for_command("diff");
    assert_eq!(semantics.interpret(Some(1)), ExitInterpretation::DifferencesFound);
}

#[test]
fn test_exit_one_is_false_not_error() {
    let semantics = ExitSemantics::for_command("test");
    assert_eq!(semantics.interpret(Some(1)), ExitInterpretation::ConditionFalse);
}

#[test]
fn missing_exit_code_is_error() {
    let semantics = ExitSemantics::for_command("echo");
    assert!(semantics.interpret(None).is_error_for_llm());
}
