use super::helpers::{
    decrypt_optional_secret, decrypt_secret, encrypt_optional_secret, encrypt_secret,
    is_valid_env_var_name,
};
use crate::app::agent::security::SecretStore;

#[test]
fn env_var_names_follow_shell_safe_subset() {
    assert!(is_valid_env_var_name("API_KEY"));
    assert!(is_valid_env_var_name("_SECRET_2"));
    assert!(!is_valid_env_var_name("2FAST"));
    assert!(!is_valid_env_var_name("BAD-NAME"));
    assert!(!is_valid_env_var_name(""));
}

#[test]
fn disabled_secret_store_leaves_values_plaintext() {
    let tmp = tempfile::tempdir().unwrap();
    let store = SecretStore::new(tmp.path(), false);
    let mut required = "secret".to_string();
    let mut optional = Some("optional".to_string());

    encrypt_secret(&store, &mut required, "required").unwrap();
    encrypt_optional_secret(&store, &mut optional, "optional").unwrap();
    decrypt_secret(&store, &mut required, "required").unwrap();
    decrypt_optional_secret(&store, &mut optional, "optional").unwrap();

    assert_eq!(required, "secret");
    assert_eq!(optional.as_deref(), Some("optional"));
}
