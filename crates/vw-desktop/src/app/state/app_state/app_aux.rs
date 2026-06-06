use super::*;

/// 通知消息
///
/// 表示应用程序中的一个通知消息，
/// 包含消息 ID、内容和创建时间。
#[derive(Debug, Clone)]
pub struct Notification {
    /// 通知的唯一标识符
    pub id: usize,
    /// 通知消息内容
    pub message: String,
    /// 通知创建时间
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub created_at: web_time::SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: usize,
    pub kind: ToastKind,
    pub message: String,
}

/// 最近项目元数据
///
/// 存储最近打开项目的元信息，
/// 包括路径、名称、图标和任务看板设置等。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentProjectMeta {
    /// 项目路径
    pub path: String,
    /// 项目显示名称
    pub name: String,
    /// 任务看板设置（可选）
    #[serde(default)]
    pub task_board_settings: Option<crate::app::task::models::TaskBoardSettings>,
    /// 会话是否自动刷新（可选）
    #[serde(default = "default_recent_project_session_auto_refresh")]
    pub session_auto_refresh: bool,
    /// 会话自动刷新间隔秒数（可选）
    #[serde(default = "default_recent_project_session_refresh_interval_seconds")]
    pub session_refresh_interval_seconds: u64,
    /// 项目图标标识符（可选）
    #[serde(default)]
    pub icon: Option<String>,
    /// 项目图标颜色（可选）
    #[serde(default)]
    pub icon_color: Option<String>,
    /// Worktree 启动命令（可选）
    #[serde(default)]
    pub worktree_start_command: Option<String>,
}

pub(crate) const fn default_recent_project_session_auto_refresh() -> bool {
    true
}

pub(crate) const fn default_recent_project_session_refresh_interval_seconds() -> u64 {
    60
}

/// 网页书签
///
/// 存储嵌入 WebView 的网页书签配置，
/// 包括标题、URL、窗口尺寸和 Cookie 配置。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebBookmark {
    /// 书签标题
    pub title: String,
    /// 网页 URL
    pub url: String,
    /// 窗口宽度（可选）
    pub width: Option<i32>,
    /// 窗口高度（可选）
    pub height: Option<i32>,
    /// Cookie 配置列表（可选）
    pub cookie_configs: Option<Vec<CookieConfig>>,
}

/// Cookie 配置
///
/// 定义 WebView 中预设 Cookie 的配置，
/// 包括名称、域名、有效期和 URL 过滤规则。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CookieConfig {
    /// Cookie 名称
    pub name: String,
    /// Cookie 作用域域名（可选）
    pub domain: Option<String>,
    /// Cookie 有效天数（可选）
    pub days: Option<i64>,
    /// URL 过滤规则（可选）
    pub url_filter: Option<String>,
}

impl App {
    /// 获取当前活跃的思维导图标签页
    ///
    /// 优先返回由 `mindmap_active_tab_id` 指定的标签页；
    /// 如果未找到，则返回第一个标签页。
    ///
    /// # 返回值
    ///
    /// 如果存在思维导图标签页，返回其不可变引用；
    /// 否则返回 `None`。
    pub fn active_mindmap_tab(&self) -> Option<&crate::apps::mindmap::state::MindMapTab> {
        self.mindmap_active_tab_id
            .as_ref()
            .and_then(|id| self.mindmap_tabs.iter().find(|t| &t.id == id))
            .or_else(|| self.mindmap_tabs.first())
    }

    /// 获取当前活跃的思维导图标签页的可变引用
    ///
    /// 优先返回由 `mindmap_active_tab_id` 指定的标签页；
    /// 如果未找到，则返回第一个标签页的可变引用。
    ///
    /// # 返回值
    ///
    /// 如果存在思维导图标签页，返回其可变引用；
    /// 否则返回 `None`。
    pub fn active_mindmap_tab_mut(
        &mut self,
    ) -> Option<&mut crate::apps::mindmap::state::MindMapTab> {
        let pos_opt = self
            .mindmap_active_tab_id
            .as_deref()
            .and_then(|id| self.mindmap_tabs.iter().position(|t| t.id == id));
        if let Some(pos) = pos_opt {
            return self.mindmap_tabs.get_mut(pos);
        }
        self.mindmap_tabs.first_mut()
    }
}

#[cfg(test)]
#[path = "app_aux_tests.rs"]
mod app_aux_tests;
