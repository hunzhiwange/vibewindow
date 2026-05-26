use super::*;

#[test]
fn init_creates_skills_directory_and_readme() {
    let dir = tempfile::tempdir().expect("temp dir");

    init_skills_dir(dir.path()).expect("skills dir should initialize");

    let skills = crate::app::agent::skills::skills_dir(dir.path());
    assert!(skills.is_dir());
    assert!(skills.join("README.md").is_file());
}
