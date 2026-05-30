use super::*;

#[test]
fn secret_scrubber_redacts_bearer_and_api_key_values() {
    let input = "Authorization: Bearer sk-secret-value api_key=abcd1234";
    let scrubbed = scrub_secret_patterns(input);
    assert!(!scrubbed.contains("sk-secret-value"));
    assert!(!scrubbed.contains("abcd1234"));
}

#[test]
fn moonshot_aliases_are_case_insensitive() {
    assert!(is_moonshot_alias("KIMI"));
    assert!(is_moonshot_alias("moonshot"));
    assert!(!is_moonshot_alias("openai"));
}
