//! 设计生成日志与聊天同步辅助。

use super::canvas::find_generation_page_mut;
use crate::app::task::{TaskExecutorBackend, TaskLogStream};
use crate::app::views::design::state::{DesignChatMessage, DesignChatRole};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;

fn generate_design_log_filename() -> String {
    let now = chrono::Local::now();
    let date_str = now.format("%Y%m%d").to_string();
    let random_suffix: u32 = rand::random();
    format!("{}_{}.log", date_str, random_suffix)
}

pub(super) fn create_design_generation_log_file(project_path: &str) -> Option<String> {
    let logs_dir = Path::new(project_path).join(".vibewindow").join("design").join("logs");
    if std::fs::create_dir_all(&logs_dir).is_err() {
        return None;
    }
    let filename = generate_design_log_filename();
    let log_path = logs_dir.join(&filename);
    if std::fs::File::create(&log_path).is_ok() { Some(filename) } else { None }
}

pub(super) fn append_design_project_log(
    project_path: &str,
    line: impl AsRef<str>,
    current_log_filename: Option<&str>,
) {
    let main_log_path = Path::new(project_path)
        .join(".vibewindow")
        .join("design")
        .join("logs")
        .join("design_generation.log");
    if let Some(parent) = main_log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let ts = crate::app::time::now_ms() as u128;
    let entry = format!("[{}] {}\n", ts, line.as_ref());
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&main_log_path)
        .and_then(|mut file| file.write_all(entry.as_bytes()));

    if let Some(filename) = current_log_filename {
        let session_log_path =
            Path::new(project_path).join(".vibewindow").join("design").join("logs").join(filename);
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&session_log_path)
            .and_then(|mut file| file.write_all(entry.as_bytes()));
    }
}

pub(super) fn push_design_generation_log(
    state: &mut crate::app::views::design::state::DesignState,
    line: impl Into<String>,
) {
    let mut line = line.into();
    if should_skip_generation_process_line(&line) {
        return;
    }
    if line.chars().count() > 280 {
        line = format!("{}…", line.chars().take(280).collect::<String>());
    }
    state.design_generation_logs.push(line);
    if state.design_generation_logs.len() > 240 {
        let overflow = state.design_generation_logs.len() - 240;
        state.design_generation_logs.drain(0..overflow);
    }
    sync_design_generation_log_editor(state);
}

pub(super) fn sync_design_generation_log_editor(
    state: &mut crate::app::views::design::state::DesignState,
) {
    let start = state.design_generation_logs.len().saturating_sub(160);
    let preview = state.design_generation_logs[start..].join("\n");
    state.design_generation_log_editor = iced::widget::text_editor::Content::with_text(&preview);
}

pub(super) fn push_module_log(
    state: &mut crate::app::views::design::state::DesignState,
    page_frame_id: &str,
    module_id: &str,
    line: impl Into<String>,
) {
    let line = line.into();
    if should_skip_generation_process_line(&line) {
        return;
    }
    if let Some(page) = find_generation_page_mut(&mut state.design_generation_pages, page_frame_id)
        && let Some(module) = page.modules.iter_mut().find(|module| module.module_id == module_id)
    {
        module.logs.push(line.clone());
        if module.logs.len() > 120 {
            let overflow = module.logs.len() - 120;
            module.logs.drain(0..overflow);
        }
    }
    push_design_generation_log(state, line);
}

fn should_skip_generation_process_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("[exec_stdin]")
        || (lower.contains("opencode")
            && (lower.contains("request")
                || lower.contains("\"request\"")
                || lower.contains("payload")))
}

#[allow(dead_code)]
fn is_error_log_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("failed")
        || lower.contains("error")
        || lower.contains("stderr")
        || lower.contains("[exec_exit] failed")
}

pub(super) fn format_design_log_stream(log: &TaskLogStream) -> Option<String> {
    match log {
        TaskLogStream::Stdout(line) | TaskLogStream::Stderr(line) => {
            let line = line.trim();
            if line.is_empty() { None } else { Some(line.to_string()) }
        }
        TaskLogStream::ExitStatus { success, code, signal } => Some(if *success {
            format!("[EXEC_EXIT] success code={:?}", code)
        } else {
            format!("[EXEC_EXIT] failed code={:?} signal={:?}", code, signal)
        }),
    }
}

pub(super) fn executor_step_label(executor: TaskExecutorBackend) -> String {
    format!("Calling tool: {}", executor.label())
}

fn merge_or_push_chat_step(
    state: &mut crate::app::views::design::state::DesignState,
    step: String,
) {
    if let Some(last) = state.design_chat_messages.last_mut()
        && matches!(last.role, DesignChatRole::Assistant)
    {
        if last.content == step {
            last.content = format!("{} ×2", step);
            return;
        }
        if let Some((prefix, count_str)) = last.content.rsplit_once(" ×")
            && prefix == step
            && let Ok(count) = count_str.trim().parse::<usize>()
        {
            last.content = format!("{} ×{}", step, count + 1);
            return;
        }
    }
    state
        .design_chat_messages
        .push(DesignChatMessage { role: DesignChatRole::Assistant, content: step });
}

pub(super) fn format_design_stream_line_for_chat(line: &str) -> Option<String> {
    let mut content = line.trim();
    if content.is_empty() {
        return None;
    }
    if let Some((_, suffix)) = content.split_once("] ")
        && content.starts_with('[')
    {
        content = suffix.trim();
    }
    if content.is_empty() {
        return None;
    }
    if let Some(value) = content.strip_prefix("[EXEC_EXIT]") {
        if value.contains("success") {
            return None;
        }
        return Some(format!("Step failed: {}", value.trim()));
    }
    if content.to_ascii_lowercase().contains("error")
        || content.to_ascii_lowercase().contains("failed")
        || content.to_ascii_lowercase().contains("stderr")
    {
        return Some(format!("Step failed: {}", content));
    }
    None
}

pub(super) fn push_design_stream_to_chat(
    state: &mut crate::app::views::design::state::DesignState,
    lines: &[String],
    max_lines: usize,
) {
    let mut emitted = 0usize;
    for line in lines {
        if emitted >= max_lines {
            break;
        }
        if let Some(message) = format_design_stream_line_for_chat(line) {
            merge_or_push_chat_step(state, message);
            emitted += 1;
        }
    }
}

pub(super) fn push_design_stream_line_to_chat(
    state: &mut crate::app::views::design::state::DesignState,
    line: &str,
) {
    if let Some(message) = format_design_stream_line_for_chat(line) {
        merge_or_push_chat_step(state, message);
    }
}

pub(super) fn collect_design_log_lines(
    scope: &str,
    receiver: &mpsc::Receiver<TaskLogStream>,
) -> Vec<String> {
    let mut lines = Vec::new();
    while let Ok(log) = receiver.try_recv() {
        if let Some(line) = format_design_log_stream(&log) {
            lines.push(format!("[{}] {}", scope, line));
        }
    }
    lines
}

