//! 版本字符串的解析与查询辅助。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;

pub const UNKNOWN_VERSION: &str = "0.0.0-unknown";

const MODULE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src");

static CACHED_VERSION: OnceLock<String> = OnceLock::new();

#[cfg(test)]
#[path = "version_tests.rs"]
mod version_tests;

#[derive(Debug, Clone, Default)]
pub struct ResolveAcpxVersionParams<'a> {
    pub env: Option<&'a HashMap<String, String>>,
    pub package_json_path: Option<&'a Path>,
}

fn parse_version(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_string())
}

fn read_package_version(package_json_path: &Path) -> Option<String> {
    let content = fs::read_to_string(package_json_path).ok()?;
    let parsed = serde_json::from_str::<Value>(&content).ok()?;
    parse_version(parsed.get("version").and_then(Value::as_str))
}

fn resolve_version_from_ancestors(start_dir: &Path) -> Option<String> {
    let mut current = Some(start_dir);
    while let Some(dir) = current {
        let package_json_path = dir.join("package.json");
        if let Some(version) = read_package_version(&package_json_path) {
            return Some(version);
        }
        current = dir.parent();
    }
    None
}

fn env_version(env: Option<&HashMap<String, String>>) -> Option<String> {
    match env {
        Some(env) => {
            let package_name = parse_version(env.get("npm_package_name").map(String::as_str))?;
            let version = parse_version(env.get("npm_package_version").map(String::as_str))?;
            if package_name == "vwacp" || package_name == env!("CARGO_PKG_NAME") {
                return Some(version);
            }
            None
        }
        None => {
            let package_name = parse_version(std::env::var("npm_package_name").ok().as_deref())?;
            let version = parse_version(std::env::var("npm_package_version").ok().as_deref())?;
            if package_name == "vwacp" || package_name == env!("CARGO_PKG_NAME") {
                return Some(version);
            }
            None
        }
    }
}

pub fn resolve_vwacp_version(params: Option<ResolveAcpxVersionParams<'_>>) -> String {
    let params = params.unwrap_or_default();

    if let Some(version) = env_version(params.env) {
        return version;
    }

    if let Some(package_json_path) = params.package_json_path {
        return read_package_version(package_json_path)
            .unwrap_or_else(|| UNKNOWN_VERSION.to_string());
    }

    resolve_version_from_ancestors(&PathBuf::from(MODULE_DIR))
        .unwrap_or_else(|| UNKNOWN_VERSION.to_string())
}

pub fn get_vwacp_version() -> String {
    CACHED_VERSION.get_or_init(|| resolve_vwacp_version(None)).clone()
}
