//! Provider 配置类型的兼容导出层。
//!
//! 具体的 provider 配置结构定义在 `vw_config_types` 中维护；本模块在
//! `vw-agent` 的配置 schema 命名空间下重新导出这些类型，避免调用方直接依赖
//! 底层 crate 的路径。

pub use vw_config_types::provider::*;
