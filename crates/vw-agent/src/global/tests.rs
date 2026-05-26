use super::*;

#[test]
fn global_paths_include_app_named_directories() {
    let paths = paths();

    assert!(paths.data.ends_with(APP));
    assert!(paths.config.ends_with(APP));
    assert!(paths.cache.ends_with(APP));
}
