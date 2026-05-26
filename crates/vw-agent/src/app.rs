//! 应用层兼容入口。
//!
//! 本文件把 crate 内部模块重新组织到 `crate::app` 命名空间下，供历史
//! 调用点、测试和桌面侧集成以稳定路径访问 agent 运行时能力。

/// Agent 运行时模块重导出。
///
/// 该模块不承载业务逻辑，只提供兼容路径，避免调用方直接依赖 crate
/// 根部模块布局。按平台条件隐藏不可用能力，特别是 wasm 目标下的守护
/// 进程、网关和 PTY 相关模块。
pub(crate) mod agent {
    pub use crate::agent;
    pub use crate::approval;
    pub use crate::auth;
    pub use crate::bus;
    pub use crate::channels;
    pub use crate::command;
    pub use crate::config;
    pub use crate::coordination;
    pub(crate) use crate::cron;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::daemon;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::doctor;
    pub use crate::env;
    pub use crate::file;
    pub use crate::flag;

    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::gateway;
    pub use crate::global;

    pub(crate) use crate::health;
    pub(crate) use crate::heartbeat;
    pub use crate::hooks;
    pub use crate::id;
    pub use crate::installation;
    pub(crate) use crate::integrations;

    pub use crate::memory;
    pub(crate) use crate::multimodal;
    pub use crate::observability;
    pub use crate::patch;
    pub use crate::permission;
    pub use crate::project;
    pub use crate::provider;
    pub use crate::providers;
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::pty;
    pub use crate::question;
    pub use crate::runtime;
    pub use crate::scheduler;
    pub use crate::security;

    pub use crate::session;
    pub use crate::shell;
    pub use crate::skill;

    pub use crate::skills;
    pub use crate::snapshot;
    pub use crate::sop;
    pub use crate::storage;
    pub use crate::tools;
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) use crate::tunnel;

    pub use crate::util;
}

/// 应用配置桥接模块。
///
/// 该模块将桌面/UI 配置读写暴露到 `crate::app::config` 路径，实际存储
/// 与序列化逻辑仍由 session UI 配置模块负责。
pub(crate) mod config {
    /// 读取当前应用配置。
    ///
    /// # 返回值
    ///
    /// 返回序列化为 `serde_json::Value` 的配置快照；读取失败时由底层
    /// 配置模块决定默认值或错误表示。
    pub fn load_app_config() -> serde_json::Value {
        crate::session::ui_config::load_app_config()
    }

    /// 设置单个配置字段。
    ///
    /// # 参数
    ///
    /// - `key`: 要更新的配置键。
    /// - `value`: 写入该键的 JSON 值。
    ///
    /// # 错误处理
    ///
    /// 本函数保持历史 fire-and-forget 语义，不向调用方返回错误；失败
    /// 行为由底层 UI 配置模块处理。
    pub fn set_config_field(key: &str, value: serde_json::Value) {
        crate::session::ui_config::set_config_field(key, value)
    }
}

/// 启动期任务兼容模型。
///
/// 当前实现只保存创建任务所需的最小字段，用于保留旧调用路径；真实
/// 调度与持久化能力由专门的任务/会话模块承担。
pub(crate) mod task {
    /// 轻量任务描述。
    ///
    /// 字段保持公开以兼容既有构造与测试代码，调用方可以直接检查任务
    /// 标识、模型和提示词。
    #[derive(Debug, Clone)]
    pub struct Task {
        /// 任务唯一标识。
        pub id: String,
        /// 执行任务时使用的模型标识。
        pub model: String,
        /// 用户或系统提供的任务提示词。
        pub prompt: String,
    }

    impl Task {
        /// 创建一个启动占位任务。
        ///
        /// # 参数
        ///
        /// - `_priority`: 历史接口保留的优先级参数，当前兼容实现不使用。
        ///
        /// # 返回值
        ///
        /// 返回带有稳定占位 id、自动模型和空提示词的任务。
        pub fn new(_priority: u32) -> Self {
            Self {
                id: "bootstrap-task".to_string(),
                model: "auto".to_string(),
                prompt: String::new(),
            }
        }
    }

    /// 创建任务并返回该任务。
    ///
    /// # 参数
    ///
    /// - `_project_path`: 历史接口保留的项目路径，当前兼容实现不使用。
    /// - `task`: 要创建的任务对象。
    ///
    /// # 返回值
    ///
    /// 成功时原样返回 `task`。
    ///
    /// # 错误处理
    ///
    /// 当前实现不会主动产生 I/O 错误，返回 `std::io::Result` 是为了保持
    /// 调用方签名兼容。
    pub fn create_task(_project_path: &str, task: Task) -> std::io::Result<Task> {
        Ok(task)
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
