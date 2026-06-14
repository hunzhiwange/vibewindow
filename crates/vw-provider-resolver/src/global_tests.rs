#[test]
fn constants_match_application_cache_contract() {
    assert_eq!(super::APP, vw_config_types::paths::APP_DIR_NAME);
    assert_eq!(super::CACHE_VERSION, "21");
}

#[test]
fn paths_returns_stable_global_directories() {
    let first = super::paths();
    let second = super::paths();

    assert!(std::ptr::eq(first, second));
    assert!(!first.home.as_os_str().is_empty());
    assert!(first.data.ends_with(super::APP));
    assert!(first.cache.ends_with(super::APP));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn paths_creates_data_cache_and_version_marker() {
    let paths = super::paths();

    assert!(paths.data.is_dir());
    assert!(paths.cache.is_dir());
    assert_eq!(
        std::fs::read_to_string(paths.cache.join("version")).unwrap_or_default().trim(),
        super::CACHE_VERSION
    );
}
