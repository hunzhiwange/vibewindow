#[cfg(feature = "whatsapp-web")]
use super::RusqliteStore;

#[cfg(feature = "whatsapp-web")]
fn test_store() -> RusqliteStore {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    RusqliteStore::new(tmp.path()).unwrap()
}

#[test]
#[cfg(not(feature = "whatsapp-web"))]
fn protocol_store_is_disabled_without_whatsapp_web_feature() {
    assert!(!cfg!(feature = "whatsapp-web"));
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn skdm_recipients_are_inserted_deduplicated_and_cleared() {
    use wa_rs_binary::jid::Jid;
    use wa_rs_core::store::traits::ProtocolStore;

    let store = test_store();
    let device_a = Jid::pn_device("15550000001", 1);
    let device_b = Jid::pn_device("15550000002", 2);

    ProtocolStore::add_skdm_recipients(
        &store,
        "120363@g.us",
        &[device_a.clone(), device_a.clone(), device_b.clone()],
    )
    .await
    .unwrap();

    let mut loaded = ProtocolStore::get_skdm_recipients(&store, "120363@g.us")
        .await
        .unwrap()
        .into_iter()
        .map(|jid| jid.to_string())
        .collect::<Vec<_>>();
    loaded.sort();
    let mut expected = vec![device_a.to_string(), device_b.to_string()];
    expected.sort();
    assert_eq!(loaded, expected);

    ProtocolStore::clear_skdm_recipients(&store, "120363@g.us").await.unwrap();
    assert!(ProtocolStore::get_skdm_recipients(&store, "120363@g.us").await.unwrap().is_empty());
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn lid_mapping_round_trip_all_and_latest_phone_mapping() {
    use wa_rs_core::store::traits::{LidPnMappingEntry, ProtocolStore};

    let store = test_store();
    let older = LidPnMappingEntry {
        lid: "lid-old".to_string(),
        phone_number: "15550000001".to_string(),
        created_at: 1,
        updated_at: 10,
        learning_source: "peer".to_string(),
    };
    let newer = LidPnMappingEntry {
        lid: "lid-new".to_string(),
        phone_number: "15550000001".to_string(),
        created_at: 2,
        updated_at: 20,
        learning_source: "usync".to_string(),
    };

    ProtocolStore::put_lid_mapping(&store, &older).await.unwrap();
    ProtocolStore::put_lid_mapping(&store, &newer).await.unwrap();

    assert_eq!(
        ProtocolStore::get_lid_mapping(&store, "lid-old").await.unwrap().unwrap().learning_source,
        "peer"
    );
    assert_eq!(
        ProtocolStore::get_pn_mapping(&store, "15550000001").await.unwrap().unwrap().lid,
        "lid-new"
    );

    let all = ProtocolStore::get_all_lid_mappings(&store).await.unwrap();
    assert_eq!(all.len(), 2);
    assert!(ProtocolStore::get_lid_mapping(&store, "missing").await.unwrap().is_none());
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn base_key_collision_detection_compares_saved_key_and_deletes() {
    use wa_rs_core::store::traits::ProtocolStore;

    let store = test_store();
    assert!(!ProtocolStore::has_same_base_key(&store, "addr", "msg", &[1, 2]).await.unwrap());

    ProtocolStore::save_base_key(&store, "addr", "msg", &[1, 2]).await.unwrap();
    assert!(ProtocolStore::has_same_base_key(&store, "addr", "msg", &[1, 2]).await.unwrap());
    assert!(!ProtocolStore::has_same_base_key(&store, "addr", "msg", &[9, 9]).await.unwrap());

    ProtocolStore::delete_base_key(&store, "addr", "msg").await.unwrap();
    assert!(!ProtocolStore::has_same_base_key(&store, "addr", "msg", &[1, 2]).await.unwrap());
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn device_registry_round_trip_and_missing_user() {
    use wa_rs_core::store::traits::{DeviceInfo, DeviceListRecord, ProtocolStore};

    let store = test_store();
    let record = DeviceListRecord {
        user: "15550000001".to_string(),
        devices: vec![
            DeviceInfo { device_id: 0, key_index: None },
            DeviceInfo { device_id: 3, key_index: Some(99) },
        ],
        timestamp: 1234,
        phash: Some("hash".to_string()),
    };

    ProtocolStore::update_device_list(&store, record).await.unwrap();

    let loaded = ProtocolStore::get_devices(&store, "15550000001").await.unwrap().unwrap();
    assert_eq!(loaded.user, "15550000001");
    assert_eq!(loaded.devices.len(), 2);
    assert_eq!(loaded.devices[1].key_index, Some(99));
    assert_eq!(loaded.phash.as_deref(), Some("hash"));
    assert!(ProtocolStore::get_devices(&store, "missing").await.unwrap().is_none());
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn forget_marks_are_consumed_once() {
    use wa_rs_core::store::traits::ProtocolStore;

    let store = test_store();
    ProtocolStore::mark_forget_sender_key(&store, "group@g.us", "alice").await.unwrap();
    ProtocolStore::mark_forget_sender_key(&store, "group@g.us", "bob").await.unwrap();

    let mut marks = ProtocolStore::consume_forget_marks(&store, "group@g.us").await.unwrap();
    marks.sort();
    assert_eq!(marks, vec!["alice".to_string(), "bob".to_string()]);
    assert!(ProtocolStore::consume_forget_marks(&store, "group@g.us").await.unwrap().is_empty());
}

#[tokio::test]
#[cfg(feature = "whatsapp-web")]
async fn tc_tokens_round_trip_list_delete_and_expire() {
    use wa_rs_core::store::traits::{ProtocolStore, TcTokenEntry};

    let store = test_store();
    let expired =
        TcTokenEntry { token: vec![1, 2], token_timestamp: 10, sender_timestamp: Some(11) };
    let fresh = TcTokenEntry { token: vec![3, 4], token_timestamp: 100, sender_timestamp: None };

    ProtocolStore::put_tc_token(&store, "jid-old", &expired).await.unwrap();
    ProtocolStore::put_tc_token(&store, "jid-new", &fresh).await.unwrap();

    assert_eq!(
        ProtocolStore::get_tc_token(&store, "jid-old").await.unwrap().unwrap().sender_timestamp,
        Some(11)
    );
    let mut jids = ProtocolStore::get_all_tc_token_jids(&store).await.unwrap();
    jids.sort();
    assert_eq!(jids, vec!["jid-new".to_string(), "jid-old".to_string()]);

    assert_eq!(ProtocolStore::delete_expired_tc_tokens(&store, 50).await.unwrap(), 1);
    assert!(ProtocolStore::get_tc_token(&store, "jid-old").await.unwrap().is_none());
    assert!(ProtocolStore::get_tc_token(&store, "jid-new").await.unwrap().is_some());

    ProtocolStore::delete_tc_token(&store, "jid-new").await.unwrap();
    assert!(ProtocolStore::get_all_tc_token_jids(&store).await.unwrap().is_empty());
}
