use super::*;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn git_source_detection_accepts_explicit_git_forms() {
    assert!(is_git_source("https://github.com/acme/skill.git"));
    assert!(is_git_source("ssh://git@github.com/acme/skill.git"));
    assert!(is_git_source("git@github.com:acme/skill.git"));
    assert!(!is_git_source("https://skills.sh/acme/repo/skill"));
    assert!(!is_git_source("/local/path"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn detects_single_new_install_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    let before = snapshot_skill_children(dir.path()).expect("snapshot");
    std::fs::create_dir(dir.path().join("new-skill")).expect("create skill dir");

    let detected = detect_newly_installed_directory(dir.path(), &before).expect("new dir");
    assert_eq!(detected.file_name().and_then(|name| name.to_str()), Some("new-skill"));
}
