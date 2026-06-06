//! 应用程序状态模块
//!
//! 本模块定义了 VibeWindow 应用程序的核心状态结构和相关数据类型。
//! 包含应用程序运行时所需的所有状态信息，包括：
//! - 会话管理和运行时状态
//! - Git 差异和版本控制相关状态
//! - 文件树和文件搜索状态
//! - 提供者和模型配置状态
//! - 各种设置面板的状态
//! - 设计编辑器状态
//! - 任务看板状态
//!
//! # 主要结构
//!
//! - [`App`]：应用程序的主状态结构，包含所有子系统和 UI 状态
//! - [`SessionRuntimeState`]：单个会话的运行时状态
//! - [`ProviderSettingsState`]：提供者设置面板状态
//! - [`ModelSettingsState`]：模型设置面板状态

use iced::widget::{Id, text_editor};
use iced::{Color, Point, Theme};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::app::components::markdown_editor::MarkdownViewMode;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::lsp::LspEvent;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::lsp::LspServiceManager;
use crate::app::message;
use crate::app::message::cleaner_tool::CleanerScanReport;
use crate::app::message::large_file_tool::{LargeFileScanProgress, LargeFileScanReport};
use crate::app::models::{self, ChatMessage, ChatSessionStep};
use crate::app::terminal::TerminalState;
use crate::app::views::design::models::ColorFormat;
use crate::app::views::design::properties::ActiveTailwindClassPicker;
use crate::app::views::design::properties::appearance::ActiveEffectPicker;
use crate::app::views::design::properties::color_picker::ActiveColorPicker;
use crate::app::views::design::properties::fill::ActiveFillPicker;
use crate::app::views::design::properties::icon::ActiveIconPicker;
use crate::app::views::design::properties::typography::ActiveFontPicker;
use crate::app::views::design::state::{DesignPlannerCorner, DesignSettingsTab, DesignState};
use crate::app::{DiffTheme, FocusArea, PreviewTab, Screen, SettingsTab, TodoPanelPlacement};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::{LspHoverPending, LspProgress};
use vw_config_types::automation::ResearchTrigger;
use vw_config_types::tools::HttpRequestConfig;

mod agent;
mod app_state;
mod knowledge;
mod pet;
mod presentation;
mod redis;
mod runtime_tools;
mod settings;
mod workspace;

#[cfg(test)]
mod runtime_tools_tests;

pub(crate) use agent::*;
pub use app_state::{App, CookieConfig, RecentProjectMeta, WebBookmark};
pub(crate) use app_state::{
    Notification, Toast, ToastKind, default_recent_project_session_auto_refresh,
    default_recent_project_session_refresh_interval_seconds,
};
pub(crate) use knowledge::*;
pub(crate) use pet::*;
pub use presentation::AppTab;
pub(crate) use presentation::{
    ConventionalCommitType, ExternalOpenApp, ModelPopoverHover, RuntimePlatform, TopBarGatewayTab,
};
pub(crate) use redis::*;
pub(crate) use runtime_tools::*;
pub(crate) use settings::*;
pub(crate) use workspace::*;
