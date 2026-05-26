//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use crate::app::{App, Message};
use iced::Task;

use super::messages::SettingsMessage;
fn normalize_observability_backend(raw: &str) -> String {
    let v = raw.trim().to_ascii_lowercase();
    match v.as_str() {
        "none" | "log" | "prometheus" | "otel" => v,
        _ => "none".to_string(),
    }
}

#[cfg(test)]
#[path = "observability_tests.rs"]
mod observability_tests;

fn normalize_runtime_trace_mode(raw: &str) -> String {
    let v = raw.trim().to_ascii_lowercase();
    match v.as_str() {
        "none" | "rolling" | "full" => v,
        _ => "none".to_string(),
    }
}

fn persist_observability_settings(app: &mut App) -> Task<Message> {
    let s = &app.observability_settings;
    let backend = normalize_observability_backend(&s.backend);
    let otel_endpoint = s.otel_endpoint_input.trim().to_string();
    let otel_service_name = s.otel_service_name_input.trim().to_string();
    let runtime_trace_mode = normalize_runtime_trace_mode(&s.runtime_trace_mode);
    let runtime_trace_path = s.runtime_trace_path_input.trim().to_string();
    let runtime_trace_max_entries = s.runtime_trace_max_entries.clamp(1, 100_000) as usize;

    crate::app::update_observability_config_async(move |observability| {
        observability.backend = backend;
        observability.otel_endpoint =
            if otel_endpoint.is_empty() { None } else { Some(otel_endpoint) };
        observability.otel_service_name =
            if otel_service_name.is_empty() { None } else { Some(otel_service_name) };
        observability.runtime_trace_mode = runtime_trace_mode;
        observability.runtime_trace_path = if runtime_trace_path.is_empty() {
            "state/runtime-trace.jsonl".to_string()
        } else {
            runtime_trace_path
        };
        observability.runtime_trace_max_entries = runtime_trace_max_entries;
    })
}

/// 处理 `update` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn update(app: &mut App, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::ObservabilityBackendChanged(v) => {
            app.observability_settings.backend = v;
            app.observability_settings.backend =
                normalize_observability_backend(&app.observability_settings.backend);
            app.observability_settings.save_error = None;
            persist_observability_settings(app)
        }
        SettingsMessage::ObservabilityOtelEndpointChanged(v) => {
            app.observability_settings.otel_endpoint_input = v;
            app.observability_settings.save_error = None;
            persist_observability_settings(app)
        }
        SettingsMessage::ObservabilityOtelServiceNameChanged(v) => {
            app.observability_settings.otel_service_name_input = v;
            app.observability_settings.save_error = None;
            persist_observability_settings(app)
        }
        SettingsMessage::ObservabilityRuntimeTraceModeChanged(v) => {
            app.observability_settings.runtime_trace_mode = v;
            app.observability_settings.runtime_trace_mode =
                normalize_runtime_trace_mode(&app.observability_settings.runtime_trace_mode);
            app.observability_settings.save_error = None;
            persist_observability_settings(app)
        }
        SettingsMessage::ObservabilityRuntimeTracePathChanged(v) => {
            app.observability_settings.runtime_trace_path_input = v;
            app.observability_settings.save_error = None;
            persist_observability_settings(app)
        }
        SettingsMessage::ObservabilityRuntimeTraceMaxEntriesChanged(v) => {
            app.observability_settings.runtime_trace_max_entries = v.clamp(1, 100_000);
            app.observability_settings.save_error = None;
            persist_observability_settings(app)
        }
        SettingsMessage::ObservabilityHelpOpen => {
            app.observability_settings.show_help_modal = true;
            Task::none()
        }
        SettingsMessage::ObservabilityHelpClose => {
            app.observability_settings.show_help_modal = false;
            Task::none()
        }
        _ => Task::none(),
    }
}
