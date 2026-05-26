use super::*;
use crate::app::agent::skill::types::SkillLoadMode;

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
