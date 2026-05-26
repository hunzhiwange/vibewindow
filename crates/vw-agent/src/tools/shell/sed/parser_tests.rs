//! sed parser 与 validation 的行为测试。
//!
//! 测试覆盖允许的原地替换/打印命令，以及会写文件或执行命令的危险 sed 脚本阻断。

use std::path::PathBuf;

use crate::tools::shell::ast::parse_command;
use crate::tools::shell::sed::{SedEdit, SedParseError, SedValidationResult, validate_sed_command};

#[test]
fn parses_in_place_substitute() {
    let edit = SedEdit::parse(&parse_command("sed -i 's/old/new/g' file.txt")).unwrap();
    assert_eq!(edit.file, PathBuf::from("file.txt"));
    assert_eq!(edit.pattern, "old");
    assert_eq!(edit.replacement, "new");
    assert_eq!(edit.flags, "g");
}

#[test]
fn parses_print_command() {
    assert!(matches!(
        validate_sed_command(&parse_command("sed -n '1,10p' file.txt")),
        SedValidationResult::Allowed { .. }
    ));
}

#[test]
fn parses_extended_regex_substitute() {
    let edit =
        SedEdit::parse(&parse_command("sed -i -E 's/pattern/replacement/' file.txt")).unwrap();
    assert!(edit.extended_regex);
}

#[test]
fn blocks_write_command() {
    assert!(matches!(
        validate_sed_command(&parse_command("sed 'w /tmp/out' file.txt")),
        SedValidationResult::Blocked { .. }
    ));
}

#[test]
fn blocks_exec_command() {
    assert!(matches!(
        validate_sed_command(&parse_command("sed 'e command' file.txt")),
        SedValidationResult::Blocked { .. }
    ));
}

#[test]
fn parses_macos_in_place_syntax() {
    let edit = SedEdit::parse(&parse_command("sed -i '' 's/old/new/g' file.txt")).unwrap();
    assert_eq!(edit.file, PathBuf::from("file.txt"));
}

#[test]
fn applies_single_line_substitute() {
    let edit = SedEdit::parse(&parse_command("sed -i 's/old/new/g' file.txt")).unwrap();
    assert_eq!(edit.apply_to_content("old text").unwrap(), "new text");
}

#[test]
fn applies_multi_line_substitute() {
    let edit = SedEdit::parse(&parse_command("sed -i 's/old/new/g' file.txt")).unwrap();
    assert_eq!(edit.apply_to_content("old\nold\n").unwrap(), "new\nnew\n");
}

#[test]
fn malformed_expression_returns_error() {
    assert_eq!(
        SedEdit::parse(&parse_command("sed -i 's/old/new' file.txt")).unwrap_err(),
        SedParseError::MalformedSubstitute
    );
}
