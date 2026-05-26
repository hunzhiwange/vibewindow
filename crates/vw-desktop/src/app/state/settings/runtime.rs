use super::*;

/// Hooks 设置面板状态
///
/// 管理运行时 hooks 总开关与内置钩子的配置。
#[derive(Debug, Clone)]
pub(crate) struct HooksSettingsState {
    /// 是否启用 hooks 运行时
    pub(crate) enabled: bool,
    /// 是否启用 command_logger 内置钩子
    pub(crate) command_logger: bool,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for HooksSettingsState {
    fn default() -> Self {
        Self { enabled: true, command_logger: false, save_error: None }
    }
}

/// 运行时设置面板状态
///
/// 管理 runtime 配置，包括 kind 选择、Docker/WASM 子配置与推理覆盖选项。
#[derive(Debug, Clone)]
pub(crate) struct RuntimeSettingsState {
    /// 运行时类型
    pub(crate) kind: String,
    /// Docker 镜像
    pub(crate) docker_image: String,
    /// Docker 网络模式
    pub(crate) docker_network: String,
    /// Docker 内存限制输入（MB）
    pub(crate) docker_memory_limit_mb_input: String,
    /// Docker CPU 限制输入
    pub(crate) docker_cpu_limit_input: String,
    /// Docker 只读根文件系统
    pub(crate) docker_read_only_rootfs: bool,
    /// Docker 挂载工作区
    pub(crate) docker_mount_workspace: bool,
    /// Docker 允许挂载的工作区根目录输入
    pub(crate) docker_allowed_workspace_roots_input: String,
    /// WASM 工具目录
    pub(crate) wasm_tools_dir: String,
    /// WASM 燃料限制输入
    pub(crate) wasm_fuel_limit_input: String,
    /// WASM 内存限制输入（MB）
    pub(crate) wasm_memory_limit_mb_input: String,
    /// WASM 模块大小限制输入（MB）
    pub(crate) wasm_max_module_size_mb_input: String,
    /// 是否允许读取工作区
    pub(crate) wasm_allow_workspace_read: bool,
    /// 是否允许写入工作区
    pub(crate) wasm_allow_workspace_write: bool,
    /// 允许访问的主机输入
    pub(crate) wasm_allowed_hosts_input: String,
    /// 工具目录必须位于工作区内
    pub(crate) wasm_require_workspace_relative_tools_dir: bool,
    /// 拒绝符号链接模块
    pub(crate) wasm_reject_symlink_modules: bool,
    /// 拒绝符号链接工具目录
    pub(crate) wasm_reject_symlink_tools_dir: bool,
    /// 严格主机校验
    pub(crate) wasm_strict_host_validation: bool,
    /// 能力升级模式
    pub(crate) wasm_capability_escalation_mode: String,
    /// 模块哈希策略
    pub(crate) wasm_module_hash_policy: String,
    /// 模块 SHA-256 映射输入
    pub(crate) wasm_module_sha256_input: String,
    /// 推理覆盖开关输入（自动/启用/禁用，对应 auto/true/false）
    pub(crate) reasoning_enabled_input: String,
    /// 推理级别输入
    pub(crate) reasoning_level_input: String,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for RuntimeSettingsState {
    fn default() -> Self {
        Self {
            kind: "native".to_string(),
            docker_image: "alpine:3.20".to_string(),
            docker_network: "none".to_string(),
            docker_memory_limit_mb_input: "512".to_string(),
            docker_cpu_limit_input: "1".to_string(),
            docker_read_only_rootfs: true,
            docker_mount_workspace: true,
            docker_allowed_workspace_roots_input: String::new(),
            wasm_tools_dir: "tools/wasm".to_string(),
            wasm_fuel_limit_input: "1000000".to_string(),
            wasm_memory_limit_mb_input: "64".to_string(),
            wasm_max_module_size_mb_input: "50".to_string(),
            wasm_allow_workspace_read: false,
            wasm_allow_workspace_write: false,
            wasm_allowed_hosts_input: String::new(),
            wasm_require_workspace_relative_tools_dir: true,
            wasm_reject_symlink_modules: true,
            wasm_reject_symlink_tools_dir: true,
            wasm_strict_host_validation: true,
            wasm_capability_escalation_mode: "deny".to_string(),
            wasm_module_hash_policy: "warn".to_string(),
            wasm_module_sha256_input: String::new(),
            reasoning_enabled_input: "auto".to_string(),
            reasoning_level_input: String::new(),
            save_error: None,
        }
    }
}

/// 技能设置面板状态
///
/// 管理技能系统的配置，包括开放技能目录和提示注入模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillsSettingsTab {
    Skills,
    Plugins,
    SystemConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillsDirectoryScope {
    Project,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillsCatalogKind {
    Recommended,
    System,
    Personal,
}

#[derive(Debug, Clone)]
pub(crate) struct SkillsSelectedDetail {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) kind: SkillsCatalogKind,
    pub(crate) installed: bool,
    pub(crate) enabled: bool,
    pub(crate) source: String,
    pub(crate) source_path: Option<String>,
    pub(crate) document_name: String,
    pub(crate) document_content: String,
    pub(crate) can_install: bool,
    pub(crate) can_toggle: bool,
    pub(crate) can_delete: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct SkillsCatalogItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) kind: SkillsCatalogKind,
    pub(crate) resource_count: usize,
    pub(crate) installed: bool,
    pub(crate) enabled: bool,
    pub(crate) source: String,
    pub(crate) source_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct SkillsSettingsState {
    /// 是否启用开放技能
    pub(crate) open_skills_enabled: bool,
    /// 开放技能目录路径输入
    pub(crate) open_skills_dir_input: String,
    /// 提示注入模式
    pub(crate) prompt_injection_mode: vw_config_types::skills::SkillsPromptInjectionMode,
    /// 当前激活的技能设置页签
    pub(crate) active_tab: SkillsSettingsTab,
    /// 技能搜索关键词
    pub(crate) query: String,
    /// 当前选中的目录范围
    pub(crate) directory_scope: SkillsDirectoryScope,
    /// 技能目录是否正在通过 gateway 加载
    pub(crate) loading: bool,
    /// 当前展示的技能目录项
    pub(crate) catalog: Vec<SkillsCatalogItem>,
    /// 当前选中的技能 ID
    pub(crate) selected_skill_id: Option<String>,
    /// 当前选中的技能详情
    pub(crate) selected_skill_detail: Option<SkillsSelectedDetail>,
    /// 技能详情是否正在加载
    pub(crate) detail_loading: bool,
    /// 技能详情错误
    pub(crate) detail_error: Option<String>,
    /// 技能操作状态提示
    pub(crate) status_message: Option<String>,
    /// 当前状态提示是否为错误
    pub(crate) status_is_error: bool,
    /// 是否显示帮助对话框
    pub(crate) show_help_modal: bool,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for SkillsSettingsState {
    fn default() -> Self {
        Self {
            open_skills_enabled: false,
            open_skills_dir_input: String::new(),
            prompt_injection_mode: vw_config_types::skills::SkillsPromptInjectionMode::Compact,
            active_tab: SkillsSettingsTab::Skills,
            query: String::new(),
            directory_scope: SkillsDirectoryScope::Project,
            loading: false,
            catalog: Vec::new(),
            selected_skill_id: None,
            selected_skill_detail: None,
            detail_loading: false,
            detail_error: None,
            status_message: None,
            status_is_error: false,
            show_help_modal: false,
            save_error: None,
        }
    }
}

/// 研究功能设置面板状态
///
/// 管理自动研究功能的配置，包括触发条件、
/// 迭代限制和系统提示等。
#[derive(Debug, Clone)]
pub(crate) struct ResearchSettingsState {
    /// 是否启用研究功能
    pub(crate) enabled: bool,
    /// 研究触发条件
    pub(crate) trigger: ResearchTrigger,
    /// 关键词输入
    pub(crate) keywords_input: String,
    /// 最小消息长度
    pub(crate) min_message_length: u32,
    /// 最大迭代次数
    pub(crate) max_iterations: u32,
    /// 是否显示进度
    pub(crate) show_progress: bool,
    /// 系统提示前缀
    pub(crate) system_prompt_prefix: String,
    /// 是否显示帮助对话框
    pub(crate) show_help_modal: bool,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for ResearchSettingsState {
    fn default() -> Self {
        Self {
            enabled: false,
            trigger: ResearchTrigger::Never,
            keywords_input: String::new(),
            min_message_length: 50,
            max_iterations: 5,
            show_progress: true,
            system_prompt_prefix: String::new(),
            show_help_modal: false,
            save_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WebSearchSettingsState {
    pub(crate) enabled: bool,
    pub(crate) provider: String,
    pub(crate) api_key_input: String,
    pub(crate) api_url_input: String,
    pub(crate) brave_api_key_input: String,
    pub(crate) max_results_input: String,
    pub(crate) timeout_secs_input: String,
    pub(crate) user_agent: String,
    pub(crate) show_help_modal: bool,
    pub(crate) save_error: Option<String>,
}

impl Default for WebSearchSettingsState {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "duckduckgo".to_string(),
            api_key_input: String::new(),
            api_url_input: String::new(),
            brave_api_key_input: String::new(),
            max_results_input: "5".to_string(),
            timeout_secs_input: "15".to_string(),
            user_agent: "VibeWindow/1.0".to_string(),
            show_help_modal: false,
            save_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BrowserSettingsState {
    pub(crate) enabled: bool,
    pub(crate) allowed_domains_input: String,
    pub(crate) allowed_domains_editor: text_editor::Content,
    pub(crate) browser_open: String,
    pub(crate) session_name_input: String,
    pub(crate) backend: String,
    pub(crate) native_headless: bool,
    pub(crate) native_webdriver_url: String,
    pub(crate) native_chrome_path_input: String,
    pub(crate) computer_use_endpoint: String,
    pub(crate) computer_use_api_key_input: String,
    pub(crate) computer_use_timeout_ms_input: String,
    pub(crate) computer_use_allow_remote_endpoint: bool,
    pub(crate) computer_use_window_allowlist_input: String,
    pub(crate) computer_use_max_coordinate_x_input: String,
    pub(crate) computer_use_max_coordinate_y_input: String,
    pub(crate) save_error: Option<String>,
}

impl Default for BrowserSettingsState {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_domains_input: String::new(),
            allowed_domains_editor: text_editor::Content::new(),
            browser_open: "default".to_string(),
            session_name_input: String::new(),
            backend: "agent_browser".to_string(),
            native_headless: true,
            native_webdriver_url: "http://127.0.0.1:9515".to_string(),
            native_chrome_path_input: String::new(),
            computer_use_endpoint: "http://127.0.0.1:8787/v1/actions".to_string(),
            computer_use_api_key_input: String::new(),
            computer_use_timeout_ms_input: "15000".to_string(),
            computer_use_allow_remote_endpoint: false,
            computer_use_window_allowlist_input: String::new(),
            computer_use_max_coordinate_x_input: String::new(),
            computer_use_max_coordinate_y_input: String::new(),
            save_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GatewaySettingsState {
    pub(crate) port: u16,
    pub(crate) host_input: String,
    pub(crate) require_pairing: bool,
    pub(crate) allow_public_bind: bool,
    pub(crate) paired_tokens: Vec<String>,
    pub(crate) new_paired_token_input: String,
    pub(crate) pair_rate_limit_per_minute: u32,
    pub(crate) webhook_rate_limit_per_minute: u32,
    pub(crate) trust_forwarded_headers: bool,
    pub(crate) rate_limit_max_keys: u32,
    pub(crate) idempotency_ttl_secs: u32,
    pub(crate) idempotency_max_keys: u32,
    pub(crate) node_control_enabled: bool,
    pub(crate) node_control_auth_token_input: String,
    pub(crate) node_control_allowed_node_ids_input: String,
    pub(crate) show_help_modal: bool,
    pub(crate) save_error: Option<String>,
}

impl Default for GatewaySettingsState {
    fn default() -> Self {
        Self {
            port: 42617,
            host_input: "127.0.0.1".to_string(),
            require_pairing: true,
            allow_public_bind: false,
            paired_tokens: Vec::new(),
            new_paired_token_input: String::new(),
            pair_rate_limit_per_minute: 10,
            webhook_rate_limit_per_minute: 60,
            trust_forwarded_headers: false,
            rate_limit_max_keys: 10_000,
            idempotency_ttl_secs: 300,
            idempotency_max_keys: 10_000,
            node_control_enabled: false,
            node_control_auth_token_input: String::new(),
            node_control_allowed_node_ids_input: String::new(),
            show_help_modal: false,
            save_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GatewayClientSettingsState {
    pub(crate) host_input: String,
    pub(crate) port: u16,
    pub(crate) bearer_token_input: String,
    pub(crate) username_input: String,
    pub(crate) password_input: String,
    pub(crate) skey_input: String,
    pub(crate) show_help_modal: bool,
    pub(crate) save_error: Option<String>,
}

impl Default for GatewayClientSettingsState {
    fn default() -> Self {
        Self {
            host_input: "127.0.0.1".to_string(),
            port: 42617,
            bearer_token_input: String::new(),
            username_input: String::new(),
            password_input: String::new(),
            skey_input: String::new(),
            show_help_modal: false,
            save_error: None,
        }
    }
}

/// Agent 间 IPC 设置面板状态
///
/// 管理 Agent 间进程间通信的配置。
#[derive(Debug, Clone)]
pub(crate) struct AgentsIpcSettingsState {
    /// 是否启用 IPC
    pub(crate) enabled: bool,
    /// 数据库路径输入
    pub(crate) db_path_input: String,
    /// 过期时间（秒）
    pub(crate) staleness_secs: u64,
    /// 是否显示帮助对话框
    pub(crate) show_help_modal: bool,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for AgentsIpcSettingsState {
    fn default() -> Self {
        Self {
            enabled: false,
            db_path_input: "~/.vibewindow/agents.db".to_string(),
            staleness_secs: 300,
            show_help_modal: false,
            save_error: None,
        }
    }
}

/// 协调设置面板状态
///
/// 管理 Agent 协调系统的配置，包括消息限制和上下文管理等。
#[derive(Debug, Clone)]
pub(crate) struct CoordinationSettingsState {
    /// 是否启用协调
    pub(crate) enabled: bool,
    /// 主 Agent 输入
    pub(crate) lead_agent_input: String,
    /// 每个 Agent 的最大收件箱消息数
    pub(crate) max_inbox_messages_per_agent: u32,
    /// 最大死信数量
    pub(crate) max_dead_letters: u32,
    /// 最大上下文条目数
    pub(crate) max_context_entries: u32,
    /// 最大已见消息 ID 数
    pub(crate) max_seen_message_ids: u32,
    /// 是否显示帮助对话框
    pub(crate) show_help_modal: bool,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for CoordinationSettingsState {
    fn default() -> Self {
        Self {
            enabled: true,
            lead_agent_input: "delegate-lead".to_string(),
            max_inbox_messages_per_agent: 256,
            max_dead_letters: 256,
            max_context_entries: 512,
            max_seen_message_ids: 4096,
            show_help_modal: false,
            save_error: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct CostPriceInput {
    pub(crate) model: String,
    pub(crate) input_price: String,
    pub(crate) output_price: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct CostSettingsState {
    pub(crate) enabled: bool,
    pub(crate) daily_limit_usd_input: String,
    pub(crate) monthly_limit_usd_input: String,
    pub(crate) warn_at_percent_input: String,
    pub(crate) allow_override: bool,
    pub(crate) prices: Vec<CostPriceInput>,
    pub(crate) show_help_modal: bool,
    pub(crate) save_error: Option<String>,
}

impl Default for CostSettingsState {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_limit_usd_input: "10".to_string(),
            monthly_limit_usd_input: "100".to_string(),
            warn_at_percent_input: "80".to_string(),
            allow_override: false,
            prices: Vec::new(),
            show_help_modal: false,
            save_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MemorySettingsState {
    pub(crate) backend: String,
    pub(crate) auto_save: bool,
    pub(crate) hygiene_enabled: bool,
    pub(crate) archive_after_days: u32,
    pub(crate) purge_after_days: u32,
    pub(crate) conversation_retention_days: u32,
    pub(crate) embedding_provider: String,
    pub(crate) embedding_model: String,
    pub(crate) embedding_dimensions: u32,
    pub(crate) vector_weight: f32,
    pub(crate) keyword_weight: f32,
    pub(crate) min_relevance_score: f32,
    pub(crate) embedding_cache_size: u32,
    pub(crate) chunk_max_tokens: u32,
    pub(crate) response_cache_enabled: bool,
    pub(crate) response_cache_ttl_minutes: u32,
    pub(crate) response_cache_max_entries: u32,
    pub(crate) snapshot_enabled: bool,
    pub(crate) snapshot_on_hygiene: bool,
    pub(crate) auto_hydrate: bool,
    pub(crate) sqlite_open_timeout_secs: u32,
    pub(crate) qdrant_url_input: String,
    pub(crate) qdrant_collection: String,
    pub(crate) qdrant_api_key_input: String,
    pub(crate) save_error: Option<String>,
}

impl Default for MemorySettingsState {
    fn default() -> Self {
        Self {
            backend: "sqlite".to_string(),
            auto_save: true,
            hygiene_enabled: true,
            archive_after_days: 7,
            purge_after_days: 30,
            conversation_retention_days: 30,
            embedding_provider: "none".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
            embedding_dimensions: 1536,
            vector_weight: 0.7,
            keyword_weight: 0.3,
            min_relevance_score: 0.4,
            embedding_cache_size: 10_000,
            chunk_max_tokens: 512,
            response_cache_enabled: false,
            response_cache_ttl_minutes: 60,
            response_cache_max_entries: 5_000,
            snapshot_enabled: false,
            snapshot_on_hygiene: false,
            auto_hydrate: true,
            sqlite_open_timeout_secs: 0,
            qdrant_url_input: String::new(),
            qdrant_collection: "vibewindow_memories".to_string(),
            qdrant_api_key_input: String::new(),
            save_error: None,
        }
    }
}

/// 可靠性设置面板状态
///
/// 管理系统可靠性的配置，包括重试策略和退避时间等。
#[derive(Debug, Clone)]
pub(crate) struct ReliabilitySettingsState {
    /// 提供者重试次数
    pub(crate) provider_retries: u32,
    /// 提供者退避时间（毫秒）
    pub(crate) provider_backoff_ms: u64,
    /// 通道初始退避时间（秒）
    pub(crate) channel_initial_backoff_secs: u64,
    /// 通道最大退避时间（秒）
    pub(crate) channel_max_backoff_secs: u64,
    /// 调度器轮询间隔（秒）
    pub(crate) scheduler_poll_secs: u64,
    /// 调度器重试次数
    pub(crate) scheduler_retries: u32,
    /// 是否显示帮助对话框
    pub(crate) show_help_modal: bool,
    /// 保存错误信息
    pub(crate) save_error: Option<String>,
}

impl Default for ReliabilitySettingsState {
    fn default() -> Self {
        Self {
            provider_retries: 2,
            provider_backoff_ms: 500,
            channel_initial_backoff_secs: 2,
            channel_max_backoff_secs: 60,
            scheduler_poll_secs: 15,
            scheduler_retries: 2,
            show_help_modal: false,
            save_error: None,
        }
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
