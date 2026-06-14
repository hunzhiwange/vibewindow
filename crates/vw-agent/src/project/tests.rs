use super::*;
use serde_json::json;

#[test]
fn extra_builds_owned_json_map() {
    let map = extra([("name", json!("demo")), ("ok", json!(true))]);
    assert_eq!(map.get("name"), Some(&json!("demo")));
    assert_eq!(map.get("ok"), Some(&json!(true)));
}

#[test]
fn project_error_display_delegates_to_inner_errors() {
    let io = Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "io boom"));
    assert_eq!(io.to_string(), "io boom");

    let json_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    assert!(Error::Json(json_err).to_string().contains("EOF"));
}

#[tokio::test]
async fn instance_bootstrap_accepts_existing_directory() {
    let dir = tempfile::TempDir::new().expect("tempdir should create");
    instance_bootstrap(dir.path()).await;
}

#[test]
fn project_event_type_is_stable_and_extra_keeps_values() {
    assert_eq!(event::UPDATED.r#type, "project.updated");
    let map = extra([("count", json!(2)), ("name", json!("demo"))]);
    assert_eq!(map.get("count"), Some(&json!(2)));
    assert_eq!(map.get("name"), Some(&json!("demo")));
}
