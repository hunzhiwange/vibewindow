use super::{FilesInput, SearchInput, files, is_hidden_path, match_globs, search, tree};
use glob::Pattern;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[test]
fn files_respects_hidden_flag_and_globs() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("visible.rs"), "fn main() {}").expect("write visible");
    fs::write(temp.path().join(".hidden.rs"), "fn hidden() {}").expect("write hidden");

    let result = files(FilesInput {
        cwd: temp.path().to_path_buf(),
        glob: Some(vec!["*.rs".to_string()]),
        hidden: Some(false),
        follow: Some(false),
        max_depth: None,
    })
    .expect("files should work");

    assert_eq!(result, vec!["visible.rs".to_string()]);
    assert!(is_hidden_path(".hidden.rs"));
    let globs = [Pattern::new("*.rs").expect("valid glob")];
    assert!(match_globs(&globs, "visible.rs"));
    assert!(!match_globs(&globs, "visible.txt"));
    assert!(!is_hidden_path("src/lib.rs"));
}

#[test]
fn files_defaults_include_hidden_and_skip_ignored_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join(".git/objects")).expect("create git");
    fs::create_dir_all(temp.path().join("node_modules/pkg")).expect("create node_modules");
    fs::create_dir_all(temp.path().join("target/debug")).expect("create target");
    fs::write(temp.path().join("visible.txt"), "visible").expect("write visible");
    fs::write(temp.path().join(".hidden.txt"), "hidden").expect("write hidden");
    fs::write(temp.path().join(".git/config"), "git").expect("write git");
    fs::write(temp.path().join("node_modules/pkg/index.js"), "pkg").expect("write pkg");
    fs::write(temp.path().join("target/debug/app"), "bin").expect("write target");

    let result = files(FilesInput {
        cwd: temp.path().to_path_buf(),
        glob: None,
        hidden: None,
        follow: None,
        max_depth: None,
    })
    .expect("files should work");

    assert_eq!(result, vec![".hidden.txt".to_string(), "visible.txt".to_string()]);
}

#[test]
fn files_ignores_invalid_globs_and_honors_max_depth() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src/nested")).expect("create nested");
    fs::write(temp.path().join("root.rs"), "root").expect("write root");
    fs::write(temp.path().join("src/lib.rs"), "lib").expect("write lib");
    fs::write(temp.path().join("src/nested/deep.rs"), "deep").expect("write deep");

    let result = files(FilesInput {
        cwd: temp.path().to_path_buf(),
        glob: Some(vec!["[".to_string()]),
        hidden: Some(true),
        follow: Some(false),
        max_depth: Some(2),
    })
    .expect("files should work");

    assert_eq!(result, vec!["root.rs".to_string(), "src/lib.rs".to_string()]);
}

#[cfg(unix)]
#[test]
fn files_follow_flag_controls_symlinked_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("real.txt"), "real").expect("write real");
    symlink(temp.path().join("real.txt"), temp.path().join("link.txt")).expect("symlink");

    let without_follow = files(FilesInput {
        cwd: temp.path().to_path_buf(),
        glob: None,
        hidden: Some(true),
        follow: Some(false),
        max_depth: None,
    })
    .expect("files should work");
    let with_follow = files(FilesInput {
        cwd: temp.path().to_path_buf(),
        glob: None,
        hidden: Some(true),
        follow: Some(true),
        max_depth: None,
    })
    .expect("files should work");

    assert_eq!(without_follow, vec!["real.txt".to_string()]);
    assert_eq!(with_follow, vec!["link.txt".to_string(), "real.txt".to_string()]);
}

#[test]
fn tree_lists_directories_and_reports_truncation() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src/bin")).expect("create src");
    fs::create_dir_all(temp.path().join("docs")).expect("create docs");
    fs::create_dir_all(temp.path().join(".vibewindow/cache")).expect("create vibewindow");
    fs::write(temp.path().join("top.txt"), "top").expect("write top");
    fs::write(temp.path().join("src/lib.rs"), "lib").expect("write lib");
    fs::write(temp.path().join("src/bin/main.rs"), "main").expect("write main");
    fs::write(temp.path().join("docs/readme.md"), "docs").expect("write docs");
    fs::write(temp.path().join(".vibewindow/cache/state.json"), "{}").expect("write state");

    let full = tree(temp.path(), None).expect("tree should work");
    assert_eq!(full, "docs\nsrc");

    let truncated = tree(temp.path(), Some(2)).expect("tree should work");
    assert_eq!(truncated, "docs\nsrc");
}

#[test]
fn search_returns_match_metadata_and_respects_glob_and_limit() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("a.txt"), "alpha\nneedle one\nneedle two\n").expect("write a");
    fs::write(temp.path().join("b.md"), "needle markdown\n").expect("write b");

    let result = search(SearchInput {
        cwd: temp.path().to_path_buf(),
        pattern: "needle".to_string(),
        glob: Some(vec!["*.txt".to_string()]),
        limit: Some(1),
        follow: Some(false),
    })
    .expect("search should work");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, "a.txt");
    assert_eq!(result[0].line_number, 2);
    assert_eq!(result[0].line, "needle one");
    assert_eq!(result[0].start, 0);
    assert_eq!(result[0].end, 6);
}

#[test]
fn search_skips_non_utf8_files_handles_zero_limit_and_reports_regex_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("binary.txt"), [0xff, 0xfe]).expect("write binary");
    fs::write(temp.path().join("text.txt"), "first\nneedle\n").expect("write text");

    let invalid = search(SearchInput {
        cwd: temp.path().to_path_buf(),
        pattern: "[".to_string(),
        glob: None,
        limit: None,
        follow: Some(false),
    })
    .expect_err("invalid regex should fail");
    assert_eq!(invalid.kind(), std::io::ErrorKind::Other);

    let no_results = search(SearchInput {
        cwd: temp.path().to_path_buf(),
        pattern: "needle".to_string(),
        glob: None,
        limit: Some(0),
        follow: Some(false),
    })
    .expect("search should work");
    assert!(no_results.is_empty());

    let result = search(SearchInput {
        cwd: temp.path().to_path_buf(),
        pattern: "needle".to_string(),
        glob: None,
        limit: None,
        follow: Some(false),
    })
    .expect("search should work");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, "text.txt");
}
