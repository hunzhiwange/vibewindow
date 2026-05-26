use super::*;

#[test]
fn pairing_handlers_are_available() {
    let _ = handle_pair;
    let _ = handle_pair_code;
    let _ = persist_pairing_tokens;
}
