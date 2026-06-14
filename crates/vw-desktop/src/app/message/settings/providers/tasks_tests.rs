use super::*;

#[test]
fn task_builders_are_callable() {
    let _ = refresh_task();
    let _ = sync_remote_task();
    let _ = load_catalog_task();
    let _ = save_popular_providers_task(vec!["OpenAI".to_string()]);
}
