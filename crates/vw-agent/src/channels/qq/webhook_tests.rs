use super::*;

#[test]
fn qq_seed_from_secret_repeats_secret_bytes_to_seed_length() {
    let seed = qq_seed_from_secret("ab").expect("seed");

    assert_eq!(seed[0], b'a');
    assert_eq!(seed[1], b'b');
    assert_eq!(seed[2], b'a');
    assert!(qq_seed_from_secret("").is_none());
}

#[test]
fn qq_webhook_validation_signature_is_deterministic() {
    let first = qq_webhook_validation_signature("secret", "123", "plain").expect("signature");
    let second = qq_webhook_validation_signature("secret", "123", "plain").expect("signature");

    assert_eq!(first, second);
    assert_eq!(first.len(), 128);
}
