#[test]
fn desktop_skills_test_module_is_loaded() {
    assert!(true);
}

#[test]
fn skill_kind_sort_key_orders_known_kinds_first() {
    assert_eq!(super::skill_kind_sort_key("recommended"), 0);
    assert_eq!(super::skill_kind_sort_key("system"), 1);
    assert_eq!(super::skill_kind_sort_key("personal"), 2);
    assert_eq!(super::skill_kind_sort_key("custom"), 3);
}

#[test]
fn parse_skill_frontmatter_reads_yaml_block() {
    let parsed = super::parse_skill_frontmatter(
        r#"---
name: Demo Skill
description: Helps with demos.
---

# Body
"#,
    )
    .expect("frontmatter should parse");

    assert_eq!(parsed.name.as_deref(), Some("Demo Skill"));
    assert_eq!(parsed.description.as_deref(), Some("Helps with demos."));
}

#[test]
fn parse_skill_frontmatter_rejects_missing_or_unclosed_blocks() {
    assert!(super::parse_skill_frontmatter("# Demo").is_none());
    assert!(super::parse_skill_frontmatter("---\nname: Demo").is_none());
}

#[test]
fn humanize_skill_id_formats_hyphen_and_underscore_words() {
    assert_eq!(super::humanize_skill_id("find-skills"), "Find Skills");
    assert_eq!(super::humanize_skill_id("agent_helper"), "Agent Helper");
    assert_eq!(super::humanize_skill_id("---"), "---");
}

#[test]
fn normalize_optional_project_path_trims_blank_values() {
    assert_eq!(super::normalize_optional_project_path(None), None);
    assert_eq!(super::normalize_optional_project_path(Some("   ".to_string())), None);
    assert_eq!(
        super::normalize_optional_project_path(Some(" /tmp/project ".to_string())),
        Some("/tmp/project".to_string())
    );
}

#[test]
fn resolve_project_path_requires_existing_directory() {
    let temp = tempfile::tempdir().expect("temp dir");

    assert_eq!(super::resolve_project_path("  ").unwrap_err(), "project_path is required");
    assert!(super::resolve_project_path(temp.path().to_str().expect("utf8 path")).is_ok());
    assert!(super::resolve_project_path("/definitely/missing/vw-skills").is_err());
}

#[test]
fn read_local_catalog_metadata_prefers_toml_manifest() {
    let temp = tempfile::tempdir().expect("temp dir");
    let skill_dir = temp.path().join("demo-skill");
    std::fs::create_dir(&skill_dir).expect("skill dir");
    std::fs::write(
        skill_dir.join("SKILL.toml"),
        "[skill]\nname = 'TOML Title'\ndescription = 'TOML description'\n",
    )
    .expect("skill toml");

    let (title, description) =
        super::read_local_catalog_metadata(&skill_dir, "demo-skill").expect("metadata");

    assert_eq!(title, "TOML Title");
    assert_eq!(description, "TOML description");
}

#[test]
fn read_local_catalog_metadata_falls_back_for_blank_toml_fields() {
    let temp = tempfile::tempdir().expect("temp dir");
    let skill_dir = temp.path().join("blank-skill");
    std::fs::create_dir(&skill_dir).expect("skill dir");
    std::fs::write(skill_dir.join("SKILL.toml"), "[skill]\nname = ''\ndescription = ''\n")
        .expect("skill toml");

    let (title, description) =
        super::read_local_catalog_metadata(&skill_dir, "blank-skill").expect("metadata");

    assert_eq!(title, "Blank Skill");
    assert_eq!(description, "本地技能。");
}

#[test]
fn count_skill_resources_ignores_skill_documents() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::write(temp.path().join("SKILL.md"), "# Demo").expect("skill md");
    std::fs::write(temp.path().join("SKILL.toml"), "[skill]\nname='Demo'\ndescription='Demo'\n")
        .expect("skill toml");
    std::fs::write(temp.path().join("notes.txt"), "resource").expect("resource");
    std::fs::create_dir(temp.path().join("assets")).expect("assets dir");

    assert_eq!(super::count_skill_resources(temp.path()), 2);
}

#[test]
fn local_skill_source_key_maps_all_kinds() {
    assert_eq!(
        super::local_skill_source_key(crate::app::agent::skills::LocalSkillSourceKind::Workspace),
        "workspace"
    );
    assert_eq!(
        super::local_skill_source_key(crate::app::agent::skills::LocalSkillSourceKind::Ancestor),
        "ancestor"
    );
    assert_eq!(
        super::local_skill_source_key(crate::app::agent::skills::LocalSkillSourceKind::Global),
        "global"
    );
}

#[test]
fn unique_skill_directory_skips_existing_names() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir(temp.path().join("new-skill")).expect("new skill");
    std::fs::create_dir(temp.path().join("new-skill-2")).expect("new skill 2");

    assert_eq!(super::unique_skill_directory(temp.path()), temp.path().join("new-skill-3"));
}

#[test]
fn create_new_skill_scaffold_writes_template() {
    let project = tempfile::tempdir().expect("project dir");

    let path = super::create_new_skill_scaffold(
        project.path(),
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("create skill");
    let skill_dir = std::path::PathBuf::from(path);
    let content = std::fs::read_to_string(skill_dir.join("SKILL.md")).expect("skill md");

    assert_eq!(skill_dir.file_name().and_then(|name| name.to_str()), Some("new-skill"));
    assert!(content.contains("name: new-skill"));
    assert!(content.contains("# new skill"));
}

#[test]
fn built_in_skill_helpers_read_metadata_and_mark_recommended_group() {
    if let Some(skill) = super::find_built_in_skill("skill-creator") {
        assert_eq!(skill.id, "skill-creator");
        assert_eq!(skill.group, super::BuiltInSkillGroup::Recommended);
        assert!(skill.resource_count >= 1);
    }
    if let Ok(markdown) = super::read_built_in_skill_markdown("skill-creator") {
        assert!(markdown.contains("skill-creator"));
    }
    assert!(super::find_built_in_skill("missing-skill").is_none());
    assert!(super::read_built_in_skill_markdown("missing-skill").is_err());
}

#[test]
fn resolve_skill_detail_returns_built_in_and_local_variants() {
    let project = tempfile::tempdir().expect("project dir");
    let built_in_result = super::resolve_skill_detail(
        Some(project.path().to_str().expect("utf8 path")),
        "find-skills",
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    );

    if let Ok(built_in) = built_in_result {
        assert_eq!(built_in.id, "find-skills");
        assert_eq!(built_in.kind, "recommended");
        assert!(!built_in.installed);
        assert!(built_in.can_install);
        assert!(!built_in.can_toggle);
        assert_eq!(built_in.document_name, "SKILL.md");
    }

    let skill_dir = project.path().join(".vibewindow/skills/find-skills");
    std::fs::create_dir_all(&skill_dir).expect("skill dir");
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: Local Find\ndescription: Local override\n---\n# Local\n",
    )
    .expect("local skill");

    let local = super::resolve_skill_detail(
        Some(project.path().to_str().expect("utf8 path")),
        "find-skills",
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("local detail");

    assert_eq!(local.title, "Local Find");
    assert_eq!(local.description, "Local override");
    assert!(local.installed);
    assert!(local.enabled);
    assert!(local.can_toggle);
    assert!(local.can_delete);
    assert!(matches!(local.kind.as_str(), "recommended" | "personal"));
}

#[test]
fn install_built_in_skill_copies_once_and_collect_catalog_marks_installed() {
    let project = tempfile::tempdir().expect("project dir");

    let first = super::install_built_in_skill(
        project.path(),
        "skill-creator",
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("install built-in");
    let second = super::install_built_in_skill(
        project.path(),
        "skill-creator",
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("install built-in again");
    let catalog = super::collect_catalog_skills(
        Some(project.path().to_str().expect("utf8 path")),
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("catalog");
    let entry = catalog.iter().find(|item| item.id == "skill-creator").expect("catalog entry");

    assert_eq!(first, second);
    assert!(std::path::Path::new(&first).join("SKILL.md").is_file());
    assert!(entry.installed);
    assert!(entry.enabled);
    assert!(matches!(entry.kind.as_str(), "recommended" | "personal"));
    assert_eq!(entry.source, "workspace");
    assert!(
        super::install_built_in_skill(
            project.path(),
            "missing-skill",
            vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
        )
        .is_err()
    );
}

#[test]
fn discover_local_skills_reads_workspace_skills_and_deduplicates() {
    let project = tempfile::tempdir().expect("project dir");
    let skills_root = project.path().join(".vibewindow/skills");
    let duplicate_root = project.path().join("skills");
    std::fs::create_dir_all(skills_root.join("demo")).expect("primary skill dir");
    std::fs::create_dir_all(duplicate_root.join("demo")).expect("duplicate skill dir");
    std::fs::write(
        skills_root.join("demo/SKILL.md"),
        "---\nname: Primary Demo\ndescription: Primary description\n---\n# Demo\n",
    )
    .expect("primary md");
    std::fs::write(
        duplicate_root.join("demo/SKILL.md"),
        "---\nname: Duplicate Demo\ndescription: Duplicate description\n---\n# Demo\n",
    )
    .expect("duplicate md");
    std::fs::write(skills_root.join("demo/notes.txt"), "notes").expect("resource");

    let skills = super::discover_local_skills(
        Some(project.path().to_str().expect("utf8 path")),
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("discover local skills");

    let demo = skills.iter().find(|skill| skill.id == "demo").expect("demo skill");
    assert_eq!(demo.title, "Primary Demo");
    assert_eq!(demo.description, "Primary description");
    assert_eq!(demo.resource_count, 1);
    assert!(demo.enabled);
    assert_eq!(demo.source, "workspace");
    assert_eq!(skills.iter().filter(|skill| skill.id == "demo").count(), 1);
}

#[test]
fn set_local_skill_enabled_toggles_disabled_marker() {
    let project = tempfile::tempdir().expect("project dir");
    let skill_dir = project.path().join(".vibewindow/skills/demo");
    std::fs::create_dir_all(&skill_dir).expect("skill dir");
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: Demo\ndescription: Demo skill\n---\n# Demo\n",
    )
    .expect("skill md");
    let marker = crate::app::agent::skills::local_skill_disabled_marker_path(&skill_dir);

    let disabled_path = super::set_local_skill_enabled(
        Some(project.path().to_str().expect("utf8 path")),
        "demo",
        false,
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("disable skill");

    assert_eq!(
        std::path::PathBuf::from(disabled_path).canonicalize().expect("disabled path"),
        skill_dir.canonicalize().expect("skill path")
    );
    assert!(marker.is_file());

    super::set_local_skill_enabled(
        Some(project.path().to_str().expect("utf8 path")),
        "demo",
        true,
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("enable skill");
    assert!(!marker.exists());
}

#[test]
fn delete_local_skill_removes_directory() {
    let project = tempfile::tempdir().expect("project dir");
    let skill_dir = project.path().join(".vibewindow/skills/demo");
    std::fs::create_dir_all(&skill_dir).expect("skill dir");
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: Demo\ndescription: Demo skill\n---\n# Demo\n",
    )
    .expect("skill md");
    let expected_deleted = skill_dir.canonicalize().expect("canonical skill path");

    let deleted = super::delete_local_skill(
        Some(project.path().to_str().expect("utf8 path")),
        "demo",
        vw_config_types::skills::SkillsDirectoryProvider::Vibewindow,
    )
    .expect("delete skill");

    assert_eq!(std::path::PathBuf::from(deleted), expected_deleted);
    assert!(!skill_dir.exists());
}
