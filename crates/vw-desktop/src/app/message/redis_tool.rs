//! Redis 客户端工具消息处理模块。
//!
//! 本模块继续保留 `RedisToolMessage` 与 `update` 作为外部入口，
//! 但将实现按职责拆分为独立子文件：
//! - `message`：消息枚举定义
//! - `navigation`：连接切换、运行时加载、历史与通知
//! - `draft_inputs`：草稿表单字段与文件选择
//! - `operations`：保存、删除、测试、导入导出与命令执行
//! - `draft` / `gateway` / `helpers`：局部辅助逻辑

use crate::app::config::{
    REDIS_HISTORY_PAGE_SIZE, RedisToolGatewaySnapshot, load_redis_tool_snapshot_async,
    redis_command_execute_async, redis_connection_activate_async,
    redis_connection_key_analyze_async, redis_connection_key_create_async,
    redis_connection_create_async, redis_connection_delete_async,
    redis_connection_keys_async, redis_connection_overview_async, redis_connection_test_async,
    redis_connection_update_async, redis_settings_update_async,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::config::{redis_export_async, redis_import_async};
use crate::app::state::{
    RedisCommandOutputEntry, RedisConnectionDraft, RedisConnectionTab, RedisDetailTab,
    RedisKeyAnalysis, RedisKeyPage, RedisKeyValueKind, RedisRuntimeOverview,
};
use crate::app::{App, Message};
use iced::Task;
use vw_gateway_client::vw_api_types::tool::{
    GatewayRedisSentinelConfig, GatewayRedisSshTunnelConfig, GatewayRedisTlsCertConfig,
};
use vw_gateway_client::{
    GatewayRedisConnectionTestResponse, GatewayRedisConnectionUpsertBody,
    GatewayRedisHistoryListQuery,
};
#[cfg(not(target_arch = "wasm32"))]
use vw_gateway_client::GatewayRedisConfigBundle;

mod draft;
mod draft_inputs;
mod gateway;
mod helpers;
mod message;
mod navigation;
mod operations;
mod update;

pub use message::RedisToolMessage;
pub use update::update;

#[cfg(test)]
#[path = "redis_tool_tests.rs"]
mod redis_tool_tests;
