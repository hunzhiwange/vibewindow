use super::{InitOptions, Level, create, file, init};

#[test]
fn level_metadata_is_stable() {
    assert_eq!(Level::Error.label(), "ERROR");
    assert!(Level::Error.priority() >= Level::Debug.priority());
}

#[test]
fn init_without_file_keeps_file_absent_and_logger_usable() {
    init(InitOptions { print: true, dev: true, level: Some(Level::Debug) });
    assert!(file().is_none());
    create(None).debug("debug message", None);
}
