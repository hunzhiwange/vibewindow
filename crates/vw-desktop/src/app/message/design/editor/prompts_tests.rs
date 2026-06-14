#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("prompts_tests"));
}

use super::prompts;
use crate::app::task::TaskExecutorBackend;
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus,
    DesignGenerationTheme, DesignStyle,
};

fn sample_page() -> DesignGenerationPage {
    DesignGenerationPage {
        frame_id: "design-page-0".to_string(),
        title: "首页".to_string(),
        objective: "展示产品入口".to_string(),
        status: DesignGenerationStatus::Queued,
        modules: vec![DesignGenerationModule {
            module_id: "page-0-module-0".to_string(),
            title: "Hero".to_string(),
            description: "展示卖点".to_string(),
            status: DesignGenerationStatus::Queued,
            target_frame_id: "page-0-module-0".to_string(),
            target_frame_options: vec!["page-0-module-0".to_string()],
            generated_doc: None,
            is_generating: false,
            logs: Vec::new(),
        }],
    }
}

#[test]
fn placeholder_and_compact_multiline_are_stable() {
    assert_eq!(prompts::design_model_input_placeholder(), "auto / provider/model / ACP 模型 ID");
    assert_eq!(prompts::compact_multiline(" a \n\n b\t"), "a b");
}

#[test]
fn executor_gateway_and_agent_resolution_match_backend() {
    assert!(prompts::design_executor_uses_gateway(TaskExecutorBackend::Internal));
    assert!(prompts::design_executor_uses_gateway(TaskExecutorBackend::OpenCode));
    assert!(!prompts::design_executor_uses_gateway(TaskExecutorBackend::Claude));
    assert!(!prompts::design_executor_uses_gateway(TaskExecutorBackend::Codex));

    assert_eq!(
        prompts::resolve_design_acp_agent(TaskExecutorBackend::Internal, Some(" codex ")),
        Some("codex".to_string())
    );
    assert_eq!(prompts::resolve_design_acp_agent(TaskExecutorBackend::Claude, Some("codex")), None);
}

#[test]
fn device_resolution_and_plan_width_cover_all_devices() {
    assert_eq!(
        prompts::resolve_design_generation_device(
            DesignGenerationDevice::Auto,
            "做一个 iPhone app"
        ),
        DesignGenerationDevice::MobileApp
    );
    assert_eq!(
        prompts::resolve_design_generation_device(
            DesignGenerationDevice::Auto,
            "做一个 iPad 平板端"
        ),
        DesignGenerationDevice::Tablet
    );
    assert_eq!(
        prompts::resolve_design_generation_device(DesignGenerationDevice::Auto, "做一个 web 后台"),
        DesignGenerationDevice::DesktopWeb
    );
    assert_eq!(
        prompts::resolve_design_generation_device(DesignGenerationDevice::MobileApp, "desktop"),
        DesignGenerationDevice::MobileApp
    );
    assert_eq!(prompts::design_plan_page_width(DesignGenerationDevice::Auto), 420.0);
    assert_eq!(prompts::design_plan_page_width(DesignGenerationDevice::DesktopWeb), 1280.0);
    assert_eq!(prompts::design_plan_page_width(DesignGenerationDevice::MobileApp), 390.0);
    assert_eq!(prompts::design_plan_page_width(DesignGenerationDevice::Tablet), 900.0);
}

#[test]
fn format_plan_parse_error_uses_empty_marker_and_truncates() {
    let empty = prompts::format_plan_parse_error("bad", " \n ");
    assert!(empty.contains("<空输出>"));

    let long = "x".repeat(400);
    let message = prompts::format_plan_parse_error("bad", &long);
    assert!(message.contains("xxx..."));
    assert!(message.len() < 520);
}

#[test]
fn theme_reference_tokens_return_theme_mode() {
    let (_, lunaris_theme) =
        prompts::design_reference_tokens_and_theme(DesignGenerationTheme::Lunaris);
    let (_, shadcn_theme) =
        prompts::design_reference_tokens_and_theme(DesignGenerationTheme::Shadcn);

    assert_eq!(
        lunaris_theme.as_ref().and_then(|value| value.get("Mode")).and_then(|value| value.as_str()),
        Some("Dark")
    );
    assert_eq!(
        shadcn_theme.as_ref().and_then(|value| value.get("Mode")).and_then(|value| value.as_str()),
        Some("Light")
    );
}

#[test]
fn generation_prompt_differs_for_gateway_and_cli_executors() {
    let gateway_prompt = prompts::build_design_generation_prompt(
        "做一个 AI 工具官网",
        TaskExecutorBackend::Internal,
        DesignGenerationTheme::Lunaris,
        DesignStyle::Dark,
        DesignGenerationDevice::Auto,
    );
    assert!(gateway_prompt.contains("输出只允许 JSON"));
    assert!(gateway_prompt.contains("Lunaris 科技"));
    assert!(gateway_prompt.contains("不要输出 module_doc"));

    let cli_prompt = prompts::build_design_generation_prompt(
        "做一个企业官网",
        TaskExecutorBackend::Claude,
        DesignGenerationTheme::Nitro,
        DesignStyle::Business,
        DesignGenerationDevice::DesktopWeb,
    );
    assert!(cli_prompt.contains("不要输出 Markdown 代码块"));
    assert!(cli_prompt.contains("Nitro 企业"));
    assert!(cli_prompt.contains("1200-1440px"));
}

#[test]
fn page_generation_prompt_uses_default_brief_and_executor_branch() {
    let page = sample_page();
    let gateway_prompt = prompts::build_page_generation_prompt(
        " ",
        TaskExecutorBackend::OpenCode,
        DesignGenerationTheme::Halo,
        DesignStyle::Creative,
        DesignGenerationDevice::MobileApp,
        &page,
        "暂无页面",
    );
    assert!(gateway_prompt.contains("项目需求：当前设计需求"));
    assert!(gateway_prompt.contains("当前按页面回调生成"));
    assert!(gateway_prompt.contains("Hero | 展示卖点"));

    let cli_prompt = prompts::build_page_generation_prompt(
        "后台系统",
        TaskExecutorBackend::Codex,
        DesignGenerationTheme::Shadcn,
        DesignStyle::Modern,
        DesignGenerationDevice::Tablet,
        &page,
        "参考页",
    );
    assert!(cli_prompt.contains("当前项目只有一个 .json 文件"));
    assert!(cli_prompt.contains("768-1024px"));
    assert!(cli_prompt.contains("参考页"));
}
