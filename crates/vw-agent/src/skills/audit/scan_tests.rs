use super::*;

#[test]
fn depth_first_collection_is_deterministic() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir(dir.path().join("b")).unwrap();
    std::fs::create_dir(dir.path().join("a")).unwrap();
    std::fs::write(dir.path().join("a").join("SKILL.md"), "# A").unwrap();

    let names = collect_paths_depth_first(dir.path())
        .unwrap()
        .into_iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()).map(str::to_string))
        .collect::<Vec<_>>();

    assert_eq!(names[1], "a");
    assert_eq!(names[2], "SKILL.md");
    assert_eq!(names[3], "b");
}
