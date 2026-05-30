use super::*;
use crate::memory::traits::MemoryCategory;

#[test]
fn postgres_category_round_trips_known_and_custom_values() {
    assert_eq!(PostgresMemory::category_to_str(&MemoryCategory::Core), "core");
    assert_eq!(PostgresMemory::parse_category("daily"), MemoryCategory::Daily);
    assert_eq!(PostgresMemory::parse_category("archive"), MemoryCategory::Custom("archive".into()));
}

#[test]
fn identifier_validation_rejects_injection_shapes() {
    assert!(validate_identifier("valid_name_1", "field").is_ok());
    assert!(validate_identifier("", "field").is_err());
    assert!(validate_identifier("name;drop", "field").is_err());
}
