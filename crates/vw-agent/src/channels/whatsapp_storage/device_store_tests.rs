#[cfg(feature = "whatsapp-web")]
use super::RusqliteStore;

#[cfg(feature = "whatsapp-web")]
fn test_store() -> (tempfile::TempDir, RusqliteStore) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("session.db");
    let store = RusqliteStore::new(&path).unwrap();
    (dir, store)
}

#[test]
#[cfg(not(feature = "whatsapp-web"))]
fn device_store_is_disabled_without_whatsapp_web_feature() {
    assert!(!cfg!(feature = "whatsapp-web"));
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn create_exists_save_and_load_device() {
    use wa_rs_binary::jid::Jid;
    use wa_rs_core::store::Device as CoreDevice;
    use wa_rs_core::store::traits::DeviceStore as DeviceStoreTrait;

    let (_dir, store) = test_store();
    assert_eq!(DeviceStoreTrait::create(&store).await.unwrap(), 1);
    assert!(!DeviceStoreTrait::exists(&store).await.unwrap());
    assert!(DeviceStoreTrait::load(&store).await.unwrap().is_none());

    let mut device = CoreDevice::new();
    device.pn = Some(Jid::pn("15550000001"));
    device.lid = Some(Jid::lid("100000000000001"));
    device.registration_id = 12345;
    device.push_name = "VibeWindow".to_string();
    device.edge_routing_info = Some(vec![1, 2, 3]);
    device.props_hash = Some("props-hash".to_string());

    DeviceStoreTrait::save(&store, &device).await.unwrap();
    assert!(DeviceStoreTrait::exists(&store).await.unwrap());

    let loaded = DeviceStoreTrait::load(&store).await.unwrap().unwrap();
    assert_eq!(loaded.pn.map(|jid| jid.to_string()), Some(Jid::pn("15550000001").to_string()));
    assert_eq!(
        loaded.lid.map(|jid| jid.to_string()),
        Some(Jid::lid("100000000000001").to_string())
    );
    assert_eq!(loaded.registration_id, 12345);
    assert_eq!(loaded.push_name, "VibeWindow");
    assert_eq!(loaded.edge_routing_info, Some(vec![1, 2, 3]));
    assert_eq!(loaded.props_hash.as_deref(), Some("props-hash"));
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn save_replaces_existing_device_row() {
    use wa_rs_core::store::Device as CoreDevice;
    use wa_rs_core::store::traits::DeviceStore as DeviceStoreTrait;

    let (_dir, store) = test_store();
    let mut first = CoreDevice::new();
    first.push_name = "First".to_string();
    DeviceStoreTrait::save(&store, &first).await.unwrap();

    let mut second = first.clone();
    second.push_name = "Second".to_string();
    second.registration_id = 999;
    DeviceStoreTrait::save(&store, &second).await.unwrap();

    let loaded = DeviceStoreTrait::load(&store).await.unwrap().unwrap();
    assert_eq!(loaded.push_name, "Second");
    assert_eq!(loaded.registration_id, 999);
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn snapshot_db_copies_database_and_optional_extra_content() {
    use wa_rs_core::store::Device as CoreDevice;
    use wa_rs_core::store::traits::DeviceStore as DeviceStoreTrait;

    let (dir, store) = test_store();
    let mut device = CoreDevice::new();
    device.push_name = "Snapshot".to_string();
    DeviceStoreTrait::save(&store, &device).await.unwrap();

    DeviceStoreTrait::snapshot_db(&store, "backup", Some(b"extra")).await.unwrap();

    let snapshot = dir.path().join("session.db.snapshot.backup");
    let extra = dir.path().join("session.db.snapshot.backup.extra");
    assert!(snapshot.exists());
    assert_eq!(std::fs::read(extra).unwrap(), b"extra");
}
