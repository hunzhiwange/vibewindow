use crate::app::agent::tools::ToolRuntimeContext;

fn ctx_for(path: &std::path::Path) -> ToolRuntimeContext {
    ToolRuntimeContext::new("file-link-tests", Some(path.to_string_lossy().to_string()))
}

#[test]
fn extract_file_link_blocks_collects_complete_blocks_only() {
    let text = concat!(
        "before\n",
        "<file_link>\npath: a.md\n</file_link>\n",
        "middle\n",
        "<file_link>\npath: b.md\n</file_link>\n",
        "<file_link>\npath: missing-end\n"
    );

    let blocks = super::extract_file_link_blocks(text);

    assert_eq!(blocks.len(), 2);
    assert!(blocks[0].contains("path: a.md"));
    assert!(blocks[1].contains("path: b.md"));
}

#[test]
fn compact_file_link_replaces_block_with_path_line() {
    let text = concat!(
        "prefix\n",
        "<file_link>\n",
        "path: docs/readme.md\n",
        "open: file:///tmp/readme.md\n",
        "size_bytes: 12\n",
        "</file_link>\n",
        "suffix"
    );

    let compacted = super::compact_file_link(text);

    assert_eq!(compacted, "prefix\npath: docs/readme.md\n\nsuffix");
}

#[test]
fn compact_file_link_keeps_unparseable_input_usable() {
    assert_eq!(super::compact_file_link("plain output"), "plain output");
    assert_eq!(
        super::compact_file_link("<file_link>\nopen: file:///tmp/a\n</file_link>\nbody"),
        "body"
    );
    assert_eq!(
        super::compact_file_link("before <file_link>\npath: a.md"),
        "before <file_link>\npath: a.md"
    );
}

#[test]
fn extract_file_path_from_input_accepts_json_aliases_and_plain_references() {
    assert_eq!(
        super::extract_file_path_from_input(r#"{"filePath":"src/main.rs"}"#),
        Some("src/main.rs".to_string())
    );
    assert_eq!(
        super::extract_file_path_from_input(r#"{"file_path":"src/lib.rs"}"#),
        Some("src/lib.rs".to_string())
    );
    assert_eq!(
        super::extract_file_path_from_input(r#"{"path":"docs/readme.md"}"#),
        Some("docs/readme.md".to_string())
    );
    assert_eq!(
        super::extract_file_path_from_input("[README](docs/readme.md#L12)"),
        Some("docs/readme.md".to_string())
    );
    assert_eq!(
        super::extract_file_path_from_input("`file:///tmp/demo.txt#line-4`"),
        Some("tmp/demo.txt".to_string())
    );
    assert_eq!(super::extract_file_path_from_input("   "), None);
    assert_eq!(super::extract_file_path_from_input(r#"{"path":42}"#), None);
    assert_eq!(super::extract_file_path_from_input("{not json}"), None);
}

#[test]
fn resolve_full_path_uses_absolute_root_or_relative_path() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let ctx = ctx_for(workspace.path());
    let absolute = workspace.path().join("already.txt");

    assert_eq!(super::resolve_full_path(&ctx, absolute.to_str().unwrap()), absolute);
    assert_eq!(super::resolve_full_path(&ctx, "docs/a.md"), workspace.path().join("docs/a.md"));

    let no_root = ToolRuntimeContext::new("file-link-no-root", None);
    assert_eq!(
        super::resolve_full_path(&no_root, "relative.md"),
        std::path::PathBuf::from("relative.md")
    );
}

#[test]
fn build_file_link_reports_relative_path_open_url_and_size() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let docs = workspace.path().join("docs");
    std::fs::create_dir_all(&docs).expect("docs dir");
    std::fs::write(docs.join("a.md"), "hello").expect("doc");
    let ctx = ctx_for(workspace.path());

    let link = super::build_file_link(&ctx, "docs/a.md").expect("link");

    assert!(link.contains("<file_link>"));
    assert!(link.contains("path: docs/a.md"));
    assert!(link.contains("open: file:///"));
    assert!(link.contains("size_bytes: 5"));
}

#[test]
fn maybe_inject_file_link_respects_eligibility_and_existing_links() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    std::fs::write(workspace.path().join("a.md"), "hello").expect("doc");
    let ctx = ctx_for(workspace.path());

    let injected = super::maybe_inject_file_link("read", r#"{"path":"a.md"}"#, &ctx, "body");
    assert!(injected.starts_with("<file_link>"));
    assert!(injected.ends_with("body"));

    let link_only = super::maybe_inject_file_link("read", r#"{"path":"a.md"}"#, &ctx, "   ");
    assert!(link_only.contains("path: a.md"));
    assert!(!link_only.contains("body"));

    let existing = "<file_link>\npath: old\n</file_link>\nbody";
    assert_eq!(super::maybe_inject_file_link("read", "a.md", &ctx, existing), existing);
    assert_eq!(super::maybe_inject_file_link("bash", "a.md", &ctx, "body"), "body");
    assert_eq!(super::maybe_inject_file_link("read", "   ", &ctx, "body"), "body");
}
