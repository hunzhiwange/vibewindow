use super::common::{build_launchd_env_vars, build_systemd_env_vars, xml_escape};

#[test]
fn xml_escape_replaces_reserved_xml_characters() {
    assert_eq!(
        xml_escape("a&b<c>d\"e'f"),
        "a&amp;b&lt;c&gt;d&quot;e&apos;f"
    );
}

#[test]
fn env_builders_return_strings_without_required_environment() {
    let launchd = build_launchd_env_vars();
    let systemd = build_systemd_env_vars();

    assert!(launchd.is_empty() || launchd.contains("EnvironmentVariables"));
    assert!(systemd.is_empty() || systemd.contains("Environment="));
}
