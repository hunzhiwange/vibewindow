#[cfg(feature = "whatsapp-web")]
use super::RusqliteStore;

#[cfg(feature = "whatsapp-web")]
fn test_store() -> RusqliteStore {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    RusqliteStore::new(tmp.path()).unwrap()
}

#[test]
#[cfg(not(feature = "whatsapp-web"))]
fn signal_store_is_disabled_without_whatsapp_web_feature() {
    assert!(!cfg!(feature = "whatsapp-web"));
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn identity_session_and_sender_key_round_trips() {
    use wa_rs_core::store::traits::SignalStore;

    let store = test_store();

    SignalStore::put_identity(&store, "user@s.whatsapp.net", [7u8; 32]).await.unwrap();
    assert_eq!(
        SignalStore::load_identity(&store, "user@s.whatsapp.net").await.unwrap(),
        Some(vec![7u8; 32])
    );
    SignalStore::delete_identity(&store, "user@s.whatsapp.net").await.unwrap();
    assert!(SignalStore::load_identity(&store, "user@s.whatsapp.net").await.unwrap().is_none());

    SignalStore::put_session(&store, "user@s.whatsapp.net", &[1, 2, 3]).await.unwrap();
    assert!(SignalStore::has_session(&store, "user@s.whatsapp.net").await.unwrap());
    assert_eq!(
        SignalStore::get_session(&store, "user@s.whatsapp.net").await.unwrap(),
        Some(vec![1, 2, 3])
    );
    SignalStore::delete_session(&store, "user@s.whatsapp.net").await.unwrap();
    assert!(!SignalStore::has_session(&store, "user@s.whatsapp.net").await.unwrap());

    SignalStore::put_sender_key(&store, "group::sender", &[9, 8]).await.unwrap();
    assert_eq!(
        SignalStore::get_sender_key(&store, "group::sender").await.unwrap(),
        Some(vec![9, 8])
    );
    SignalStore::delete_sender_key(&store, "group::sender").await.unwrap();
    assert!(SignalStore::get_sender_key(&store, "group::sender").await.unwrap().is_none());
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn prekeys_and_signed_prekeys_round_trip_and_delete() {
    use wa_rs_core::store::traits::SignalStore;

    let store = test_store();

    SignalStore::store_prekey(&store, 10, &[1, 1, 2], false).await.unwrap();
    assert_eq!(SignalStore::load_prekey(&store, 10).await.unwrap(), Some(vec![1, 1, 2]));
    SignalStore::remove_prekey(&store, 10).await.unwrap();
    assert!(SignalStore::load_prekey(&store, 10).await.unwrap().is_none());

    SignalStore::store_signed_prekey(&store, 20, &[3, 5, 8]).await.unwrap();
    SignalStore::store_signed_prekey(&store, 21, &[13, 21]).await.unwrap();
    assert_eq!(SignalStore::load_signed_prekey(&store, 20).await.unwrap(), Some(vec![3, 5, 8]));

    let mut all = SignalStore::load_all_signed_prekeys(&store).await.unwrap();
    all.sort_by_key(|(id, _)| *id);
    assert_eq!(all, vec![(20, vec![3, 5, 8]), (21, vec![13, 21])]);

    SignalStore::remove_signed_prekey(&store, 20).await.unwrap();
    assert!(SignalStore::load_signed_prekey(&store, 20).await.unwrap().is_none());
}
