use super::{
    attachment_name, build_image_attachment_snapshot_path, collect_gateway_file_paths,
    collect_local_attachments, copy_recursively, is_supported_image_attachment,
    map_gateway_project, map_loaded_project_info, map_timestamp, normalize_path,
    relative_to_project, resolve_absolute_gateway_path, sanitize_attachment_stem,
    stabilize_image_attachment, unique_name_in_dir,
};
use crate::app::App;
use std::fs;
use tempfile::tempdir;
use vw_gateway_client::vw_api_types::common::TimestampMs;
use vw_gateway_client::vw_api_types::file::{FileNodeDto, FileNodeKind};
use vw_gateway_client::vw_api_types::id::ProjectId;
use vw_gateway_client::vw_api_types::project::{ProjectDto, ProjectGitStateDto, ProjectStatus};

#[test]
fn supported_image_attachment_detection_is_case_insensitive() {
    for path in ["a.png", "b.JPG", "c.jpeg", "d.WEBP", "e.gif", "f.BmP"] {
        assert!(is_supported_image_attachment(std::path::Path::new(path)), "{path}");
    }

    assert!(!is_supported_image_attachment(std::path::Path::new("notes.txt")));
    assert!(!is_supported_image_attachment(std::path::Path::new("no-extension")));
}

#[test]
fn sanitize_attachment_stem_replaces_unsafe_characters_and_falls_back() {
    assert_eq!(sanitize_attachment_stem(std::path::Path::new("screen shot!.png")), "screen-shot");
    assert_eq!(sanitize_attachment_stem(std::path::Path::new("你好.png")), "image");
    assert_eq!(sanitize_attachment_stem(std::path::Path::new("!!!.png")), "image");
}

#[test]
fn attachment_name_uses_file_name_or_display_path() {
    assert_eq!(attachment_name(std::path::Path::new("/tmp/capture.png")), "capture.png");
    assert_eq!(attachment_name(std::path::Path::new("/")), "/");
}

#[test]
fn external_image_is_copied_into_snapshot_dir() {
    let workspace_dir = tempdir().expect("workspace tempdir");
    let external_dir = tempdir().expect("external tempdir");
    let snapshot_dir = tempdir().expect("snapshot tempdir");
    let source = external_dir.path().join("screen shot.png");
    fs::write(&source, b"png-bytes").expect("write source image");
    let metadata = fs::metadata(&source).expect("metadata");

    let copied = stabilize_image_attachment(
        &source,
        &metadata,
        Some(workspace_dir.path()),
        snapshot_dir.path(),
    )
    .expect("stabilize image");

    assert_ne!(copied, source);
    assert!(copied.starts_with(snapshot_dir.path()));
    assert_eq!(fs::read(&copied).expect("read copied"), b"png-bytes");
}

#[test]
fn workspace_image_keeps_original_path() {
    let workspace_dir = tempdir().expect("workspace tempdir");
    let snapshot_dir = tempdir().expect("snapshot tempdir");
    let source = workspace_dir.path().join("diagram.png");
    fs::write(&source, b"workspace-image").expect("write workspace image");
    let metadata = fs::metadata(&source).expect("metadata");

    let kept = stabilize_image_attachment(
        &source,
        &metadata,
        Some(workspace_dir.path()),
        snapshot_dir.path(),
    )
    .expect("stabilize image");

    assert_eq!(kept, source);
}

#[test]
fn snapshot_path_is_stable_for_same_source_version() {
    let snapshot_dir = tempdir().expect("snapshot tempdir");
    let external_dir = tempdir().expect("external tempdir");
    let source = external_dir.path().join("capture.png");
    fs::write(&source, b"stable-bytes").expect("write source image");
    let metadata = fs::metadata(&source).expect("metadata");

    let first = build_image_attachment_snapshot_path(&source, &metadata, snapshot_dir.path());
    let second = build_image_attachment_snapshot_path(&source, &metadata, snapshot_dir.path());

    assert_eq!(first, second);
}

#[test]
fn snapshot_path_uses_img_extension_when_source_has_no_extension() {
    let snapshot_dir = tempdir().expect("snapshot tempdir");
    let external_dir = tempdir().expect("external tempdir");
    let source = external_dir.path().join("clipboard");
    fs::write(&source, b"raw-bytes").expect("write source image");
    let metadata = fs::metadata(&source).expect("metadata");

    let path = build_image_attachment_snapshot_path(&source, &metadata, snapshot_dir.path());

    assert_eq!(path.extension().and_then(|value| value.to_str()), Some("img"));
    assert!(path.file_name().unwrap().to_string_lossy().starts_with("clipboard-"));
}

#[test]
fn image_already_inside_snapshot_dir_keeps_original_path() {
    let snapshot_dir = tempdir().expect("snapshot tempdir");
    let source = snapshot_dir.path().join("existing.png");
    fs::write(&source, b"snapshot-image").expect("write snapshot image");
    let metadata = fs::metadata(&source).expect("metadata");

    let kept =
        stabilize_image_attachment(&source, &metadata, None, snapshot_dir.path()).expect("kept");

    assert_eq!(kept, source);
}

#[test]
fn collect_local_attachments_skips_empty_missing_folders_and_duplicates() {
    let (mut app, _task) = App::new();
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("notes.txt");
    fs::write(&file, "hello").expect("write file");
    let existing = fs::canonicalize(&file).expect("canonical file").to_string_lossy().to_string();
    app.files = vec![existing.clone()];

    let (accepted, errors) = collect_local_attachments(
        &app,
        vec![
            "   ".to_string(),
            file.to_string_lossy().to_string(),
            dir.path().to_string_lossy().to_string(),
            dir.path().join("missing.txt").to_string_lossy().to_string(),
        ],
    );

    assert!(accepted.is_empty());
    assert!(errors.iter().any(|error| error.contains("不支持文件夹")));
    assert!(errors.iter().any(|error| error.contains("读取失败")));
}

#[test]
fn collect_local_attachments_accepts_non_image_files_once() {
    let (app, _task) = App::new();
    let dir = tempdir().expect("tempdir");
    let first = dir.path().join("a.txt");
    let second = dir.path().join("b.md");
    fs::write(&first, "a").expect("write first");
    fs::write(&second, "b").expect("write second");

    let (accepted, errors) = collect_local_attachments(
        &app,
        vec![
            first.to_string_lossy().to_string(),
            first.to_string_lossy().to_string(),
            second.to_string_lossy().to_string(),
        ],
    );

    assert!(errors.is_empty());
    assert_eq!(accepted.len(), 2);
    assert!(accepted.iter().any(|path| path.ends_with("a.txt")));
    assert!(accepted.iter().any(|path| path.ends_with("b.md")));
}

#[test]
fn collect_local_attachments_reports_image_limit_before_copying() {
    let (mut app, _task) = App::new();
    app.multimodal_settings.max_images = 1;
    app.files = vec!["already.png".to_string()];
    let dir = tempdir().expect("tempdir");
    let image = dir.path().join("new.png");
    fs::write(&image, b"png").expect("write image");

    let (accepted, errors) =
        collect_local_attachments(&app, vec![image.to_string_lossy().to_string()]);

    assert!(accepted.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("已达图片上限"));
}

#[test]
fn unique_name_in_dir_returns_original_or_copy_suffix() {
    let dir = tempdir().expect("tempdir");
    assert_eq!(unique_name_in_dir(dir.path(), "note.txt"), dir.path().join("note.txt"));

    fs::write(dir.path().join("note.txt"), "original").expect("write original");
    assert_eq!(unique_name_in_dir(dir.path(), "note.txt"), dir.path().join("note copy.txt"));

    fs::write(dir.path().join("note copy.txt"), "copy").expect("write copy");
    assert_eq!(unique_name_in_dir(dir.path(), "note.txt"), dir.path().join("note copy 2.txt"));
}

#[test]
fn copy_recursively_copies_nested_directories_and_files() {
    let src = tempdir().expect("src");
    let dst = tempdir().expect("dst");
    fs::create_dir_all(src.path().join("nested")).expect("create nested");
    fs::write(src.path().join("root.txt"), "root").expect("write root");
    fs::write(src.path().join("nested/child.txt"), "child").expect("write child");
    let target = dst.path().join("copied");

    copy_recursively(src.path(), &target).expect("copy recursively");

    assert_eq!(fs::read_to_string(target.join("root.txt")).unwrap(), "root");
    assert_eq!(fs::read_to_string(target.join("nested/child.txt")).unwrap(), "child");
}

#[test]
fn copy_recursively_creates_parent_for_single_file_target() {
    let src = tempdir().expect("src");
    let dst = tempdir().expect("dst");
    let source_file = src.path().join("source.txt");
    fs::write(&source_file, "body").expect("write source");
    let target_file = dst.path().join("missing/target.txt");

    copy_recursively(&source_file, &target_file).expect("copy file");

    assert_eq!(fs::read_to_string(target_file).unwrap(), "body");
}

#[test]
fn relative_to_project_handles_exact_nested_and_outside_paths() {
    assert_eq!(relative_to_project("/tmp/project", "/tmp/project"), Some(".".to_string()));
    assert_eq!(
        relative_to_project("/tmp/project", "/tmp/project/src/main.rs"),
        Some("src/main.rs".to_string())
    );
    assert_eq!(relative_to_project("/tmp/project", "/tmp/other/main.rs"), None);
}

#[test]
fn relative_to_project_uses_canonical_paths_when_string_prefix_fails() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested");
    fs::create_dir_all(&nested).expect("create nested");
    let file = nested.join("file.txt");
    fs::write(&file, "body").expect("write file");
    let root_with_dot = dir.path().join(".");

    assert_eq!(
        relative_to_project(&root_with_dot.to_string_lossy(), &file.to_string_lossy()),
        Some("nested/file.txt".to_string())
    );
}

#[test]
fn resolve_absolute_gateway_path_handles_empty_dot_and_relative_paths() {
    assert_eq!(resolve_absolute_gateway_path("/project", ""), "/project");
    assert_eq!(resolve_absolute_gateway_path("/project", "."), "/project");
    assert_eq!(resolve_absolute_gateway_path("/project", "src/lib.rs"), "/project/src/lib.rs");
}

#[test]
fn normalize_path_rewrites_separators_and_trims_trailing_slashes() {
    assert_eq!(normalize_path(r"C:\work\project\"), "C:/work/project");
    assert_eq!(normalize_path("/tmp/project///"), "/tmp/project");
}

#[test]
fn collect_gateway_file_paths_recurses_directories_and_sorts_by_caller() {
    let tree = FileNodeDto {
        path: ".".to_string(),
        name: ".".to_string(),
        kind: FileNodeKind::Directory,
        size_bytes: None,
        children: Some(vec![
            FileNodeDto {
                path: "src".to_string(),
                name: "src".to_string(),
                kind: FileNodeKind::Directory,
                size_bytes: None,
                children: Some(vec![FileNodeDto {
                    path: "src/lib.rs".to_string(),
                    name: "lib.rs".to_string(),
                    kind: FileNodeKind::File,
                    size_bytes: Some(3),
                    children: None,
                }]),
            },
            FileNodeDto {
                path: "README.md".to_string(),
                name: "README.md".to_string(),
                kind: FileNodeKind::File,
                size_bytes: Some(4),
                children: None,
            },
        ]),
    };
    let mut paths = Vec::new();

    collect_gateway_file_paths("/project", &tree, &mut paths);
    paths.sort();

    assert_eq!(paths, vec!["/project/README.md".to_string(), "/project/src/lib.rs".to_string()]);
}

#[test]
fn map_timestamp_clamps_negative_values_to_zero() {
    assert_eq!(map_timestamp(-1), 0);
    assert_eq!(map_timestamp(42), 42);
}

#[test]
fn map_gateway_project_converts_git_and_time_fields() {
    let info = map_gateway_project(project_dto(Some("main")));

    assert_eq!(info.id, "project-id");
    assert_eq!(info.worktree, "/project");
    assert_eq!(info.name.as_deref(), Some("Gateway Project"));
    assert!(matches!(info.vcs, Some(vw_shared::project::Vcs::Git)));
    assert_eq!(info.time.created, 0);
    assert_eq!(info.time.updated, 5000);
    assert_eq!(info.sandboxes, vec!["/project".to_string()]);
}

#[test]
fn map_loaded_project_info_drops_blank_current_branch() {
    let loaded = map_loaded_project_info("/current".to_string(), project_dto(Some("  ")));

    assert_eq!(loaded.project_path, "/current");
    assert_eq!(loaded.info.id, "project-id");
    assert!(loaded.current_branch.is_none());
}

#[test]
fn map_loaded_project_info_keeps_non_blank_current_branch() {
    let loaded = map_loaded_project_info("/current".to_string(), project_dto(Some("feature")));

    assert_eq!(loaded.current_branch.as_deref(), Some("feature"));
}

fn project_dto(current_branch: Option<&str>) -> ProjectDto {
    ProjectDto {
        id: ProjectId("project-id".to_string()),
        name: "Gateway Project".to_string(),
        directory: "/project".to_string(),
        display_path: "/project".to_string(),
        status: ProjectStatus::Ready,
        created_at_ms: TimestampMs(-100),
        updated_at_ms: TimestampMs(5000),
        default_branch: Some("main".to_string()),
        current_branch: current_branch.map(ToString::to_string),
        git: ProjectGitStateDto {
            is_repo: true,
            has_uncommitted_changes: false,
            ahead: None,
            behind: None,
        },
        active_worktree_id: None,
        session_count: Some(2),
    }
}
