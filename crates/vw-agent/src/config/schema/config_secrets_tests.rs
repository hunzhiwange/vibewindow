use super::config_secrets::{
    decrypt_map_secrets, decrypt_vec_secrets, encrypt_map_secrets, encrypt_vec_secrets,
};
use crate::app::agent::security::SecretStore;
use std::collections::HashMap;

#[test]
fn vec_and_map_secrets_noop_when_encryption_disabled() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SecretStore::new(tmp.path(), false);
    let mut vec_values = vec!["alpha".to_string()];
    let mut map_values = HashMap::from([("one".to_string(), "bravo".to_string())]);

    encrypt_vec_secrets(&store, &mut vec_values, "vec").unwrap();
    encrypt_map_secrets(&store, &mut map_values, "map").unwrap();
    decrypt_vec_secrets(&store, &mut vec_values[..], "vec").unwrap();
    decrypt_map_secrets(&store, &mut map_values, "map").unwrap();

    assert_eq!(vec_values, vec!["alpha".to_string()]);
    assert_eq!(map_values.get("one").map(String::as_str), Some("bravo"));
}
