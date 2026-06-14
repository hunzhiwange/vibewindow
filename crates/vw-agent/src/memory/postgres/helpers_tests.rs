use super::*;
use crate::memory::traits::MemoryCategory;
use chrono::{DateTime, Utc};

#[test]
fn postgres_category_round_trips_known_and_custom_values() {
    assert_eq!(PostgresMemory::category_to_str(&MemoryCategory::Core), "core");
    assert_eq!(PostgresMemory::category_to_str(&MemoryCategory::Daily), "daily");
    assert_eq!(PostgresMemory::category_to_str(&MemoryCategory::Conversation), "conversation");
    assert_eq!(
        PostgresMemory::category_to_str(&MemoryCategory::Custom("archive".into())),
        "archive"
    );
    assert_eq!(PostgresMemory::parse_category("core"), MemoryCategory::Core);
    assert_eq!(PostgresMemory::parse_category("daily"), MemoryCategory::Daily);
    assert_eq!(PostgresMemory::parse_category("conversation"), MemoryCategory::Conversation);
    assert_eq!(PostgresMemory::parse_category("archive"), MemoryCategory::Custom("archive".into()));
}

#[test]
fn identifier_validation_rejects_injection_shapes() {
    assert!(validate_identifier("valid_name_1", "field").is_ok());
    assert!(validate_identifier("_starts_with_underscore", "field").is_ok());
    assert!(validate_identifier("", "field").is_err());
    assert!(validate_identifier("1name", "field").is_err());
    assert!(validate_identifier("name;drop", "field").is_err());
    assert!(validate_identifier("with-hyphen", "field").is_err());
    assert!(validate_identifier("contains space", "field").is_err());
}

#[test]
fn identifier_validation_errors_name_the_invalid_field() {
    let empty = validate_identifier("", "storage schema").unwrap_err().to_string();
    let bad_start = validate_identifier("9bad", "storage table").unwrap_err().to_string();
    let bad_char = validate_identifier("bad.table", "storage table").unwrap_err().to_string();

    assert!(empty.contains("storage schema must not be empty"));
    assert!(bad_start.contains("storage table must start with an ASCII letter or underscore"));
    assert!(bad_char.contains("storage table can only contain ASCII letters"));
}

#[test]
fn quote_identifier_wraps_without_mutating_contents() {
    assert_eq!(quote_identifier("memories"), "\"memories\"");
    assert_eq!(quote_identifier("_table_1"), "\"_table_1\"");
}

#[test]
fn entry_from_values_maps_database_values_to_memory_entry() {
    let timestamp: DateTime<Utc> =
        DateTime::parse_from_rfc3339("2026-06-13T08:09:10Z").unwrap().with_timezone(&Utc);

    let entry = PostgresMemory::entry_from_values(
        "id-1".to_string(),
        "key".to_string(),
        "content".to_string(),
        "daily".to_string(),
        timestamp,
        Some("session".to_string()),
        Some(0.75),
    );

    assert_eq!(entry.id, "id-1");
    assert_eq!(entry.key, "key");
    assert_eq!(entry.content, "content");
    assert_eq!(entry.category, MemoryCategory::Daily);
    assert_eq!(entry.timestamp, "2026-06-13T08:09:10+00:00");
    assert_eq!(entry.session_id.as_deref(), Some("session"));
    assert_eq!(entry.score, Some(0.75));
}

#[test]
fn entry_from_values_preserves_missing_session_and_score() {
    let timestamp: DateTime<Utc> =
        DateTime::parse_from_rfc3339("2026-06-13T08:09:10Z").unwrap().with_timezone(&Utc);

    let entry = PostgresMemory::entry_from_values(
        "id-2".to_string(),
        "key".to_string(),
        "content".to_string(),
        "custom".to_string(),
        timestamp,
        None,
        None,
    );

    assert_eq!(entry.category, MemoryCategory::Custom("custom".into()));
    assert!(entry.session_id.is_none());
    assert!(entry.score.is_none());
}
