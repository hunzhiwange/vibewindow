use super::*;
use std::collections::HashMap;
use std::fs;

#[test]
fn env_version_accepts_vwacp_package_only() {
    let mut env = HashMap::new();
    env.insert("npm_package_name".to_string(), "vwacp".to_string());
    env.insert("npm_package_version".to_string(), "1.2.3".to_string());

    assert_eq!(
        resolve_vwacp_version(Some(ResolveAcpxVersionParams {
            env: Some(&env),
            package_json_path: None
        })),
        "1.2.3"
    );

    env.insert("npm_package_name".to_string(), "other".to_string());
    assert_eq!(env_version(Some(&env)), None);
}

#[test]
fn read_package_version_rejects_blank_or_missing_version() {
    let path = std::env::temp_dir().join(format!("vwacp-version-{}.json", std::process::id()));
    fs::write(&path, r#"{ "version": "  2.0.0-beta  " }"#).unwrap();
    assert_eq!(read_package_version(&path), Some("2.0.0-beta".to_string()));

    fs::write(&path, r#"{ "version": "   " }"#).unwrap();
    assert_eq!(read_package_version(&path), None);
    let _ = fs::remove_file(path);
}
