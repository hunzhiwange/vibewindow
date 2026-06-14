use super::*;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn not_found_error_display_uses_message_only() {
    let err = NotFoundError { message: "missing thing".to_string() };
    assert_eq!(err.to_string(), "missing thing");
}

#[test]
fn error_display_delegates_to_inner_error() {
    let err = Error::NotFound(NotFoundError { message: "gone".to_string() });
    assert_eq!(err.to_string(), "gone");

    let io = Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "io boom"));
    assert_eq!(io.to_string(), "io boom");

    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    assert!(Error::Json(json).to_string().contains("EOF"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn now_ms_returns_epoch_milliseconds() {
    assert!(now_ms() > 1_000_000_000_000);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn target_path_joins_segments_and_sets_json_extension() {
    let path = target_path(Path::new("/tmp/storage"), &["project", "alpha"]);

    assert_eq!(path, Path::new("/tmp/storage").join("project").join("alpha.json"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn map_io_error_maps_not_found_to_storage_error() {
    let target = Path::new("/tmp/missing.json");
    let err = map_io_error(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"), target);

    match err {
        Error::NotFound(not_found) => {
            assert!(not_found.message.contains("Resource not found"));
            assert!(not_found.message.contains("missing.json"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn write_read_update_list_and_remove_round_trip() {
    let run_id = Uuid::new_v4().to_string();
    let prefix = ["test-storage", run_id.as_str()];
    let first = ["test-storage", run_id.as_str(), "nested", "first"];
    let second = ["test-storage", run_id.as_str(), "second"];

    write(&first, &json!({"name": "Ada", "count": 1})).await.expect("first write should succeed");
    write(&second, &json!({"name": "Grace"})).await.expect("second write should succeed");

    let read_back: serde_json::Value = read(&first).await.expect("first value should read");
    assert_eq!(read_back["name"], "Ada");

    let updated: serde_json::Value = update(&first, |value: &mut serde_json::Value| {
        value["count"] = json!(2);
    })
    .await
    .expect("update should rewrite JSON");
    assert_eq!(updated["count"], 2);

    let keys = list(&prefix).await.expect("list should succeed");
    assert!(keys.contains(&vec![
        "test-storage".to_string(),
        run_id.clone(),
        "nested".to_string(),
        "first".to_string()
    ]));
    assert!(keys.contains(&vec!["test-storage".to_string(), run_id.clone(), "second".to_string()]));

    remove(&first).await.expect("remove existing should succeed");
    let err = read::<serde_json::Value>(&first).await.expect_err("removed key should be missing");
    assert!(matches!(err, Error::NotFound(_)));

    remove(&first).await.expect("remove missing should stay idempotent");
    remove(&second).await.expect("cleanup should succeed");
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn read_and_update_report_missing_or_invalid_json() {
    let run_id = Uuid::new_v4().to_string();
    let missing = ["test-storage", run_id.as_str(), "missing"];
    let invalid = ["test-storage", run_id.as_str(), "invalid"];

    let err =
        read::<serde_json::Value>(&missing).await.expect_err("missing key should return NotFound");
    assert!(matches!(err, Error::NotFound(_)));

    write(&invalid, &json!({"ok": true})).await.expect("initial write should succeed");
    let state_dir = state().await.dir.clone();
    let path = target_path(&state_dir, &invalid);
    tokio::fs::write(&path, "{").await.expect("invalid JSON should write");

    let err =
        read::<serde_json::Value>(&invalid).await.expect_err("invalid JSON should be reported");
    assert!(matches!(err, Error::Json(_)));

    let err = update::<serde_json::Value>(&missing, |_| {})
        .await
        .expect_err("missing update should return NotFound");
    assert!(matches!(err, Error::NotFound(_)));

    remove(&invalid).await.expect("cleanup should succeed");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn migration_helpers_handle_absent_project_dir_and_root_commit() {
    let temp = TempDir::new().expect("tempdir should create");
    migration_0_blocking(temp.path()).expect("missing legacy project dir should be ignored");

    let repo_dir = temp.path().join("repo");
    std::fs::create_dir_all(&repo_dir).expect("repo dir should create");
    let repo = Repository::init(&repo_dir).expect("repo should init");
    let sig = git2::Signature::now("Vibe Window", "vw@example.com").expect("signature");
    let tree_id = {
        let mut index = repo.index().expect("index");
        index.write_tree().expect("tree")
    };
    let tree = repo.find_tree(tree_id).expect("tree should load");
    let commit =
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).expect("commit should write");

    assert_eq!(root_commit_id(&repo), Some(commit.to_string()));
}
