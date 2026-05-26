use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::{ReadTextFileRequest, WriteTextFileRequest};
use parking_lot::Mutex;

use super::filesystem::{FileSystemHandlers, FileSystemHandlersOptions};
use super::types::{
    ClientOperation, ClientOperationStatus, NonInteractivePermissionPolicy, PermissionMode,
};

fn temp_root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("vw-acp-filesystem-tests-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");
    root
}

#[tokio::test]
async fn read_text_file_slices_requested_window_and_emits_completion() {
    let root = temp_root("read");
    let file_path = root.join("nested").join("sample.txt");
    std::fs::create_dir_all(file_path.parent().expect("parent")).expect("create parent");
    std::fs::write(&file_path, "one\ntwo\nthree\nfour").expect("write sample");

    let operations = Arc::new(Mutex::new(Vec::<ClientOperation>::new()));
    let handlers = FileSystemHandlers::new(FileSystemHandlersOptions {
        cwd: root.clone(),
        on_operation: Some({
            let operations = Arc::clone(&operations);
            Arc::new(move |operation| operations.lock().push(operation))
        }),
        ..FileSystemHandlersOptions::default()
    });

    let response = handlers
        .read_text_file(&ReadTextFileRequest::new("session-1", &file_path).line(2_u32).limit(2_u32))
        .await
        .expect("read text file");

    assert_eq!(response.content, "two\nthree");
    let operations = operations.lock();
    assert_eq!(operations.len(), 2);
    assert_eq!(operations[1].status, ClientOperationStatus::Completed);
    assert_eq!(operations[1].details.as_deref(), Some("line=2, limit=2"));
}

#[tokio::test]
async fn write_text_file_denies_paths_outside_root() {
    let root = temp_root("outside");
    let outside = root.parent().expect("parent").join("outside-vw-acp-test.txt");
    let handlers = FileSystemHandlers::new(FileSystemHandlersOptions {
        cwd: root,
        permission_mode: PermissionMode::ApproveAll,
        ..FileSystemHandlersOptions::default()
    });

    let err = handlers
        .write_text_file(&WriteTextFileRequest::new("session-1", outside, "secret"))
        .await
        .expect_err("outside path must fail");

    assert!(err.to_string().contains("outside allowed cwd subtree"));
}

#[tokio::test]
async fn approve_reads_uses_confirmation_for_writes() {
    let root = temp_root("confirm");
    let target = root.join("approved.txt");
    let handlers = FileSystemHandlers::new(FileSystemHandlersOptions {
        cwd: root,
        permission_mode: PermissionMode::ApproveReads,
        non_interactive_permissions: Some(NonInteractivePermissionPolicy::Deny),
        confirm_write: Some(Arc::new(|path, preview| {
            Box::pin(async move {
                assert!(path.ends_with("approved.txt"));
                assert_eq!(preview, "content");
                Ok(true)
            })
        })),
        ..FileSystemHandlersOptions::default()
    });

    handlers
        .write_text_file(&WriteTextFileRequest::new("session-1", &target, "content"))
        .await
        .expect("write approved file");

    assert_eq!(std::fs::read_to_string(target).expect("read file"), "content");
}
