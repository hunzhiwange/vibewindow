use super::*;
use crate::app::agent::skill::types::{Skill, SkillTool, SkillsPromptInjectionMode};
use std::collections::HashMap;

#[test]
fn prompt_escapes_xml_and_compact_uses_relative_location() {
    let skill = Skill {
        name: "review&fix".to_string(),
        description: "Use <care>".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        tags: Vec::new(),
        tools: Vec::new(),
        prompts: vec!["Keep \"scope\" tight".to_string()],
        location: None,
    };
    let prompt = skills_to_prompt_with_mode(
        &[skill],
        std::path::Path::new("/workspace"),
        SkillsPromptInjectionMode::Compact,
    );

    assert!(prompt.contains("review&amp;fix"));
    assert!(prompt.contains("Use &lt;care&gt;"));
    assert!(prompt.contains("skills/review&amp;fix/SKILL.md"));
    assert!(!prompt.contains("<tools>"));
}

#[test]
fn empty_skill_list_produces_empty_prompt() {
    assert_eq!(skills_to_prompt(&[], std::path::Path::new("/workspace")), "");
}

#[test]
fn full_prompt_includes_instructions_tools_and_absolute_location() {
    let workspace = tempfile::tempdir().expect("workspace");
    let location = workspace.path().join("custom/SKILL.md");
    let skill = Skill {
        name: "build".to_string(),
        description: "Build things".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        tags: Vec::new(),
        tools: vec![SkillTool {
            name: "fmt".to_string(),
            description: "Format".to_string(),
            kind: "shell".to_string(),
            command: "cargo fmt".to_string(),
            args: HashMap::new(),
        }],
        prompts: vec!["Run checks".to_string()],
        location: Some(location.clone()),
    };

    let prompt = skills_to_prompt(&[skill], workspace.path());

    assert!(prompt.contains("<instructions>"));
    assert!(prompt.contains("<tools>"));
    assert!(prompt.contains(location.to_string_lossy().as_ref()));
}

#[test]
fn skills_dir_appends_skills_component() {
    assert_eq!(
        skills_dir(std::path::Path::new("/workspace")),
        std::path::Path::new("/workspace/skills")
    );
}
