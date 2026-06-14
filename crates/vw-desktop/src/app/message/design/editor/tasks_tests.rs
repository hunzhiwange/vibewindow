#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("tasks_tests"));
}

use super::tasks;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus,
    DesignGenerationTheme, DesignState,
};

fn module(
    id: &str,
    status: DesignGenerationStatus,
    generated_doc: Option<DesignDoc>,
) -> DesignGenerationModule {
    DesignGenerationModule {
        module_id: id.to_string(),
        title: format!("模块 {id}"),
        description: "描述".to_string(),
        status,
        target_frame_id: id.to_string(),
        target_frame_options: vec![id.to_string()],
        generated_doc,
        is_generating: false,
        logs: Vec::new(),
    }
}

fn page(
    id: &str,
    status: DesignGenerationStatus,
    modules: Vec<DesignGenerationModule>,
) -> DesignGenerationPage {
    DesignGenerationPage {
        frame_id: id.to_string(),
        title: format!("页面 {id}"),
        objective: "目标".to_string(),
        status,
        modules,
    }
}

fn doc_with_root_height(height: serde_json::Value) -> DesignDoc {
    DesignDoc {
        children: vec![DesignElement {
            kind: "frame".to_string(),
            id: "root".to_string(),
            height: Some(height),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn next_queued_pages_respects_limit_and_skips_running_or_generated() {
    let mut state = DesignState::new(DesignDoc::default());
    let mut running = module("running", DesignGenerationStatus::Queued, None);
    running.is_generating = true;
    state.design_generation_pages = vec![
        page("p0", DesignGenerationStatus::Queued, vec![running]),
        page(
            "p1",
            DesignGenerationStatus::Queued,
            vec![module("m1", DesignGenerationStatus::Queued, None)],
        ),
        page(
            "p2",
            DesignGenerationStatus::Queued,
            vec![module("m2", DesignGenerationStatus::Generated, Some(DesignDoc::default()))],
        ),
        page(
            "p3",
            DesignGenerationStatus::Queued,
            vec![module("m3", DesignGenerationStatus::Placeholder, None)],
        ),
    ];

    assert!(tasks::next_queued_generation_pages(&state, 0).is_empty());
    assert_eq!(
        tasks::next_queued_generation_pages(&state, 2),
        [("p1".to_string(), "m1".to_string()), ("p3".to_string(), "m3".to_string())]
    );
}

#[test]
fn running_and_progress_counts_cover_statuses() {
    let mut state = DesignState::new(DesignDoc::default());
    let mut generating = module("m0", DesignGenerationStatus::Queued, None);
    generating.is_generating = true;
    state.design_generation_pages = vec![
        page(
            "p0",
            DesignGenerationStatus::Running,
            vec![
                generating,
                module("m1", DesignGenerationStatus::Filled, None),
                module("m2", DesignGenerationStatus::Failed, None),
                module("m3", DesignGenerationStatus::Generated, Some(DesignDoc::default())),
                module("m4", DesignGenerationStatus::Placeholder, None),
                module("m5", DesignGenerationStatus::Aggregated, None),
            ],
        ),
        page(
            "p1",
            DesignGenerationStatus::Queued,
            vec![module("m6", DesignGenerationStatus::Running, None)],
        ),
    ];
    state.design_generation_parallel_pages = 100;

    assert_eq!(tasks::count_running_generation_pages(&state), 2);
    assert_eq!(tasks::design_page_parallel_limit(&state), 16);
    assert_eq!(tasks::count_generation_progress(&state), (1, 4, 1));
}

#[test]
fn summarize_generated_pages_skips_current_and_counts_filled() {
    let pages = vec![
        page("current", DesignGenerationStatus::Queued, vec![]),
        page(
            "other",
            DesignGenerationStatus::Filled,
            vec![
                module("m1", DesignGenerationStatus::Filled, None),
                module("m2", DesignGenerationStatus::Queued, None),
            ],
        ),
    ];

    let summary = tasks::summarize_generated_pages_for_prompt(&pages, "current");

    assert!(summary.contains("页面:页面 other"));
    assert!(summary.contains("filled_modules=1"));
    assert_eq!(
        tasks::summarize_generated_pages_for_prompt(&pages[..1], "current"),
        "暂无可参考的已完成页面，请严格遵循当前页面目标并保持主题风格一致。"
    );
}

#[test]
fn summarize_page_modules_includes_generated_height_or_unknown() {
    let page = page(
        "p0",
        DesignGenerationStatus::Queued,
        vec![
            module(
                "m1",
                DesignGenerationStatus::Generated,
                Some(doc_with_root_height(serde_json::json!(240))),
            ),
            module("m2", DesignGenerationStatus::Generated, Some(DesignDoc::default())),
            module("m3", DesignGenerationStatus::Queued, None),
        ],
    );

    let summary = tasks::summarize_page_modules_for_prompt(&page);

    assert!(summary.contains("已生成高度≈240px"));
    assert!(summary.contains("已生成高度=unknown"));
    assert!(summary.contains("待生成"));
}

#[test]
fn build_design_plan_canvas_creates_page_and_module_frames() {
    let pages = vec![page(
        "design-page-0",
        DesignGenerationStatus::Queued,
        vec![
            module("module-queued", DesignGenerationStatus::Queued, None),
            module("module-failed", DesignGenerationStatus::Failed, None),
        ],
    )];

    let doc = tasks::build_design_plan_canvas(
        &pages,
        DesignGenerationTheme::Lunaris,
        DesignGenerationDevice::MobileApp,
        Some("summary"),
    );

    assert_eq!(doc.version, "2.6");
    assert_eq!(doc.children[0].id, "design-page-0");
    assert_eq!(doc.children[0].width, Some(serde_json::json!(390.0)));
    assert_eq!(doc.theme.as_ref().map(|theme| theme.mode.as_str()), Some("Dark"));
    assert!(doc.children[0].children.iter().any(|child| child.id == "module-queued"));
    assert!(doc.children[0].children.iter().any(|child| child.id == "module-failed"));
}
