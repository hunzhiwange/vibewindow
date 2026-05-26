use super::*;

#[test]
fn base64_decode_accepts_unpadded_discord_token_prefix() {
    assert_eq!(base64_decode("MTIzNDU2"), Some("123456".to_string()));
    assert_eq!(bot_user_id_from_token("MTIzNDU2.timestamp.signature"), Some("123456".to_string()));
}

#[test]
fn base64_decode_rejects_invalid_or_non_utf8_input() {
    assert_eq!(base64_decode("not valid!"), None);
    assert_eq!(base64_decode("//8="), None);
}
