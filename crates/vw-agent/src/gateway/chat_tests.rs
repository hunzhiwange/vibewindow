use super::*;

#[test]
fn sanitize_gateway_response_truncates_long_text() {
    let input = "x".repeat(20_000);
    let sanitized = sanitize_gateway_response(&input, &[]);

    assert!(sanitized.len() < input.len());
    assert!(sanitized.ends_with('…'));
}
