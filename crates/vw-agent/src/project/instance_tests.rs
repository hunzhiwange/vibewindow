use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

#[test]
fn contains_path_is_false_without_context() {
    assert!(!contains_path("/tmp/outside"));
}

#[tokio::test]
async fn provide_scopes_context_runs_init_once_and_disposes() {
    dispose_all().await;
    let temp = TempDir::new().expect("tempdir should create");
    let dir = temp.path().to_path_buf();
    let init_count = Arc::new(AtomicUsize::new(0));

    let init_count_first = init_count.clone();
    let observed = provide(
        &dir,
        Some(Box::new(move || {
            let init_count = init_count_first.clone();
            Box::pin(async move {
                init_count.fetch_add(1, Ordering::SeqCst);
            })
        })),
        {
            let dir = dir.clone();
            move || {
                Box::pin(async move {
                    assert_eq!(directory(), dir.to_string_lossy().to_string());
                    assert!(contains_path(dir.join("child.txt")));
                    assert!(project().is_some());
                    worktree()
                })
            }
        },
    )
    .await
    .expect("provide should succeed");
    assert!(!observed.is_empty());

    provide(
        &dir,
        Some(Box::new(|| {
            Box::pin(async {
                panic!("cached context should not run init again");
            })
        })),
        || Box::pin(async { assert!(project().is_some()) }),
    )
    .await
    .expect("cached provide should succeed");
    assert_eq!(init_count.load(Ordering::SeqCst), 1);

    dispose().await;
    assert!(directory().is_empty());
}
