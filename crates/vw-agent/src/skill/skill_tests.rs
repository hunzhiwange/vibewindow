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
