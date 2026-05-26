//! 覆盖项目消息辅助函数的行为，确保路径、配置和会话状态处理稳定。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{build_image_attachment_snapshot_path, stabilize_image_attachment};
use std::fs;
use tempfile::tempdir;

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
