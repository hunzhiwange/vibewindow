use super::config_load::config_json_path;

#[test]
fn config_json_path_appends_stable_filename() {
    assert_eq!(
        config_json_path(std::path::Path::new("/tmp/vw")),
        std::path::Path::new("/tmp/vw/vibewindow.json")
    );
}
