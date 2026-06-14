//! AI-DATA 持久化辅助函数。
//!
//! 该模块把 settings、connections、reports 映射到稳定的 storage key，并提供
//! 排序与时间戳工具，供 HTTP handler 保持一致的数据读写行为。

use vw_api_types::data::{AiDataConnectionDto, AiDataReportDto, AiDataSettings};

use crate::app::agent::gateway::ApiError;
use crate::storage;

/// 读取 AI-DATA 设置。
///
/// # 返回值
///
/// 读取失败时返回默认设置，避免配置缺失阻断页面初始化。
pub(super) async fn load_settings() -> AiDataSettings {
    storage::read(&["ai_data", "settings"]).await.unwrap_or_default()
}

/// 保存 AI-DATA 设置。
///
/// # 错误
///
/// 当底层 storage 写入失败时返回内部错误。
pub(super) async fn save_settings(settings: &AiDataSettings) -> Result<(), ApiError> {
    storage::write(&["ai_data", "settings"], settings)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// 读取全部 AI-DATA 连接。
pub(super) async fn load_connections() -> Vec<AiDataConnectionDto> {
    storage::read(&["ai_data", "connections"]).await.unwrap_or_default()
}

/// 保存全部 AI-DATA 连接。
///
/// # 错误
///
/// 当底层 storage 写入失败时返回内部错误。
pub(super) async fn save_connections(connections: &[AiDataConnectionDto]) -> Result<(), ApiError> {
    let owned = connections.to_vec();
    storage::write(&["ai_data", "connections"], &owned)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// 读取全部 AI-DATA 报表。
pub(super) async fn load_reports() -> Vec<AiDataReportDto> {
    storage::read(&["ai_data", "reports"]).await.unwrap_or_default()
}

/// 保存全部 AI-DATA 报表。
///
/// # 错误
///
/// 当底层 storage 写入失败时返回内部错误。
pub(super) async fn save_reports(reports: &[AiDataReportDto]) -> Result<(), ApiError> {
    let owned = reports.to_vec();
    storage::write(&["ai_data", "reports"], &owned)
        .await
        .map_err(|error| ApiError::internal(error.to_string()))
}

/// 按最近使用或更新时间倒序排列连接。
pub(super) fn sort_connections(connections: &mut [AiDataConnectionDto]) {
    connections.sort_by(|left, right| {
        let left_key = left.last_used_ms.unwrap_or(left.updated_at_ms);
        let right_key = right.last_used_ms.unwrap_or(right.updated_at_ms);
        right_key.cmp(&left_key).then_with(|| left.name.cmp(&right.name))
    });
}

/// 按更新时间倒序排列报表。
pub(super) fn sort_reports(reports: &mut [AiDataReportDto]) {
    reports.sort_by(|left, right| {
        right.updated_at_ms.cmp(&left.updated_at_ms).then_with(|| left.name.cmp(&right.name))
    });
}

/// 返回当前 Unix 毫秒时间戳。
///
/// # 返回值
///
/// 系统时间异常或数值溢出时返回安全的兜底值。
pub(super) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
#[path = "data_storage_support_tests.rs"]
mod data_storage_support_tests;
