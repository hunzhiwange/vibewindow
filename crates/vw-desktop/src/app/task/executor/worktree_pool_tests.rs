#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("worktree_pool_tests"));
}

#[test]
fn needs_maintenance_uses_cached_pool_before_git_lookup() {
    let repo_root = format!(
        "/tmp/vibewindow-cached-pool-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );

    {
        let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
        pools.insert(
            repo_root.clone(),
            super::RepoWorktreePool {
                repo_root: repo_root.clone(),
                base_branch: "main".to_string(),
                slots: Vec::new(),
                task_slots: std::collections::HashMap::new(),
                merge_target_locks: std::collections::HashMap::new(),
                last_synced_at_ms: 0,
            },
        );
    }

    assert!(super::worktree_pool_needs_maintenance(&repo_root, 1));
    assert!(super::worktree_pool_needs_maintenance(&format!("{repo_root}/nested"), 1));

    let mut pools = super::worktree_pools().lock().expect("worktree pool lock");
    pools.remove(&repo_root);
}
