use super::*;
use crate::app::agent::project::{Icon, TimeInfo};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn discover_function_type_is_available() {
    let _ = discover;
}

fn info(id: &str, worktree: &str) -> Info {
    Info {
        id: id.to_string(),
        worktree: worktree.to_string(),
        vcs: Some(Vcs::Git),
        name: None,
        icon: None,
        commands: None,
        time: TimeInfo { created: 1, updated: 1, initialized: None },
        sandboxes: Vec::new(),
    }
}

#[tokio::test]
async fn discover_writes_shortest_favicon_as_data_url() {
    let temp = TempDir::new().expect("tempdir should create");
    let id = format!("icon-test-{}", Uuid::new_v4());
    let nested = temp.path().join("deep").join("assets");
    std::fs::create_dir_all(&nested).expect("nested dir should create");
    std::fs::write(nested.join("favicon.png"), b"nested").expect("nested icon should write");
    std::fs::write(temp.path().join("favicon.svg"), b"<svg/>").expect("root icon should write");

    let input = info(&id, &temp.path().to_string_lossy());
    crate::app::agent::storage::write(&["project", &id], &input)
        .await
        .expect("project should write");

    discover(&input).await.expect("icon discovery should succeed");

    let stored: Info =
        crate::app::agent::storage::read(&["project", &id]).await.expect("project should read");
    let url = stored.icon.and_then(|i| i.url).expect("icon url should be set");
    assert!(url.starts_with("data:image/svg+xml;base64,"));

    crate::app::agent::storage::remove(&["project", &id]).await.expect("cleanup should succeed");
}

#[tokio::test]
async fn discover_skips_non_git_missing_dirs_and_existing_icons() {
    let temp = TempDir::new().expect("tempdir should create");
    let mut non_git = info("non-git", &temp.path().to_string_lossy());
    non_git.vcs = None;
    discover(&non_git).await.expect("non git should be ignored");

    let mut missing = info("missing", &temp.path().join("missing").to_string_lossy());
    discover(&missing).await.expect("missing dir should be ignored");

    missing.worktree = temp.path().to_string_lossy().to_string();
    missing.icon =
        Some(Icon { url: Some("https://example.test/icon.png".to_string()), ..Default::default() });
    discover(&missing).await.expect("existing url should be ignored");

    missing.icon = Some(Icon { override_icon: Some("rocket".to_string()), ..Default::default() });
    discover(&missing).await.expect("override icon should be ignored");
}
