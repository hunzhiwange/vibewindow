use super::*;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[test]
fn extra_builds_json_map() {
    let map = extra([("task", json!("tick"))]);
    assert_eq!(map.get("task"), Some(&json!("tick")));
}

fn counting_task(id: &str, counter: Arc<AtomicUsize>) -> Task {
    Task::new(
        id,
        Duration::from_secs(3600),
        Arc::new(move || {
            let counter = counter.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        }),
    )
}

async fn wait_for_counter(counter: &AtomicUsize, minimum: usize) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        if counter.load(Ordering::SeqCst) >= minimum {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("counter did not reach {minimum}");
}

#[tokio::test]
async fn instance_register_runs_task_and_reset_removes_entry() {
    let id = format!("instance-run-{}", uuid::Uuid::new_v4());
    let instance = format!("inst-{}", uuid::Uuid::new_v4());
    let counter = Arc::new(AtomicUsize::new(0));
    let mut task = counting_task(&id, counter.clone());
    task.scope = Scope::Instance(instance.clone());

    register(task);
    wait_for_counter(&counter, 1).await;
    assert!(INSTANCES.lock().unwrap().contains_key(&instance));

    reset_instance(&instance);
    assert!(!INSTANCES.lock().unwrap().contains_key(&instance));
}

#[tokio::test]
async fn instance_register_replaces_existing_timer_with_same_id() {
    let id = format!("replace-{}", uuid::Uuid::new_v4());
    let instance = format!("inst-{}", uuid::Uuid::new_v4());
    let first = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(AtomicUsize::new(0));

    let mut first_task = counting_task(&id, first.clone());
    first_task.scope = Scope::Instance(instance.clone());
    register(first_task);
    wait_for_counter(&first, 1).await;

    let mut second_task = counting_task(&id, second.clone());
    second_task.scope = Scope::Instance(instance.clone());
    register(second_task);
    wait_for_counter(&second, 1).await;

    let instances = INSTANCES.lock().unwrap();
    let entry = instances.get(&instance).expect("instance entry should exist");
    assert_eq!(entry.tasks.len(), 1);
    assert_eq!(entry.timers.len(), 1);
    drop(instances);
    reset_instance(&instance);
}

#[tokio::test]
async fn global_register_ignores_duplicate_task_id() {
    let id = format!("global-{}", uuid::Uuid::new_v4());
    let first = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(AtomicUsize::new(0));

    let mut first_task = counting_task(&id, first.clone());
    first_task.scope = Scope::Global;
    register(first_task);
    wait_for_counter(&first, 1).await;

    let mut second_task = counting_task(&id, second.clone());
    second_task.scope = Scope::Global;
    register(second_task);
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(second.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn run_inner_swallows_task_errors() {
    let run: RunFn = Arc::new(|| Box::pin(async { Err("boom".to_string()) }));
    run_inner("failing-task", run).await;
}
