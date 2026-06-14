#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::super::*;
    use crate::app::Shell;
    use git2::{IndexAddOption, Repository, Signature};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    const SOURCE: &str = include_str!("git.rs");

    fn source_declares_symbol(name: &str) -> bool {
        let needles = [
            format!("fn {name}"),
            format!("pub fn {name}"),
            format!("struct {name}"),
            format!("pub struct {name}"),
            format!("enum {name}"),
            format!("pub enum {name}"),
            format!("type {name}"),
            format!("pub type {name}"),
            format!("const {name}"),
            format!("pub const {name}"),
            format!("static {name}"),
            format!("pub static {name}"),
            format!("impl {name}"),
        ];

        needles.iter().any(|needle| SOURCE.contains(needle))
    }

    fn repo_path(dir: &TempDir) -> &str {
        dir.path().to_str().expect("temp path should be utf-8")
    }

    fn write_file(repo_dir: &Path, file: &str, content: &str) {
        let path = repo_dir.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, content).expect("write test file");
    }

    fn init_repo() -> (TempDir, Repository) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let repo = Repository::init(dir.path()).expect("init git repo");
        {
            let mut config = repo.config().expect("open git config");
            config.set_str("user.name", "Vibe Window").expect("set user name");
            config.set_str("user.email", "vibe@example.test").expect("set user email");
        }
        (dir, repo)
    }

    fn commit_all(repo: &Repository, message: &str) {
        let mut index = repo.index().expect("open index");
        index.add_all(["*"], IndexAddOption::DEFAULT, None).expect("add all files");
        index.write().expect("write index");
        let tree_id = index.write_tree().expect("write tree");
        let tree = repo.find_tree(tree_id).expect("find tree");
        let sig = Signature::now("Vibe Window", "vibe@example.test").expect("signature");
        let parents = if let Ok(head) = repo.head() {
            vec![head.peel_to_commit().expect("head commit")]
        } else {
            Vec::new()
        };
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs).expect("commit");
    }

    fn index_contains(repo: &Repository, file: &str, content: &str) -> bool {
        let repo = Repository::open(repo.workdir().expect("repo should have workdir"))
            .expect("reopen test repo");
        let index = repo.index().expect("open index");
        let oid = index.get_path(Path::new(file), 0).expect("index entry").id;
        let blob = repo.find_blob(oid).expect("find indexed blob");
        std::str::from_utf8(blob.content()).expect("utf-8 blob") == content
    }

    #[test]
    fn git_tests_keeps_planned_coverage_targets() {
        for name in [
            "DIFF_CONTEXT",
            "git_stage_file",
            "sh_quote",
            "git_commit",
            "git_commit_with_body",
            "git_log",
            "git_discard_file",
            "git_diff_for_file",
            "get_file_content_pair",
            "git_discard_hunk",
            "git_stage_hunk",
            "git_stage_line_insert",
            "git_stage_line_delete",
            "git_revert_line_delete",
            "git_revert_line_restore",
            "run_shell_command",
        ] {
            assert!(
                source_declares_symbol(name),
                "expected source to declare coverage target {name}"
            );
        }
    }

    #[test]
    fn git_stage_file_quotes_paths_and_stages_new_file() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "quote's file.txt", "fresh\n");

        git_stage_file(repo_path(&dir), "quote's file.txt", Shell::Bash).expect("stage file");

        assert!(index_contains(&repo, "quote's file.txt", "fresh\n"));
    }

    #[test]
    fn git_commit_variants_create_log_entries_with_quoted_messages() {
        let (dir, _) = init_repo();
        write_file(dir.path(), "a.txt", "one\n");
        git_stage_file(repo_path(&dir), "a.txt", Shell::Bash).expect("stage file");

        git_commit(repo_path(&dir), "feat: it's quoted", Shell::Bash).expect("commit");
        write_file(dir.path(), "a.txt", "one\ntwo\n");
        git_stage_file(repo_path(&dir), "a.txt", Shell::Bash).expect("stage update");
        git_commit_with_body(
            repo_path(&dir),
            "docs: add body",
            "body keeps it's quotes",
            Shell::Bash,
        )
        .expect("commit with body");

        let log = git_log(repo_path(&dir), 2);
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].1, "docs: add body");
        assert_eq!(log[1].1, "feat: it's quoted");
    }

    #[test]
    fn git_log_returns_empty_for_repo_without_commits() {
        let (dir, _) = init_repo();

        assert!(git_log(repo_path(&dir), 3).is_empty());
    }

    #[test]
    fn git_diff_for_file_reports_modify_create_delete_and_none() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "tracked.txt", "old\nsame\n");
        write_file(dir.path(), "delete.txt", "gone\n");
        commit_all(&repo, "initial");

        assert!(git_diff_for_file(repo_path(&dir), "tracked.txt").is_none());

        write_file(dir.path(), "tracked.txt", "new\nsame\n");
        let changed = git_diff_for_file(repo_path(&dir), "tracked.txt").expect("changed diff");
        assert!(changed.contains("-old"));
        assert!(changed.contains("+new"));

        write_file(dir.path(), "created.txt", "fresh\n");
        let created = git_diff_for_file(repo_path(&dir), "created.txt").expect("created diff");
        assert!(created.contains("+fresh"));

        fs::remove_file(dir.path().join("delete.txt")).expect("delete file");
        let deleted = git_diff_for_file(repo_path(&dir), "delete.txt").expect("deleted diff");
        assert!(deleted.contains("-gone"));
    }

    #[test]
    fn get_file_content_pair_reads_head_and_worktree_for_existing_and_new_files() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "tracked.txt", "old\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "tracked.txt", "new\n");
        write_file(dir.path(), "new.txt", "fresh\n");

        assert_eq!(
            get_file_content_pair(repo_path(&dir), "tracked.txt"),
            Some(("old\n".to_string(), "new\n".to_string()))
        );
        assert_eq!(
            get_file_content_pair(repo_path(&dir), "new.txt"),
            Some((String::new(), "fresh\n".to_string()))
        );
    }

    #[test]
    fn git_discard_file_restores_modified_file_and_removes_new_files() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "tracked.txt", "clean\n");
        commit_all(&repo, "initial");

        write_file(dir.path(), "tracked.txt", "dirty\n");
        git_discard_file(repo_path(&dir), "tracked.txt").expect("discard tracked file");
        assert_eq!(
            fs::read_to_string(dir.path().join("tracked.txt")).expect("read tracked"),
            "clean\n"
        );

        write_file(dir.path(), "untracked.txt", "temp\n");
        git_discard_file(repo_path(&dir), "untracked.txt").expect("discard untracked");
        assert!(!dir.path().join("untracked.txt").exists());

        write_file(dir.path(), "indexed.txt", "temp\n");
        git_stage_file(repo_path(&dir), "indexed.txt", Shell::Bash).expect("stage indexed");
        git_discard_file(repo_path(&dir), "indexed.txt").expect("discard indexed");
        assert!(!dir.path().join("indexed.txt").exists());
        assert!(repo.index().expect("open index").get_path(Path::new("indexed.txt"), 0).is_none());
    }

    #[test]
    fn git_discard_file_errors_for_invalid_repository() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let err = git_discard_file(repo_path(&dir), "missing.txt").expect_err("invalid repo");

        assert!(err.contains("repository") || err.contains("not found"));
    }

    #[test]
    fn git_stage_hunk_stages_selected_change_and_rejects_bad_index() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "lines.txt", "one\ntwo\nthree\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "lines.txt", "one\ntwo changed\nthree\n");

        assert_eq!(
            git_stage_hunk(repo_path(&dir), "lines.txt", 99, Shell::Bash),
            Err("bad hunk index".to_string())
        );
        git_stage_hunk(repo_path(&dir), "lines.txt", 0, Shell::Bash).expect("stage hunk");

        assert!(index_contains(&repo, "lines.txt", "one\ntwo changed\nthree\n"));
    }

    #[test]
    fn git_discard_hunk_restores_selected_change_and_rejects_bad_index() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "lines.txt", "one\ntwo\nthree\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "lines.txt", "one\ntwo changed\nthree\n");

        assert_eq!(
            git_discard_hunk(repo_path(&dir), "lines.txt", 99, Shell::Bash),
            Err("bad hunk index".to_string())
        );
        git_discard_hunk(repo_path(&dir), "lines.txt", 0, Shell::Bash).expect("discard hunk");

        assert_eq!(
            fs::read_to_string(dir.path().join("lines.txt")).expect("read lines"),
            "one\ntwo\nthree\n"
        );
    }

    #[test]
    fn git_stage_line_insert_stages_inserted_line_and_errors_without_anchor() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "lines.txt", "one\nthree\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "lines.txt", "one\ntwo\nthree\n");

        git_stage_line_insert(repo_path(&dir), "lines.txt", 1, Shell::Bash).expect("stage insert");
        assert!(index_contains(&repo, "lines.txt", "one\ntwo\nthree\n"));

        let err = git_stage_line_insert(repo_path(&dir), "lines.txt", 50, Shell::Bash)
            .expect_err("missing insertion anchor");
        assert_eq!(err, "Cannot locate insertion anchor");
    }

    #[test]
    fn git_stage_line_delete_stages_deleted_line_and_rejects_out_of_bounds_index() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "lines.txt", "one\ntwo\nthree\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "lines.txt", "one\nthree\n");

        assert_eq!(
            git_stage_line_delete(repo_path(&dir), "lines.txt", 99, Shell::Bash),
            Err("Old line index out of bounds".to_string())
        );
        git_stage_line_delete(repo_path(&dir), "lines.txt", 1, Shell::Bash).expect("stage delete");

        assert!(index_contains(&repo, "lines.txt", "one\nthree\n"));
    }

    #[test]
    fn git_revert_line_delete_removes_selected_worktree_line() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "lines.txt", "one\nthree\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "lines.txt", "one\ntwo\nthree\n");

        assert_eq!(
            git_revert_line_delete(repo_path(&dir), "lines.txt", 99),
            Err("Line index out of bounds".to_string())
        );
        git_revert_line_delete(repo_path(&dir), "lines.txt", 1).expect("remove inserted line");

        assert_eq!(
            fs::read_to_string(dir.path().join("lines.txt")).expect("read lines"),
            "one\nthree\n"
        );
    }

    #[test]
    fn git_revert_line_restore_inserts_head_line_or_appends_when_out_of_range() {
        let (dir, repo) = init_repo();
        write_file(dir.path(), "lines.txt", "one\ntwo\nthree\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "lines.txt", "one\nthree\n");

        assert_eq!(
            git_revert_line_restore(repo_path(&dir), "lines.txt", 0, 99),
            Err("Old line index out of bounds".to_string())
        );
        git_revert_line_restore(repo_path(&dir), "lines.txt", 1, 1).expect("restore middle line");
        assert_eq!(
            fs::read_to_string(dir.path().join("lines.txt")).expect("read lines"),
            "one\ntwo\nthree\n"
        );

        git_revert_line_restore(repo_path(&dir), "lines.txt", 99, 0).expect("append old line");
        assert_eq!(
            fs::read_to_string(dir.path().join("lines.txt")).expect("read lines"),
            "one\ntwo\nthree\none\n"
        );
    }

    #[test]
    fn run_shell_command_returns_stdout_and_stderr_and_normalizes_line_endings() {
        let out = run_shell_command(
            None,
            "printf 'out\\r\\n'; printf 'err\\r\\n' >&2".to_string(),
            Shell::Bash,
        );

        assert!(out.contains("out\n"));
        assert!(out.contains("err\n"));
        assert!(!out.contains('\r'));
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::super::*;
    use crate::app::Shell;

    #[test]
    fn wasm_git_operations_return_explicit_unsupported_results() {
        assert_eq!(git_discard_file("/repo", "file.txt"), Err("Not supported on Web".to_string()));
        assert_eq!(git_diff_for_file("/repo", "file.txt"), None);
        assert_eq!(
            git_discard_hunk("/repo", "file.txt", 0, Shell::Bash),
            Err("Not supported on Web".to_string())
        );
        assert_eq!(
            git_revert_line_delete("/repo", "file.txt", 0),
            Err("Not supported on Web".to_string())
        );
        assert_eq!(
            git_revert_line_restore("/repo", "file.txt", 0, 0),
            Err("Not supported on Web".to_string())
        );
    }
}
