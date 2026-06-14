use super::*;

#[test]
fn hex_decode_rejects_invalid_input() {
    assert_eq!(hex_decode("00ff").unwrap(), vec![0, 255]);
    assert!(hex_decode("not hex").is_err());
}

#[test]
fn encrypted_prefix_checks_are_distinct() {
    assert!(SecretStore::is_encrypted("encrypted:v1:data"));
    assert!(SecretStore::is_secure_encrypted("encrypted:v2:data"));
    assert!(!SecretStore::is_secure_encrypted("encrypted:v1:data"));
}

#[test]
fn disabled_store_and_empty_plaintext_bypass_encryption() {
    let dir = tempfile::tempdir().unwrap();
    let disabled = SecretStore::new(dir.path(), false);
    let enabled = SecretStore::new(dir.path(), true);

    assert_eq!(disabled.encrypt("secret").unwrap(), "secret");
    assert_eq!(enabled.encrypt("").unwrap(), "");
    assert_eq!(disabled.decrypt("plain").unwrap(), "plain");
    assert_eq!(disabled.decrypt_and_migrate("plain").unwrap(), ("plain".to_string(), None));
}

#[test]
fn chacha20_encryption_roundtrips_and_rejects_tampering() {
    let dir = tempfile::tempdir().unwrap();
    let store = SecretStore::new(dir.path(), true);

    let encrypted = store.encrypt("top-secret").unwrap();
    assert!(encrypted.starts_with("enc2:"));
    assert_ne!(encrypted, "top-secret");
    assert_eq!(store.decrypt(&encrypted).unwrap(), "top-secret");
    assert_eq!(store.decrypt_and_migrate(&encrypted).unwrap(), ("top-secret".to_string(), None));

    let mut tampered = encrypted.clone();
    tampered.pop();
    tampered.push(if encrypted.ends_with('0') { '1' } else { '0' });
    assert!(store.decrypt(&tampered).is_err());
}

#[test]
fn legacy_xor_secret_decrypts_and_migrates_to_enc2() {
    let dir = tempfile::tempdir().unwrap();
    let store = SecretStore::new(dir.path(), true);
    let _ = store.encrypt("warm-key").unwrap();
    let key = hex_decode(&std::fs::read_to_string(&store.key_path).unwrap()).unwrap();
    let legacy = format!("enc:{}", hex_encode(&xor_cipher(b"legacy-secret", &key)));

    assert!(SecretStore::needs_migration(&legacy));
    let (plaintext, migrated) = store.decrypt_and_migrate(&legacy).unwrap();

    assert_eq!(plaintext, "legacy-secret");
    let migrated = migrated.expect("legacy value should migrate");
    assert!(migrated.starts_with("enc2:"));
    assert_eq!(store.decrypt(&migrated).unwrap(), "legacy-secret");
}

#[test]
fn malformed_encrypted_values_return_errors() {
    let dir = tempfile::tempdir().unwrap();
    let store = SecretStore::new(dir.path(), true);

    assert!(store.decrypt("enc2:not-hex").unwrap_err().to_string().contains("corrupt hex"));
    assert!(store.decrypt("enc2:00").unwrap_err().to_string().contains("too short"));
    assert!(store.decrypt("enc:not-hex").unwrap_err().to_string().contains("legacy"));
}

#[test]
fn key_file_is_reused_and_corruption_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let store = SecretStore::new(dir.path(), true);
    let first = store.encrypt("first").unwrap();
    let second = SecretStore::new(dir.path(), true);

    assert_eq!(second.decrypt(&first).unwrap(), "first");

    std::fs::write(&store.key_path, "bad-key").unwrap();
    assert!(second.encrypt("fails").is_err());
}

#[test]
fn hex_and_xor_helpers_cover_edge_cases() {
    assert_eq!(hex_encode(&[0x00, 0xab, 0xff]), "00abff");
    assert!(hex_decode("abc").unwrap_err().to_string().contains("odd length"));
    assert_eq!(xor_cipher(b"data", b""), b"data");

    let encrypted = xor_cipher(b"data", b"k");
    assert_eq!(xor_cipher(&encrypted, b"k"), b"data");
    assert_eq!(build_windows_icacls_grant_arg("  Ada "), Some("Ada:F".to_string()));
    assert_eq!(build_windows_icacls_grant_arg("  "), None);
}
