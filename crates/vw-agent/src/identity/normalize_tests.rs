use super::*;
use serde_json::json;

#[test]
fn normalize_builds_full_name_from_first_and_last() {
    let payload = json!({"identity": {"names": {"first": "Ada", "last": "Lovelace"}}});
    let normalized = normalize_aieos_identity(&payload);
    let names = normalized.identity.unwrap().names.unwrap();
    assert_eq!(names.full.as_deref(), Some("Ada Lovelace"));
}

#[test]
fn normalize_ignores_empty_sections_and_dedupes_lists() {
    let payload = json!({"linguistics": {}, "psychology": {"moral_compass": ["care", "", "care"]}});
    let normalized = normalize_aieos_identity(&payload);
    assert!(normalized.linguistics.is_none());
    assert_eq!(normalized.psychology.unwrap().moral_compass.unwrap(), vec!["care"]);
}
