use super::*;

#[test]
fn postgres_memory_type_name_remains_stable() {
    assert!(std::any::type_name::<PostgresMemory>().contains("PostgresMemory"));
}
