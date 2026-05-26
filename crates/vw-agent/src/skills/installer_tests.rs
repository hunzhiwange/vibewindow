use super::*;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn git_source_detection_accepts_schemes_and_scp_form() {
    assert!(is_git_source("https://github.com/acme/skill.git"));
    assert!(is_git_source("git://github.com/acme/skill.git"));
    assert!(is_git_source("git@github.com:acme/skill.git"));
    assert!(!is_git_source("https://skills.sh/acme/repo/skill"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn install_directory_detection_requires_exactly_one_new_dir() {
    let dir = tempfile::tempdir().expect("temp dir");
    let before = snapshot_skill_children(dir.path()).unwrap();
    std::fs::create_dir(dir.path().join("skill-one")).unwrap();

    let detected = detect_newly_installed_directory(dir.path(), &before).unwrap();
    assert_eq!(detected.file_name().and_then(|name| name.to_str()), Some("skill-one"));
}
