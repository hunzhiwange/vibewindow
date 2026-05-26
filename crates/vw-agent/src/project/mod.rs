//! 项目管理模块
//!
//! 本模块提供项目的完整生命周期管理，包括：
//! - 项目信息存储与检索（基于 Git worktree 或本地目录）
//! - 版本控制系统（VCS）检测与集成
//! - 项目图标自动发现
//! - 沙箱目录管理
//! - 项目实例上下文管理
//! - 项目状态管理

use crate::app::agent::snapshot;
use crate::app::agent::storage;
use crate::app::agent::util::log;
use serde_json::{Map, Value};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::LazyLock;

mod core;
mod icon_discovery;
pub mod instance;
pub mod state;
pub mod vcs;

/// 模块专用日志记录器
static LOGGER: LazyLock<log::Logger> = LazyLock::new(|| {
    let mut tags = Map::new();
    tags.insert("service".to_string(), Value::String("project".to_string()));
    log::create(Some(tags))
});

/// 装箱的 Future 类型别名
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// 项目相关事件定义模块
pub mod event {
    use crate::app::agent::bus;

    pub const UPDATED: bus::Definition = bus::Definition { r#type: "project.updated" };
}

pub use core::{
    add_sandbox, from_directory, list, remove_sandbox, sandboxes, set_initialized, update,
};
pub use icon_discovery::discover;
pub use vw_shared::project::{
    Commands, CommandsUpdate, Icon, IconUpdate, Info, TimeInfo, UpdateInput, Vcs,
};

/// 项目模块错误类型
#[derive(Debug)]
pub enum Error {
    Storage(storage::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Join(tokio::task::JoinError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Storage(e) => write!(f, "{}", e),
            Error::Io(e) => write!(f, "{}", e),
            Error::Json(e) => write!(f, "{}", e),
            Error::Join(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<storage::Error> for Error {
    fn from(value: storage::Error) -> Self {
        Error::Storage(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

/// 引导项目实例
pub async fn instance_bootstrap(worktree: impl AsRef<Path>) {
    let worktree = worktree.as_ref().to_path_buf();
    snapshot::init(&worktree);
}

fn extra<const N: usize>(pairs: [(&'static str, Value); N]) -> Map<String, Value> {
    let mut m = Map::new();
    for (k, v) in pairs {
        m.insert(k.to_string(), v);
    }
    m
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
