//! 验证 vwacp 版本解析的优先级与缺省行为。
//!
//! 版本可来自 npm 环境变量或显式 package.json；当来源不可用时应返回稳定的
//! UNKNOWN_VERSION，而不是让调用方处理文件系统错误。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use vw_acp::{ResolveAcpxVersionParams, UNKNOWN_VERSION, resolve_vwacp_version};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 生成版本解析测试使用的临时目录。
///
/// 返回值用于创建隔离的 package.json，调用方负责清理目录。
fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let pid = std::process::id();
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("vw-acp-version-{pid}-{nanos}-{counter}"))
}

#[test]
fn resolve_vwacp_version_prefers_matching_env_version() {
    let mut env = HashMap::new();
    env.insert("npm_package_name".to_string(), "vwacp".to_string());
    env.insert("npm_package_version".to_string(), "1.2.3".to_string());

    let version = resolve_vwacp_version(Some(ResolveAcpxVersionParams {
        env: Some(&env),
        package_json_path: None,
    }));

    assert_eq!(version, "1.2.3");
}

#[test]
fn resolve_vwacp_version_reads_explicit_package_json_path() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let package_json_path = dir.join("package.json");
    fs::write(&package_json_path, r#"{"name":"vwacp","version":" 2.3.4 "}"#).unwrap();

    let version = resolve_vwacp_version(Some(ResolveAcpxVersionParams {
        env: None,
        package_json_path: Some(&package_json_path),
    }));

    assert_eq!(version, "2.3.4");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn resolve_vwacp_version_returns_unknown_for_missing_package_json() {
    let dir = unique_temp_dir();
    fs::create_dir_all(&dir).unwrap();
    let missing_path = dir.join("missing-package.json");

    let version = resolve_vwacp_version(Some(ResolveAcpxVersionParams {
        env: None,
        package_json_path: Some(&missing_path),
    }));

    assert_eq!(version, UNKNOWN_VERSION);

    let _ = fs::remove_dir_all(dir);
}
