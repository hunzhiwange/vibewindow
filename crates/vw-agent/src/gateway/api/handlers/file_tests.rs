use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn file_write_body_defaults_create_if_missing_to_false() {
    let body: FileWriteBody = serde_json::from_value(serde_json::json!({
        "path": "notes/todo.md",
        "content": "hello"
    }))
    .expect("valid body");

    assert_eq!(body.path, "notes/todo.md");
    assert_eq!(body.content, "hello");
    assert!(!body.create_if_missing);
}

#[test]
fn resolve_workspace_path_rejects_absolute_path_outside_root() {
    let root = std::path::PathBuf::from("/tmp/vibewindow-root");
    let result = resolve_workspace_path(&root, "/etc/passwd");

    assert!(result.is_err());
}

#[test]
fn find_file_query_defaults_are_optional() {
    let query: FindFileQuery = serde_json::from_value(serde_json::json!({
        "query": "main"
    }))
    .expect("valid query");

    assert_eq!(query.query, "main");
    assert!(query.directory.is_none());
    assert!(query.dirs.is_none());
    assert!(query.r#type.is_none());
    assert!(query.limit.is_none());
}

#[test]
fn large_file_status_query_requires_job_id() {
    let query: LargeFileScanStatusQuery =
        serde_json::from_value(serde_json::json!({ "job_id": "job-1" })).expect("valid query");

    assert_eq!(query.job_id, "job-1");
}

#[test]
fn next_large_file_job_id_is_unique_and_prefixed() {
    let first = next_large_file_job_id();
    let second = next_large_file_job_id();

    assert!(first.starts_with("large-file-"));
    assert!(second.starts_with("large-file-"));
    assert_ne!(first, second);
}

#[test]
fn large_file_progress_helpers_clamp_and_advance() {
    let progress = Some(std::sync::Arc::new(std::sync::Mutex::new(LargeFileScanProgressDto {
        phase_label: String::new(),
        current_path: String::new(),
        total_files: 2,
        processed_files: 0,
        matched_files: 0,
        progress_value: 0.0,
    })));

    set_large_file_progress(&progress, "phase", "root".to_string(), 0, 2, 0, 2.0);
    {
        let state = progress.as_ref().expect("progress").lock().expect("lock");
        assert_eq!(state.phase_label, "phase");
        assert_eq!(state.progress_value, 1.0);
    }

    advance_large_file_progress(&progress, "file.bin".to_string(), true);
    let state = progress.as_ref().expect("progress").lock().expect("lock");
    assert_eq!(state.phase_label, "扫描文件大小");
    assert_eq!(state.current_path, "file.bin");
    assert_eq!(state.processed_files, 1);
    assert_eq!(state.matched_files, 1);
    assert!((0.2..=0.98).contains(&state.progress_value));
}

#[test]
fn large_file_cancel_flag_controls_scan_cancellation() {
    assert!(!is_large_file_scan_cancelled(&None));

    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel = Some(flag.clone());
    assert!(!is_large_file_scan_cancelled(&cancel));
    flag.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(is_large_file_scan_cancelled(&cancel));
}

#[test]
fn classify_large_file_returns_only_files_at_or_above_threshold() {
    let temp = tempfile::tempdir().expect("temp dir");
    let small = temp.path().join("small.bin");
    let large = temp.path().join("large.bin");
    std::fs::write(&small, b"small").expect("small file");
    write_sparse_file(&large, LARGE_FILE_MIN_BYTES).expect("large file");

    assert!(classify_large_file(temp.path()).expect("dir classification").is_none());
    assert!(classify_large_file(&small).expect("small classification").is_none());
    let entry = classify_large_file(&large).expect("large classification").expect("large entry");
    assert_eq!(entry.name, "large.bin");
    assert_eq!(entry.size_bytes, LARGE_FILE_MIN_BYTES);
}

#[test]
fn build_large_file_category_sorts_files_by_size_descending() {
    let category = build_large_file_category(
        "test",
        "Test",
        "Subtitle",
        vec![
            LargeFileEntryDto {
                name: "small".to_string(),
                path: "/tmp/small".to_string(),
                parent: "/tmp".to_string(),
                size_bytes: 3,
            },
            LargeFileEntryDto {
                name: "large".to_string(),
                path: "/tmp/large".to_string(),
                parent: "/tmp".to_string(),
                size_bytes: 9,
            },
        ],
    );

    assert_eq!(category.total_bytes, 12);
    assert_eq!(
        category.files.iter().map(|file| file.name.as_str()).collect::<Vec<_>>(),
        ["large", "small"]
    );
}

#[test]
fn scan_large_files_groups_matching_files_and_updates_progress() {
    let temp = tempfile::tempdir().expect("temp dir");
    write_sparse_file(&temp.path().join("tiny.bin"), LARGE_FILE_MIN_BYTES - 1).expect("tiny file");
    write_sparse_file(&temp.path().join("small.bin"), LARGE_FILE_MIN_BYTES).expect("small file");
    write_sparse_file(&temp.path().join("medium.bin"), ONE_HUNDRED_MB).expect("medium file");

    let progress = Some(std::sync::Arc::new(std::sync::Mutex::new(LargeFileScanProgressDto {
        phase_label: String::new(),
        current_path: String::new(),
        total_files: 0,
        processed_files: 0,
        matched_files: 0,
        progress_value: 0.0,
    })));
    let report = scan_large_files_with_progress(
        temp.path().to_string_lossy().to_string(),
        progress.clone(),
        None,
    )
    .expect("scan");

    assert_eq!(report.total_files, 2);
    assert_eq!(report.total_bytes, LARGE_FILE_MIN_BYTES + ONE_HUNDRED_MB);
    assert_eq!(
        report.categories.iter().map(|category| category.id.as_str()).collect::<Vec<_>>(),
        ["100m", "50m"]
    );
    let state = progress.as_ref().expect("progress").lock().expect("lock");
    assert_eq!(state.phase_label, "扫描完成");
    assert_eq!(state.progress_value, 1.0);
}

#[test]
fn scan_large_files_rejects_missing_or_file_roots() {
    let temp = tempfile::tempdir().expect("temp dir");
    let file = temp.path().join("file.txt");
    std::fs::write(&file, "content").expect("file");

    assert!(scan_large_files(temp.path().join("missing").to_string_lossy().to_string()).is_err());
    assert!(scan_large_files(file.to_string_lossy().to_string()).is_err());
}

#[test]
fn scan_large_files_observes_pre_cancelled_flag() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::write(temp.path().join("file.txt"), "content").expect("file");
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

    let error = scan_large_files_with_progress(
        temp.path().to_string_lossy().to_string(),
        None,
        Some(cancel),
    )
    .expect_err("cancelled scan should fail");

    assert!(format!("{error:?}").contains("已取消扫描"));
}

#[test]
fn delete_large_files_deletes_relative_and_missing_paths_but_blocks_escape() {
    let temp = tempfile::tempdir().expect("temp dir");
    let delete_me = temp.path().join("delete.bin");
    std::fs::write(&delete_me, "delete").expect("delete file");

    let response = delete_large_files(
        temp.path().to_str().expect("utf8 path"),
        vec!["delete.bin".to_string(), "missing.bin".to_string(), "../escape.bin".to_string()],
    )
    .expect("delete files");

    assert_eq!(response.deleted_paths, ["delete.bin", "missing.bin"]);
    assert_eq!(response.failed_paths.len(), 1);
    assert_eq!(response.failed_paths[0].path, "../escape.bin");
    assert!(!delete_me.exists());
}

#[test]
fn contains_path_handles_existing_and_future_children() {
    let temp = tempfile::tempdir().expect("temp dir");
    let child = temp.path().join("child.txt");
    std::fs::write(&child, "child").expect("child");

    assert!(contains_path(temp.path(), &child));
    assert!(contains_path(temp.path(), &temp.path().join("future.txt")));
    assert!(!contains_path(temp.path(), &temp.path().join("../outside.txt")));
}

#[test]
fn resolve_agent_workspace_root_uses_env_and_agent_suffix() {
    let temp = tempfile::tempdir().expect("temp dir");
    let previous = std::env::var_os("VIBEWINDOW_WORKSPACE");
    unsafe {
        std::env::set_var("VIBEWINDOW_WORKSPACE", temp.path());
    }

    assert_eq!(
        resolve_agent_workspace_root(Some("main")),
        Some(temp.path().to_string_lossy().to_string())
    );
    assert_eq!(
        resolve_agent_workspace_root(Some("worker")),
        Some(format!("{}-worker", temp.path().to_string_lossy()))
    );

    unsafe {
        match previous {
            Some(value) => std::env::set_var("VIBEWINDOW_WORKSPACE", value),
            None => std::env::remove_var("VIBEWINDOW_WORKSPACE"),
        }
    }
}

#[test]
fn decode_worktree_directory_decodes_url_safe_base64() {
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("/tmp/worktree");

    assert_eq!(decode_worktree_directory(&encoded).expect("decoded"), "/tmp/worktree");
    assert!(decode_worktree_directory("%%%").is_err());
}

#[test]
fn remove_existing_path_requires_recursive_for_directories() {
    let temp = tempfile::tempdir().expect("temp dir");
    let file = temp.path().join("file.txt");
    let dir = temp.path().join("dir");
    std::fs::write(&file, "file").expect("file");
    std::fs::create_dir(&dir).expect("dir");

    assert!(remove_existing_path(&dir, false).is_err());
    remove_existing_path(&file, false).expect("remove file");
    remove_existing_path(&dir, true).expect("remove dir");
    assert!(!file.exists());
    assert!(!dir.exists());
}

#[test]
fn copy_path_recursive_copies_files_and_directories() {
    let temp = tempfile::tempdir().expect("temp dir");
    let source = temp.path().join("source");
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(source.join("nested")).expect("nested");
    std::fs::write(source.join("nested/file.txt"), "hello").expect("source file");

    copy_path_recursive(&source, &dest).expect("copy dir");

    assert_eq!(std::fs::read_to_string(dest.join("nested/file.txt")).expect("dest file"), "hello");
}

#[tokio::test]
async fn file_read_and_write_v1_round_trip_with_directory_body() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path().canonicalize().expect("canonical temp dir");
    let headers = axum::http::HeaderMap::new();

    let missing = file_read_v1(
        headers.clone(),
        Json(FileReadBody {
            directory: Some(root.to_string_lossy().to_string()),
            agent_key: None,
            path: "notes/todo.md".to_string(),
        }),
    )
    .await
    .expect("missing file reads empty")
    .0;
    assert_eq!(missing.content, "");

    let written = file_write_v1(
        headers.clone(),
        Json(FileWriteBody {
            directory: Some(root.to_string_lossy().to_string()),
            agent_key: None,
            path: "notes/todo.md".to_string(),
            content: "hello".to_string(),
            create_if_missing: true,
        }),
    )
    .await
    .expect("write file")
    .0;
    assert!(written.ok);
    assert_eq!(written.bytes_written, 5);

    let read = file_read_v1(
        headers,
        Json(FileReadBody {
            directory: Some(root.to_string_lossy().to_string()),
            agent_key: None,
            path: "notes/todo.md".to_string(),
        }),
    )
    .await
    .expect("read file")
    .0;
    assert_eq!(read.content, "hello");
}

#[tokio::test]
async fn file_move_copy_and_delete_v1_use_worktree_id_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path().canonicalize().expect("canonical temp dir");
    std::fs::write(root.join("source.txt"), "source").expect("source");
    let encoded =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(root.to_string_lossy().as_ref());
    let project_id = vw_api_types::id::ProjectId("project".to_string());
    let worktree_id = Some(vw_api_types::id::WorktreeId(encoded));

    let _ = file_copy_v1(Json(CopyFileRequest {
        project_id: project_id.clone(),
        worktree_id: worktree_id.clone(),
        from_path: "source.txt".to_string(),
        to_path: "copies/source.txt".to_string(),
        overwrite: false,
    }))
    .await
    .expect("copy file");
    assert_eq!(std::fs::read_to_string(root.join("copies/source.txt")).expect("copy"), "source");

    let _ = file_move_v1(Json(MoveFileRequest {
        project_id: project_id.clone(),
        worktree_id: worktree_id.clone(),
        from_path: "copies/source.txt".to_string(),
        to_path: "moved.txt".to_string(),
        overwrite: false,
    }))
    .await
    .expect("move file");
    assert!(root.join("moved.txt").is_file());
    assert!(!root.join("copies/source.txt").exists());

    let _ = file_delete_v1(Json(DeleteFileRequest {
        project_id,
        worktree_id,
        path: "moved.txt".to_string(),
        recursive: false,
    }))
    .await
    .expect("delete file");
    assert!(!root.join("moved.txt").exists());
}

#[tokio::test]
async fn large_file_scan_start_status_and_cancel_handlers_cover_job_lifecycle() {
    let temp = tempfile::tempdir().expect("temp dir");
    write_sparse_file(&temp.path().join("large.bin"), LARGE_FILE_MIN_BYTES).expect("large file");

    let started = large_file_scan_start_v1(Json(LargeFileScanStartRequest {
        root: temp.path().to_string_lossy().to_string(),
    }))
    .await
    .expect("start scan")
    .0;
    assert!(started.job_id.starts_with("large-file-"));

    let mut final_status = None;
    for _ in 0..20 {
        let status = large_file_scan_status_v1(Query(LargeFileScanStatusQuery {
            job_id: started.job_id.clone(),
        }))
        .await
        .expect("scan status")
        .0;
        if status.finished {
            final_status = Some(status);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let status = final_status.expect("scan should finish");
    assert!(status.error.is_none());
    assert_eq!(status.report.expect("report").total_files, 1);

    let cancel_started = large_file_scan_start_v1(Json(LargeFileScanStartRequest {
        root: temp.path().to_string_lossy().to_string(),
    }))
    .await
    .expect("start cancel scan")
    .0;
    let ack = large_file_scan_cancel_v1(Json(LargeFileScanCancelRequest {
        job_id: cancel_started.job_id,
    }))
    .await
    .expect("cancel scan")
    .0;
    assert!(ack.ok);
}

fn write_sparse_file(path: &std::path::Path, len: u64) -> std::io::Result<()> {
    let file = std::fs::File::create(path)?;
    file.set_len(len)
}
