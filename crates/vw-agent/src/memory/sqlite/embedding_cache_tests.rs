use super::*;

#[test]
fn embedding_cache_module_is_linked() {
    assert!(std::any::type_name::<SqliteMemory>().contains("SqliteMemory"));
}
