use super::environment_checks::check_command_available;
use super::{DiagItem, Severity};

#[test]
fn missing_command_is_reported_as_unavailable() {
    let command = "vibewindow-command-that-should-not-exist";
    let mut items = Vec::<DiagItem>::new();

    check_command_available(command, &["--version"], "env", &mut items);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].severity, Severity::Warn);
}
