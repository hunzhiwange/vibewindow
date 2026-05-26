//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use std::collections::BTreeMap;

use crate::app::config::update_runtime_config_async;
use crate::app::{App, Message};
use iced::Task;
use vw_config_types::runtime::{RuntimeConfig, WasmCapabilityEscalationMode, WasmModuleHashPolicy};

use super::messages::{RuntimeMessage, SettingsMessage};

fn normalize_runtime_kind(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "native" | "docker" | "wasm" => raw.trim().to_ascii_lowercase(),
        _ => "native".to_string(),
    }
}

fn normalize_reasoning_enabled(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" => "true".to_string(),
        "false" => "false".to_string(),
        _ => "auto".to_string(),
    }
}

fn parse_reasoning_level(input: &str) -> Result<Option<String>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let normalized = trimmed.to_ascii_lowercase().replace(['-', '_'], "");
    match normalized.as_str() {
        "minimal" | "low" | "medium" | "high" | "xhigh" => Ok(Some(normalized)),
        _ => Err("reasoning_level 仅支持 minimal / low / medium / high / xhigh".to_string()),
    }
}

fn parse_optional_u64(input: &str, field: &str) -> Result<Option<u64>, String> {
    let value = input.trim();
    if value.is_empty() {
        return Ok(None);
    }
    value.parse::<u64>().map(Some).map_err(|_| format!("{field} 必须是非负整数"))
}

fn parse_optional_f64(input: &str, field: &str) -> Result<Option<f64>, String> {
    let value = input.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let parsed = value.parse::<f64>().map_err(|_| format!("{field} 必须是数字"))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err(format!("{field} 必须大于 0"));
    }
    Ok(Some(parsed))
}

fn parse_required_u64(input: &str, field: &str, min: u64, max: u64) -> Result<u64, String> {
    let parsed = input.trim().parse::<u64>().map_err(|_| format!("{field} 必须是整数"))?;
    Ok(parsed.clamp(min, max))
}

fn parse_csv_lines(input: &str) -> Vec<String> {
    input
        .split([',', '\n'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_module_sha256_map(input: &str) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    for (index, line) in input.split([',', '\n']).enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((module, hash)) = line.split_once(':') else {
            return Err(format!("第 {} 行模块摘要必须使用 module:sha256 格式", index + 1));
        };

        let module = module.trim();
        let hash = hash.trim();
        if module.is_empty() || hash.is_empty() {
            return Err(format!("第 {} 行模块摘要不能为空", index + 1));
        }

        map.insert(module.to_string(), hash.to_string());
    }
    Ok(map)
}

fn normalize_capability_mode(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "clamp" => "clamp".to_string(),
        _ => "deny".to_string(),
    }
}

fn normalize_module_hash_policy(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "disabled" => "disabled".to_string(),
        "enforce" => "enforce".to_string(),
        _ => "warn".to_string(),
    }
}

fn persist_runtime_settings(app: &mut App) -> Result<Task<Message>, String> {
    let s = &app.runtime_settings;
    let kind = normalize_runtime_kind(&s.kind);
    let docker_image = s.docker_image.trim().to_string();
    let docker_network = s.docker_network.trim().to_string();
    let docker_read_only_rootfs = s.docker_read_only_rootfs;
    let docker_mount_workspace = s.docker_mount_workspace;
    let docker_allowed_workspace_roots = parse_csv_lines(&s.docker_allowed_workspace_roots_input);
    let wasm_tools_dir = s.wasm_tools_dir.trim().to_string();
    let wasm_allow_workspace_read = s.wasm_allow_workspace_read;
    let wasm_allow_workspace_write = s.wasm_allow_workspace_write;
    let wasm_allowed_hosts = parse_csv_lines(&s.wasm_allowed_hosts_input);
    let wasm_require_workspace_relative_tools_dir = s.wasm_require_workspace_relative_tools_dir;
    let wasm_reject_symlink_modules = s.wasm_reject_symlink_modules;
    let wasm_reject_symlink_tools_dir = s.wasm_reject_symlink_tools_dir;
    let wasm_strict_host_validation = s.wasm_strict_host_validation;
    let docker_memory_limit_mb =
        parse_optional_u64(&s.docker_memory_limit_mb_input, "Docker 内存限制")?;
    let docker_cpu_limit = parse_optional_f64(&s.docker_cpu_limit_input, "Docker CPU 限制")?;
    let wasm_fuel_limit =
        parse_required_u64(&s.wasm_fuel_limit_input, "WASM fuel_limit", 1, 100_000_000)?;
    let wasm_memory_limit_mb =
        parse_required_u64(&s.wasm_memory_limit_mb_input, "WASM memory_limit_mb", 1, 4096)?;
    let wasm_max_module_size_mb =
        parse_required_u64(&s.wasm_max_module_size_mb_input, "WASM max_module_size_mb", 1, 4096)?;
    let capability_mode = normalize_capability_mode(&s.wasm_capability_escalation_mode);
    let module_hash_policy = normalize_module_hash_policy(&s.wasm_module_hash_policy);
    let module_sha256 = parse_module_sha256_map(&s.wasm_module_sha256_input)?;
    let reasoning_enabled = match normalize_reasoning_enabled(&s.reasoning_enabled_input).as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    };
    let reasoning_level = parse_reasoning_level(&s.reasoning_level_input)?;

    Ok(update_runtime_config_async(move |runtime| {
        *runtime = RuntimeConfig::default();
        runtime.kind = kind.clone();
        runtime.docker.image = if docker_image.is_empty() {
            RuntimeConfig::default().docker.image
        } else {
            docker_image
        };
        runtime.docker.network = if docker_network.is_empty() {
            RuntimeConfig::default().docker.network
        } else {
            docker_network
        };
        runtime.docker.memory_limit_mb = docker_memory_limit_mb;
        runtime.docker.cpu_limit = docker_cpu_limit;
        runtime.docker.read_only_rootfs = docker_read_only_rootfs;
        runtime.docker.mount_workspace = docker_mount_workspace;
        runtime.docker.allowed_workspace_roots = docker_allowed_workspace_roots;

        runtime.wasm.tools_dir = if wasm_tools_dir.is_empty() {
            RuntimeConfig::default().wasm.tools_dir
        } else {
            wasm_tools_dir
        };
        runtime.wasm.fuel_limit = wasm_fuel_limit;
        runtime.wasm.memory_limit_mb = wasm_memory_limit_mb;
        runtime.wasm.max_module_size_mb = wasm_max_module_size_mb;
        runtime.wasm.allow_workspace_read = wasm_allow_workspace_read;
        runtime.wasm.allow_workspace_write = wasm_allow_workspace_write;
        runtime.wasm.allowed_hosts = wasm_allowed_hosts;
        runtime.wasm.security.require_workspace_relative_tools_dir =
            wasm_require_workspace_relative_tools_dir;
        runtime.wasm.security.reject_symlink_modules = wasm_reject_symlink_modules;
        runtime.wasm.security.reject_symlink_tools_dir = wasm_reject_symlink_tools_dir;
        runtime.wasm.security.strict_host_validation = wasm_strict_host_validation;
        runtime.wasm.security.capability_escalation_mode = if capability_mode == "clamp" {
            WasmCapabilityEscalationMode::Clamp
        } else {
            WasmCapabilityEscalationMode::Deny
        };
        runtime.wasm.security.module_hash_policy = match module_hash_policy.as_str() {
            "disabled" => WasmModuleHashPolicy::Disabled,
            "enforce" => WasmModuleHashPolicy::Enforce,
            _ => WasmModuleHashPolicy::Warn,
        };
        runtime.wasm.security.module_sha256 = module_sha256.clone();
        runtime.reasoning_enabled = reasoning_enabled;
        runtime.reasoning_level = reasoning_level.clone();
    }))
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    let SettingsMessage::Runtime(message) = message else {
        return Task::none();
    };

    if matches!(message, RuntimeMessage::Refresh) {
        app.runtime_settings.save_error = None;
        return Task::none();
    }

    match message {
        RuntimeMessage::KindChanged(value) => {
            app.runtime_settings.kind = normalize_runtime_kind(&value)
        }
        RuntimeMessage::DockerImageChanged(value) => app.runtime_settings.docker_image = value,
        RuntimeMessage::DockerNetworkChanged(value) => app.runtime_settings.docker_network = value,
        RuntimeMessage::DockerMemoryLimitMbChanged(value) => {
            app.runtime_settings.docker_memory_limit_mb_input = value
        }
        RuntimeMessage::DockerCpuLimitChanged(value) => {
            app.runtime_settings.docker_cpu_limit_input = value
        }
        RuntimeMessage::DockerReadOnlyRootfsToggled(value) => {
            app.runtime_settings.docker_read_only_rootfs = value
        }
        RuntimeMessage::DockerMountWorkspaceToggled(value) => {
            app.runtime_settings.docker_mount_workspace = value
        }
        RuntimeMessage::DockerAllowedWorkspaceRootsChanged(value) => {
            app.runtime_settings.docker_allowed_workspace_roots_input = value
        }
        RuntimeMessage::WasmToolsDirChanged(value) => app.runtime_settings.wasm_tools_dir = value,
        RuntimeMessage::WasmFuelLimitChanged(value) => {
            app.runtime_settings.wasm_fuel_limit_input = value
        }
        RuntimeMessage::WasmMemoryLimitMbChanged(value) => {
            app.runtime_settings.wasm_memory_limit_mb_input = value
        }
        RuntimeMessage::WasmMaxModuleSizeMbChanged(value) => {
            app.runtime_settings.wasm_max_module_size_mb_input = value
        }
        RuntimeMessage::WasmAllowWorkspaceReadToggled(value) => {
            app.runtime_settings.wasm_allow_workspace_read = value
        }
        RuntimeMessage::WasmAllowWorkspaceWriteToggled(value) => {
            app.runtime_settings.wasm_allow_workspace_write = value
        }
        RuntimeMessage::WasmAllowedHostsChanged(value) => {
            app.runtime_settings.wasm_allowed_hosts_input = value
        }
        RuntimeMessage::WasmRequireWorkspaceRelativeToolsDirToggled(value) => {
            app.runtime_settings.wasm_require_workspace_relative_tools_dir = value
        }
        RuntimeMessage::WasmRejectSymlinkModulesToggled(value) => {
            app.runtime_settings.wasm_reject_symlink_modules = value
        }
        RuntimeMessage::WasmRejectSymlinkToolsDirToggled(value) => {
            app.runtime_settings.wasm_reject_symlink_tools_dir = value
        }
        RuntimeMessage::WasmStrictHostValidationToggled(value) => {
            app.runtime_settings.wasm_strict_host_validation = value
        }
        RuntimeMessage::WasmCapabilityEscalationModeChanged(value) => {
            app.runtime_settings.wasm_capability_escalation_mode = normalize_capability_mode(&value)
        }
        RuntimeMessage::WasmModuleHashPolicyChanged(value) => {
            app.runtime_settings.wasm_module_hash_policy = normalize_module_hash_policy(&value)
        }
        RuntimeMessage::WasmModuleSha256Changed(value) => {
            app.runtime_settings.wasm_module_sha256_input = value
        }
        RuntimeMessage::ReasoningEnabledChanged(value) => {
            app.runtime_settings.reasoning_enabled_input = normalize_reasoning_enabled(&value)
        }
        RuntimeMessage::ReasoningLevelChanged(value) => {
            app.runtime_settings.reasoning_level_input = value
        }
        RuntimeMessage::Refresh => unreachable!(),
    }

    match persist_runtime_settings(app) {
        Ok(task) => {
            app.runtime_settings.save_error = None;
            task
        }
        Err(err) => {
            app.runtime_settings.save_error = Some(err);
            Task::none()
        }
    }
}
#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
