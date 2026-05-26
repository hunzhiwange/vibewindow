use super::*;

#[test]
fn prefix_keeps_progressive_command_parts() {
    assert_eq!(prefix(&["cargo", "clippy", "--all-targets"]), vec!["cargo", "cargo clippy", "cargo clippy --all-targets"]);
}

#[test]
fn prefix_handles_empty_input() {
    assert!(prefix(&[]).is_empty());
}

