//! Redis 相关 API。
//!
//! 本模块封装 `/v1/redis` 下的细粒度 REST 接口，覆盖：
//! - 设置读取与更新
//! - 单连接增删改查与激活
//! - 按连接测试
//! - Key 列表读取、内容分析与默认创建
//! - 历史分页查询
//! - 导入导出

use super::GatewayClient;
use vw_api_types::tool::{
    GatewayRedisCommandRequest, GatewayRedisCommandResponse, GatewayRedisConfigBundle,
    GatewayRedisConnectionConfig, GatewayRedisConnectionTestResponse,
    GatewayRedisConnectionUpsertBody, GatewayRedisDeleteResponse, GatewayRedisHistoryListQuery,
    GatewayRedisHistoryPage, GatewayRedisImportResponse, GatewayRedisKeyAnalysis,
    GatewayRedisKeyAnalysisRequest, GatewayRedisKeyCreateRequest, GatewayRedisKeyListQuery,
    GatewayRedisKeyPage, GatewayRedisRuntimeOverview, GatewayRedisSettings,
    GatewayRedisSettingsUpdateBody,
};

impl GatewayClient {
    /// 读取 Redis 设置。
    pub async fn redis_settings_get(&self) -> Result<GatewayRedisSettings, String> {
        self.get_json("/v1/redis/settings", &[]).await
    }

    /// 更新 Redis 设置。
    pub async fn redis_settings_put(
        &self,
        body: &GatewayRedisSettingsUpdateBody,
    ) -> Result<GatewayRedisSettings, String> {
        self.put_json("/v1/redis/settings", &[], body).await
    }

    /// 列出全部 Redis 连接。
    pub async fn redis_connections_list(
        &self,
    ) -> Result<Vec<GatewayRedisConnectionConfig>, String> {
        self.get_json("/v1/redis/connections", &[]).await
    }

    /// 读取单个 Redis 连接。
    pub async fn redis_connection_get(
        &self,
        connection_id: &str,
    ) -> Result<GatewayRedisConnectionConfig, String> {
        self.get_json(&format!("/v1/redis/connections/{connection_id}"), &[]).await
    }

    /// 创建 Redis 连接。
    pub async fn redis_connection_create(
        &self,
        body: &GatewayRedisConnectionUpsertBody,
    ) -> Result<GatewayRedisConnectionConfig, String> {
        self.post_json("/v1/redis/connections", &[], body).await
    }

    /// 更新 Redis 连接。
    pub async fn redis_connection_update(
        &self,
        connection_id: &str,
        body: &GatewayRedisConnectionUpsertBody,
    ) -> Result<GatewayRedisConnectionConfig, String> {
        self.put_json(&format!("/v1/redis/connections/{connection_id}"), &[], body).await
    }

    /// 删除 Redis 连接。
    pub async fn redis_connection_delete(
        &self,
        connection_id: &str,
    ) -> Result<GatewayRedisDeleteResponse, String> {
        self.delete_json(
            &format!("/v1/redis/connections/{connection_id}"),
            &[],
            &serde_json::json!({}),
        )
        .await
    }

    /// 激活 Redis 连接，并更新服务端最近使用时间。
    pub async fn redis_connection_activate(
        &self,
        connection_id: &str,
    ) -> Result<GatewayRedisSettings, String> {
        self.post_json(
            &format!("/v1/redis/connections/{connection_id}/activate"),
            &[],
            &serde_json::json!({}),
        )
        .await
    }

    /// 测试指定 Redis 连接。
    pub async fn redis_connection_test(
        &self,
        connection_id: &str,
    ) -> Result<GatewayRedisConnectionTestResponse, String> {
        self.post_json(
            &format!("/v1/redis/connections/{connection_id}/test"),
            &[],
            &serde_json::json!({}),
        )
        .await
    }

    /// 读取指定 Redis 连接的运行时概览。
    pub async fn redis_connection_overview(
        &self,
        connection_id: &str,
    ) -> Result<GatewayRedisRuntimeOverview, String> {
        self.get_json(&format!("/v1/redis/connections/{connection_id}/overview"), &[]).await
    }

    /// 读取指定 Redis 连接的键分页。
    pub async fn redis_connection_keys(
        &self,
        connection_id: &str,
        query: &GatewayRedisKeyListQuery,
    ) -> Result<GatewayRedisKeyPage, String> {
        let mut params = Vec::new();
        if let Some(cursor) = query.cursor {
            params.push(("cursor".to_string(), cursor.to_string()));
        }
        if let Some(count) = query.count {
            params.push(("count".to_string(), count.to_string()));
        }
        if let Some(pattern) = &query.pattern
            && !pattern.trim().is_empty()
        {
            params.push(("pattern".to_string(), pattern.clone()));
        }
        self.get_json(&format!("/v1/redis/connections/{connection_id}/keys"), &params).await
    }

    /// 分析指定 Redis Key 的类型、TTL、内存占用与值预览。
    pub async fn redis_connection_key_analyze(
        &self,
        connection_id: &str,
        body: &GatewayRedisKeyAnalysisRequest,
    ) -> Result<GatewayRedisKeyAnalysis, String> {
        self.post_json(&format!("/v1/redis/connections/{connection_id}/keys/analyze"), &[], body)
            .await
    }

    /// 以默认值初始化方式创建指定 Redis Key，并返回分析结果。
    pub async fn redis_connection_key_create(
        &self,
        connection_id: &str,
        body: &GatewayRedisKeyCreateRequest,
    ) -> Result<GatewayRedisKeyAnalysis, String> {
        self.post_json(&format!("/v1/redis/connections/{connection_id}/keys"), &[], body).await
    }

    /// 在指定 Redis 连接上执行命令。
    pub async fn redis_command_execute(
        &self,
        connection_id: &str,
        body: &GatewayRedisCommandRequest,
    ) -> Result<GatewayRedisCommandResponse, String> {
        self.post_json(&format!("/v1/redis/connections/{connection_id}/command"), &[], body).await
    }

    /// 读取 Redis 历史分页。
    pub async fn redis_history_list(
        &self,
        query: &GatewayRedisHistoryListQuery,
    ) -> Result<GatewayRedisHistoryPage, String> {
        let mut params = Vec::new();
        if let Some(offset) = query.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = query.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(connection_id) = &query.connection_id
            && !connection_id.trim().is_empty()
        {
            params.push(("connection_id".to_string(), connection_id.clone()));
        }
        if let Some(text) = &query.query
            && !text.trim().is_empty()
        {
            params.push(("query".to_string(), text.clone()));
        }
        if let Some(only_write) = query.only_write {
            params.push(("only_write".to_string(), only_write.to_string()));
        }
        self.get_json("/v1/redis/history", &params).await
    }

    /// 导出 Redis 配置。
    pub async fn redis_export(&self) -> Result<GatewayRedisConfigBundle, String> {
        self.get_json("/v1/redis/export", &[]).await
    }

    /// 导入 Redis 配置。
    pub async fn redis_import(
        &self,
        bundle: &GatewayRedisConfigBundle,
    ) -> Result<GatewayRedisImportResponse, String> {
        self.post_json("/v1/redis/import", &[], bundle).await
    }
}

#[cfg(test)]
#[path = "redis_api_tests.rs"]
mod redis_api_tests;
