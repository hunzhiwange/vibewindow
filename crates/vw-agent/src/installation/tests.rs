use super::*;

#[test]
fn method_as_str_is_stable() {
    assert_eq!(Method::Npm.as_str(), "npm");
    assert_eq!(Method::Brew.as_str(), "brew");
    assert_eq!(Method::Curl.as_str(), "curl");
    assert_eq!(Method::Unknown.as_str(), "unknown");
}

#[test]
fn upgrade_failed_error_displays_stderr_only() {
    let err = UpgradeFailedError { stderr: "upgrade failed".into() };
    assert_eq!(err.to_string(), "upgrade failed");
}
