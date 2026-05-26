use super::*;

#[test]
fn from_command_extracts_name_args_and_redirects() {
    let info = CommandInfo::from_command("git status --short > out.txt").unwrap();
    assert_eq!(info.name, "git");
    assert!(info.args.contains(&"status".to_string()));
    assert_eq!(info.redirects.len(), 1);
}

#[test]
fn from_command_rejects_empty_command() {
    assert!(CommandInfo::from_command("   ").is_none());
}
