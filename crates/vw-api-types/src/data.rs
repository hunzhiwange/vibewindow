//! AI-DATA 相关共享 DTO。
//!
//! 本模块用于表达数据连接、报表配置、执行请求与 AI 规划结果，
//! 供网关 API、桌面端与后续数据执行器共享统一协议。

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

fn default_ai_data_schema_version() -> u32 {
    1
}

fn default_ai_data_limit() -> u32 {
    100
}

fn default_ai_data_timeout_secs() -> u32 {
    30
}

fn default_http_method() -> String {
    "GET".to_string()
}

fn default_true() -> bool {
    true
}

fn deserialize_optional_search_fields<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => Ok(Some(
            items
                .into_iter()
                .map(|item| match item {
                    Value::String(text) => text,
                    other => other.to_string(),
                })
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        )),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.starts_with('[') {
                return serde_json::from_str::<Vec<String>>(trimmed)
                    .map(Some)
                    .map_err(serde::de::Error::custom);
            }
            Ok(Some(
                trimmed
                    .split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(ToOwned::to_owned)
                    .collect(),
            ))
        }
        Some(other) => Err(serde::de::Error::custom(format!(
            "invalid searchFields payload: {other}"
        ))),
    }
}

fn deserialize_optional_template_fields<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<AiDataTemplateFieldDto>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => serde_json::from_value(Value::Array(items))
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            serde_json::from_str::<Vec<AiDataTemplateFieldDto>>(trimmed)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        Some(other) => Err(serde::de::Error::custom(format!(
            "invalid templateFields payload: {other}"
        ))),
    }
}

fn deserialize_optional_transformer_fields<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<AiDataTransformerFieldDto>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => serde_json::from_value(Value::Array(items))
            .map(Some)
            .map_err(serde::de::Error::custom),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            serde_json::from_str::<Vec<AiDataTransformerFieldDto>>(trimmed)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        Some(other) => Err(serde::de::Error::custom(format!(
            "invalid transformerFields payload: {other}"
        ))),
    }
}

fn deserialize_optional_count_mode<'de, D>(
    deserializer: D,
) -> Result<Option<AiDataCountMode>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => match number.as_i64() {
            Some(0) => Ok(Some(AiDataCountMode::Disabled)),
            Some(1) => Ok(Some(AiDataCountMode::Enabled)),
            Some(2) => Ok(Some(AiDataCountMode::Only)),
            _ => Err(serde::de::Error::custom("invalid count mode number")),
        },
        Some(Value::String(text)) => match text.trim().to_ascii_lowercase().as_str() {
            "0" | "disabled" => Ok(Some(AiDataCountMode::Disabled)),
            "1" | "enabled" => Ok(Some(AiDataCountMode::Enabled)),
            "2" | "only" | "count_only" => Ok(Some(AiDataCountMode::Only)),
            other => Err(serde::de::Error::custom(format!("invalid count mode: {other}"))),
        },
        Some(other) => Err(serde::de::Error::custom(format!(
            "invalid count mode payload: {other}"
        ))),
    }
}

fn deserialize_optional_json_or_string<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            serde_json::from_str::<Value>(trimmed)
                .map(Some)
                .or_else(|_| Ok(Some(Value::String(text))))
        }
        Some(other) => Ok(Some(other)),
    }
}

/// AI-DATA 连接类型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiDataConnectionKind {
    Sqlite,
    Mysql,
    Postgres,
    Cube,
    Http,
}

/// AI-DATA 报表数据源模式，对齐 PHP 中 normal / ai 宽表的区分。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiDataSourceMode {
    Normal,
    Ai,
}

/// AI-DATA 查询执行类型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiDataQueryKind {
    Sql,
    Cube,
    Http,
}

/// 查询总数模式。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiDataCountMode {
    Disabled,
    Enabled,
    Only,
}

/// AI-DATA 全局设置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataSettings {
    #[serde(default = "default_ai_data_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_ai_data_limit")]
    pub default_limit: u32,
    #[serde(default = "default_ai_data_timeout_secs")]
    pub default_timeout_secs: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_connection_id: Option<String>,
}

impl Default for AiDataSettings {
    fn default() -> Self {
        Self {
            schema_version: default_ai_data_schema_version(),
            default_limit: default_ai_data_limit(),
            default_timeout_secs: default_ai_data_timeout_secs(),
            selected_connection_id: None,
        }
    }
}

/// AI-DATA 设置更新请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataSettingsUpdateBody {
    pub default_limit: u32,
    pub default_timeout_secs: u32,
}

/// 数据连接定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataConnectionDto {
    pub id: String,
    pub name: String,
    pub kind: AiDataConnectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sqlite_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_hint: Option<String>,
    #[serde(default)]
    pub updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_ms: Option<u64>,
}

/// 数据连接写入请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataConnectionUpsertBody {
    pub name: String,
    pub kind: AiDataConnectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sqlite_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_hint: Option<String>,
}

/// 连接测试结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataConnectionTestResponse {
    #[serde(default)]
    pub ok: bool,
    pub message: String,
    #[serde(default)]
    pub latency_ms: u64,
}

/// 连接目录/元数据响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataConnectionCatalogResponse {
    pub connection_id: String,
    pub kind: AiDataConnectionKind,
    pub catalog: Value,
}

/// 报表中的单个数据源绑定。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataReportSourceDto {
    pub source_key: String,
    pub connection_id: String,
    pub query_kind: AiDataQueryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count_sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cube_query: Option<Value>,
    #[serde(default = "default_http_method")]
    pub http_method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_body: Option<Value>,
    #[serde(default = "default_true")]
    pub append_pagination: bool,
}

/// 报表定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataReportDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub data_source: AiDataSourceMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_source_key: Option<String>,
    #[serde(default)]
    pub report_config: Value,
    #[serde(default)]
    pub sources: Vec<AiDataReportSourceDto>,
    #[serde(default)]
    pub updated_at_ms: u64,
}

/// 报表写入请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataReportUpsertBody {
    pub name: String,
    pub slug: String,
    pub data_source: AiDataSourceMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_source_key: Option<String>,
    #[serde(default)]
    pub report_config: Value,
    #[serde(default)]
    pub sources: Vec<AiDataReportSourceDto>,
}

/// 模板字段格式化定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataTemplateFieldDto {
    pub code: String,
    #[serde(rename = "valueType")]
    pub value_type: i64,
}

/// 转换字段定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataTransformerFieldDto {
    pub code: String,
    #[serde(rename = "transformerType")]
    pub transformer_type: String,
    #[serde(rename = "transformerArgs", default)]
    pub transformer_args: BTreeMap<String, Value>,
}

/// 分页信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataPageDto {
    pub per_page: u32,
    pub current_page: u32,
    pub total_page: u32,
    pub total_record: u64,
    pub from: u64,
    pub to: u64,
}

/// 统一查询请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AiDataQueryRequest {
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "report")]
    pub report_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "source")]
    pub source_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_kind: Option<AiDataQueryKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count_sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cube_query: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_body: Option<Value>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "pageSize")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_count_mode")]
    pub count: Option<AiDataCountMode>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "orderBy")]
    pub order_by: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "searchFields",
        deserialize_with = "deserialize_optional_search_fields"
    )]
    pub search_fields: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "templateFields",
        deserialize_with = "deserialize_optional_template_fields"
    )]
    pub template_fields: Option<Vec<AiDataTemplateFieldDto>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "transformerFields",
        deserialize_with = "deserialize_optional_transformer_fields"
    )]
    pub transformer_fields: Option<Vec<AiDataTransformerFieldDto>>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "templateCode")]
    pub template_code: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "searchCondition",
        deserialize_with = "deserialize_optional_json_or_string"
    )]
    pub search_condition: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
}

/// 查询响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataQueryResponse {
    pub page: AiDataPageDto,
    #[serde(default)]
    pub items: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_config: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_next_page: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug: Option<Value>,
}

/// AI 查询请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AiDataAiQueryRequest {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "report")]
    pub report_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "source")]
    pub source_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// AI 生成的执行计划。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataExecutionPlan {
    pub connection_id: String,
    pub query_kind: AiDataQueryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count_sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cube_query: Option<Value>,
    #[serde(default = "default_http_method")]
    pub http_method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_body: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

/// AI 查询响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiDataAiQueryResponse {
    pub plan: AiDataExecutionPlan,
    pub result: AiDataQueryResponse,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_model_response: Option<String>,
}