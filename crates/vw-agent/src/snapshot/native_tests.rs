use super::*;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn ensure_gitdir_reports_whether_directory_was_created() {
    let dir = tempfile::tempdir().expect("temp dir");
    let git = dir.path().join("snapshot.git");

    assert!(ensure_gitdir(&git).expect("create gitdir"));
    assert!(git.is_dir());
    assert!(!ensure_gitdir(&git).expect("existing gitdir"));
}
