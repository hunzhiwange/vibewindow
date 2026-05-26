use super::*;

#[test]
fn mask_and_restore_optional_secret_round_trips_masked_value() {
    let mut value = Some("secret".to_string());
    mask_optional_secret(&mut value);
    assert_eq!(value.as_deref(), Some(MASKED_SECRET));

    restore_optional_secret(&mut value, &Some("secret".to_string()));
    assert_eq!(value.as_deref(), Some("secret"));
}

#[test]
fn mask_vec_secrets_preserves_empty_values() {
    let mut values = vec!["one".to_string(), String::new()];
    mask_vec_secrets(&mut values);

    assert_eq!(values, vec![MASKED_SECRET.to_string(), String::new()]);
}
