use super::*;

#[test]
fn sqlite_search_module_is_linked() {
    assert!(std::any::type_name::<SqliteMemory>().contains("SqliteMemory"));
}
