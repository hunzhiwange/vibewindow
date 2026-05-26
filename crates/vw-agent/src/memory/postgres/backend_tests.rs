use super::super::{quote_identifier, validate_identifier};

#[test]
fn validates_schema_and_table_identifiers_before_connecting() {
    assert!(validate_identifier("agent_memory", "schema").is_ok());
    assert!(validate_identifier("1bad", "schema").is_err());
    assert!(validate_identifier("bad-name", "table").is_err());
    assert_eq!(quote_identifier("memories"), "\"memories\"");
}

