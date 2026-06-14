use super::event::UPDATED;
use super::{UpdatedProperties, publish_updated};
use crate::app::agent::bus;
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[test]
fn watcher_event_contract_is_stable() {
    let props = UpdatedProperties { file: "src/lib.rs".to_string(), event: "change".to_string() };
    let json = serde_json::to_value(props).expect("serialize");

    assert_eq!(UPDATED.r#type, "file.watcher.updated");
    assert_eq!(json["event"], "change");
}

#[test]
fn publish_updated_emits_expected_payload() {
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let slot = Arc::clone(&seen);
    let unsubscribe = bus::subscribe(UPDATED, move |payload| {
        slot.lock().unwrap().push(payload);
    });

    publish_updated("src/main.rs", "change");
    unsubscribe();

    let lock = seen.lock().unwrap();
    assert_eq!(lock.len(), 1);
    assert_eq!(lock[0]["type"], UPDATED.r#type);
    assert_eq!(lock[0]["properties"]["file"], "src/main.rs");
    assert_eq!(lock[0]["properties"]["event"], "change");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn scan_snapshot_collects_files_and_skips_ignored_entries() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::create_dir_all(root.join(".git/objects")).expect("create git");
    std::fs::create_dir_all(root.join("target/debug")).expect("create target");
    std::fs::write(root.join("src/lib.rs"), "pub fn ok() {}\n").expect("write lib");
    std::fs::write(root.join("app.log"), "noise\n").expect("write log");
    std::fs::write(root.join(".git/objects/ignored"), "git\n").expect("write git");
    std::fs::write(root.join("target/debug/app"), "bin\n").expect("write target");

    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("src/lib.rs"), root.join("src/link.rs"))
        .expect("create symlink");

    let snapshot = super::scan_snapshot(root);

    assert!(snapshot.contains_key("src/lib.rs"));
    assert!(!snapshot.contains_key("app.log"));
    assert!(!snapshot.contains_key(".git/objects/ignored"));
    assert!(!snapshot.contains_key("target/debug/app"));
    #[cfg(unix)]
    assert!(!snapshot.contains_key("src/link.rs"));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn running_and_stop_reflect_task_registry() {
    let temp = tempfile::tempdir().expect("tempdir");
    let key = temp.path().to_string_lossy().to_string();
    super::stop(temp.path());

    assert!(!super::running(temp.path()));

    let handle = tokio::spawn(async {
        std::future::pending::<()>().await;
    });
    {
        let mut tasks = super::TASKS.lock().unwrap_or_else(|e| e.into_inner());
        tasks.insert(key.clone(), handle);
    }

    assert!(super::running(temp.path()));
    super::stop(temp.path());
    assert!(!super::running(temp.path()));
    assert!(!super::TASKS.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&key));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn init_is_noop_when_filewatcher_flag_is_disabled() {
    let temp = tempfile::tempdir().expect("tempdir");
    super::stop(temp.path());

    if !*crate::app::agent::flag::VIBEWINDOW_EXPERIMENTAL_FILEWATCHER {
        super::init(temp.path());
        assert!(!super::running(temp.path()));
    }
}

#[cfg(target_arch = "wasm32")]
#[test]
fn wasm_watcher_functions_are_noops() {
    super::init("/tmp/project");
    assert!(!super::running("/tmp/project"));
    super::stop("/tmp/project");
}
