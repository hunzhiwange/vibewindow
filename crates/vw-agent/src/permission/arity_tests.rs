use super::*;

#[test]
fn prefix_keeps_progressive_command_parts() {
    assert_eq!(
        prefix(&["cargo", "clippy", "--all-targets"]),
        vec!["cargo", "cargo clippy", "cargo clippy --all-targets"]
    );
}

#[test]
fn prefix_handles_empty_input() {
    assert!(prefix(&[]).is_empty());
}

#[test]
fn prefix_stops_when_known_arity_is_reached() {
    assert_eq!(prefix(&["ls", "-la"]), vec!["ls"]);
}

#[test]
fn prefix_continues_progressive_parts_for_higher_arity_parent_commands() {
    assert_eq!(
        prefix(&["docker", "compose", "up", "-d"]),
        vec!["docker", "docker compose", "docker compose up", "docker compose up -d"]
    );
    assert_eq!(
        prefix(&["npm", "run", "build", "--", "--prod"]),
        vec!["npm", "npm run", "npm run build", "npm run build --", "npm run build -- --prod"]
    );
}

#[test]
fn prefix_returns_all_progressive_parts_for_unknown_command() {
    assert_eq!(
        prefix(&["unknown-tool", "sub", "arg"]),
        vec!["unknown-tool", "unknown-tool sub", "unknown-tool sub arg"]
    );
}
