use super::*;

#[test]
fn ensure_https_accepts_only_https_urls() {
    assert!(ensure_https("https://api.sgroup.qq.com").is_ok());
    assert!(ensure_https("http://api.sgroup.qq.com").is_err());
    assert!(ensure_https("file:///tmp/token").is_err());
}
