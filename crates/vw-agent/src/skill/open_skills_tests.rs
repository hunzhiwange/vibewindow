use super::*;

#[test]
fn resolves_open_skills_dir_by_source_priority() {
    let home = std::path::Path::new("/home/example");
    assert_eq!(
        resolve_open_skills_dir_from_sources(Some(" /env/open "), Some("/config/open"), Some(home)),
        Some(PathBuf::from("/env/open"))
    );
    assert_eq!(
        resolve_open_skills_dir_from_sources(None, Some(" /config/open "), Some(home)),
        Some(PathBuf::from("/config/open"))
    );
    assert_eq!(
        resolve_open_skills_dir_from_sources(None, None, Some(home)),
        Some(PathBuf::from("/home/example/open-skills"))
    );
}
