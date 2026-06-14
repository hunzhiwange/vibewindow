use vw_shared::question;
use vw_shared::todo::Todo;

use super::overlay::{
    McpServerTransport, OverlayFocus, OverlayState, UiConfirmOverlay, UiErrorOverlay, UiOverlay,
    UiOverlayKind, UiQuestionOverlay, UiQuestionSurfaceKind, UiSearchOverlay, UiTaskOverlay,
    UiTodoOverlay,
};

#[test]
fn overlay_state_push_pop_and_clear_manage_focus() {
    let mut state = OverlayState::default();
    assert_eq!(state.focus, OverlayFocus::Prompt);
    assert!(state.active().is_none());

    state.push(UiOverlay::Confirm(UiConfirmOverlay {
        title: "Confirm".to_string(),
        body: "Proceed?".to_string(),
        confirm_label: "Yes".to_string(),
        cancel_label: "No".to_string(),
        destructive: false,
    }));
    assert_eq!(state.focus, OverlayFocus::Overlay);
    assert_eq!(state.active().map(UiOverlay::kind), Some(UiOverlayKind::Confirm));

    state.push(UiOverlay::Error(UiErrorOverlay {
        title: "Oops".to_string(),
        message: "failed".to_string(),
        recoverable: true,
    }));
    assert_eq!(state.pop().map(|overlay| overlay.kind()), Some(UiOverlayKind::Error));
    assert_eq!(state.focus, OverlayFocus::Overlay);
    assert_eq!(state.pop().map(|overlay| overlay.kind()), Some(UiOverlayKind::Confirm));
    assert_eq!(state.focus, OverlayFocus::Prompt);
    assert_eq!(state.pop(), None);

    state.push(UiOverlay::Search(UiSearchOverlay::default()));
    state.clear();
    assert!(state.stack.is_empty());
    assert_eq!(state.focus, OverlayFocus::Prompt);
}

#[test]
fn ui_overlay_kind_covers_core_variants() {
    let overlays = [
        UiOverlay::Confirm(UiConfirmOverlay {
            title: String::new(),
            body: String::new(),
            confirm_label: String::new(),
            cancel_label: String::new(),
            destructive: false,
        }),
        UiOverlay::Search(UiSearchOverlay::default()),
        UiOverlay::Question(question_overlay(None, "Need input?", "Choose")),
        UiOverlay::Todo(UiTodoOverlay::default()),
        UiOverlay::Task(UiTaskOverlay::default()),
        UiOverlay::Error(UiErrorOverlay {
            title: "Error".to_string(),
            message: "boom".to_string(),
            recoverable: false,
        }),
    ];
    let expected = [
        UiOverlayKind::Confirm,
        UiOverlayKind::Search,
        UiOverlayKind::Question,
        UiOverlayKind::Todo,
        UiOverlayKind::Task,
        UiOverlayKind::Error,
    ];

    for (overlay, kind) in overlays.iter().zip(expected) {
        assert_eq!(overlay.kind(), kind);
    }
}

#[test]
fn question_overlay_from_request_preserves_prompts_answers_and_tool_context() {
    let request = question::Request {
        id: "req-1".to_string(),
        session_id: "session-1".to_string(),
        questions: vec![
            question::Info {
                question: "Pick one".to_string(),
                header: "Header".to_string(),
                options: vec![question::OptionInfo {
                    label: "A".to_string(),
                    description: "Alpha".to_string(),
                    preview: Some("preview".to_string()),
                }],
                multiple: Some(true),
                custom: Some(true),
            },
            question::Info {
                question: "Second".to_string(),
                header: "More".to_string(),
                options: Vec::new(),
                multiple: None,
                custom: None,
            },
        ],
        tool: Some(question::ToolMeta {
            message_id: "msg-1".to_string(),
            call_id: "call-1".to_string(),
        }),
    };

    let overlay = UiQuestionOverlay::from_request(&request);
    assert_eq!(overlay.request_id, "req-1");
    assert_eq!(overlay.session_id, "session-1");
    assert_eq!(overlay.prompts.len(), 2);
    assert_eq!(overlay.answers, vec![Vec::<String>::new(), Vec::<String>::new()]);
    assert!(overlay.prompts[0].multiple);
    assert!(overlay.prompts[0].allow_custom_input);
    assert_eq!(overlay.prompts[0].options[0].preview.as_deref(), Some("preview"));
    assert!(overlay.is_tool_backed());
    assert_eq!(overlay.surface_kind(), UiQuestionSurfaceKind::ToolFallback);
}

#[test]
fn question_overlay_permission_markers_choose_permission_surface_labels() {
    let overlay = question_overlay(
        Some(question::ToolMeta { message_id: "m".to_string(), call_id: "c".to_string() }),
        "Permission required",
        "Allow once",
    );

    assert!(overlay.is_permission_request());
    assert_eq!(overlay.surface_kind(), UiQuestionSurfaceKind::PermissionRequest);
    assert_eq!(overlay.modal_title(), "权限请求");
    assert_eq!(overlay.request_label(), "权限请求");
    assert_eq!(overlay.reply_error_title(), "权限请求回复失败");
    assert_eq!(overlay.reject_error_title(), "权限请求拒绝失败");
    assert_eq!(overlay.empty_submission_title(), "提交权限请求回复");
    assert!(overlay.empty_submission_message().contains("授权选项"));

    let plain = question_overlay(None, "Need input?", "Continue");
    assert!(!plain.is_tool_backed());
    assert_eq!(plain.surface_kind(), UiQuestionSurfaceKind::Question);
    assert_eq!(plain.modal_title(), "提问");
}

#[test]
fn todo_overlay_from_todos_maps_shared_items_and_defaults() {
    let todos = vec![Todo {
        id: "1".to_string(),
        content: "write tests".to_string(),
        status: "in_progress".to_string(),
        priority: "high".to_string(),
    }];

    let overlay = UiTodoOverlay::from_todos(Some("session-1"), &todos);
    assert_eq!(overlay.session_id.as_deref(), Some("session-1"));
    assert_eq!(overlay.items.len(), 1);
    assert_eq!(overlay.items[0].id, "1");
    assert_eq!(overlay.items[0].content, "write tests");
    assert_eq!(overlay.items[0].status, "in_progress");
    assert_eq!(overlay.items[0].priority, "high");
    assert_eq!(overlay.selected_index, 0);
    assert!(!overlay.dirty);
}

#[test]
fn mcp_transport_labels_are_stable() {
    assert_eq!(McpServerTransport::Stdio.label(), "stdio");
    assert_eq!(McpServerTransport::Sse.label(), "sse");
    assert_eq!(McpServerTransport::Http.label(), "http");
}

fn question_overlay(
    tool: Option<question::ToolMeta>,
    header: &str,
    option_label: &str,
) -> UiQuestionOverlay {
    UiQuestionOverlay::from_request(&question::Request {
        id: "req".to_string(),
        session_id: "session".to_string(),
        questions: vec![question::Info {
            question: "Question?".to_string(),
            header: header.to_string(),
            options: vec![question::OptionInfo {
                label: option_label.to_string(),
                description: "Description".to_string(),
                preview: None,
            }],
            multiple: None,
            custom: None,
        }],
        tool,
    })
}
