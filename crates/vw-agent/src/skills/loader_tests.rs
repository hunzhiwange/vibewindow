use super::*;
use crate::app::agent::skills::types::SkillLoadMode;

#[test]
fn markdown_metadata_parser_uses_frontmatter_when_present() {
    let metadata = parse_markdown_skill_metadata(
        "---\nname: custom\ndescription: Front matter description\n---\n# Ignored\n",
    );

    assert_eq!(metadata.display_name.as_deref(), Some("custom"));
    assert_eq!(metadata.description.as_deref(), Some("Front matter description"));
}

#[test]
fn open_skill_markdown_metadata_only_omits_prompt_body() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("browser.md");
    std::fs::write(&path, "# Browser\n\nUse browser safely.").unwrap();

    let skill = load_open_skill_md(&path, SkillLoadMode::MetadataOnly).unwrap();
    assert_eq!(skill.name, "browser");
    assert!(skill.prompts.is_empty());
    assert!(skill.tags.contains(&"open-skills".to_string()));
}
