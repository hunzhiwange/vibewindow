use super::*;

#[test]
fn sqlite_memory_impl_module_is_linked() {
    assert!(std::any::type_name::<SqliteMemory>().contains("SqliteMemory"));
}
