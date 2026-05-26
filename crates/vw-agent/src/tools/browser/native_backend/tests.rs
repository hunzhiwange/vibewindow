use super::*;

#[test]
fn invalid_webdriver_endpoint_is_not_available() {
    assert!(!NativeBrowserState::is_available(true, "not a url", None));
}
