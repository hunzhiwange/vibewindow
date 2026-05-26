use super::*;

#[test]
fn detector_allows_plain_text_and_flags_api_key_shape() {
    let detector = LeakDetector::default();
    assert!(matches!(detector.scan("plain operational message"), LeakResult::Clean));
    assert!(matches!(detector.scan("OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz123456"), LeakResult::Detected { .. }));
}
