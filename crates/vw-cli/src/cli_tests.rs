use clap::CommandFactory;

#[test]
fn cli_definition_builds_without_panicking() {
    let command = crate::cli::Cli::command();
    assert_eq!(command.get_name(), "vibewindow");
}
