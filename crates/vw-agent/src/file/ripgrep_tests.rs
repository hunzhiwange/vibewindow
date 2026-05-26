use super::{files, is_hidden_path, match_globs, FilesInput};
use glob::Pattern;

#[test]
fn files_respects_hidden_flag_and_globs() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("visible.rs"), "fn main() {}").expect("write visible");
    std::fs::write(temp.path().join(".hidden.rs"), "fn hidden() {}").expect("write hidden");

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
}
