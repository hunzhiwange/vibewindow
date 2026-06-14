use super::*;
use crate::app::agent::project;
use std::sync::Arc;

#[test]
fn clear_approval_manager_is_idempotent_for_unknown_directory() {
    clear_approval_manager_for_directory("/tmp/not-present");
    clear_approval_manager_for_directory("/tmp/not-present");
}

#[tokio::test]
async fn approval_manager_reuses_current_instance_cache_entry() {
    let temp = tempfile::tempdir().expect("tempdir");
    let directory = temp.path().to_string_lossy().to_string();
    clear_approval_manager_for_directory(&directory);

    let (first, second) = project::instance::provide(temp.path(), None, || {
        Box::pin(async {
            let first = approval_manager_for_current_instance().await;
            let second = approval_manager_for_current_instance().await;
            Ok::<_, project::Error>((first, second))
        })
    })
    .await
    .expect("instance context should be provided")
    .expect("approval managers should load");

    assert!(Arc::ptr_eq(&first, &second));
    clear_approval_manager_for_directory(&directory);
}

#[tokio::test]
async fn approval_manager_uses_separate_entries_for_separate_instances() {
    let first_temp = tempfile::tempdir().expect("first tempdir");
    let second_temp = tempfile::tempdir().expect("second tempdir");
    let first_directory = first_temp.path().to_string_lossy().to_string();
    let second_directory = second_temp.path().to_string_lossy().to_string();
    clear_approval_manager_for_directory(&first_directory);
    clear_approval_manager_for_directory(&second_directory);

    let first = project::instance::provide(first_temp.path(), None, || {
        Box::pin(async { Ok::<_, project::Error>(approval_manager_for_current_instance().await) })
    })
    .await
    .expect("first instance context should be provided")
    .expect("first approval manager should load");
    let second = project::instance::provide(second_temp.path(), None, || {
        Box::pin(async { Ok::<_, project::Error>(approval_manager_for_current_instance().await) })
    })
    .await
    .expect("second instance context should be provided")
    .expect("second approval manager should load");

    assert!(!Arc::ptr_eq(&first, &second));
    clear_approval_manager_for_directory(&first_directory);
    clear_approval_manager_for_directory(&second_directory);
}

#[tokio::test]
async fn clear_approval_manager_removes_existing_directory_entry() {
    let temp = tempfile::tempdir().expect("tempdir");
    let directory = temp.path().to_string_lossy().to_string();
    clear_approval_manager_for_directory(&directory);

    let first = project::instance::provide(temp.path(), None, || {
        Box::pin(async { Ok::<_, project::Error>(approval_manager_for_current_instance().await) })
    })
    .await
    .expect("instance context should be provided")
    .expect("first approval manager should load");
    clear_approval_manager_for_directory(&directory);
    let second = project::instance::provide(temp.path(), None, || {
        Box::pin(async { Ok::<_, project::Error>(approval_manager_for_current_instance().await) })
    })
    .await
    .expect("instance context should be provided")
    .expect("second approval manager should load");

    assert!(!Arc::ptr_eq(&first, &second));
    clear_approval_manager_for_directory(&directory);
}
