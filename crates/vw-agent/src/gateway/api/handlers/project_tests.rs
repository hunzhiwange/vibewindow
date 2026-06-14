use super::*;
use axum::extract::{Path, Query};
use std::path::Path as FsPath;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn normalize_path_removes_current_directory_prefix() {
    assert_eq!(normalize_path("./crates/vw-agent"), "crates/vw-agent");
    assert_eq!(normalize_path("/crates/vw-agent"), "crates/vw-agent");
    assert_eq!(normalize_path(r"\crates\vw-agent\"), "crates/vw-agent");
    assert_eq!(normalize_path("crates/vw-agent/"), "crates/vw-agent");
}

#[test]
fn worktree_id_round_trips_directory() {
    let directory = "/tmp/vibe window/project";
    let id = worktree_id_from_directory(directory);

    assert_eq!(directory_from_worktree_id(&id).expect("decode"), directory);
}

#[test]
fn directory_from_worktree_id_rejects_invalid_values() {
    let bad_base64 = directory_from_worktree_id("not valid base64!");
    assert_eq!(
        bad_base64.expect_err("bad base64 should fail").status,
        axum::http::StatusCode::NOT_FOUND
    );

    let invalid_utf8 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0xff]);
    assert_eq!(
        directory_from_worktree_id(&invalid_utf8).expect_err("bad utf8 should fail").status,
        axum::http::StatusCode::NOT_FOUND
    );
}

#[test]
fn timestamp_ms_clamps_to_i64_range() {
    assert_eq!(timestamp_ms(42), TimestampMs(42));
    assert_eq!(timestamp_ms(u64::MAX), TimestampMs(i64::MAX));
}

fn project_info(id: &str, worktree: String, sandboxes: Vec<String>) -> project::Info {
    project::Info {
        id: id.to_string(),
        worktree,
        vcs: None,
        name: None,
        icon: None,
        commands: None,
        time: project::TimeInfo { created: 10, updated: 20, initialized: None },
        sandboxes,
    }
}

#[test]
fn project_context_directory_prefers_worktree_then_sandbox() {
    let with_worktree =
        project_info("project-1", "/repo/main".to_string(), vec!["/repo/sandbox".to_string()]);
    assert_eq!(project_context_directory(&with_worktree).expect("worktree"), "/repo/main");

    let with_sandbox = project_info(
        "project-2",
        "   ".to_string(),
        vec!["   ".to_string(), "/repo/sandbox".to_string()],
    );
    assert_eq!(project_context_directory(&with_sandbox).expect("sandbox"), "/repo/sandbox");

    let missing = project_info("project-3", String::new(), vec![" ".to_string()]);
    let error = project_context_directory(&missing).expect_err("missing directory should fail");
    assert_eq!(error.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(error.message, "project directory missing");
}

#[test]
fn map_project_uses_sandbox_directory_and_path_name_fallback() {
    let temp = TempDir::new().expect("tempdir");
    let directory = temp.path().join("alpha-project");
    std::fs::create_dir_all(&directory).expect("project dir");
    let info =
        project_info("project-map", String::new(), vec![directory.to_string_lossy().to_string()]);

    let dto = map_project(&info);

    assert_eq!(dto.id.0, "project-map");
    assert_eq!(dto.name, "alpha-project");
    assert_eq!(dto.directory, directory.to_string_lossy());
    assert_eq!(dto.status, ProjectStatus::Ready);
    assert_eq!(dto.created_at_ms, TimestampMs(10));
    assert_eq!(dto.updated_at_ms, TimestampMs(20));
    assert!(!dto.git.is_repo);
    assert_eq!(dto.session_count, Some(0));
}

#[test]
fn map_project_uses_explicit_non_blank_name() {
    let temp = TempDir::new().expect("tempdir");
    let mut info =
        project_info("project-named", temp.path().to_string_lossy().to_string(), Vec::new());
    info.name = Some("Named Project".to_string());

    let dto = map_project(&info);

    assert_eq!(dto.name, "Named Project");
    assert_eq!(dto.directory, temp.path().to_string_lossy());
}

#[test]
fn map_worktree_derives_name_id_and_timestamps() {
    let temp = TempDir::new().expect("tempdir");
    let directory = temp.path().join("feature-tree");
    std::fs::create_dir_all(&directory).expect("worktree dir");
    let info = project_info("project-worktree", temp.path().to_string_lossy().to_string(), vec![]);

    let dto = map_worktree("project-worktree", &info, directory.to_string_lossy().to_string());

    assert_eq!(dto.project_id.0, "project-worktree");
    assert_eq!(dto.name, "feature-tree");
    assert_eq!(directory_from_worktree_id(&dto.id.0).expect("id should decode"), dto.directory);
    assert_eq!(dto.status, WorktreeStatus::Ready);
    assert_eq!(dto.created_at_ms, TimestampMs(10));
    assert_eq!(dto.updated_at_ms, TimestampMs(20));
}

#[test]
fn git_stdout_and_change_records_are_empty_for_non_repo() {
    let temp = TempDir::new().expect("tempdir");

    assert!(
        git_stdout(&temp.path().to_string_lossy(), &["rev-parse", "--abbrev-ref", "HEAD"])
            .is_none()
    );
    assert!(collect_project_change_records(&temp.path().to_string_lossy()).is_empty());
}

fn write_file(path: &FsPath, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent dir");
    }
    std::fs::write(path, content).expect("file write");
}

fn commit_all(repo: &git2::Repository, message: &str) {
    let mut index = repo.index().expect("index");
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None).expect("add all");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("tree lookup");
    let signature = git2::Signature::now("Vibe Window", "vibe@example.test").expect("signature");
    let parents = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .and_then(|oid| repo.find_commit(oid).ok())
        .into_iter()
        .collect::<Vec<_>>();
    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &parent_refs)
        .expect("commit");
    index.write().expect("index write");
}

#[test]
fn collect_project_change_records_groups_modified_files() {
    let temp = TempDir::new().expect("tempdir");
    let repo = git2::Repository::init(temp.path()).expect("repo init");
    write_file(&temp.path().join("src/main.rs"), "fn main() {}\n");
    commit_all(&repo, "initial");

    write_file(&temp.path().join("src/main.rs"), "fn main() {\n    println!(\"hi\");\n}\n");
    write_file(&temp.path().join("README.md"), "# demo\n");

    let items = collect_project_change_records(&temp.path().to_string_lossy());

    assert!(
        items
            .iter()
            .any(|item| { item.path == "src/main.rs" && item.patch.contains("println!(\"hi\")") })
    );
}

#[tokio::test]
async fn load_project_filters_missing_sandboxes_and_maps_missing_project() {
    let temp = TempDir::new().expect("tempdir");
    let id = format!("handler-project-{}", Uuid::new_v4());
    let existing = temp.path().join("sandbox");
    std::fs::create_dir_all(&existing).expect("sandbox dir");
    let info = project_info(
        &id,
        existing.to_string_lossy().to_string(),
        vec![existing.to_string_lossy().to_string(), "/definitely/missing".to_string()],
    );
    storage::write(&["project", &id], &info).await.expect("project write");

    let loaded = load_project(&id).await.expect("project should load");
    assert_eq!(loaded.sandboxes, vec![existing.to_string_lossy().to_string()]);

    storage::remove(&["project", &id]).await.expect("cleanup");
    let missing = load_project(&id).await.expect_err("missing project should map to not found");
    assert_eq!(missing.status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn project_get_and_update_round_trip_storage() {
    let temp = TempDir::new().expect("tempdir");
    let id = format!("handler-project-update-{}", Uuid::new_v4());
    let info = project_info(&id, temp.path().to_string_lossy().to_string(), Vec::new());
    storage::write(&["project", &id], &info).await.expect("project write");

    let Json(response) = project_get_v1(Path(id.clone())).await.expect("project should get");
    assert_eq!(response.project.id.0, id);

    let Json(response) = project_update_v1(
        Path(id.clone()),
        Json(UpdateProjectRequest {
            name: Some("Updated".to_string()),
            active_worktree_id: None,
            icon: Some(vw_api_types::project::IconUpdateDto {
                override_icon: Some("rocket".to_string()),
                color: Some("#123456".to_string()),
            }),
            commands: Some(vw_api_types::project::CommandsUpdateDto {
                start: Some("cargo run".to_string()),
            }),
        }),
    )
    .await
    .expect("project should update");

    assert_eq!(response.project.name, "Updated");
    storage::remove(&["project", &id]).await.expect("cleanup");
}

#[tokio::test]
async fn project_list_filters_sorts_limits_and_cursors() {
    let temp = TempDir::new().expect("tempdir");
    let token = Uuid::new_v4().to_string();
    let alpha_dir = temp.path().join("alpha");
    let beta_dir = temp.path().join("beta");
    std::fs::create_dir_all(&alpha_dir).expect("alpha dir");
    std::fs::create_dir_all(&beta_dir).expect("beta dir");
    let alpha_id = format!("handler-list-alpha-{}", Uuid::new_v4());
    let beta_id = format!("handler-list-beta-{}", Uuid::new_v4());
    let mut alpha = project_info(&alpha_id, alpha_dir.to_string_lossy().to_string(), Vec::new());
    alpha.name = Some(format!("Alpha {token}"));
    alpha.time.updated = 100;
    let mut beta = project_info(&beta_id, beta_dir.to_string_lossy().to_string(), Vec::new());
    beta.name = Some(format!("Beta {token}"));
    beta.time.updated = 200;
    storage::write(&["project", &alpha_id], &alpha).await.expect("alpha write");
    storage::write(&["project", &beta_id], &beta).await.expect("beta write");

    let Json(page) = project_list_v1(Query(ListProjectsRequest {
        cursor: None,
        limit: Some(1),
        query: Some(token.clone()),
        status: Some(ProjectStatus::Ready),
    }))
    .await
    .expect("projects should list");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id.0, beta_id);
    assert_eq!(page.next_cursor.as_deref(), Some(beta_id.as_str()));

    let Json(next_page) = project_list_v1(Query(ListProjectsRequest {
        cursor: page.next_cursor,
        limit: Some(50),
        query: Some(token),
        status: Some(ProjectStatus::Ready),
    }))
    .await
    .expect("next projects should list");
    assert!(next_page.items.iter().any(|item| item.id.0 == alpha_id));

    storage::remove(&["project", &alpha_id]).await.expect("alpha cleanup");
    storage::remove(&["project", &beta_id]).await.expect("beta cleanup");
}

#[tokio::test]
async fn project_change_records_requires_directory() {
    let err = project_change_records_v1(Query(ListProjectChangeRecordsRequest { directory: None }))
        .await
        .expect_err("directory is required");

    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(err.message, "directory is required");
}

#[tokio::test]
async fn project_resolve_and_change_records_handle_plain_directory() {
    let temp = TempDir::new().expect("tempdir");

    let Json(resolved) = project_resolve_v1(Json(ResolveProjectRequest {
        directory: temp.path().to_string_lossy().to_string(),
        create_if_missing: true,
    }))
    .await
    .expect("plain directory should resolve");
    assert_eq!(resolved.project.directory, temp.path().to_string_lossy());

    let Json(changes) = project_change_records_v1(Query(ListProjectChangeRecordsRequest {
        directory: Some(temp.path().to_string_lossy().to_string()),
    }))
    .await
    .expect("plain directory changes should list");
    assert!(changes.items.is_empty());

    storage::remove(&["project", &resolved.project.id.0]).await.expect("cleanup");
}

#[tokio::test]
async fn project_worktrees_return_empty_for_non_git_project() {
    let temp = TempDir::new().expect("tempdir");
    let id = format!("handler-project-worktrees-{}", Uuid::new_v4());
    let info = project_info(&id, temp.path().to_string_lossy().to_string(), Vec::new());
    storage::write(&["project", &id], &info).await.expect("project write");

    let Json(response) =
        project_worktrees_v1(Path(id.clone())).await.expect("worktrees should list");

    assert!(response.items.is_empty());
    storage::remove(&["project", &id]).await.expect("cleanup");
}

#[tokio::test]
async fn project_worktree_create_rejects_project_without_directory() {
    let id = format!("handler-project-worktree-create-{}", Uuid::new_v4());
    let info = project_info(&id, String::new(), vec![String::new()]);
    storage::write(&["project", &id], &info).await.expect("project write");

    let err = project_worktree_create_v1(
        Path(id.clone()),
        Json(CreateWorktreeRequest {
            name: "feature".to_string(),
            branch: "feature".to_string(),
            from_ref: None,
            checkout: false,
        }),
    )
    .await
    .expect_err("missing project directory should fail");

    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
    storage::remove(&["project", &id]).await.expect("cleanup");
}

#[tokio::test]
async fn worktree_get_rejects_invalid_id_before_project_lookup() {
    let err = worktree_get_v1(Path("invalid!".to_string()))
        .await
        .expect_err("invalid worktree id should fail");

    assert_eq!(err.status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn worktree_delete_and_reset_reject_invalid_ids_before_project_lookup() {
    let delete_err = worktree_delete_v1(
        Path("invalid!".to_string()),
        Json(DeleteWorktreeRequest { force: true }),
    )
    .await
    .expect_err("invalid delete id should fail");
    assert_eq!(delete_err.status, axum::http::StatusCode::NOT_FOUND);

    let reset_err = worktree_reset_v1(
        Path("invalid!".to_string()),
        Json(ResetWorktreeRequest { mode: ResetMode::Hard, target_ref: Some("HEAD".to_string()) }),
    )
    .await
    .expect_err("invalid reset id should fail");
    assert_eq!(reset_err.status, axum::http::StatusCode::NOT_FOUND);
}
