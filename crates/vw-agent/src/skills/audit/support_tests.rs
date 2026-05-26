use super::*;
use std::fs;

#[test]
fn extension_helpers_are_case_insensitive() {
    assert!(is_markdown_file(Path::new("Guide.MD")));
    assert!(is_markdown_file(Path::new("Guide.markdown")));
    assert!(is_toml_file(Path::new("SKILL.TOML")));
    assert!(has_script_suffix("install.PS1"));
}

#[test]
fn link_text_helpers_strip_and_classify_targets() {
    assert_eq!(strip_query_and_fragment("docs/readme.md?x=1#top"), "docs/readme.md");
    assert_eq!(url_scheme("https://example.com"), Some("https"));
    assert_eq!(url_scheme("bad scheme://example.com"), None);
    assert!(looks_like_absolute_path("/etc/passwd"));
    assert!(looks_like_absolute_path("C:/Windows/System32"));
    assert!(looks_like_absolute_path("~/secret"));
    assert!(has_markdown_suffix("README.Markdown"));
}

#[test]
fn unsupported_script_detects_shell_shebang() {
    let dir = std::env::temp_dir().join(format!("vw-audit-support-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let script = dir.join("run");
    fs::write(&script, "#!/bin/sh\necho hi\n").unwrap();
    assert!(is_unsupported_script_file(&script));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn relative_display_prefers_root_relative_paths() {
    let root = Path::new("/tmp/skill");
    assert_eq!(relative_display(root, root), ".");
    assert_eq!(relative_display(root, &root.join("SKILL.md")), "SKILL.md");
}
