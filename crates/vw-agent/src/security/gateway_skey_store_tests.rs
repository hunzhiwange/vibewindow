use super::*;

#[test]
fn load_existing_gateway_skeys_returns_none_when_db_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("gateway").join("skeys.sqlite");

    let loaded = load_existing_gateway_skeys_from_path(&db_path).unwrap();

    assert!(loaded.is_none());
}

#[test]
fn gateway_skey_db_path_for_config_dir_uses_vibewindow_gateway_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = gateway_skey_db_path_for_config_dir(dir.path());

    assert_eq!(db_path, dir.path().join("gateway").join("skeys.sqlite"));
}

#[test]
fn load_existing_gateway_skeys_falls_back_to_legacy_path_when_primary_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let primary_path = dir.path().join("primary").join("gateway").join("skeys.sqlite");
    let legacy_path = dir.path().join("legacy").join("gateway").join("skeys.sqlite");
    let skeys = vec![GatewaySkey {
        enabled: true,
        skey: None,
        skey_hash: "c".repeat(64),
        masked_skey: "sk-cccc***************ddddddddd".to_string(),
        name: "legacy".to_string(),
        expires_at: None,
    }];

    save_gateway_skeys_to_path(&legacy_path, &skeys).unwrap();
    let loaded =
        load_existing_gateway_skeys_from_paths(&primary_path, &legacy_path).unwrap().unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "legacy");
    assert_eq!(loaded[0].masked_skey, "sk-cccc***************ddddddddd");
}

#[test]
fn gateway_skey_store_roundtrips_metadata_without_raw_skeys() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("gateway").join("skeys.sqlite");
    let skeys = vec![
        GatewaySkey {
            enabled: true,
            skey: Some("sk-raw-never-stored".to_string()),
            skey_hash: "a".repeat(64),
            masked_skey: "sk-raw-never-st***************er-stored".to_string(),
            name: "desktop".to_string(),
            expires_at: Some("2026-12-31T23:59:59Z".to_string()),
        },
        GatewaySkey {
            enabled: false,
            skey: None,
            skey_hash: "b".repeat(64),
            masked_skey: "sk-ci***************tail".to_string(),
            name: "ci".to_string(),
            expires_at: None,
        },
        GatewaySkey {
            enabled: true,
            skey: None,
            skey_hash: String::new(),
            masked_skey: String::new(),
            name: "ignored".to_string(),
            expires_at: None,
        },
    ];

    save_gateway_skeys_to_path(&db_path, &skeys).unwrap();
    let loaded = load_existing_gateway_skeys_from_path(&db_path).unwrap().unwrap();

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "desktop");
    assert_eq!(loaded[0].masked_skey, "sk-raw-never-st***************er-stored");
    assert!(loaded[0].enabled);
    assert!(loaded[0].skey.is_none());
    assert_eq!(loaded[0].expires_at.as_deref(), Some("2026-12-31T23:59:59Z"));
    assert_eq!(loaded[1].name, "ci");
    assert_eq!(loaded[1].masked_skey, "sk-ci***************tail");
    assert!(!loaded[1].enabled);
    assert!(loaded[1].skey.is_none());
}

#[test]
fn gateway_skey_store_hashes_raw_skeys_before_persisting() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("gateway").join("skeys.sqlite");
    let raw_skey = "sk-1234567890abcdef1234567890abcdef111111111";
    let skeys = vec![GatewaySkey {
        enabled: true,
        skey: Some(raw_skey.to_string()),
        skey_hash: String::new(),
        masked_skey: String::new(),
        name: "raw-only".to_string(),
        expires_at: None,
    }];

    save_gateway_skeys_to_path(&db_path, &skeys).unwrap();
    let loaded = load_existing_gateway_skeys_from_path(&db_path).unwrap().unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].skey_hash, hash_skey(raw_skey));
    assert_eq!(loaded[0].masked_skey, "sk-1234567890abc***************111111111");
    assert_eq!(loaded[0].name, "raw-only");
    assert!(loaded[0].skey.is_none());
}
