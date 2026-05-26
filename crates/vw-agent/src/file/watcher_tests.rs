use super::event::UPDATED;
use super::UpdatedProperties;

#[test]
fn watcher_event_contract_is_stable() {
    let props = UpdatedProperties { file: "src/lib.rs".to_string(), event: "change".to_string() };
    let json = serde_json::to_value(props).expect("serialize");

    assert_eq!(UPDATED.r#type, "file.watcher.updated");
    assert_eq!(json["event"], "change");
}
