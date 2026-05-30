//! 为 crates/vw-desktop/src/app/message/design/editor/tests.rs 提供消息处理或测试辅助逻辑。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::canvas::{find_generation_module_index, normalize_target_frame_id};
use super::logging::{executor_step_label, format_design_stream_line_for_chat};
use super::parser::{parse_design_generation_module_doc, parse_design_generation_pages};
use super::prompts::{
    build_design_generation_prompt, build_page_generation_prompt, format_plan_parse_error,
};
use super::tasks::build_design_plan_canvas;
use crate::app::task::TaskExecutorBackend;
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationModule, DesignGenerationPage, DesignGenerationPlan,
    DesignGenerationStatus, DesignGenerationTheme, DesignStyle,
};

#[test]
fn parse_design_generation_pages_extracts_json_from_code_fence() {
    let raw = r#"这里是设计说明

```json
[
  {
    "title": "商城首页",
    "objective": "展示主促销与推荐商品",
    "modules": [
      {
        "title": "首屏 Banner",
        "description": "突出品牌与主促销"
      },
      {
        "title": "推荐商品",
        "description": "展示热卖商品卡片"
      }
    ]
  }
]
```
"#;

    let plan = parse_design_generation_pages(raw, DesignGenerationTheme::Shadcn)
        .expect("should parse fenced json");
    assert_eq!(plan.pages.len(), 1);
    assert_eq!(plan.pages[0].title, "商城首页");
    assert_eq!(plan.pages[0].modules.len(), 2);
}

#[test]
fn parse_design_generation_pages_supports_alias_fields() {
    let raw = r#"{
  "summary": "企业官网信息架构",
  "site_map": [
    {
      "page_title": "首页",
      "goal": "展示品牌价值和核心服务",
      "components": [
        {
          "module_title": "Hero 区",
          "desc": "突出价值主张与主 CTA"
        }
      ]
    }
  ]
}"#;

    let plan = parse_design_generation_pages(raw, DesignGenerationTheme::Shadcn)
        .expect("should parse alias json");
    assert_eq!(plan.pages.len(), 1);
    assert_eq!(plan.pages[0].title, "首页");
    assert_eq!(plan.pages[0].objective, "展示品牌价值和核心服务");
    assert_eq!(plan.pages[0].modules.len(), 1);
    assert_eq!(plan.pages[0].modules[0].title, "Hero 区");
}

#[test]
fn parse_design_generation_pages_keeps_searching_after_empty_pages_envelope() {
    let raw = r#"{
  "summary": "站点结构",
  "pages": [],
  "plan": [
    {
      "title": "产品页",
      "objective": "展示产品能力与功能细节",
      "modules": [
        {
          "title": "能力矩阵",
          "description": "按场景展示核心能力"
        }
      ]
    }
  ]
}"#;

    let plan = parse_design_generation_pages(raw, DesignGenerationTheme::Shadcn)
        .expect("should parse nested plan");
    assert_eq!(plan.pages.len(), 1);
    assert_eq!(plan.pages[0].title, "产品页");
    assert_eq!(plan.pages[0].modules.len(), 1);
}

#[test]
fn parse_design_generation_pages_supports_claude_stream_result_field() {
    let raw = r#"{"type":"assistant","result":"{\"summary\":\"电商信息架构\",\"pages\":[{\"title\":\"首页\",\"objective\":\"展示核心商品入口\",\"modules\":[{\"title\":\"首屏Banner\",\"description\":\"展示主视觉与CTA\"}]}]}"}"#;

    let plan = parse_design_generation_pages(raw, DesignGenerationTheme::Shadcn)
        .expect("should parse stream-json result field");
    assert_eq!(plan.pages.len(), 1);
    assert_eq!(plan.pages[0].title, "首页");
    assert_eq!(plan.pages[0].modules.len(), 1);
}

#[test]
fn parse_design_generation_pages_supports_opencode_part_text_field() {
    let raw = r#"{"type":"text","part":{"type":"text","text":"{\"summary\":\"站点结构\",\"pages\":[{\"title\":\"首页\",\"objective\":\"展示核心入口\",\"modules\":[{\"title\":\"主视觉\",\"description\":\"展示卖点与CTA\"}]}]}"}}"#;

    let plan = parse_design_generation_pages(raw, DesignGenerationTheme::Shadcn)
        .expect("should parse opencode part text json");
    assert_eq!(plan.pages.len(), 1);
    assert_eq!(plan.pages[0].title, "首页");
    assert_eq!(plan.pages[0].modules.len(), 1);
}

#[test]
fn format_plan_parse_error_includes_truncated_raw_snippet() {
    let raw = "```json\n{\"pages\":[]}\n```";
    let message = format_plan_parse_error("生成结果为空，未得到可用页面计划", raw);
    assert!(message.contains("页面计划解析失败"));
    assert!(message.contains("原始输出片段（截断）"));
    assert!(message.contains("\"pages\":[]"));
}

#[test]
fn build_page_generation_prompt_keeps_user_brief_generic() {
    let page = DesignGenerationPage {
        frame_id: "design-page-0".to_string(),
        title: "内容页".to_string(),
        objective: "展示主要内容结构".to_string(),
        status: DesignGenerationStatus::Placeholder,
        modules: vec![DesignGenerationModule {
            module_id: "page-0-module-0".to_string(),
            title: "主体内容区".to_string(),
            description: "展示主要内容和必要交互。".to_string(),
            status: DesignGenerationStatus::Queued,
            target_frame_id: "page-0-module-0".to_string(),
            target_frame_options: vec!["page-0-module-0".to_string()],
            generated_doc: None,
            is_generating: false,
            logs: Vec::new(),
        }],
    };
    let prompt = build_page_generation_prompt(
        "做一个通用网站，业务信息由用户自己定义",
        TaskExecutorBackend::Internal,
        DesignGenerationTheme::Shadcn,
        DesignStyle::Tech,
        DesignGenerationDevice::Auto,
        &page,
        "暂无可参考页面",
    );

    assert!(prompt.contains("业务信息由用户自己定义"));
    assert!(prompt.contains("当前按页面回调生成"));
    assert!(prompt.contains("页面标题: 内容页"));
    assert!(prompt.contains("当前按页面回调生成"));
    assert!(prompt.contains("主题参考摘要"));
    assert!(prompt.contains("当前页面模块清单"));
    assert!(!prompt.contains("电商网站"));
}

#[test]
fn build_design_generation_prompt_includes_mobile_width_hint() {
    let prompt = build_design_generation_prompt(
        "做一个移动端APP首页",
        TaskExecutorBackend::Internal,
        DesignGenerationTheme::Shadcn,
        DesignStyle::Tech,
        DesignGenerationDevice::Auto,
    );
    assert!(prompt.contains("端类型宽度策略"));
    assert!(prompt.contains("移动端 / APP"));
    assert!(prompt.contains("360-430px"));
}

#[test]
fn format_design_stream_line_for_chat_only_keeps_failures() {
    assert_eq!(format_design_stream_line_for_chat("[plan] [EXEC_EXIT] success code=Some(0)"), None);
    assert_eq!(format_design_stream_line_for_chat("[plan] [EXEC_STDIN] full prompt body"), None);
    assert_eq!(
        format_design_stream_line_for_chat("[plan] stderr: failed to execute"),
        Some("Step failed: stderr: failed to execute".to_string())
    );
}

#[test]
fn executor_step_label_uses_executor_name() {
    assert_eq!(executor_step_label(TaskExecutorBackend::Internal), "Calling tool: ACP 智能体");
}

#[test]
fn parse_design_generation_module_doc_tolerates_invalid_children_type() {
    let raw = r#"{"children":"run"}"#;
    let doc = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect("should tolerate invalid children type");
    assert!(doc.children.is_empty());
}

#[test]
fn parse_design_generation_module_doc_tolerates_invalid_children_items() {
    let raw = r#"{"children":[{"type":"frame","id":"module-root"},"run"]}"#;
    let doc = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect("should tolerate invalid children item");
    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.children[0].id, "module-root");
}

#[test]
fn parse_design_generation_pages_ignores_module_doc_payload() {
    let raw = r#"{
  "summary": "营销站结构",
  "pages": [
    {
      "title": "首页",
      "objective": "展示品牌与主转化入口",
      "modules": [
        {
          "title": "首屏模块",
          "description": "展示卖点与 CTA",
          "module_doc": {
            "version": "2.6",
            "children": [
              {
                "type": "frame",
                "id": "hero-root",
                "name": "Hero Root"
              }
            ],
            "variables": {},
            "theme": { "Mode": "Light" }
          }
        }
      ]
    }
  ]
}"#;

    let plan = parse_design_generation_pages(raw, DesignGenerationTheme::Shadcn)
        .expect("should parse plan");
    assert_eq!(plan.pages.len(), 1);
    assert_eq!(plan.pages[0].modules.len(), 1);
    assert_eq!(plan.pages[0].status, DesignGenerationStatus::Queued);
    assert_eq!(plan.pages[0].modules[0].status, DesignGenerationStatus::Queued);
    assert!(plan.pages[0].modules[0].generated_doc.is_none());
}

#[test]
fn build_design_plan_canvas_uses_page_frames_as_doc_children() {
    let plan = DesignGenerationPlan {
        summary: Some("生成多页面官网项目".to_string()),
        pages: vec![DesignGenerationPage {
            frame_id: "design-page-0".to_string(),
            title: "首页".to_string(),
            objective: "展示品牌和主转化入口".to_string(),
            status: DesignGenerationStatus::Placeholder,
            modules: vec![DesignGenerationModule {
                module_id: "page-0-module-0".to_string(),
                title: "品牌首屏".to_string(),
                description: "展示价值主张、视觉焦点和 CTA。".to_string(),
                status: DesignGenerationStatus::Placeholder,
                target_frame_id: "page-0-module-0".to_string(),
                target_frame_options: vec!["page-0-module-0".to_string()],
                generated_doc: None,
                is_generating: false,
                logs: Vec::new(),
            }],
        }],
    };
    let doc = build_design_plan_canvas(
        &plan.pages,
        DesignGenerationTheme::Nitro,
        DesignGenerationDevice::DesktopWeb,
        plan.summary.as_deref(),
    );

    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.children[0].id, "design-page-0");
    assert_eq!(doc.theme.as_ref().map(|theme| theme.mode.as_str()), Some("Light"));
}

#[test]
fn normalize_target_frame_id_extracts_embedded_id_field() {
    let raw = r#""id": "cmp-button-primary","#;
    assert_eq!(normalize_target_frame_id(raw), "cmp-button-primary");
}

#[test]
fn parse_design_generation_module_doc_supports_part_text_and_patches_root_fields() {
    let raw = r#"{"type":"text","part":{"type":"text","text":"{\"children\":[{\"type\":\"frame\",\"id\":\"module-root\",\"name\":\"模块根容器\"}]}"}}"#;
    let doc = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect("should parse module doc from part.text");
    assert_eq!(doc.version, "2.6");
    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.children[0].id, "module-root");
}

#[test]
fn parse_design_generation_module_doc_supports_stream_logs_fallback() {
    let raw = r#"{"type":"step_start"}"#;
    let logs = vec![r#"[module] [OPENCODE_RAW] {"type":"text","part":{"type":"text","text":"{\"children\":[{\"type\":\"frame\",\"id\":\"module-root-2\"}]}"}}"#.to_string()];
    let doc = parse_design_generation_module_doc(&raw, &logs, DesignGenerationTheme::Shadcn)
        .expect("should parse module doc from stream logs");
    assert_eq!(doc.version, "2.6");
    assert_eq!(doc.children[0].id, "module-root-2");
}

#[test]
fn parse_design_generation_module_doc_supports_markdown_fenced_part_text() {
    let raw = r#"{"type":"text","part":{"type":"text","text":"```json\n{\"children\":[{\"type\":\"frame\",\"id\":\"module-root-3\",\"name\":\"模块根容器\"}]}\n```"}}"#;
    let doc = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect("should parse module doc from fenced json");
    assert_eq!(doc.version, "2.6");
    assert_eq!(doc.children[0].id, "module-root-3");
}

#[test]
fn parse_design_generation_module_doc_patches_missing_ids_and_kind() {
    let raw = r#"{"children":[{"kind":"frame","children":[{"type":"text","content":"标题"}]}]}"#;
    let doc = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect("should patch missing ids");
    assert_eq!(doc.children[0].kind, "frame");
    assert!(!doc.children[0].id.is_empty());
    assert!(!doc.children[0].children[0].id.is_empty());
}

#[test]
fn parse_design_generation_module_doc_supports_stroke_string() {
    let raw = r#"{"children":[{"type":"frame","id":"module-root-4","stroke":"$--border"}]}"#;
    let doc = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect("should convert stroke string");
    let stroke_fill = doc.children[0].stroke.as_ref().and_then(|stroke| stroke.fill.as_deref());
    assert_eq!(stroke_fill, Some("$--border"));
}

#[test]
fn parse_design_generation_module_doc_rejects_protocol_event_payload() {
    let raw = r#"{"type":"step_start","timestamp":1774180239421,"sessionID":"ses_x","part":{"type":"step-start"}}"#;
    let error = parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .expect_err("protocol event payload should be rejected");
    assert!(error.contains("生成结果不是合法模块文档 JSON"));
}

#[test]
fn find_generation_module_index_matches_target_frame_id_fallback() {
    let page = DesignGenerationPage {
        frame_id: "design-page-0".to_string(),
        title: "首页".to_string(),
        objective: "目标".to_string(),
        status: DesignGenerationStatus::Placeholder,
        modules: vec![DesignGenerationModule {
            module_id: "page-0-module-0".to_string(),
            title: "模块".to_string(),
            description: "描述".to_string(),
            status: DesignGenerationStatus::Placeholder,
            target_frame_id: "custom-target-frame".to_string(),
            target_frame_options: vec!["custom-target-frame".to_string()],
            generated_doc: None,
            is_generating: false,
            logs: Vec::new(),
        }],
    };
    let index = find_generation_module_index(&page, r#""id": "custom-target-frame","#);
    assert_eq!(index, Some(0));
}
