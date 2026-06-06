//! Desktop cleaner API DTOs.
//!
//! These types describe the protocol between the desktop UI and the gateway.
//! They intentionally contain no filesystem or command execution behavior.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerScanReport {
    pub total_bytes: u64,
    pub matched_items: usize,
    pub groups: Vec<CleanerScanGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerScanGroup {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub total_bytes: u64,
    pub items: Vec<CleanerScanItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerScanItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub sensitive: bool,
    pub total_bytes: u64,
    pub details: Vec<CleanerScanDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerScanDetail {
    pub label: String,
    pub path: String,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerCleanupRequest {
    #[serde(default)]
    pub clear_system_temp: bool,
    #[serde(default)]
    pub clear_app_cache: bool,
    #[serde(default)]
    pub clear_logs: bool,
    #[serde(default)]
    pub clear_package_cache: bool,
    #[serde(default)]
    pub clear_downloads: bool,
    #[serde(default)]
    pub empty_trash: bool,
    #[serde(default)]
    pub clear_installers: bool,
    #[serde(default)]
    pub clear_other_apps: bool,
    #[serde(default)]
    pub clear_wechat_work: bool,
    #[serde(default)]
    pub clear_wechat: bool,
    #[serde(default)]
    pub clear_qq: bool,
    #[serde(default)]
    pub clear_dingtalk: bool,
    #[serde(default)]
    pub clear_feishu: bool,
    #[serde(default)]
    pub clear_safari: bool,
    #[serde(default)]
    pub clear_chrome: bool,
    #[serde(default)]
    pub clear_edge: bool,
    #[serde(default)]
    pub clear_firefox: bool,
    #[serde(default)]
    pub clear_mail: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerInfoResponse {
    pub platform: String,
    pub supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerRunResponse {
    pub output: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanerStatusResponse {
    pub running: bool,
    pub output: String,
}
