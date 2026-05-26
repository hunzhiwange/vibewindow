use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vw_shared::provider::types::Info;

use super::GatewayClient;
use crate::http::directory_query;

/// Provider 列表接口的聚合响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderListResponse {
    /// 当前可见的全部 provider 信息。
    pub all: Vec<Info>,
    /// 各能力对应的默认 provider 映射。
    pub default: HashMap<String, String>,
    /// 已完成连接校验的 provider 标识列表。
    pub connected: Vec<String>,
}

impl GatewayClient {
    /// 获取指定目录上下文下可用的 provider 列表。
    pub async fn provider_list(
        &self,
        directory: Option<&str>,
    ) -> Result<ProviderListResponse, String> {
        self.get_json("/v1/provider", &directory_query(directory)).await
    }
}

#[cfg(test)]
#[path = "provider_api_tests.rs"]
mod provider_api_tests;
