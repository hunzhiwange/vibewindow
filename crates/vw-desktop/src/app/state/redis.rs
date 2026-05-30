use super::*;

fn default_redis_tool_schema_version() -> u32 {
    1
}

fn default_redis_tool_load_count() -> u32 {
    500
}

fn default_redis_ssh_port() -> u16 {
    22
}

fn default_redis_ssh_timeout_secs() -> u32 {
    30
}

/// Redis TLS 证书配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisTlsCertConfig {
    #[serde(default)]
    pub(crate) private_key_path: String,
    #[serde(default)]
    pub(crate) public_cert_path: String,
    #[serde(default)]
    pub(crate) ca_cert_path: String,
}

impl Default for RedisTlsCertConfig {
    fn default() -> Self {
        Self {
            private_key_path: String::new(),
            public_cert_path: String::new(),
            ca_cert_path: String::new(),
        }
    }
}

impl RedisTlsCertConfig {
    pub(crate) fn has_custom_paths(&self) -> bool {
        !self.private_key_path.trim().is_empty()
            || !self.public_cert_path.trim().is_empty()
            || !self.ca_cert_path.trim().is_empty()
    }
}

/// Redis SSH 隧道配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSshTunnelConfig {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) host: String,
    #[serde(default = "default_redis_ssh_port")]
    pub(crate) port: u16,
    #[serde(default)]
    pub(crate) username: String,
    #[serde(default)]
    pub(crate) password: String,
    #[serde(default)]
    pub(crate) private_key_path: String,
    #[serde(default)]
    pub(crate) passphrase: String,
    #[serde(default = "default_redis_ssh_timeout_secs")]
    pub(crate) timeout_secs: u32,
}

impl Default for RedisSshTunnelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_redis_ssh_port(),
            username: String::new(),
            password: String::new(),
            private_key_path: String::new(),
            passphrase: String::new(),
            timeout_secs: default_redis_ssh_timeout_secs(),
        }
    }
}

/// Redis Sentinel 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSentinelConfig {
    #[serde(default)]
    pub(crate) enabled: bool,
    #[serde(default)]
    pub(crate) master_name: String,
    #[serde(default)]
    pub(crate) node_password: String,
}

impl Default for RedisSentinelConfig {
    fn default() -> Self {
        Self { enabled: false, master_name: String::new(), node_password: String::new() }
    }
}

/// Redis 连接配置。
///
/// 当前结构同时保存基础直连参数与高级配置草稿，便于连接模板复用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConnectionConfig {
    /// 连接唯一标识，用于列表选择与更新覆盖。
    pub(crate) id: String,
    /// 用户定义的连接名称。
    pub(crate) name: String,
    /// Redis 主机地址。
    pub(crate) host: String,
    /// Redis 端口。
    pub(crate) port: u16,
    /// 默认数据库编号。
    pub(crate) db: i64,
    /// 用户名，可为空。
    #[serde(default)]
    pub(crate) username: String,
    /// 密码，可为空。
    #[serde(default)]
    pub(crate) password: String,
    /// 是否启用 TLS。
    #[serde(default)]
    pub(crate) use_tls: bool,
    /// TLS 证书材料配置。
    #[serde(default)]
    pub(crate) tls_cert: RedisTlsCertConfig,
    /// SSH 隧道配置。
    #[serde(default)]
    pub(crate) ssh_tunnel: RedisSshTunnelConfig,
    /// Sentinel 配置。
    #[serde(default)]
    pub(crate) sentinel: RedisSentinelConfig,
    /// 是否启用 Cluster 模式。
    #[serde(default)]
    pub(crate) use_cluster: bool,
    /// 是否只读访问。
    #[serde(default)]
    pub(crate) read_only: bool,
    /// 首版用于后续键浏览的匹配模式。
    #[serde(default = "RedisConnectionConfig::default_key_pattern")]
    pub(crate) key_pattern: String,
    /// 最近一次进入该连接工作区的时间戳。
    #[serde(default)]
    pub(crate) last_used_ms: Option<u64>,
    /// 最近一次保存该连接配置的时间戳。
    #[serde(default)]
    pub(crate) updated_at_ms: u64,
}

impl RedisConnectionConfig {
    fn default_key_pattern() -> String {
        "*".to_string()
    }
}

/// Redis 连接配置编辑页签。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedisConnectionTab {
    #[default]
    Basic,
    Ssh,
    Tls,
    Sentinel,
    Cluster,
}

impl RedisConnectionTab {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Basic => "基础",
            Self::Ssh => "SSH",
            Self::Tls => "SSL/TLS",
            Self::Sentinel => "Sentinel",
            Self::Cluster => "Cluster",
        }
    }
}

/// Redis 右侧详情页签。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedisDetailTab {
    #[default]
    Connection,
    Keys,
    Analysis,
    Command,
    Overview,
    Info,
}

impl RedisDetailTab {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Connection => "连接配置",
            Self::Keys => "键树",
            Self::Analysis => "内容分析",
            Self::Command => "命令",
            Self::Overview => "概览",
            Self::Info => "INFO",
        }
    }

    pub(crate) fn requires_runtime(self) -> bool {
        matches!(self, Self::Overview | Self::Info)
    }

    pub(crate) fn requires_keys(self) -> bool {
        matches!(self, Self::Keys)
    }

    pub(crate) fn requires_key_analysis(self) -> bool {
        matches!(self, Self::Analysis)
    }
}

/// Redis Key 默认创建类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedisKeyValueKind {
    #[default]
    String,
    Hash,
    List,
    Set,
    Zset,
    Stream,
    ReJson,
}

impl RedisKeyValueKind {
    pub(crate) const ALL: [Self; 7] =
        [Self::String, Self::Hash, Self::List, Self::Set, Self::Zset, Self::Stream, Self::ReJson];

    pub(crate) fn gateway_value(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Hash => "Hash",
            Self::List => "List",
            Self::Set => "Set",
            Self::Zset => "Zset",
            Self::Stream => "Stream",
            Self::ReJson => "ReJSON",
        }
    }
}

impl std::fmt::Display for RedisKeyValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.gateway_value())
    }
}

/// Redis Key 新建草稿。
#[derive(Debug, Clone, Default)]
pub struct RedisCreateKeyDraft {
    pub(crate) name: String,
    pub(crate) key_type: RedisKeyValueKind,
}

/// Redis Key 内容分析结果。
#[derive(Debug, Clone, Default)]
pub struct RedisKeyAnalysis {
    pub(crate) connection_id: String,
    pub(crate) key: String,
    pub(crate) key_type: String,
    pub(crate) ttl_secs: i64,
    pub(crate) memory_usage_bytes: Option<u64>,
    pub(crate) preview_command: String,
    pub(crate) preview_output: String,
}

/// Redis 日志记录。
///
/// 该结构用于首版历史弹窗，记录连接配置相关动作与后续 Redis 命令轨迹。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisHistoryRecord {
    /// 记录时间戳（毫秒）。
    pub(crate) time_ms: u64,
    /// 关联的连接 ID，可为空。
    #[serde(default)]
    pub(crate) connection_id: Option<String>,
    /// 显示用连接名称。
    pub(crate) connection_label: String,
    /// 操作或命令名称。
    pub(crate) command: String,
    /// 参数摘要，不包含密码等敏感内容。
    pub(crate) args: String,
    /// 操作耗时（毫秒）。
    #[serde(default)]
    pub(crate) cost_ms: u64,
    /// 是否属于写操作。
    #[serde(default)]
    pub(crate) is_write: bool,
}

/// Redis INFO 键值条目。
#[derive(Debug, Clone, Default)]
pub struct RedisInfoEntry {
    pub(crate) key: String,
    pub(crate) value: String,
}

/// Redis Keyspace 统计条目。
#[derive(Debug, Clone, Default)]
pub struct RedisKeyspaceStat {
    pub(crate) db: String,
    pub(crate) keys: u64,
    pub(crate) expires: u64,
    pub(crate) avg_ttl: i64,
}

/// Redis 运行时概览。
#[derive(Debug, Clone, Default)]
pub struct RedisRuntimeOverview {
    pub(crate) connection_id: String,
    pub(crate) connection_label: String,
    pub(crate) server_version: String,
    pub(crate) os: String,
    pub(crate) process_id: String,
    pub(crate) used_memory_human: String,
    pub(crate) used_memory_peak_human: String,
    pub(crate) used_memory_lua_human: String,
    pub(crate) connected_clients: u64,
    pub(crate) total_connections_received: u64,
    pub(crate) total_commands_processed: u64,
    pub(crate) keyspace: Vec<RedisKeyspaceStat>,
    pub(crate) info_entries: Vec<RedisInfoEntry>,
}

/// Redis 命令执行结果。
#[derive(Debug, Clone, Default)]
pub struct RedisCommandOutputEntry {
    pub(crate) command: String,
    pub(crate) output: String,
    pub(crate) cost_ms: u64,
    pub(crate) is_error: bool,
    pub(crate) time_ms: u64,
}

/// Redis 键分页结果。
#[derive(Debug, Clone, Default)]
pub struct RedisKeyPage {
    pub(crate) connection_id: String,
    pub(crate) pattern: String,
    pub(crate) keys: Vec<String>,
    pub(crate) next_cursor: u64,
    pub(crate) has_more: bool,
}

/// Redis 工具持久化状态。
///
/// 该结构写入本地工具内容存储，用于恢复连接列表、默认加载数量与历史记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisToolPersistedState {
    /// 数据结构版本号，用于后续兼容升级。
    #[serde(default = "default_redis_tool_schema_version")]
    pub(crate) schema_version: u32,
    /// 默认加载数量。
    #[serde(default = "default_redis_tool_load_count")]
    pub(crate) default_load_count: u32,
    /// 已保存连接列表。
    #[serde(default)]
    pub(crate) connections: Vec<RedisConnectionConfig>,
    /// 操作历史。
    #[serde(default)]
    pub(crate) history: Vec<RedisHistoryRecord>,
    /// 上次选中的连接。
    #[serde(default)]
    pub(crate) selected_connection_id: Option<String>,
}

impl Default for RedisToolPersistedState {
    fn default() -> Self {
        Self {
            schema_version: default_redis_tool_schema_version(),
            default_load_count: default_redis_tool_load_count(),
            connections: Vec::new(),
            history: Vec::new(),
            selected_connection_id: None,
        }
    }
}

/// Redis 连接草稿。
///
/// 左侧选择连接后，右侧表单会把当前连接映射为该草稿；
/// 点击“新建连接”时则重置为默认模板。
#[derive(Debug, Clone)]
pub struct RedisTlsCertDraft {
    pub(crate) private_key_path: String,
    pub(crate) public_cert_path: String,
    pub(crate) ca_cert_path: String,
}

impl Default for RedisTlsCertDraft {
    fn default() -> Self {
        Self {
            private_key_path: String::new(),
            public_cert_path: String::new(),
            ca_cert_path: String::new(),
        }
    }
}

impl RedisTlsCertDraft {
    pub(crate) fn has_custom_paths(&self) -> bool {
        !self.private_key_path.trim().is_empty()
            || !self.public_cert_path.trim().is_empty()
            || !self.ca_cert_path.trim().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct RedisSshTunnelDraft {
    pub(crate) enabled: bool,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) private_key_path: String,
    pub(crate) passphrase: String,
    pub(crate) timeout_secs: String,
}

impl Default for RedisSshTunnelDraft {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_redis_ssh_port().to_string(),
            username: String::new(),
            password: String::new(),
            private_key_path: String::new(),
            passphrase: String::new(),
            timeout_secs: default_redis_ssh_timeout_secs().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedisSentinelDraft {
    pub(crate) enabled: bool,
    pub(crate) master_name: String,
    pub(crate) node_password: String,
}

impl Default for RedisSentinelDraft {
    fn default() -> Self {
        Self { enabled: false, master_name: String::new(), node_password: String::new() }
    }
}

#[derive(Debug, Clone)]
pub struct RedisConnectionDraft {
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) db: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) use_tls: bool,
    pub(crate) tls_cert: RedisTlsCertDraft,
    pub(crate) ssh_tunnel: RedisSshTunnelDraft,
    pub(crate) sentinel: RedisSentinelDraft,
    pub(crate) use_cluster: bool,
    pub(crate) read_only: bool,
    pub(crate) key_pattern: String,
}

impl Default for RedisConnectionDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: "127.0.0.1".to_string(),
            port: "6379".to_string(),
            db: "0".to_string(),
            username: String::new(),
            password: String::new(),
            use_tls: false,
            tls_cert: RedisTlsCertDraft::default(),
            ssh_tunnel: RedisSshTunnelDraft::default(),
            sentinel: RedisSentinelDraft::default(),
            use_cluster: false,
            read_only: false,
            key_pattern: "*".to_string(),
        }
    }
}

/// Redis 工具运行时 UI 状态。
///
/// 持久化字段与短生命周期字段在这里集中管理，避免继续向 `App` 顶层平铺更多字段。
#[derive(Debug, Clone)]
pub struct RedisToolUiState {
    /// 已保存连接列表。
    pub(crate) connections: Vec<RedisConnectionConfig>,
    /// 历史记录。
    pub(crate) history: Vec<RedisHistoryRecord>,
    /// 当前选中的连接 ID。
    pub(crate) selected_connection_id: Option<String>,
    /// 右侧表单草稿。
    pub(crate) draft: RedisConnectionDraft,
    /// 草稿是否处于“新建连接”模式。
    pub(crate) draft_is_new: bool,
    /// 当前高级连接配置页签。
    pub(crate) draft_tab: RedisConnectionTab,
    /// 当前右侧详情页签。
    pub(crate) detail_tab: RedisDetailTab,
    /// 默认加载数量输入值。
    pub(crate) default_load_count_input: String,
    /// 左侧连接搜索关键字。
    pub(crate) connection_search_query: String,
    /// 历史关键字过滤。
    pub(crate) history_filter: String,
    /// 历史是否仅显示写操作。
    pub(crate) history_only_write: bool,
    /// 是否显示设置弹窗。
    pub(crate) show_settings_modal: bool,
    /// 是否显示历史弹窗。
    pub(crate) show_history_modal: bool,
    /// 是否显示连接配置弹窗。
    pub(crate) show_connection_modal: bool,
    /// 是否显示新增 Key 弹窗。
    pub(crate) show_create_key_modal: bool,
    /// 顶部通知文本。
    pub(crate) notification: Option<String>,
    /// 当前网关请求中的状态文案。
    pub(crate) gateway_loading_label: Option<String>,
    /// 最近一次网关请求错误。
    pub(crate) gateway_error: Option<String>,
    /// 当前连接的运行时概览。
    pub(crate) runtime_overview: Option<RedisRuntimeOverview>,
    /// 命令执行输入框内容。
    pub(crate) command_input: String,
    /// 命令输出缓冲。
    pub(crate) command_output: Vec<RedisCommandOutputEntry>,
    /// 键浏览匹配模式。
    pub(crate) key_browser_pattern: String,
    /// 当前键浏览所属连接。
    pub(crate) key_browser_connection_id: Option<String>,
    /// 已加载的键列表。
    pub(crate) key_browser_items: Vec<String>,
    /// 下一次键分页游标。
    pub(crate) key_browser_cursor: u64,
    /// 是否仍有更多键可继续加载。
    pub(crate) key_browser_has_more: bool,
    /// 已展开的键树路径。
    pub(crate) key_tree_expanded_paths: HashSet<String>,
    /// 当前选中的 Key。
    pub(crate) selected_key: Option<String>,
    /// 新增 Key 草稿。
    pub(crate) create_key_draft: RedisCreateKeyDraft,
    /// 当前 Key 的内容分析结果。
    pub(crate) key_analysis: Option<RedisKeyAnalysis>,
    /// INFO 过滤关键字。
    pub(crate) info_filter: String,
    /// 历史分页偏移。
    pub(crate) history_page_offset: usize,
    /// 历史分页大小。
    pub(crate) history_page_limit: usize,
    /// 历史总量。
    pub(crate) history_total: usize,
    /// 当前历史页后是否仍有更多数据。
    pub(crate) history_has_more: bool,
}

impl RedisToolUiState {
    /// 根据持久化状态恢复 Redis 工具的运行时状态。
    pub(crate) fn from_persisted(state: RedisToolPersistedState) -> Self {
        let selected_connection_id = state.selected_connection_id.clone().filter(|selected_id| {
            state.connections.iter().any(|connection| connection.id == *selected_id)
        });
        let history_total = state.history.len();

        let mut ui = Self {
            connections: state.connections,
            history: state.history,
            selected_connection_id,
            draft: RedisConnectionDraft::default(),
            draft_is_new: true,
            draft_tab: RedisConnectionTab::Basic,
            detail_tab: RedisDetailTab::Connection,
            default_load_count_input: state.default_load_count.max(1).to_string(),
            connection_search_query: String::new(),
            history_filter: String::new(),
            history_only_write: false,
            show_settings_modal: false,
            show_history_modal: false,
            show_connection_modal: false,
            show_create_key_modal: false,
            notification: None,
            gateway_loading_label: None,
            gateway_error: None,
            runtime_overview: None,
            command_input: String::new(),
            command_output: Vec::new(),
            key_browser_pattern: String::new(),
            key_browser_connection_id: None,
            key_browser_items: Vec::new(),
            key_browser_cursor: 0,
            key_browser_has_more: false,
            key_tree_expanded_paths: HashSet::new(),
            selected_key: None,
            create_key_draft: RedisCreateKeyDraft::default(),
            key_analysis: None,
            info_filter: String::new(),
            history_page_offset: 0,
            history_page_limit: 50,
            history_total,
            history_has_more: false,
        };

        if let Some(selected_id) = ui.selected_connection_id.clone()
            && let Some(connection) =
                ui.connections.iter().find(|item| item.id == selected_id).cloned()
        {
            ui.load_connection_into_draft(&connection);
            ui.key_browser_pattern = connection.key_pattern.clone();
            ui.draft_is_new = false;
        }

        ui
    }

    /// 应用一次来自网关的聚合快照。
    pub(crate) fn apply_gateway_snapshot(
        &mut self,
        snapshot: crate::app::config::RedisToolGatewaySnapshot,
    ) {
        let previous_selected_connection_id = self.selected_connection_id.clone();
        let crate::app::config::RedisToolGatewaySnapshot {
            persisted_state,
            history_offset,
            history_limit,
            history_total,
            history_has_more,
        } = snapshot;
        let RedisToolPersistedState {
            default_load_count,
            connections,
            history,
            selected_connection_id,
            ..
        } = persisted_state;

        self.connections = connections;
        self.history = history;
        self.selected_connection_id = selected_connection_id;
        self.default_load_count_input = default_load_count.max(1).to_string();
        self.history_page_offset = history_offset;
        self.history_page_limit = history_limit.max(1);
        self.history_total = history_total;
        self.history_has_more = history_has_more;
        self.sync_connection_scoped_state();

        if let Some(selected_id) = self.selected_connection_id.clone()
            && let Some(connection) =
                self.connections.iter().find(|item| item.id == selected_id).cloned()
        {
            self.load_connection_into_draft(&connection);
            if previous_selected_connection_id.as_deref() != Some(connection.id.as_str())
                || self.key_browser_pattern.trim().is_empty()
            {
                self.key_browser_pattern = connection.key_pattern.clone();
            }
            self.draft_is_new = false;
            return;
        }

        if !self.draft_is_new {
            self.reset_draft();
        }
    }

    /// 标记网关请求开始。
    pub(crate) fn begin_gateway_request(&mut self, label: impl Into<String>) {
        self.gateway_loading_label = Some(label.into());
        self.gateway_error = None;
    }

    /// 清理当前网关请求状态。
    pub(crate) fn finish_gateway_request(&mut self) {
        self.gateway_loading_label = None;
    }

    /// 设置网关错误并清理 loading。
    pub(crate) fn fail_gateway_request(&mut self, error: String) {
        self.gateway_loading_label = None;
        self.gateway_error = Some(error);
    }

    /// 清除网关错误。
    pub(crate) fn clear_gateway_error(&mut self) {
        self.gateway_error = None;
    }

    /// 应用运行时概览。
    pub(crate) fn apply_runtime_overview(&mut self, overview: RedisRuntimeOverview) {
        self.runtime_overview = Some(overview);
    }

    /// 应用一页 Redis 键列表。
    pub(crate) fn apply_key_page(&mut self, page: RedisKeyPage, append: bool) {
        if self.selected_connection_id.as_deref() != Some(page.connection_id.as_str()) {
            return;
        }

        self.key_browser_connection_id = Some(page.connection_id);
        self.key_browser_pattern = page.pattern;

        let mut keys = if append { self.key_browser_items.clone() } else { Vec::new() };
        keys.extend(page.keys);
        keys.sort();
        keys.dedup();
        self.key_browser_items = keys;
        self.key_browser_cursor = page.next_cursor;
        self.key_browser_has_more = page.has_more;

        if !append {
            self.key_tree_expanded_paths.clear();
        }
    }

    /// 附加一条命令输出。
    pub(crate) fn push_command_output(&mut self, entry: RedisCommandOutputEntry) {
        self.command_output.push(entry);
        if self.command_output.len() > 60 {
            self.command_output.remove(0);
        }
    }

    /// 应用当前 Key 的分析结果。
    pub(crate) fn apply_key_analysis(&mut self, analysis: RedisKeyAnalysis) {
        self.selected_key = Some(analysis.key.clone());
        self.key_analysis = Some(analysis);
    }

    /// 当前是否已有匹配所选连接与 Key 的分析结果。
    pub(crate) fn has_key_analysis_for_selected(&self) -> bool {
        self.key_analysis.as_ref().is_some_and(|analysis| {
            self.selected_connection_id.as_deref() == Some(analysis.connection_id.as_str())
                && self.selected_key.as_deref() == Some(analysis.key.as_str())
        })
    }

    /// 选中指定 Key，并在切换 Key 时清理旧分析结果。
    pub(crate) fn select_key(&mut self, key: String) {
        let changed = self.selected_key.as_deref() != Some(key.as_str());
        self.selected_key = Some(key.clone());
        if changed && !self.has_key_analysis_for_selected() {
            self.key_analysis = None;
        }
    }

    /// 将 Key 插入当前键树缓存，以便创建后立即可见。
    pub(crate) fn include_key_browser_item(&mut self, key: String) {
        self.key_browser_items.push(key.clone());
        self.key_browser_items.sort();
        self.key_browser_items.dedup();

        let mut current_path = String::new();
        let mut seen_children = false;
        for segment in key.split(':').filter(|segment| !segment.trim().is_empty()) {
            if current_path.is_empty() {
                current_path.push_str(segment);
            } else {
                seen_children = true;
                self.key_tree_expanded_paths.insert(current_path.clone());
                current_path.push(':');
                current_path.push_str(segment);
            }
        }

        if seen_children {
            self.key_tree_expanded_paths.insert(current_path);
        }
    }

    /// 打开新增 Key 弹窗并重置草稿。
    pub(crate) fn open_create_key_modal(&mut self) {
        self.show_create_key_modal = true;
        self.create_key_draft = RedisCreateKeyDraft::default();
    }

    /// 关闭新增 Key 弹窗。
    pub(crate) fn close_create_key_modal(&mut self) {
        self.show_create_key_modal = false;
    }

    /// 打开连接配置弹窗。
    pub(crate) fn open_connection_modal(&mut self) {
        self.show_connection_modal = true;
    }

    /// 关闭连接配置弹窗。
    pub(crate) fn close_connection_modal(&mut self) {
        self.show_connection_modal = false;
    }

    /// 清理运行时数据与命令面板。
    pub(crate) fn clear_runtime_state(&mut self) {
        self.runtime_overview = None;
        self.detail_tab = RedisDetailTab::Connection;
        self.command_input.clear();
        self.command_output.clear();
        self.info_filter.clear();
        self.clear_key_browser_state();
    }

    /// 清理键树浏览状态。
    pub(crate) fn clear_key_browser_state(&mut self) {
        self.key_browser_pattern.clear();
        self.key_browser_connection_id = None;
        self.key_browser_items.clear();
        self.key_browser_cursor = 0;
        self.key_browser_has_more = false;
        self.key_tree_expanded_paths.clear();
        self.selected_key = None;
        self.key_analysis = None;
        self.show_create_key_modal = false;
    }

    /// 当前是否存在网关请求进行中。
    pub(crate) fn is_gateway_loading(&self) -> bool {
        self.gateway_loading_label.is_some()
    }

    /// 当前是否已有匹配所选连接的运行时概览。
    pub(crate) fn has_runtime_for_selected(&self) -> bool {
        self.runtime_overview.as_ref().is_some_and(|overview| {
            self.selected_connection_id.as_deref() == Some(overview.connection_id.as_str())
        })
    }

    /// 当前是否已有匹配所选连接的键树数据。
    pub(crate) fn has_key_page_for_selected(&self) -> bool {
        self.key_browser_connection_id.as_deref() == self.selected_connection_id.as_deref()
    }

    /// 当前所选详情页签是否已经具备展示数据。
    pub(crate) fn has_detail_tab_data_for_selected(&self, tab: RedisDetailTab) -> bool {
        if tab.requires_runtime() {
            self.has_runtime_for_selected()
        } else if tab.requires_keys() {
            self.has_key_page_for_selected()
        } else if tab.requires_key_analysis() {
            self.has_key_analysis_for_selected()
        } else {
            true
        }
    }

    /// 切换键树路径展开状态。
    pub(crate) fn toggle_key_tree_path(&mut self, path: String) {
        if !self.key_tree_expanded_paths.insert(path.clone()) {
            self.key_tree_expanded_paths.remove(&path);
        }
    }

    /// 将指定连接装载到右侧草稿表单。
    pub(crate) fn load_connection_into_draft(&mut self, connection: &RedisConnectionConfig) {
        self.draft = RedisConnectionDraft {
            name: connection.name.clone(),
            host: connection.host.clone(),
            port: connection.port.to_string(),
            db: connection.db.to_string(),
            username: connection.username.clone(),
            password: connection.password.clone(),
            use_tls: connection.use_tls,
            tls_cert: RedisTlsCertDraft {
                private_key_path: connection.tls_cert.private_key_path.clone(),
                public_cert_path: connection.tls_cert.public_cert_path.clone(),
                ca_cert_path: connection.tls_cert.ca_cert_path.clone(),
            },
            ssh_tunnel: RedisSshTunnelDraft {
                enabled: connection.ssh_tunnel.enabled,
                host: connection.ssh_tunnel.host.clone(),
                port: connection.ssh_tunnel.port.to_string(),
                username: connection.ssh_tunnel.username.clone(),
                password: connection.ssh_tunnel.password.clone(),
                private_key_path: connection.ssh_tunnel.private_key_path.clone(),
                passphrase: connection.ssh_tunnel.passphrase.clone(),
                timeout_secs: connection.ssh_tunnel.timeout_secs.to_string(),
            },
            sentinel: RedisSentinelDraft {
                enabled: connection.sentinel.enabled,
                master_name: connection.sentinel.master_name.clone(),
                node_password: connection.sentinel.node_password.clone(),
            },
            use_cluster: connection.use_cluster,
            read_only: connection.read_only,
            key_pattern: connection.key_pattern.clone(),
        };
        self.draft_tab = if connection.ssh_tunnel.enabled {
            RedisConnectionTab::Ssh
        } else if connection.sentinel.enabled {
            RedisConnectionTab::Sentinel
        } else if connection.use_cluster {
            RedisConnectionTab::Cluster
        } else if connection.use_tls || connection.tls_cert.has_custom_paths() {
            RedisConnectionTab::Tls
        } else {
            RedisConnectionTab::Basic
        };
    }

    /// 重置为新建连接草稿。
    pub(crate) fn reset_draft(&mut self) {
        self.draft = RedisConnectionDraft::default();
        self.draft_is_new = true;
        self.draft_tab = RedisConnectionTab::Basic;
    }

    fn sync_connection_scoped_state(&mut self) {
        let runtime_matches_selection = self.runtime_overview.as_ref().is_some_and(|overview| {
            self.selected_connection_id.as_deref() == Some(overview.connection_id.as_str())
        });
        if !runtime_matches_selection {
            self.runtime_overview = None;
            self.command_input.clear();
            self.command_output.clear();
            self.info_filter.clear();
        }

        let keys_match_selection =
            self.key_browser_connection_id.as_deref() == self.selected_connection_id.as_deref();
        if !keys_match_selection {
            self.clear_key_browser_state();
        }
    }
}

#[cfg(test)]
#[path = "redis_tests.rs"]
mod redis_tests;
