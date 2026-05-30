//! session_ui 与会话元数据访问封装。
//!
//! 本模块只负责把 `GatewayUiRuntime` 扩展为新 TUI 可直接消费的
//! session_ui 访问层，覆盖：
//! - session_ui 快照读写
//! - session 列表预览与单条预览元数据
//! - 会话持久化路径读取
//! - 全局 session scope 读写
//!
//! 这里不引入状态层或缓存逻辑，只保留稳定、可复用的 gateway 调用入口。

use serde::Deserialize;
use std::path::PathBuf;
use vw_gateway_client::GatewaySessionCreateBody;
use vw_shared::session::ui_types::{ChatSession, ChatSessionMeta};

#[cfg(not(target_arch = "wasm32"))]
use super::gateway::block_on_gateway;
use super::gateway::{
    GatewayUiRuntime, annotate_gateway_transport_error, normalize_optional_str_ref,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct UiSessionCreateInfo {
    pub(crate) id: String,
    pub(crate) title: String,
}

impl GatewayUiRuntime {
    /// 在当前工作区上下文下创建一个新 session。
    pub(crate) async fn session_create(
        &self,
        title: Option<&str>,
    ) -> Result<UiSessionCreateInfo, String> {
        let directory = self
            .directory_value()
            .unwrap_or_else(|| self.directory().to_string_lossy().to_string());
        let body = Some(GatewaySessionCreateBody {
            parent_id: None,
            title: normalize_optional_str_ref(title).map(ToOwned::to_owned),
        });
        self.client()
            .session_create(directory.as_str(), &body)
            .await
            .map_err(|err| annotate_gateway_transport_error(err, self.endpoint()))
    }

    /// 读取当前 runtime 作用域下的 session_ui 快照。
    ///
    /// 当未显式传入 `session_id` 时，会回退到 runtime 自身绑定的当前会话。
    pub(crate) async fn session_ui_load(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<ChatSession>, String> {
        let session_id = self.resolve_session_id(session_id)?;
        let directory = self.directory_value();
        self.client().session_ui_get(session_id, directory.as_deref()).await
    }

    /// 无目录限制地读取 session_ui 快照。
    pub(crate) async fn session_ui_load_any(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<ChatSession>, String> {
        let session_id = self.resolve_session_id(session_id)?;
        self.client().session_ui_get_any(session_id).await
    }

    /// 保存当前 session_ui 快照。
    ///
    /// 保存时会透传 runtime 的目录上下文，让 gateway 根据工作区解析 scope。
    pub(crate) async fn session_ui_save(&self, session: &ChatSession) -> Result<(), String> {
        let session_id = normalize_optional_str_ref(Some(session.id.as_str()))
            .ok_or_else(|| "session_ui save requires a non-empty session id".to_string())?;
        let directory = self.directory_value();
        self.client().session_ui_save(session_id, directory.as_deref(), session).await.map(|_| ())
    }

    /// 读取当前 runtime 作用域下的 session_ui 预览列表。
    pub(crate) async fn session_ui_previews(&self) -> Result<Vec<ChatSessionMeta>, String> {
        let directory = self.directory_value();
        self.client().session_ui_previews(directory.as_deref()).await
    }

    /// 读取单个会话的预览元数据。
    pub(crate) async fn session_preview_meta(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<ChatSessionMeta>, String> {
        let session_id = self.resolve_session_id(session_id)?;
        let directory = self.directory_value();
        self.client().session_preview_meta_get(session_id, directory.as_deref()).await
    }

    /// 读取单个会话对应的持久化文件路径。
    pub(crate) async fn session_path(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<PathBuf>, String> {
        let session_id = self.resolve_session_id(session_id)?;
        let directory = self.directory_value();
        self.client()
            .session_path_get(session_id, directory.as_deref())
            .await
            .map(|path| path.map(PathBuf::from))
    }

    /// 读取当前全局 session scope。
    pub(crate) async fn session_scope_get(&self) -> Result<Option<String>, String> {
        self.client().session_scope_get(None, None).await
    }

    /// 更新当前全局 session scope。
    pub(crate) async fn session_scope_put(&self, scope: Option<&str>) -> Result<(), String> {
        self.client().session_scope_put(normalize_optional_str_ref(scope)).await.map(|_| ())
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式在当前工作区上下文下创建一个新 session。
    pub(crate) fn session_create_blocking(
        &self,
        title: Option<&str>,
    ) -> Result<UiSessionCreateInfo, String> {
        block_on_gateway(self.session_create(title))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式读取当前 runtime 作用域下的 session_ui 快照。
    pub(crate) fn session_ui_load_blocking(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<ChatSession>, String> {
        block_on_gateway(self.session_ui_load(session_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式无目录限制地读取 session_ui 快照。
    pub(crate) fn session_ui_load_any_blocking(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<ChatSession>, String> {
        block_on_gateway(self.session_ui_load_any(session_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式保存 session_ui 快照。
    pub(crate) fn session_ui_save_blocking(&self, session: &ChatSession) -> Result<(), String> {
        block_on_gateway(self.session_ui_save(session))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式读取 session_ui 预览列表。
    pub(crate) fn session_ui_previews_blocking(&self) -> Result<Vec<ChatSessionMeta>, String> {
        block_on_gateway(self.session_ui_previews())
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式读取单个会话的预览元数据。
    pub(crate) fn session_preview_meta_blocking(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<ChatSessionMeta>, String> {
        block_on_gateway(self.session_preview_meta(session_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式读取会话持久化文件路径。
    pub(crate) fn session_path_blocking(
        &self,
        session_id: Option<&str>,
    ) -> Result<Option<PathBuf>, String> {
        block_on_gateway(self.session_path(session_id))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式读取当前全局 session scope。
    pub(crate) fn session_scope_get_blocking(&self) -> Result<Option<String>, String> {
        block_on_gateway(self.session_scope_get())
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// 以阻塞方式更新当前全局 session scope。
    pub(crate) fn session_scope_put_blocking(&self, scope: Option<&str>) -> Result<(), String> {
        block_on_gateway(self.session_scope_put(scope))
    }
}
