use super::*;

#[test]
fn split_frontmatter_separates_yaml_header_from_body() {
    let input = "---\nname: test\ndescription: demo\n---\nBody text";
    let (frontmatter, body) = split_frontmatter(input);

    assert!(frontmatter.unwrap().contains("name: test"));
    assert_eq!(body.trim(), "Body text");
}

#[test]
fn split_frontmatter_returns_body_when_missing_header() {
    let (frontmatter, body) = split_frontmatter("Body only");
    assert!(frontmatter.is_none());
    assert_eq!(body, "Body only");
}

#[test]
fn split_frontmatter_requires_closing_marker() {
    let input = "---\nname: missing-close\nBody";
    let (frontmatter, body) = split_frontmatter(input);

    assert!(frontmatter.is_none());
    assert_eq!(body, input);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn split_frontmatter_handles_empty_header_and_requires_marker_newline() {
    let (frontmatter, body) = split_frontmatter("---\n\n---\nBody");
    assert_eq!(frontmatter, Some(""));
    assert_eq!(body, "Body");

    let input = "---name: inline\n---\nBody";
    let (frontmatter, body) = split_frontmatter(input);
    assert!(frontmatter.is_none());
    assert_eq!(body, input);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn parse_skill_md_reads_frontmatter_and_normalizes_newlines() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("SKILL.md");
    std::fs::write(&path, "---\r\nname: reviewer\r\ndescription: Review code\r\n---\r\nBody\r\n")
        .unwrap();

    let parsed = parse_skill_md(&path).await.expect("parsed skill");

    assert_eq!(parsed.0, "reviewer");
    assert_eq!(parsed.1, "Review code");
    assert_eq!(parsed.2, "Body");
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn parse_skill_md_rejects_missing_or_blank_name() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("SKILL.md");

    std::fs::write(&path, "---\ndescription: Missing name\n---\nBody").unwrap();
    assert!(parse_skill_md(&path).await.is_none());

    std::fs::write(&path, "---\nname: '   '\n---\nBody").unwrap();
    assert!(parse_skill_md(&path).await.is_none());
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn parse_skill_md_defaults_description_and_rejects_invalid_yaml() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("SKILL.md");

    std::fs::write(&path, "---\nname: helper\n---\n\nBody").unwrap();
    let parsed = parse_skill_md(&path).await.expect("parsed skill");
    assert_eq!(parsed.0, "helper");
    assert_eq!(parsed.1, "");
    assert_eq!(parsed.2, "Body");

    std::fs::write(&path, "---\nname: [unterminated\n---\nBody").unwrap();
    assert!(parse_skill_md(&path).await.is_none());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn glob_files_returns_only_regular_files() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("nested")).unwrap();
    std::fs::write(dir.path().join("nested/SKILL.md"), "content").unwrap();
    std::fs::create_dir(dir.path().join("nested/dir.md")).unwrap();

    let files = glob_files(dir.path(), "**/*.md");

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().and_then(|name| name.to_str()), Some("SKILL.md"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn glob_files_returns_empty_for_invalid_patterns() {
    let dir = tempfile::tempdir().expect("temp dir");

    assert!(glob_files(dir.path(), "[invalid").is_empty());
}
