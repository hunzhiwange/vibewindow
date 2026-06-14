use super::*;
use crate::app::agent::skill::types::{
    SkillLoadMode, SkillRuntimeConfig, SkillsConfig, SkillsPromptInjectionMode,
};

#[test]
fn open_skill_markdown_uses_file_stem_and_metadata_defaults() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("code-review.md");
    std::fs::write(&path, "# Code Review\n\nReview changes carefully.").unwrap();

    let skill = load_open_skill_md(&path, SkillLoadMode::MetadataOnly).unwrap();
    assert_eq!(skill.name, "code-review");
    assert_eq!(skill.version, "open-skills");
    assert_eq!(skill.author.as_deref(), Some("besoeasy/open-skills"));
    assert!(skill.prompts.is_empty());
    assert!(skill.tags.contains(&"open-skills".to_string()));
}

#[test]
fn extract_description_skips_headings_and_blank_lines() {
    assert_eq!(extract_description("# Heading\n\n  Useful description  "), "Useful description");
    assert_eq!(extract_description("# Heading\n\n"), "No description");
}

#[test]
fn load_skill_toml_supports_full_and_metadata_only_modes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("SKILL.toml");
    std::fs::write(
        &path,
        r#"
        prompts = ["Be tidy"]

        [skill]
        name = "format"
        description = "Format code"
        version = "2.0.0"
        author = "team"
        tags = ["rust"]

        [[tools]]
        name = "fmt"
        description = "Format"
        kind = "shell"
        command = "cargo fmt"
        "#,
    )
    .unwrap();

    let full = load_skill_toml(&path, SkillLoadMode::Full).unwrap();
    assert_eq!(full.name, "format");
    assert_eq!(full.tools.len(), 1);
    assert_eq!(full.prompts, vec!["Be tidy"]);

    let metadata = load_skill_toml(&path, SkillLoadMode::MetadataOnly).unwrap();
    assert_eq!(metadata.name, "format");
    assert!(metadata.tools.is_empty());
    assert!(metadata.prompts.is_empty());
}

#[test]
fn load_skill_md_uses_directory_name_and_mode_specific_prompts() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skill_dir = dir.path().join("skills/reviewer");
    std::fs::create_dir_all(&skill_dir).unwrap();
    let path = skill_dir.join("SKILL.md");
    std::fs::write(&path, "# Reviewer\n\nReview carefully.").unwrap();

    let full = load_skill_md(&path, &skill_dir, SkillLoadMode::Full).unwrap();
    assert_eq!(full.name, "reviewer");
    assert_eq!(full.description, "Review carefully.");
    assert_eq!(full.prompts.len(), 1);

    let metadata = load_skill_md(&path, &skill_dir, SkillLoadMode::MetadataOnly).unwrap();
    assert_eq!(metadata.description, "Review carefully.");
    assert!(metadata.prompts.is_empty());
}

#[test]
fn load_skills_from_directory_skips_files_and_loads_clean_skill_dirs() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skills_dir = dir.path().join("skills");
    let good = skills_dir.join("good");
    std::fs::create_dir_all(&good).unwrap();
    std::fs::write(good.join("SKILL.md"), "# Good\n\nA safe skill.").unwrap();
    std::fs::write(skills_dir.join("not-a-dir"), "ignored").unwrap();

    let skills = load_skills_from_directory(&skills_dir, SkillLoadMode::MetadataOnly);

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "good");
    assert!(skills[0].prompts.is_empty());
}

#[test]
fn load_open_skills_prefers_nested_layout_over_legacy_markdown() {
    let dir = tempfile::tempdir().expect("temp dir");
    let repo = dir.path().join("repo");
    let nested = repo.join("skills/nested-review");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("SKILL.md"), "# Nested\n\nNested description.").unwrap();
    std::fs::write(repo.join("legacy.md"), "# Legacy\n\nShould not load.").unwrap();

    let skills = load_open_skills(&repo, SkillLoadMode::Full);

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "nested-review");
    assert_eq!(skills[0].description, "Nested description.");
    assert_eq!(skills[0].prompts.len(), 1);
}

#[test]
fn load_open_skills_legacy_layout_skips_readme_non_markdown_and_directories() {
    let dir = tempfile::tempdir().expect("temp dir");
    let repo = dir.path().join("repo");
    std::fs::create_dir_all(repo.join("folder.md")).unwrap();
    std::fs::write(repo.join("README.md"), "# Readme").unwrap();
    std::fs::write(repo.join("notes.txt"), "ignored").unwrap();
    std::fs::write(repo.join("review.md"), "# Review\n\nReview carefully.").unwrap();

    let skills = load_open_skills(&repo, SkillLoadMode::MetadataOnly);

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "review");
    assert_eq!(skills[0].version, "open-skills");
    assert!(skills[0].prompts.is_empty());
}

#[test]
fn workspace_loading_prefers_toml_and_config_compact_mode_omits_prompts() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skill_dir = dir.path().join("skills/dual");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "# Markdown\n\nShould lose priority.").unwrap();
    std::fs::write(
        skill_dir.join("SKILL.toml"),
        r#"
        prompts = ["Full instructions"]

        [skill]
        name = "toml-skill"
        description = "Loaded from TOML"
        "#,
    )
    .unwrap();

    let direct = load_workspace_skills(dir.path(), SkillLoadMode::Full);
    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0].name, "toml-skill");
    assert_eq!(direct[0].prompts, vec!["Full instructions"]);

    let config = SkillRuntimeConfig {
        workspace_dir: dir.path().to_path_buf(),
        skills: SkillsConfig {
            open_skills_enabled: false,
            open_skills_dir: None,
            prompt_injection_mode: SkillsPromptInjectionMode::Compact,
        },
    };
    let compact = load_skills_with_config(dir.path(), &config);
    assert_eq!(compact.len(), 1);
    assert_eq!(compact[0].name, "toml-skill");
    assert!(compact[0].prompts.is_empty());
}

#[test]
fn markdown_description_stream_returns_default_when_no_body_exists() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("SKILL.md");
    std::fs::write(&path, "# Only heading\n\n## Another heading").unwrap();

    assert_eq!(extract_description_from_markdown(&path).unwrap(), "No description");
}
