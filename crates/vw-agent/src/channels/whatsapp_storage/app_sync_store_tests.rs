#[cfg(feature = "whatsapp-web")]
use super::RusqliteStore;

#[cfg(feature = "whatsapp-web")]
fn test_store() -> RusqliteStore {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    RusqliteStore::new(tmp.path()).unwrap()
}

#[test]
#[cfg(not(feature = "whatsapp-web"))]
fn app_sync_store_is_disabled_without_whatsapp_web_feature() {
    assert!(!cfg!(feature = "whatsapp-web"));
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn sync_key_round_trip_and_missing_key_returns_none() {
    use wa_rs_core::store::traits::{AppStateSyncKey, AppSyncStore};

    let store = test_store();
    assert!(AppSyncStore::get_sync_key(&store, b"missing").await.unwrap().is_none());

    let key =
        AppStateSyncKey { key_data: vec![1, 2, 3], fingerprint: vec![4, 5, 6], timestamp: 123 };
    AppSyncStore::set_sync_key(&store, b"key-1", key.clone()).await.unwrap();

    let loaded = AppSyncStore::get_sync_key(&store, b"key-1").await.unwrap().unwrap();
    assert_eq!(loaded.key_data, key.key_data);
    assert_eq!(loaded.fingerprint, key.fingerprint);
    assert_eq!(loaded.timestamp, key.timestamp);
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn version_round_trip_preserves_hash_state() {
    use std::collections::HashMap;
    use wa_rs_core::appstate::hash::HashState;
    use wa_rs_core::store::traits::AppSyncStore;

    let store = test_store();
    let mut index_value_map = HashMap::new();
    index_value_map.insert("idx".to_string(), vec![9, 8, 7]);
    let mut hash = [0u8; 128];
    hash[0] = 42;
    hash[127] = 24;
    let state = HashState { version: 77, hash, index_value_map };

    AppSyncStore::set_version(&store, "regular_high", state.clone()).await.unwrap();

    let loaded = AppSyncStore::get_version(&store, "regular_high").await.unwrap();
    assert_eq!(loaded.version, 77);
    assert_eq!(loaded.hash[0], 42);
    assert_eq!(loaded.hash[127], 24);
    assert_eq!(loaded.index_value_map.get("idx"), Some(&vec![9, 8, 7]));
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn mutation_macs_can_be_inserted_loaded_and_deleted() {
    use wa_rs_core::appstate::processor::AppStateMutationMAC;
    use wa_rs_core::store::traits::AppSyncStore;

    let store = test_store();
    let mutation = AppStateMutationMAC { index_mac: vec![1, 2], value_mac: vec![3, 4] };

    AppSyncStore::put_mutation_macs(&store, "critical_block", 5, &[mutation.clone()])
        .await
        .unwrap();

    let loaded = AppSyncStore::get_mutation_mac(&store, "critical_block", &mutation.index_mac)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded, serde_json::to_vec(&mutation.value_mac).unwrap());

    AppSyncStore::delete_mutation_macs(&store, "critical_block", &[mutation.index_mac])
        .await
        .unwrap();
    assert!(
        AppSyncStore::get_mutation_mac(&store, "critical_block", &[1, 2]).await.unwrap().is_none()
    );
}
