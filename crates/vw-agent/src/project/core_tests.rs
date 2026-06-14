use super::super::{Commands, CommandsUpdate, Icon, IconUpdate};
use super::*;
use std::path::Path;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn local_project_id_is_deterministic_for_path() {
    let path = Path::new("/tmp/vibe-window-project");
    assert_eq!(local_project_id_from_path(path), local_project_id_from_path(path));
}

#[test]
fn now_ms_returns_millisecond_timestamp() {
    assert!(now_ms() > 1_000_000_000_000);
}

fn project_info(updated: u64) -> Info {
    Info {
        id: "project-1".to_string(),
        worktree: "/repo/alpha".to_string(),
        vcs: Some(Vcs::Git),
        name: Some("alpha".to_string()),
        icon: None,
        commands: None,
        time: TimeInfo { created: 1, updated, initialized: None },
        sandboxes: vec!["/repo/alpha-worktree".to_string()],
    }
}

#[test]
fn project_discovery_changed_ignores_time_only_changes() {
    let previous = project_info(100);
    let next = project_info(200);

    assert!(!project_discovery_changed(&previous, &next));
}

#[test]
fn project_discovery_changed_detects_discovery_field_changes() {
    let previous = project_info(100);
    let mut next = project_info(100);
    next.sandboxes.push("/repo/alpha-feature".to_string());

    assert!(project_discovery_changed(&previous, &next));
}

#[test]
fn local_project_id_changes_for_different_paths() {
    let first = Path::new("/tmp/vibe-window-project-a");
    let second = Path::new("/tmp/vibe-window-project-b");

    assert_ne!(local_project_id_from_path(first), local_project_id_from_path(second));
    assert!(local_project_id_from_path(first).starts_with("local-"));
}

#[test]
fn git_entry_and_cached_id_helpers_handle_git_marker() {
    let temp = TempDir::new().expect("tempdir should create");
    let root = temp.path().join("repo");
    let nested = root.join("src").join("bin");
    std::fs::create_dir_all(root.join(".git")).expect("git dir should create");
    std::fs::create_dir_all(&nested).expect("nested dir should create");

    let (found_root, git_entry) = find_git_entry(&nested).expect("git entry should be found");
    assert_eq!(found_root, root);
    assert_eq!(git_entry, found_root.join(".git"));

    assert!(read_cached_id(&git_entry).is_none());
    write_cached_id(&git_entry, "project-123");
    assert_eq!(read_cached_id(&git_entry), Some("project-123".to_string()));
}

#[test]
fn project_discovery_changed_detects_user_visible_fields() {
    let previous = project_info(100);

    let mut renamed = previous.clone();
    renamed.name = Some("renamed".to_string());
    assert!(project_discovery_changed(&previous, &renamed));

    let mut icon = previous.clone();
    icon.icon =
        Some(Icon { url: Some("data:image/png;base64,xx".to_string()), ..Default::default() });
    assert!(project_discovery_changed(&previous, &icon));

    let mut commands = previous.clone();
    commands.commands = Some(Commands { start: Some("cargo run".to_string()) });
    assert!(project_discovery_changed(&previous, &commands));
}

fn stored_project(id: &str, sandbox: &Path) -> Info {
    Info {
        id: id.to_string(),
        worktree: sandbox.to_string_lossy().to_string(),
        vcs: None,
        name: None,
        icon: None,
        commands: None,
        time: TimeInfo { created: 1, updated: 1, initialized: None },
        sandboxes: vec![sandbox.to_string_lossy().to_string(), "/definitely/missing".to_string()],
    }
}

#[tokio::test]
async fn update_list_sandbox_and_initialized_round_trip() {
    let temp = TempDir::new().expect("tempdir should create");
    let id = format!("test-project-{}", Uuid::new_v4());
    let sandbox = temp.path().join("sandbox");
    std::fs::create_dir_all(&sandbox).expect("sandbox should create");
    let info = stored_project(&id, &sandbox);

    crate::app::agent::storage::write(&["project", &id], &info)
        .await
        .expect("project should write");

    let updated = update(UpdateInput {
        project_id: id.clone(),
        name: Some(Some("Demo".to_string())),
        icon: Some(IconUpdate {
            url: Some(Some("https://example.test/icon.png".to_string())),
            override_icon: Some(Some(String::new())),
            color: Some(Some("#fff".to_string())),
        }),
        commands: Some(CommandsUpdate { start: Some(Some("cargo run".to_string())) }),
    })
    .await
    .expect("project should update");
    assert_eq!(updated.name.as_deref(), Some("Demo"));
    assert_eq!(updated.icon.as_ref().and_then(|i| i.override_icon.as_deref()), None);
    assert_eq!(updated.commands.as_ref().and_then(|c| c.start.as_deref()), Some("cargo run"));

    let listed = list().await.expect("projects should list");
    let listed_project = listed.iter().find(|p| p.id == id).expect("project should be listed");
    assert_eq!(listed_project.sandboxes, vec![sandbox.to_string_lossy().to_string()]);

    let sandbox2 = temp.path().join("sandbox2");
    std::fs::create_dir_all(&sandbox2).expect("second sandbox should create");
    let sandbox2_s = sandbox2.to_string_lossy().to_string();
    let added = add_sandbox(&id, &sandbox2_s).await.expect("sandbox add");
    assert!(added.sandboxes.iter().any(|p| p == &sandbox2_s));
    let added_again = add_sandbox(&id, &sandbox2_s).await.expect("sandbox add");
    assert_eq!(added_again.sandboxes.iter().filter(|p| *p == &sandbox2_s).count(), 1);

    let existing = sandboxes(&id).await.expect("sandboxes should read");
    assert!(existing.iter().any(|p| p == &sandbox2_s));

    let removed = remove_sandbox(&id, &sandbox2_s).await.expect("sandbox remove");
    assert!(!removed.sandboxes.iter().any(|p| p == &sandbox2_s));

    set_initialized(&id).await.expect("project should initialize");
    let initialized: Info =
        crate::app::agent::storage::read(&["project", &id]).await.expect("project should read");
    assert!(initialized.time.initialized.is_some());

    let cleared = update(UpdateInput {
        project_id: id.clone(),
        name: None,
        icon: None,
        commands: Some(CommandsUpdate { start: Some(Some(String::new())) }),
    })
    .await
    .expect("project should clear commands");
    assert!(cleared.commands.is_none());

    crate::app::agent::storage::remove(&["project", &id]).await.expect("cleanup should succeed");
}

#[tokio::test]
async fn sandboxes_returns_empty_for_missing_project() {
    let id = format!("missing-project-{}", Uuid::new_v4());
    assert!(sandboxes(&id).await.expect("missing project should be empty").is_empty());
}
