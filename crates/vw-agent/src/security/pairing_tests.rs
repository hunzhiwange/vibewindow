use super::*;
use vw_config_types::gateway::GatewaySkey;

#[test]
fn constant_time_eq_matches_equal_strings_only() {
    assert!(constant_time_eq("same", "same"));
    assert!(!constant_time_eq("same", "diff"));
}

#[test]
fn public_bind_detection_is_explicit() {
    assert!(is_public_bind("0.0.0.0"));
    assert!(!is_public_bind("127.0.0.1"));
}

#[test]
fn disabled_pairing_authenticates_without_code_or_token() {
    let guard = PairingGuard::new(false, &[]);

    assert!(!guard.require_pairing());
    assert!(guard.pairing_code().is_none());
    assert!(guard.ensure_pairing_code().is_none());
    assert!(guard.is_authenticated("anything"));
    assert!(!guard.is_paired());
}

#[tokio::test]
async fn successful_pairing_consumes_code_and_stores_token_hash() {
    let guard = PairingGuard::new(true, &[]);
    let code = guard.pairing_code().expect("pairing code");

    let token = guard
        .try_pair(&format!(" {code}\n"), " client-a ")
        .await
        .expect("not locked out")
        .expect("token");

    assert!(token.starts_with("zc_"));
    assert_eq!(token.len(), 67);
    assert!(guard.pairing_code().is_none());
    assert!(guard.is_paired());
    assert!(guard.is_authenticated(&token));
    assert!(!guard.is_authenticated("zc_wrong"));

    let tokens = guard.tokens();
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].chars().all(|ch| ch.is_ascii_hexdigit()));
    assert_eq!(tokens[0].len(), 64);
}

#[tokio::test]
async fn failed_attempts_lock_out_client_but_not_other_clients() {
    let guard = PairingGuard::new(true, &[]);
    let code = guard.pairing_code().expect("pairing code");

    for _ in 0..5 {
        assert!(guard.try_pair("000000", "client-a").await.unwrap().is_none());
    }

    let lockout = guard.try_pair(&code, "client-a").await.unwrap_err();
    assert!(lockout > 0);
    assert!(guard.try_pair(&code, "client-b").await.unwrap().is_some());
}

#[test]
fn existing_plain_tokens_are_hashed_and_existing_hashes_are_preserved() {
    let plain = "zc_existing_plain";
    let hashed = hash_token("zc_existing_hashed");
    let guard = PairingGuard::new(true, &[plain.to_string(), hashed.clone()]);

    assert!(guard.pairing_code().is_none());
    assert!(guard.is_authenticated(plain));
    assert!(guard.is_authenticated("zc_existing_hashed"));

    let tokens = guard.tokens();
    assert!(tokens.contains(&hash_token(plain)));
    assert!(tokens.contains(&hashed));
}

#[test]
fn configured_skeys_respect_enabled_flag_and_expiration() {
    let enabled = GatewaySkey {
        enabled: true,
        skey: Some("enabled-skey".to_string()),
        skey_hash: String::new(),
        masked_skey: String::new(),
        name: "enabled".to_string(),
        expires_at: None,
    };
    let disabled = GatewaySkey {
        enabled: false,
        skey: Some("disabled-skey".to_string()),
        skey_hash: String::new(),
        masked_skey: String::new(),
        name: "disabled".to_string(),
        expires_at: None,
    };
    let expired = GatewaySkey {
        enabled: true,
        skey: Some("expired-skey".to_string()),
        skey_hash: String::new(),
        masked_skey: String::new(),
        name: "expired".to_string(),
        expires_at: Some("2000-01-01T00:00:00Z".to_string()),
    };
    let guard = PairingGuard::from_skeys(true, &[enabled, disabled, expired]);

    assert_eq!(guard.active_skey_count(), 1);
    assert!(guard.is_authenticated("enabled-skey"));
    assert!(!guard.is_authenticated("disabled-skey"));
    assert!(!guard.is_authenticated("expired-skey"));
}

#[test]
fn skey_config_updates_apply_to_existing_guard() {
    let guard = PairingGuard::from_skeys(false, &[]);
    assert!(!guard.auth_enabled());
    assert!(guard.is_authenticated(""));

    let enabled = GatewaySkey {
        enabled: true,
        skey: Some("hot-skey".to_string()),
        skey_hash: String::new(),
        masked_skey: String::new(),
        name: "hot".to_string(),
        expires_at: None,
    };

    guard.update_from_skeys(true, &[enabled]);
    assert!(guard.auth_enabled());
    assert!(guard.is_authenticated("hot-skey"));
    assert!(!guard.is_authenticated("wrong-skey"));

    guard.update_from_skeys(false, &[]);
    assert!(!guard.auth_enabled());
    assert!(guard.is_authenticated(""));
}

#[test]
fn ensure_pairing_code_regenerates_for_paired_guard() {
    let guard = PairingGuard::new(true, &[hash_token("existing")]);

    assert!(guard.pairing_code().is_none());
    let code = guard.ensure_pairing_code().expect("bootstrap code");
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|ch| ch.is_ascii_digit()));
    assert_eq!(guard.ensure_pairing_code(), Some(code));
}

#[test]
fn helper_functions_cover_edge_cases() {
    assert_eq!(normalize_client_key("  "), "unknown");
    assert_eq!(normalize_client_key("  ip-1 "), "ip-1");

    let token = generate_token();
    assert!(token.starts_with("zc_"));
    assert_eq!(token.len(), 67);
    assert!(is_token_hash(&hash_token(&token)));
    assert!(!is_token_hash("not-a-hash"));

    let code = generate_code();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|ch| ch.is_ascii_digit()));

    assert!(!constant_time_eq("same", "same-but-longer"));
    assert!(!is_public_bind("localhost"));
    assert!(!is_public_bind("::1"));
    assert!(is_public_bind("192.168.1.10"));
}
