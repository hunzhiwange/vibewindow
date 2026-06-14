#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("parser_tests"));
}

use super::parser;
use crate::app::views::design::state::{DesignGenerationStatus, DesignGenerationTheme};

#[test]
fn parse_pages_accepts_array_and_status_values() {
    let raw = r#"[
      {
        "title": "首页",
        "objective": "展示入口",
        "status": "planned",
        "modules": [
          { "title": "Hero", "description": "主视觉", "status": "generated" },
          { "title": "Footer", "description": "页脚", "status": "unknown" }
        ]
      }
    ]"#;

    let plan = parser::parse_design_generation_pages(raw, DesignGenerationTheme::Halo).unwrap();

    assert_eq!(plan.summary, None);
    assert_eq!(plan.pages[0].status, DesignGenerationStatus::Planned);
    assert_eq!(plan.pages[0].modules[0].status, DesignGenerationStatus::Generated);
    assert_eq!(plan.pages[0].modules[1].status, DesignGenerationStatus::Placeholder);
    assert_eq!(
        plan.pages[0].modules[0].target_frame_options,
        ["page-0-module-0", "design-page-0", "canvas-root"]
    );
}

#[test]
fn parse_pages_uses_nested_text_candidates() {
    let nested = r#"{"summary":"站点","pages":[{"title":"首页","objective":"入口","modules":[{"title":"Hero","description":"主视觉"}]}]}"#;
    let raw = serde_json::json!({
        "message": {
            "content": [
                { "text": "not json" },
                { "delta": { "text": nested } }
            ]
        }
    })
    .to_string();

    let plan = parser::parse_design_generation_pages(&raw, DesignGenerationTheme::Shadcn).unwrap();

    assert_eq!(plan.summary.as_deref(), Some("站点"));
    assert_eq!(plan.pages[0].title, "首页");
}

#[test]
fn parse_pages_reports_empty_or_invalid() {
    assert_eq!(
        parser::parse_design_generation_pages("  ", DesignGenerationTheme::Shadcn).unwrap_err(),
        "生成结果为空，未得到可用页面计划"
    );
    assert_eq!(
        parser::parse_design_generation_pages("plain text", DesignGenerationTheme::Shadcn)
            .unwrap_err(),
        "生成结果不是合法 JSON 页面计划。"
    );
}

#[test]
fn parse_module_doc_wraps_array_and_sets_lunaris_dark_theme() {
    let raw = r##"[{"type":"frame","id":"root","stroke":{"color":"#111","width":2}}]"##;

    let doc = parser::parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Lunaris)
        .unwrap();

    assert_eq!(doc.version, "2.6");
    assert_eq!(doc.children[0].id, "root");
    assert_eq!(
        doc.children[0].stroke.as_ref().and_then(|stroke| stroke.fill.as_deref()),
        Some("#111")
    );
    assert_eq!(
        doc.children[0].stroke.as_ref().and_then(|stroke| stroke.thickness.as_ref()),
        Some(&serde_json::json!(2))
    );
    assert_eq!(doc.theme.as_ref().map(|theme| theme.mode.as_str()), Some("Dark"));
}

#[test]
fn parse_module_doc_wraps_single_design_element() {
    let raw = r#"{"type":"text","content":"标题"}"#;

    let doc = parser::parse_design_generation_module_doc(raw, &[], DesignGenerationTheme::Shadcn)
        .unwrap();

    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.children[0].kind, "text");
    assert!(doc.children[0].id.starts_with("auto-"));
}

#[test]
fn parse_module_doc_uses_logs_when_raw_is_empty_protocol() {
    let logs = vec![
        "noise".to_string(),
        r#"{"content":{"text":"{\"children\":[{\"type\":\"frame\",\"id\":\"from-log\"}]}"}}"#
            .to_string(),
    ];

    let doc =
        parser::parse_design_generation_module_doc("event", &logs, DesignGenerationTheme::Nitro)
            .unwrap();

    assert_eq!(doc.children[0].id, "from-log");
}

#[test]
fn default_target_frame_options_are_stable() {
    assert_eq!(
        parser::default_target_frame_options(2, 3),
        ["page-2-module-3", "design-page-2", "canvas-root"]
    );
}
