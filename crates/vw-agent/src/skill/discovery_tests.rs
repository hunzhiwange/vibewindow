use super::*;

#[test]
fn index_deserializes_skill_file_lists() {
    let index: Index = serde_json::from_str(
        r#"{"skills":[{"name":"rust","description":"Rust helper","files":["SKILL.md"]}]}"#,
    )
    .unwrap();

    assert_eq!(index.skills.len(), 1);
    assert_eq!(index.skills[0].name, "rust");
    assert_eq!(index.skills[0].files, vec!["SKILL.md"]);
}
