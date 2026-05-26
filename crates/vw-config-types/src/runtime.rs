//! 运行时适配器配置模块
//!
//! 本模块定义了代理运行时的配置结构，支持三种运行时类型：
//! - `native`: 原生系统运行时（默认）
//! - `docker`: Docker 容器隔离运行时
//! - `wasm`: WebAssembly 沙箱运行时
//!
//! # 配置结构
//!
//! 运行时配置通过 `[runtime]` 配置段进行设置，包含：
//! - 运行时类型选择（`kind`）
//! - Docker 运行时特定配置（`[runtime.docker]`）
//! - WASM 运行时特定配置（`[runtime.wasm]`）
//! - 推理能力覆盖设置（`reasoning_enabled`、`reasoning_level`）
//!
//! # 安全性
//!
//! - Docker 运行时默认采用只读根文件系统和网络隔离
//! - WASM 运行时默认启用模块哈希校验和严格主机验证
//! - 所有运行时类型均采用最小权限原则

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 运行时适配器主配置（`[runtime]` 配置段）
///
/// 定义代理的执行环境类型及相关配置。运行时负责隔离和执行工具调用，
/// 是系统安全边界的关键组成部分。
///
/// # 示例配置
///
/// ```toml
/// [runtime]
/// kind = "docker"
/// reasoning_enabled = true
///
/// [runtime.docker]
/// image = "alpine:3.20"
/// memory_limit_mb = 512
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeConfig {
    /// 运行时类型（`native` | `docker` | `wasm`）
    ///
    /// - `native`: 直接在宿主机执行，性能最高但隔离性最弱
    /// - `docker`: 在 Docker 容器中执行，提供进程级隔离
    /// - `wasm`: 在 WebAssembly 沙箱中执行，提供最强隔离
    #[serde(default = "default_runtime_kind")]
    pub kind: String,

    /// Docker 运行时配置（当 `kind = "docker"` 时生效）
    ///
    /// 控制 Docker 容器的资源限制、网络配置和文件系统挂载等。
    #[serde(default)]
    pub docker: DockerRuntimeConfig,

    /// WASM 运行时配置（当 `kind = "wasm"` 时生效）
    ///
    /// 控制 WebAssembly 模块的执行环境，包括资源限制和安全策略。
    #[serde(default)]
    pub wasm: WasmRuntimeConfig,

    /// 全局推理能力覆盖开关（用于支持显式控制的 Provider）
    ///
    /// 推理能力允许模型在响应前进行内部思考链推理。
    ///
    /// - `None`: 使用 Provider 默认行为
    /// - `Some(true)`: 在支持时请求启用推理/思考功能
    /// - `Some(false)`: 在支持时禁用推理/思考功能
    #[serde(default)]
    pub reasoning_enabled: Option<bool>,

    /// 推理级别兼容性别名（已弃用，保留用于向后兼容）
    ///
    /// # 迁移说明
    ///
    /// - 推荐配置键: `provider.reasoning_level`
    /// - 兼容配置键: `runtime.reasoning_level`（已弃用）
    /// - 当两者同时设置时，Provider 级别的值优先
    #[serde(default)]
    pub reasoning_level: Option<String>,
}

/// Docker 运行时配置（`[runtime.docker]` 配置段）
///
/// 定义 Docker 容器的执行环境参数，包括镜像、资源限制和安全设置。
/// 默认配置采用最小权限原则，提供安全的隔离环境。
///
/// # 安全默认值
///
/// - 网络模式默认为 `none`（无网络访问）
/// - 根文件系统默认为只读
/// - 默认启用工作区挂载
///
/// # 示例配置
///
/// ```toml
/// [runtime.docker]
/// image = "alpine:3.20"
/// network = "bridge"
/// memory_limit_mb = 1024
/// cpu_limit = 2.0
/// read_only_rootfs = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DockerRuntimeConfig {
    /// 执行 Shell 命令的 Docker 镜像
    ///
    /// 镜像应包含执行工具所需的基本命令（如 `sh`、`ls` 等）。
    /// 默认使用轻量级的 Alpine Linux 镜像。
    #[serde(default = "default_docker_image")]
    pub image: String,

    /// Docker 网络模式
    ///
    /// 常用选项：
    /// - `none`: 无网络访问（最安全）
    /// - `bridge`: 使用 Docker 默认网桥
    /// - `host`: 使用宿主机网络（需谨慎使用）
    #[serde(default = "default_docker_network")]
    pub network: String,

    /// 内存限制（单位：MB）
    ///
    /// - `Some(n)`: 限制容器最大使用 n MB 内存
    /// - `None`: 不设置显式内存限制
    #[serde(default = "default_docker_memory_limit_mb")]
    pub memory_limit_mb: Option<u64>,

    /// CPU 限制
    ///
    /// - `Some(n)`: 限制容器最多使用 n 个 CPU 核心
    /// - `None`: 不设置显式 CPU 限制
    #[serde(default = "default_docker_cpu_limit")]
    pub cpu_limit: Option<f64>,

    /// 是否将根文件系统挂载为只读
    ///
    /// 启用后容器无法修改系统文件，提升安全性。
    /// 默认启用。
    #[serde(default = "default_true")]
    pub read_only_rootfs: bool,

    /// 是否将工作区挂载到容器的 `/workspace` 目录
    ///
    /// 启用后代理可以在容器内访问和修改工作区文件。
    /// 默认启用。
    #[serde(default = "default_true")]
    pub mount_workspace: bool,

    /// 工作区根目录白名单
    ///
    /// 用于验证 Docker 挂载路径的合法性。
    /// 只有在此列表中的目录才允许被挂载到容器。
    #[serde(default)]
    pub allowed_workspace_roots: Vec<String>,
}

/// WASM 运行时配置（`[runtime.wasm]` 配置段）
///
/// 定义 WebAssembly 模块的执行环境，提供细粒度的资源限制和安全控制。
/// WASM 运行时提供比 Docker 更强的隔离性，适合执行不可信代码。
///
/// # 安全特性
///
/// - 燃料限制：控制指令执行数量，防止无限循环
/// - 内存限制：限制模块可用内存大小
/// - 模块大小限制：防止加载过大的模块
/// - 主机访问控制：限制对文件系统和网络的访问
///
/// # 示例配置
///
/// ```toml
/// [runtime.wasm]
/// tools_dir = "tools/wasm"
/// fuel_limit = 1000000
/// memory_limit_mb = 64
/// max_module_size_mb = 50
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WasmRuntimeConfig {
    /// WASM 工具模块目录（相对于工作区）
    ///
    /// 存放 `.wasm` 模块文件的目录路径。安全策略要求此目录必须位于工作区内。
    #[serde(default = "default_wasm_tools_dir")]
    pub tools_dir: String,

    /// 每次调用的燃料限制（指令预算）
    ///
    /// 燃料是 WASM 运行时用于计算指令执行的抽象资源。
    /// 当燃料耗尽时，执行将被终止，防止无限循环攻击。
    #[serde(default = "default_wasm_fuel_limit")]
    pub fuel_limit: u64,

    /// 每次调用的内存限制（单位：MB）
    ///
    /// 限制 WASM 模块在单次调用中可分配的最大内存。
    /// 超过此限制的内存分配将失败。
    #[serde(default = "default_wasm_memory_limit_mb")]
    pub memory_limit_mb: u64,

    /// WASM 模块最大文件大小（单位：MB）
    ///
    /// 限制可加载的 `.wasm` 模块文件的最大尺寸。
    /// 防止加载过大的模块消耗过多资源。
    #[serde(default = "default_wasm_max_module_size_mb")]
    pub max_module_size_mb: u64,

    /// 是否允许在 WASM 宿主调用中读取工作区文件（未来功能）
    ///
    /// 启用后 WASM 模块可以通过宿主函数读取工作区内的文件。
    /// 目前为未来预留功能。
    #[serde(default)]
    pub allow_workspace_read: bool,

    /// 是否允许在 WASM 宿主调用中写入工作区文件（未来功能）
    ///
    /// 启用后 WASM 模块可以通过宿主函数写入工作区内的文件。
    /// 目前为未来预留功能。
    #[serde(default)]
    pub allow_workspace_write: bool,

    /// 出站 HTTP 请求的主机白名单（未来功能）
    ///
    /// 限制 WASM 模块可以访问的外部主机列表。
    /// 只有在此列表中的主机才允许进行 HTTP 请求。
    /// 目前为未来预留功能。
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    /// WASM 运行时安全控制（`[runtime.wasm.security]` 配置段）
    ///
    /// 定义 WASM 模块的安全策略，包括完整性校验和能力限制。
    #[serde(default)]
    pub security: WasmSecurityConfig,
}

/// WASM 能力升级处理策略
///
/// 定义当 WASM 调用请求的能力超过运行时基线策略时如何处理。
/// 能力包括内存限制、燃料限制、文件系统访问等权限。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WasmCapabilityEscalationMode {
    /// 拒绝：拒绝任何超出运行时配置的能力请求
    ///
    /// 严格模式，当调用请求的能力超过配置限制时直接拒绝执行。
    /// 这是最安全的选项。
    #[default]
    Deny,

    /// 裁剪：自动将调用能力限制到运行时配置的上限
    ///
    /// 宽松模式，自动将请求的能力调整为不超过配置限制的值。
    /// 允许执行但可能影响功能。
    Clamp,
}

/// WASM 模块哈希校验策略
///
/// 定义是否以及如何验证通过 SHA-256 摘要固定的 WASM 模块完整性。
/// 模块哈希可以防止模块被篡改。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WasmModuleHashPolicy {
    /// 禁用：不进行模块哈希校验
    ///
    /// 不验证模块文件的完整性。不推荐在生产环境使用。
    Disabled,

    /// 警告：哈希缺失或不匹配时警告，但允许执行
    ///
    /// 宽松模式，当模块哈希与配置不匹配时发出警告但继续执行。
    /// 这是默认选项，平衡了安全性和可用性。
    #[default]
    Warn,

    /// 强制：执行前必须匹配哈希
    ///
    /// 严格模式，模块哈希必须与配置完全匹配才能执行。
    /// 提供最强的完整性保护。
    Enforce,
}

/// WASM 运行时安全策略配置（`[runtime.wasm.security]` 配置段）
///
/// 定义 WASM 模块执行的加固安全策略，包括路径验证、符号链接处理、
/// 主机验证和能力控制等。默认启用所有安全检查。
///
/// # 安全默认值
///
/// - 要求工具目录位于工作区内
/// - 拒绝符号链接模块文件
/// - 启用严格主机验证
/// - 默认拒绝能力升级请求
/// - 启用模块哈希警告
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WasmSecurityConfig {
    /// 要求 `runtime.wasm.tools_dir` 必须位于工作区内且无路径穿越
    ///
    /// 启用后验证工具目录路径的安全性：
    /// - 必须是相对路径
    /// - 不能包含 `..` 等路径穿越序列
    /// - 不能指向工作区外部
    #[serde(default = "default_true")]
    pub require_workspace_relative_tools_dir: bool,

    /// 是否拒绝符号链接模块文件
    ///
    /// 启用后在执行前检查模块文件是否为符号链接。
    /// 符号链接可能被利用指向意外的文件，拒绝它们提升安全性。
    #[serde(default = "default_true")]
    pub reject_symlink_modules: bool,

    /// 是否拒绝符号链接工具目录
    ///
    /// 启用后检查 `tools_dir` 本身是否为符号链接。
    /// 防止工具目录被替换为指向其他位置的链接。
    #[serde(default = "default_true")]
    pub reject_symlink_tools_dir: bool,

    /// 是否启用严格主机验证
    ///
    /// 启用后验证 `allowed_hosts` 中的条目格式：
    /// - 仅允许 `host` 或 `host:port` 格式
    /// - 拒绝包含协议前缀（如 `http://`）的条目
    /// - 拒绝包含路径（如 `host/path`）的条目
    #[serde(default = "default_true")]
    pub strict_host_validation: bool,

    /// 能力升级处理策略
    ///
    /// 定义当调用请求的能力超过运行时基线配置时如何处理。
    /// 默认为拒绝模式（`Deny`），提供最强的安全性。
    #[serde(default)]
    pub capability_escalation_mode: WasmCapabilityEscalationMode,

    /// 模块摘要校验策略
    ///
    /// 定义 WASM 模块 SHA-256 摘要的验证方式。
    /// 默认为警告模式（`Warn`），平衡安全性和可用性。
    #[serde(default)]
    pub module_hash_policy: WasmModuleHashPolicy,

    /// 模块 SHA-256 摘要映射表
    ///
    /// 键为模块名称（不含 `.wasm` 后缀），值为期望的 SHA-256 摘要（十六进制字符串）。
    /// 用于验证模块文件的完整性，防止篡改。
    ///
    /// # 示例
    ///
    /// ```toml
    /// [runtime.wasm.security.module_sha256]
    /// "my_tool" = "abc123..."
    /// "another_tool" = "def456..."
    /// ```
    #[serde(default)]
    pub module_sha256: BTreeMap<String, String>,
}

// ============================================================================
// 默认值函数
// ============================================================================

/// 返回默认运行时类型
fn default_runtime_kind() -> String {
    "native".into()
}

/// 返回默认 Docker 镜像
fn default_docker_image() -> String {
    "alpine:3.20".into()
}

/// 返回默认 Docker 网络模式
fn default_docker_network() -> String {
    "none".into()
}

/// 返回默认 Docker 内存限制
fn default_docker_memory_limit_mb() -> Option<u64> {
    Some(512)
}

/// 返回默认 Docker CPU 限制
fn default_docker_cpu_limit() -> Option<f64> {
    Some(1.0)
}

/// 返回默认 WASM 工具目录
fn default_wasm_tools_dir() -> String {
    "tools/wasm".into()
}

/// 返回默认 WASM 燃料限制
fn default_wasm_fuel_limit() -> u64 {
    1_000_000
}

/// 返回默认 WASM 内存限制
fn default_wasm_memory_limit_mb() -> u64 {
    64
}

/// 返回默认 WASM 模块大小限制
fn default_wasm_max_module_size_mb() -> u64 {
    50
}

/// 返回默认布尔值 `true`
fn default_true() -> bool {
    true
}

// ============================================================================
// 默认实现
// ============================================================================

impl Default for DockerRuntimeConfig {
    fn default() -> Self {
        Self {
            image: default_docker_image(),
            network: default_docker_network(),
            memory_limit_mb: default_docker_memory_limit_mb(),
            cpu_limit: default_docker_cpu_limit(),
            read_only_rootfs: true,
            mount_workspace: true,
            allowed_workspace_roots: Vec::new(),
        }
    }
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            tools_dir: default_wasm_tools_dir(),
            fuel_limit: default_wasm_fuel_limit(),
            memory_limit_mb: default_wasm_memory_limit_mb(),
            max_module_size_mb: default_wasm_max_module_size_mb(),
            allow_workspace_read: false,
            allow_workspace_write: false,
            allowed_hosts: Vec::new(),
            security: WasmSecurityConfig::default(),
        }
    }
}

impl Default for WasmSecurityConfig {
    fn default() -> Self {
        Self {
            require_workspace_relative_tools_dir: true,
            reject_symlink_modules: true,
            reject_symlink_tools_dir: true,
            strict_host_validation: true,
            capability_escalation_mode: WasmCapabilityEscalationMode::Deny,
            module_hash_policy: WasmModuleHashPolicy::Warn,
            module_sha256: BTreeMap::new(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            kind: default_runtime_kind(),
            docker: DockerRuntimeConfig::default(),
            wasm: WasmRuntimeConfig::default(),
            reasoning_enabled: None,
            reasoning_level: None,
        }
    }
}
#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
