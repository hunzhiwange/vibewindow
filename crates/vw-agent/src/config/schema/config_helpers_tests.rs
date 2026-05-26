use super::config_helpers::config_dir_creation_error;

#[test]
fn config_dir_creation_error_mentions_path_and_openrc_hint() {
    let message = config_dir_creation_error(std::path::Path::new("/restricted/vw"));

    assert!(message.contains("/restricted/vw"));
    assert!(message.contains("OpenRC"));
}
