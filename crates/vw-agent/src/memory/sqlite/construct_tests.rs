use super::*;

#[test]
fn sqlite_memory_type_name_remains_stable() {
    assert!(std::any::type_name::<SqliteMemory>().contains("SqliteMemory"));
}

