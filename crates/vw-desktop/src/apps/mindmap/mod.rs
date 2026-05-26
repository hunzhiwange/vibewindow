//! 思维导图应用模块
//!
//! 本模块提供了思维导图功能的核心实现，包括：
//! - 画布渲染和管理（canvas）
//! - 消息处理和状态更新（message）
//! - 数据模型定义（model）
//! - 状态管理（state）
//! - 视图渲染（view）
//!
//! # 架构
//!
//! 模块采用 Model-Update-View (MUV) 架构模式：
//! - `model`: 定义思维导图的数据结构
//! - `state`: 管理应用状态
//! - `message`: 处理消息和状态更新
//! - `view`: 负责视图渲染
//! - `canvas`: 画布相关功能
//!
//! # 主要功能
//!
//! - 支持多标签页管理
//! - 数据持久化和恢复
//! - 响应式 UI 更新
//! - 消息驱动的状态管理

/// 画布模块，负责思维导图的画布渲染和交互
pub mod canvas;

/// 消息模块，定义思维导图相关的消息类型和处理逻辑
pub mod message;

/// 数据模型模块，定义思维导图的核心数据结构
pub mod model;

/// 状态模块，管理思维导图的应用状态
pub mod state;

/// 视图模块，负责思维导图的 UI 渲染
pub mod view;

#[cfg(test)]
#[path = "model_tests.rs"]
mod model_tests;
#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
#[cfg(test)]
mod tests;

/// 重导出思维导图消息类型
///
/// 便于外部模块直接使用 `MindMapMessage` 而不需要了解内部模块结构
pub use message::MindMapMessage;

use crate::app::App;

/// 确保思维导图已正确初始化
///
/// 该函数执行以下初始化步骤：
/// 1. 如果没有打开的标签页，尝试加载已持久化的数据
/// 2. 如果仍然没有标签页，创建一个新的空白标签页
/// 3. 确保当前激活的标签页 ID 指向一个有效的标签页
///
/// # 参数
///
/// * `app` - 可变引用的应用实例，将修改其思维导图相关的状态
///
/// # 副作用
///
/// 该函数会修改 `app` 的以下字段：
/// - `mindmap_tabs`: 可能添加新的标签页
/// - `mindmap_active_tab_id`: 可能更新当前激活的标签页 ID
///
/// # 示例
///
/// ```ignore
/// use crate::app::App;
/// use crate::apps::mindmap::ensure_initialized;
///
/// let mut app = App::default();
/// ensure_initialized(&mut app);
/// // 现在可以确保 app.mindmap_tabs 不为空
/// ```
pub fn ensure_initialized(app: &mut App) -> iced::Task<crate::app::Message> {
    // 如果没有任何标签页，首先尝试加载持久化的数据
    if app.mindmap_tabs.is_empty() {
        #[cfg(target_arch = "wasm32")]
        {
            return message::load_persisted(app);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = message::load_persisted(app);
        }
    }

    // 如果加载后仍然没有标签页，创建一个新的空白标签页
    if app.mindmap_tabs.is_empty() {
        return message::new_blank_tab(app);
    } else if app
        .mindmap_active_tab_id
        .as_ref()
        // 查找当前激活的标签页是否存在
        .and_then(|id| app.mindmap_tabs.iter().find(|t| &t.id == id))
        .is_none()
    {
        // 如果当前激活的标签页 ID 无效，设置为第一个标签页
        app.mindmap_active_tab_id = app.mindmap_tabs.first().map(|t| t.id.clone());
    }

    iced::Task::none()
}

/// 渲染思维导图视图
///
/// 该函数创建并返回思维导图的 UI 元素，用于显示在应用界面中。
/// 委托给 `view` 模块的具体实现。
///
/// # 参数
///
/// * `app` - 应用实例的不可变引用，用于读取当前状态
///
/// # 返回值
///
/// 返回一个 iced `Element`，代表思维导图的完整 UI 视图。
/// 该元素包含了画布、工具栏、节点等所有可见组件。
///
/// # 示例
///
/// ```ignore
/// use crate::app::App;
/// use crate::apps::mindmap::view;
///
/// let app = App::default();
/// let element = view(&app);
/// // 将 element 渲染到界面上
/// ```
pub fn view(app: &App) -> iced::Element<'_, crate::app::Message> {
    view::view(app)
}

/// 处理思维导图消息并更新应用状态
///
/// 该函数是思维导图模块的核心消息处理器，接收消息并根据消息类型
/// 执行相应的状态更新和副作用操作。委托给 `message` 模块的具体实现。
///
/// # 参数
///
/// * `app` - 可变引用的应用实例，用于更新状态
/// * `message` - 要处理的思维导图消息，包含具体的操作指令
///
/// # 返回值
///
/// 返回一个 iced `Task`，可能包含需要执行的异步操作或命令。
/// 例如：
/// - 保存数据到磁盘
/// - 加载远程资源
/// - 执行动画
///
/// # 示例
///
/// ```ignore
/// use crate::app::App;
/// use crate::apps::mindmap::{update, MindMapMessage};
///
/// let mut app = App::default();
/// let message = MindMapMessage::CreateNode;
/// let task = update(&mut app, message);
/// // 执行 task 中的操作
/// ```
pub fn update(app: &mut App, message: MindMapMessage) -> iced::Task<crate::app::Message> {
    message::update(app, message)
}
