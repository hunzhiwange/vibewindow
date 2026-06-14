use super::super::{quote_identifier, validate_identifier};
use super::*;

#[test]
fn validates_schema_and_table_identifiers_before_connecting() {
    assert!(validate_identifier("agent_memory", "schema").is_ok());
    assert!(validate_identifier("1bad", "schema").is_err());
    assert!(validate_identifier("bad-name", "table").is_err());
    assert_eq!(quote_identifier("memories"), "\"memories\"");
}

#[test]
fn connect_timeout_is_bounded_to_postgres_cap() {
    assert_eq!(PostgresMemory::bounded_connect_timeout(0), std::time::Duration::from_secs(0));
    assert_eq!(PostgresMemory::bounded_connect_timeout(42), std::time::Duration::from_secs(42));
    assert_eq!(
        PostgresMemory::bounded_connect_timeout(POSTGRES_CONNECT_TIMEOUT_CAP_SECS + 1),
        std::time::Duration::from_secs(POSTGRES_CONNECT_TIMEOUT_CAP_SECS)
    );
}

#[test]
fn schema_sql_uses_quoted_schema_and_table_everywhere() {
    let sql = PostgresMemory::schema_sql("\"agent\"", "\"agent\".\"memories\"");

    assert!(sql.contains("CREATE SCHEMA IF NOT EXISTS \"agent\""));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"agent\".\"memories\""));
    assert!(sql.contains("id TEXT PRIMARY KEY"));
    assert!(sql.contains("key TEXT UNIQUE NOT NULL"));
    assert!(sql.contains("session_id TEXT"));
    assert!(sql.contains("idx_memories_category ON \"agent\".\"memories\"(category)"));
    assert!(sql.contains("idx_memories_session_id ON \"agent\".\"memories\"(session_id)"));
    assert!(sql.contains("idx_memories_updated_at ON \"agent\".\"memories\"(updated_at DESC)"));
}

#[test]
fn initialize_client_reports_invalid_connection_url_without_connecting() {
    let error = match PostgresMemory::initialize_client(
        "postgres://[invalid-host/db".to_string(),
        Some(999),
        false,
        "\"agent\"".to_string(),
        "\"agent\".\"memories\"".to_string(),
    ) {
        Ok(_) => panic!("invalid connection URL should fail"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("invalid PostgreSQL connection URL"));
}
