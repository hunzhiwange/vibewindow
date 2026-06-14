#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("logging_tests"));
}

use super::logging;
use crate::app::task::{TaskExecutorBackend, TaskLogStream};
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::{
    DesignChatRole, DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus,
    DesignState,
};
use std::sync::mpsc;

fn state_with_module() -> DesignState {
    let mut state = DesignState::new(DesignDoc::default());
    state.design_generation_pages = vec![DesignGenerationPage {
        frame_id: "design-page-0".to_string(),
        title: "首页".to_string(),
        objective: "展示入口".to_string(),
        status: DesignGenerationStatus::Queued,
        modules: vec![DesignGenerationModule {
            module_id: "page-0-module-0".to_string(),
            title: "Hero".to_string(),
            description: "主视觉".to_string(),
            status: DesignGenerationStatus::Queued,
            target_frame_id: "page-0-module-0".to_string(),
            target_frame_options: vec!["page-0-module-0".to_string()],
            generated_doc: None,
            is_generating: false,
            logs: Vec::new(),
        }],
    }];
    state
}

#[test]
fn push_generation_log_skips_sensitive_process_lines() {
    let mut state = DesignState::new(DesignDoc::default());

    logging::push_design_generation_log(&mut state, "[EXEC_STDIN] secret prompt");
    logging::push_design_generation_log(&mut state, "opencode request payload");

    assert!(state.design_generation_logs.is_empty());
}

#[test]
fn push_generation_log_truncates_and_keeps_recent_entries() {
    let mut state = DesignState::new(DesignDoc::default());
    let long = "a".repeat(400);

    logging::push_design_generation_log(&mut state, long);
    for index in 0..250 {
        logging::push_design_generation_log(&mut state, format!("line-{index}"));
    }

    assert_eq!(state.design_generation_logs.len(), 240);
    assert_eq!(state.design_generation_logs.first().map(String::as_str), Some("line-10"));
    assert!(state.design_generation_log_editor.text().contains("line-249"));
}

#[test]
fn push_module_log_updates_module_and_global_logs() {
    let mut state = state_with_module();

    logging::push_module_log(&mut state, "design-page-0", "page-0-module-0", "module line");

    assert_eq!(state.design_generation_pages[0].modules[0].logs, ["module line"]);
    assert_eq!(state.design_generation_logs, ["module line"]);
}

#[test]
fn format_log_stream_covers_all_variants() {
    assert_eq!(
        logging::format_design_log_stream(&TaskLogStream::Stdout(" ok \n".to_string())),
        Some("ok".to_string())
    );
    assert_eq!(logging::format_design_log_stream(&TaskLogStream::Stderr("   ".to_string())), None);
    assert_eq!(
        logging::format_design_log_stream(&TaskLogStream::SubTaskStarted {
            subtask_id: "1".to_string(),
            content: "scan".to_string(),
        }),
        Some("[SUBTASK] start scan".to_string())
    );
    assert_eq!(
        logging::format_design_log_stream(&TaskLogStream::SubTaskCompleted {
            subtask_id: "1".to_string(),
        }),
        Some("[SUBTASK] completed 1".to_string())
    );
    assert_eq!(
        logging::format_design_log_stream(&TaskLogStream::SubTaskFailed {
            subtask_id: "1".to_string(),
            error: "boom".to_string(),
        }),
        Some("[SUBTASK] failed 1 boom".to_string())
    );
    assert_eq!(
        logging::format_design_log_stream(&TaskLogStream::ExitStatus {
            success: true,
            code: Some(0),
            signal: None,
        }),
        Some("[EXEC_EXIT] success code=Some(0)".to_string())
    );
    assert_eq!(
        logging::format_design_log_stream(&TaskLogStream::ExitStatus {
            success: false,
            code: Some(1),
            signal: Some(9),
        }),
        Some("[EXEC_EXIT] failed code=Some(1) signal=Some(9)".to_string())
    );
}

#[test]
fn chat_stream_failures_merge_repeated_steps() {
    let mut state = DesignState::new(DesignDoc::default());
    state.design_chat_messages.clear();
    let lines = vec![
        "[page] stderr: failed".to_string(),
        "[page] stderr: failed".to_string(),
        "[page] [EXEC_EXIT] failed code=Some(1) signal=None".to_string(),
        "[page] [EXEC_EXIT] success code=Some(0)".to_string(),
    ];

    logging::push_design_stream_to_chat(&mut state, &lines, 10);
    logging::push_design_stream_line_to_chat(&mut state, "[page] stderr: failed");

    assert_eq!(state.design_chat_messages.len(), 3);
    assert_eq!(state.design_chat_messages[0].role, DesignChatRole::Assistant);
    assert_eq!(state.design_chat_messages[0].content, "Step failed: stderr: failed ×2");
    assert!(state.design_chat_messages.iter().any(|message| message.content.contains("failed")));
}

#[test]
fn collect_design_log_lines_drains_receiver_with_scope() {
    let (tx, rx) = mpsc::channel();
    tx.send(TaskLogStream::Stdout("hello".to_string())).unwrap();
    tx.send(TaskLogStream::Stderr(" ".to_string())).unwrap();
    tx.send(TaskLogStream::ExitStatus { success: true, code: Some(0), signal: None }).unwrap();

    let lines = logging::collect_design_log_lines("page:home", &rx);

    assert_eq!(lines, ["[page:home] hello", "[page:home] [EXEC_EXIT] success code=Some(0)"]);
}

#[test]
fn executor_step_label_uses_backend_label() {
    assert_eq!(logging::executor_step_label(TaskExecutorBackend::Codex), "Calling tool: Codex CLI");
}
