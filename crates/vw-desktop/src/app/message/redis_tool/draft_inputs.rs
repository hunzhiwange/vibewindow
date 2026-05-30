//! 处理 Redis 连接草稿表单输入、文件选择结果和开关状态。

use super::helpers::pick_file_path;
use super::{App, Message, RedisToolMessage, Task};

/// 处理 `draft_name_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_name_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.name = value;
    Task::none()
}

#[cfg(test)]
#[path = "draft_inputs_tests.rs"]
mod draft_inputs_tests;

/// 处理 `draft_host_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_host_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.host = value;
    Task::none()
}

/// 处理 `draft_port_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_port_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.port = value;
    Task::none()
}

/// 处理 `draft_db_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_db_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.db = value;
    Task::none()
}

/// 处理 `draft_username_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_username_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.username = value;
    Task::none()
}

/// 处理 `draft_password_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_password_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.password = value;
    Task::none()
}

/// 处理 `draft_tab_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_tab_changed(app: &mut App, value: super::RedisConnectionTab) -> Task<Message> {
    app.redis_tool.draft_tab = value;
    Task::none()
}

/// 处理 `draft_tls_toggled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_tls_toggled(app: &mut App, value: bool) -> Task<Message> {
    app.redis_tool.draft.use_tls = value;
    Task::none()
}

/// 处理 `draft_tls_private_key_path_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_tls_private_key_path_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.tls_cert.private_key_path = value;
    Task::none()
}

/// 处理 `draft_tls_public_cert_path_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_tls_public_cert_path_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.tls_cert.public_cert_path = value;
    Task::none()
}

/// 处理 `draft_tls_ca_cert_path_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_tls_ca_cert_path_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.tls_cert.ca_cert_path = value;
    Task::none()
}

/// 处理 `pick_tls_private_key_file` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn pick_tls_private_key_file() -> Task<Message> {
    Task::perform(
        pick_file_path(vec![("Key", vec!["key", "pem"]), ("All Files", vec!["*"])]),
        |opt| Message::RedisTool(RedisToolMessage::TlsPrivateKeyFilePicked(opt)),
    )
}

/// 处理 `tls_private_key_file_picked` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn tls_private_key_file_picked(app: &mut App, opt: Option<String>) -> Task<Message> {
    if let Some(path) = opt {
        app.redis_tool.draft.tls_cert.private_key_path = path;
    }
    Task::none()
}

/// 处理 `pick_tls_public_cert_file` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn pick_tls_public_cert_file() -> Task<Message> {
    Task::perform(
        pick_file_path(vec![("Certificate", vec!["crt", "cer", "pem"]), ("All Files", vec!["*"])]),
        |opt| Message::RedisTool(RedisToolMessage::TlsPublicCertFilePicked(opt)),
    )
}

/// 处理 `tls_public_cert_file_picked` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn tls_public_cert_file_picked(app: &mut App, opt: Option<String>) -> Task<Message> {
    if let Some(path) = opt {
        app.redis_tool.draft.tls_cert.public_cert_path = path;
    }
    Task::none()
}

/// 处理 `pick_tls_ca_cert_file` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn pick_tls_ca_cert_file() -> Task<Message> {
    Task::perform(
        pick_file_path(vec![("Certificate", vec!["crt", "cer", "pem"]), ("All Files", vec!["*"])]),
        |opt| Message::RedisTool(RedisToolMessage::TlsCaCertFilePicked(opt)),
    )
}

/// 处理 `tls_ca_cert_file_picked` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn tls_ca_cert_file_picked(app: &mut App, opt: Option<String>) -> Task<Message> {
    if let Some(path) = opt {
        app.redis_tool.draft.tls_cert.ca_cert_path = path;
    }
    Task::none()
}

/// 处理 `draft_ssh_enabled_toggled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_enabled_toggled(app: &mut App, value: bool) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.enabled = value;
    Task::none()
}

/// 处理 `draft_ssh_host_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_host_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.host = value;
    Task::none()
}

/// 处理 `draft_ssh_port_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_port_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.port = value;
    Task::none()
}

/// 处理 `draft_ssh_username_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_username_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.username = value;
    Task::none()
}

/// 处理 `draft_ssh_password_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_password_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.password = value;
    Task::none()
}

/// 处理 `draft_ssh_private_key_path_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn draft_ssh_private_key_path_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.private_key_path = value;
    Task::none()
}

/// 处理 `pick_ssh_private_key_file` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn pick_ssh_private_key_file() -> Task<Message> {
    Task::perform(
        pick_file_path(vec![("SSH Key", vec!["pem", "key"]), ("All Files", vec!["*"])]),
        |opt| Message::RedisTool(RedisToolMessage::SshPrivateKeyFilePicked(opt)),
    )
}

/// 处理 `ssh_private_key_file_picked` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn ssh_private_key_file_picked(app: &mut App, opt: Option<String>) -> Task<Message> {
    if let Some(path) = opt {
        app.redis_tool.draft.ssh_tunnel.private_key_path = path;
    }
    Task::none()
}

/// 处理 `draft_ssh_passphrase_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_passphrase_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.passphrase = value;
    Task::none()
}

/// 处理 `draft_ssh_timeout_secs_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_ssh_timeout_secs_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.ssh_tunnel.timeout_secs = value;
    Task::none()
}

/// 处理 `draft_sentinel_enabled_toggled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_sentinel_enabled_toggled(app: &mut App, value: bool) -> Task<Message> {
    app.redis_tool.draft.sentinel.enabled = value;
    Task::none()
}

/// 处理 `draft_sentinel_master_name_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn draft_sentinel_master_name_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.sentinel.master_name = value;
    Task::none()
}

/// 处理 `draft_sentinel_node_password_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
pub(super) fn draft_sentinel_node_password_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.sentinel.node_password = value;
    Task::none()
}

/// 处理 `draft_cluster_toggled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_cluster_toggled(app: &mut App, value: bool) -> Task<Message> {
    app.redis_tool.draft.use_cluster = value;
    Task::none()
}

/// 处理 `draft_read_only_toggled` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_read_only_toggled(app: &mut App, value: bool) -> Task<Message> {
    app.redis_tool.draft.read_only = value;
    Task::none()
}

/// 处理 `draft_key_pattern_changed` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub(super) fn draft_key_pattern_changed(app: &mut App, value: String) -> Task<Message> {
    app.redis_tool.draft.key_pattern = value;
    Task::none()
}
