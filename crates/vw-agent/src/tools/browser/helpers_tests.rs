use super::*;

#[test]
fn domain_helpers_normalize_and_match_allowlist() {
    let allowed = normalize_domains(vec![
        " Example.COM ".to_string(),
        "*.trusted.test".to_string(),
        " ".to_string(),
    ]);
    assert_eq!(allowed, vec!["example.com", "*.trusted.test"]);
    assert!(host_matches_allowlist("www.example.com", &allowed));
    assert!(host_matches_allowlist("api.trusted.test", &allowed));
    assert!(!host_matches_allowlist("example.net", &allowed));
}

#[test]
fn host_extraction_and_private_detection_are_conservative() {
    assert_eq!(extract_host("https://Example.COM:443/path").unwrap(), "example.com");
    assert_eq!(extract_host("http://[::1]:8080/").unwrap(), "[::1]");
    assert!(is_private_host("localhost"));
    assert!(is_private_host("192.168.1.1"));
    assert!(!is_private_host("example.com"));
}
