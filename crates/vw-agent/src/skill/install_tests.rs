use super::*;

#[cfg(not(target_arch = "wasm32"))]
fn write_clean_skill(dir: &Path, title: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("SKILL.md"), format!("# {title}\n\nSafe instructions.")).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
fn run_git(repo: &Path, args: &[&str]) {
    let output = git_std_command().args(args).current_dir(repo).output().expect("run git");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

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

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn detects_missing_and_ambiguous_new_install_directories() {
    let dir = tempfile::tempdir().expect("temp dir");
    let before = snapshot_skill_children(dir.path()).expect("snapshot");
    let missing = detect_newly_installed_directory(dir.path(), &before).unwrap_err();
    assert!(missing.to_string().contains("no new directory"));

    std::fs::create_dir(dir.path().join("one")).unwrap();
    std::fs::create_dir(dir.path().join("two")).unwrap();
    let multiple = detect_newly_installed_directory(dir.path(), &before).unwrap_err();
    assert!(multiple.to_string().contains("multiple new directories"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn git_source_detection_rejects_malformed_scheme_and_scp_forms() {
    assert!(!is_git_source("https:///owner/repo.git"));
    assert!(!is_git_source("https://github.com/owner/repo"));
    assert!(!is_git_source("git@github.com:"));
    assert!(!is_git_source("git/github.com:owner/repo.git"));
    assert!(!is_git_source("git@github.com/owner:repo.git"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn init_skills_dir_writes_readme_policy_and_builtin_skills() {
    let dir = tempfile::tempdir().expect("temp dir");

    init_skills_dir(dir.path()).unwrap();

    let skills = skills_dir(dir.path());
    assert!(skills.join("README.md").is_file());
    assert!(skills.join(crate::app::agent::skill::constants::SKILL_DOWNLOAD_POLICY_FILE).is_file());
    for builtin in crate::app::agent::skill::constants::BUILTIN_PRELOADED_SKILLS {
        assert!(skills.join(builtin.dir_name).join("SKILL.md").is_file());
        assert!(skills.join(builtin.dir_name).join("_meta.json").is_file());
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn local_install_copies_clean_skill_and_rejects_existing_destination() {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("source-skill");
    let skills_path = dir.path().join("skills");
    std::fs::create_dir_all(&skills_path).unwrap();
    write_clean_skill(&source, "Source");

    let (dest, scanned) =
        install_local_skill_source(source.to_str().unwrap(), &skills_path).unwrap();
    assert!(dest.join("SKILL.md").is_file());
    assert!(scanned >= 2);

    let err = install_local_skill_source(source.to_str().unwrap(), &skills_path).unwrap_err();
    assert!(err.to_string().contains("Destination skill already exists"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn local_install_rejects_missing_file_source_and_bad_destination_copy() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skills_path = dir.path().join("skills");
    std::fs::create_dir_all(&skills_path).unwrap();

    let missing = install_local_skill_source("does-not-exist", &skills_path).unwrap_err();
    assert!(missing.to_string().contains("Source path does not exist"));

    let source_file = dir.path().join("not-a-directory");
    std::fs::write(&source_file, "# File").unwrap();
    let file_err = copy_dir_recursive_secure(&source_file, &skills_path.join("copy")).unwrap_err();
    assert!(file_err.to_string().contains("Skill source must be a directory"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn secure_copy_recursively_copies_nested_regular_files() {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("source");
    let dest = dir.path().join("dest");
    write_clean_skill(&source, "Copy");
    std::fs::create_dir_all(source.join("nested")).unwrap();
    std::fs::write(source.join("nested/info.txt"), "nested").unwrap();

    copy_dir_recursive_secure(&source, &dest).unwrap();

    assert_eq!(
        std::fs::read_to_string(dest.join("SKILL.md")).unwrap(),
        "# Copy\n\nSafe instructions."
    );
    assert_eq!(std::fs::read_to_string(dest.join("nested/info.txt")).unwrap(), "nested");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn remove_git_metadata_deletes_nested_git_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skill = dir.path().join("skill");
    std::fs::create_dir_all(skill.join(".git/objects")).unwrap();

    remove_git_metadata(&skill).unwrap();

    assert!(!skill.join(".git").exists());
}

#[cfg(all(not(target_arch = "wasm32"), unix))]
#[test]
fn secure_copy_rejects_symlinks_inside_skill_source() {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("source");
    let dest = dir.path().join("dest");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("SKILL.md"), "# Skill").unwrap();
    std::os::unix::fs::symlink(source.join("SKILL.md"), source.join("linked.md")).unwrap();

    let err = copy_dir_recursive_secure(&source, &dest).unwrap_err();

    assert!(err.to_string().contains("Refusing to copy symlink"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn git_install_clones_local_repo_removes_metadata_and_audits() {
    let dir = tempfile::tempdir().expect("temp dir");
    let repo = dir.path().join("repo-skill");
    let skills_path = dir.path().join("skills");
    write_clean_skill(&repo, "Repo Skill");
    std::fs::create_dir_all(&skills_path).unwrap();

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "Test User"]);
    run_git(&repo, &["add", "SKILL.md"]);
    run_git(&repo, &["commit", "-m", "init"]);

    let (installed, scanned) =
        install_git_skill_source(repo.to_str().unwrap(), &skills_path).unwrap();

    assert_eq!(installed.file_name().and_then(|name| name.to_str()), Some("repo-skill"));
    assert!(installed.join("SKILL.md").is_file());
    assert!(!installed.join(".git").exists());
    assert!(scanned >= 1);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn handle_command_covers_list_audit_and_remove_paths() {
    let dir = tempfile::tempdir().expect("temp dir");
    let config =
        SkillRuntimeConfig { workspace_dir: dir.path().to_path_buf(), skills: Default::default() };
    let skills_path = skills_dir(dir.path());
    let skill = skills_path.join("remove-me");
    write_clean_skill(&skill, "Remove Me");

    handle_command(SkillCommands::List, &config).unwrap();
    handle_command(SkillCommands::Audit { source: skill.to_string_lossy().to_string() }, &config)
        .unwrap();
    let missing_audit =
        handle_command(SkillCommands::Audit { source: "missing".to_string() }, &config)
            .unwrap_err();
    assert!(missing_audit.to_string().contains("not found"));

    let invalid_remove =
        handle_command(SkillCommands::Remove { name: "../bad".to_string() }, &config).unwrap_err();
    assert!(invalid_remove.to_string().contains("Invalid skill name"));

    handle_command(SkillCommands::Remove { name: "remove-me".to_string() }, &config).unwrap();
    assert!(!skill.exists());
    let missing_remove =
        handle_command(SkillCommands::Remove { name: "remove-me".to_string() }, &config)
            .unwrap_err();
    assert!(missing_remove.to_string().contains("Skill not found"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn handle_command_installs_local_skill_source() {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("install-me");
    write_clean_skill(&source, "Install Me");
    let config = SkillRuntimeConfig {
        workspace_dir: dir.path().join("workspace"),
        skills: Default::default(),
    };

    handle_command(
        SkillCommands::Install { source: source.to_string_lossy().to_string() },
        &config,
    )
    .unwrap();

    assert!(skills_dir(&config.workspace_dir).join("install-me/SKILL.md").is_file());
}
