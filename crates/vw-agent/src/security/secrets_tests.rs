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
