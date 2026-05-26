use super::*;

#[test]
fn open_skills_enabled_sources_prefer_valid_env_override() {
    assert!(open_skills_enabled_from_sources(Some(false), Some("yes")));
    assert!(!open_skills_enabled_from_sources(Some(true), Some("off")));
    assert!(open_skills_enabled_from_sources(Some(true), Some("invalid")));
    assert!(!open_skills_enabled_from_sources(None, None));
}

#[test]
fn open_skills_dir_sources_use_env_then_config_then_home() {
    let home = std::path::Path::new("/home/example");
    assert_eq!(
        resolve_open_skills_dir_from_sources(Some(" /env/open "), Some("/config/open"), Some(home)),
        Some(PathBuf::from("/env/open"))
    );
    assert_eq!(
        resolve_open_skills_dir_from_sources(None, Some("/config/open"), Some(home)),
        Some(PathBuf::from("/config/open"))
    );
    assert_eq!(
        resolve_open_skills_dir_from_sources(None, None, Some(home)),
        Some(PathBuf::from("/home/example/open-skills"))
    );
}
