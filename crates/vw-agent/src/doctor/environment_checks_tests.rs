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

#[test]
fn command_with_non_zero_exit_is_reported_as_warning() {
    let mut items = Vec::<DiagItem>::new();

    check_command_available("sh", &["-c", "exit 7"], "env", &mut items);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].severity, Severity::Warn);
    assert!(items[0].message.contains("found but returned non-zero"));
}

#[test]
fn successful_command_records_first_stdout_line() {
    let mut items = Vec::<DiagItem>::new();

    check_command_available("sh", &["-c", "printf 'doctor-ok\\nsecond-line'"], "env", &mut items);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].severity, Severity::Ok);
    assert!(items[0].message.contains("doctor-ok"));
    assert!(!items[0].message.contains("second-line"));
}
