use super::*;

#[test]
fn sqlite_memory_type_name_remains_stable() {
    assert!(std::any::type_name::<SqliteMemory>().contains("SqliteMemory"));
}

#[test]
fn sqlite_memory_uses_user_scoped_data_dir() {
    let workspace = tempfile::TempDir::new().unwrap();
    let storage = crate::app::agent::memory::paths::project_data_dir(workspace.path()).unwrap();

    let _memory = SqliteMemory::new(workspace.path()).unwrap();

    assert!(!workspace.path().join("memory").join("brain.db").exists());
    assert!(storage.join("memory").join("brain.db").exists());
}
