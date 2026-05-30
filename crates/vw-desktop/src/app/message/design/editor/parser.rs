//! 设计生成结果解析与规范化。
//!
//! 本模块负责把模型输出的 JSON、代码块或协议事件包装解析为页面计划与模块文档，
//! 并在进入画布前补齐必要字段，保证兼容当前 DesignDoc 结构。

use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{
    DesignGenerationModule, DesignGenerationPage, DesignGenerationPlan, DesignGenerationStatus,
    DesignGenerationTheme,
};

fn parse_generation_status(value: &str) -> DesignGenerationStatus {
    match value.trim().to_ascii_lowercase().as_str() {
        "planned" => DesignGenerationStatus::Planned,
        "generated" => DesignGenerationStatus::Generated,
        "aggregated" => DesignGenerationStatus::Aggregated,
        _ => DesignGenerationStatus::Placeholder,
    }
}

#[derive(serde::Deserialize)]
struct DesignGenerationPagePayload {
    #[serde(alias = "name", alias = "page", alias = "page_title")]
    title: String,
    #[serde(alias = "goal", alias = "purpose", alias = "description")]
    objective: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    #[serde(alias = "sections", alias = "blocks", alias = "components")]
    modules: Vec<DesignGenerationModulePayload>,
}

#[derive(serde::Deserialize)]
struct DesignGenerationModulePayload {
    #[serde(alias = "name", alias = "module", alias = "module_title")]
    title: String,
    #[serde(alias = "desc", alias = "objective", alias = "content")]
    description: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    #[serde(
        alias = "module_doc_json",
        alias = "module_json",
        alias = "module_doc_design",
        alias = "design_doc",
        alias = "doc"
    )]
    module_doc: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct DesignGenerationPagesEnvelope {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    pages: Vec<DesignGenerationPagePayload>,
    #[serde(default)]
    #[serde(alias = "site_map", alias = "website_pages")]
    plan: Vec<DesignGenerationPagePayload>,
}

fn module_frame_id(page_index: usize, module_index: usize) -> String {
    format!("page-{}-module-{}", page_index, module_index)
}

pub(super) fn default_target_frame_options(page_index: usize, module_index: usize) -> Vec<String> {
    vec![
        module_frame_id(page_index, module_index),
        format!("design-page-{}", page_index),
        "canvas-root".to_string(),
    ]
}

fn materialize_design_generation_pages(
    pages: Vec<DesignGenerationPagePayload>,
    _theme: DesignGenerationTheme,
) -> Result<Vec<DesignGenerationPage>, String> {
    let mut materialized_pages = Vec::new();
    for (page_index, page) in pages.into_iter().enumerate() {
        if page.title.trim().is_empty() {
            continue;
        }
        let mut modules = Vec::new();
        for (module_index, module) in page.modules.into_iter().enumerate() {
            if module.title.trim().is_empty() {
                continue;
            }
            let _ = module.module_doc;
            let generated_doc = None;
            let status = module
                .status
                .as_deref()
                .map(parse_generation_status)
                .unwrap_or(DesignGenerationStatus::Queued);
            modules.push(DesignGenerationModule {
                module_id: module_frame_id(page_index, module_index),
                title: module.title,
                description: module.description,
                status,
                target_frame_id: module_frame_id(page_index, module_index),
                target_frame_options: default_target_frame_options(page_index, module_index),
                generated_doc,
                is_generating: false,
                logs: Vec::new(),
            });
        }
        materialized_pages.push(DesignGenerationPage {
            frame_id: format!("design-page-{}", page_index),
            title: page.title,
            objective: page.objective,
            status: page
                .status
                .as_deref()
                .map(parse_generation_status)
                .unwrap_or(DesignGenerationStatus::Queued),
            modules,
        });
    }

    if materialized_pages.is_empty() {
        Err("生成结果为空，未得到可用页面计划".to_string())
    } else {
        Ok(materialized_pages)
    }
}

fn parse_design_generation_plan_from_value(
    value: serde_json::Value,
    theme: DesignGenerationTheme,
    matched_but_empty: &mut bool,
) -> Result<Option<DesignGenerationPlan>, String> {
    const EMPTY_PLAN_ERROR: &str = "生成结果为空，未得到可用页面计划";

    if let Ok(pages) = serde_json::from_value::<Vec<DesignGenerationPagePayload>>(value.clone()) {
        match materialize_design_generation_pages(pages, theme) {
            Ok(pages) => {
                return Ok(Some(DesignGenerationPlan { summary: None, pages }));
            }
            Err(error) => {
                if error == EMPTY_PLAN_ERROR {
                    *matched_but_empty = true;
                } else {
                    return Err(error);
                }
            }
        }
    }

    if let Ok(envelope) = serde_json::from_value::<DesignGenerationPagesEnvelope>(value) {
        let pages_payload = if envelope.pages.is_empty() { envelope.plan } else { envelope.pages };
        match materialize_design_generation_pages(pages_payload, theme) {
            Ok(pages) => {
                return Ok(Some(DesignGenerationPlan { summary: envelope.summary, pages }));
            }
            Err(error) => {
                if error == EMPTY_PLAN_ERROR {
                    *matched_but_empty = true;
                } else {
                    return Err(error);
                }
            }
        }
    }

    Ok(None)
}

fn parse_design_generation_plan_from_text_candidate(
    text: &str,
    theme: DesignGenerationTheme,
    matched_but_empty: &mut bool,
) -> Result<Option<DesignGenerationPlan>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed)
        && let Some(plan) =
            parse_design_generation_plan_from_value(value, theme, matched_but_empty)?
    {
        return Ok(Some(plan));
    }

    for value in vw_shared::json::extract_json_values(trimmed) {
        if let Some(plan) =
            parse_design_generation_plan_from_value(value, theme, matched_but_empty)?
        {
            return Ok(Some(plan));
        }
    }

    Ok(None)
}

fn collect_plan_json_text_candidates(value: &serde_json::Value) -> Vec<String> {
    let mut candidates = Vec::new();
    let pointer_candidates = [
        "/result",
        "/message",
        "/text",
        "/delta",
        "/delta/text",
        "/message/text",
        "/content",
        "/content/text",
        "/part/text",
        "/part/content",
        "/part/result",
    ];
    for pointer in pointer_candidates {
        if let Some(text) = value.pointer(pointer).and_then(|v| v.as_str()) {
            candidates.push(text.to_string());
        }
    }

    for array_pointer in ["/message/content", "/content", "/part/content"] {
        if let Some(items) = value.pointer(array_pointer).and_then(|v| v.as_array()) {
            for item in items {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    candidates.push(text.to_string());
                }
                if let Some(text) = item.pointer("/delta/text").and_then(|v| v.as_str()) {
                    candidates.push(text.to_string());
                }
                if let Some(text) = item.get("content").and_then(|v| v.as_str()) {
                    candidates.push(text.to_string());
                }
                if let Some(text) = item.pointer("/part/text").and_then(|v| v.as_str()) {
                    candidates.push(text.to_string());
                }
            }
        }
    }

    candidates
}

pub(super) fn parse_design_generation_pages(
    raw: &str,
    theme: DesignGenerationTheme,
) -> Result<DesignGenerationPlan, String> {
    const EMPTY_PLAN_ERROR: &str = "生成结果为空，未得到可用页面计划";
    const INVALID_PLAN_ERROR: &str = "生成结果不是合法 JSON 页面计划。";

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EMPTY_PLAN_ERROR.to_string());
    }

    let mut matched_but_empty = false;

    if let Some(plan) =
        parse_design_generation_plan_from_text_candidate(trimmed, theme, &mut matched_but_empty)?
    {
        return Ok(plan);
    }

    for value in vw_shared::json::extract_json_values(trimmed) {
        for text in collect_plan_json_text_candidates(&value) {
            if let Some(plan) = parse_design_generation_plan_from_text_candidate(
                &text,
                theme,
                &mut matched_but_empty,
            )? {
                return Ok(plan);
            }
        }
    }

    if matched_but_empty {
        Err(EMPTY_PLAN_ERROR.to_string())
    } else {
        Err(INVALID_PLAN_ERROR.to_string())
    }
}

fn default_module_doc_theme(theme: DesignGenerationTheme) -> serde_json::Value {
    match theme {
        DesignGenerationTheme::Lunaris => serde_json::json!({ "Mode": "Dark" }),
        _ => serde_json::json!({ "Mode": "Light" }),
    }
}

fn normalize_module_doc_value(
    value: serde_json::Value,
    theme: DesignGenerationTheme,
) -> Option<serde_json::Value> {
    if let serde_json::Value::Array(children) = value {
        return Some(serde_json::json!({
            "version": "2.6",
            "children": children,
            "variables": {},
            "theme": default_module_doc_theme(theme),
        }));
    }

    let serde_json::Value::Object(mut object) = value else {
        return None;
    };

    let looks_like_design_doc = object.contains_key("version")
        || object.contains_key("children")
        || object.contains_key("variables")
        || object.contains_key("theme");
    let has_design_payload = object.contains_key("id")
        || object.contains_key("children")
        || object.contains_key("width")
        || object.contains_key("height")
        || object.contains_key("x")
        || object.contains_key("y")
        || object.contains_key("content")
        || object.contains_key("fill")
        || object.contains_key("layout")
        || object.contains_key("name")
        || object.contains_key("slot")
        || object.contains_key("stroke")
        || object.contains_key("cornerRadius")
        || object.contains_key("ref");
    let looks_like_design_element = has_design_payload
        && (object.contains_key("type")
            || object.contains_key("kind")
            || object.contains_key("id"));

    if looks_like_design_doc {
        if !object.contains_key("version") {
            object.insert("version".to_string(), serde_json::Value::String("2.6".to_string()));
        }
        if !object.contains_key("children") {
            object.insert("children".to_string(), serde_json::Value::Array(Vec::new()));
        }
        if !object.contains_key("variables") {
            object
                .insert("variables".to_string(), serde_json::Value::Object(serde_json::Map::new()));
        }
        if !object.contains_key("theme") {
            object.insert("theme".to_string(), default_module_doc_theme(theme));
        }
        return Some(serde_json::Value::Object(object));
    }

    if looks_like_design_element {
        return Some(serde_json::json!({
            "version": "2.6",
            "children": [serde_json::Value::Object(object)],
            "variables": {},
            "theme": default_module_doc_theme(theme),
        }));
    }

    None
}

fn patch_design_element_value(value: &mut serde_json::Value, seed: &str) {
    let Some(object) = value.as_object_mut() else {
        return;
    };

    if !object.contains_key("type")
        && let Some(kind) = object.get("kind").and_then(|kind| kind.as_str())
        && !kind.trim().is_empty()
    {
        object.insert("type".to_string(), serde_json::Value::String(kind.to_string()));
    }

    let needs_id =
        object.get("id").and_then(|id| id.as_str()).map(|id| id.trim().is_empty()).unwrap_or(true);
    if needs_id {
        object.insert("id".to_string(), serde_json::Value::String(format!("auto-{}", seed)));
    }

    if let Some(stroke) = object.get_mut("stroke") {
        normalize_design_stroke_value(stroke);
    }

    if let Some(children) = object.get_mut("children").and_then(|children| children.as_array_mut())
    {
        for (index, child) in children.iter_mut().enumerate() {
            patch_design_element_value(child, &format!("{}-{}", seed, index));
        }
    }
}

fn normalize_design_stroke_value(stroke: &mut serde_json::Value) {
    match stroke {
        serde_json::Value::String(fill) => {
            *stroke = serde_json::json!({ "fill": fill });
        }
        serde_json::Value::Object(object) => {
            if !object.contains_key("fill")
                && let Some(color) = object.get("color").and_then(|value| value.as_str())
            {
                object.insert("fill".to_string(), serde_json::Value::String(color.to_string()));
            }
            if !object.contains_key("thickness")
                && let Some(width) = object.get("width").cloned()
            {
                object.insert("thickness".to_string(), width);
            }
        }
        _ => {}
    }
}

fn patch_module_doc_value(value: &mut serde_json::Value) {
    let Some(children_value) = value.get_mut("children") else {
        return;
    };
    if !children_value.is_array() {
        *children_value = serde_json::Value::Array(Vec::new());
    }
    let Some(children) = children_value.as_array_mut() else {
        return;
    };
    let mut sanitized_children = Vec::new();
    for (index, mut child) in children.drain(..).enumerate() {
        if !child.is_object() {
            continue;
        }
        patch_design_element_value(&mut child, &format!("node-{}", index));
        if serde_json::from_value::<DesignElement>(child.clone()).is_ok() {
            sanitized_children.push(child);
        }
    }
    *children = sanitized_children;
}

fn parse_module_doc_from_value(
    value: serde_json::Value,
    theme: DesignGenerationTheme,
) -> Result<Option<DesignDoc>, String> {
    let Some(mut normalized) = normalize_module_doc_value(value, theme) else {
        return Ok(None);
    };
    patch_module_doc_value(&mut normalized);
    let doc = serde_json::from_value::<DesignDoc>(normalized).map_err(|error| error.to_string())?;
    Ok(Some(doc))
}

fn parse_module_doc_from_text_candidate(
    text: &str,
    theme: DesignGenerationTheme,
) -> Result<Option<DesignDoc>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let mut candidates = vec![trimmed.to_string()];
    if let Some(fenced) = parse_markdown_fenced_json_candidate(trimmed) {
        candidates.push(fenced);
    }

    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&candidate)
            && let Some(doc) = parse_module_doc_from_value(value, theme)?
        {
            return Ok(Some(doc));
        }

        for value in vw_shared::json::extract_json_values(&candidate) {
            if let Some(doc) = parse_module_doc_from_value(value, theme)? {
                return Ok(Some(doc));
            }
        }
    }

    Ok(None)
}

fn parse_markdown_fenced_json_candidate(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    let mut lines = trimmed.lines();
    let first_line = lines.next()?;
    if !first_line.trim_start().starts_with("```") {
        return None;
    }
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            break;
        }
        body.push(line);
    }
    let candidate = body.join("\n");
    let candidate = candidate.trim();
    if candidate.is_empty() { None } else { Some(candidate.to_string()) }
}

pub(super) fn parse_design_generation_module_doc(
    raw: &str,
    logs: &[String],
    theme: DesignGenerationTheme,
) -> Result<DesignDoc, String> {
    const EMPTY_DOC_ERROR: &str = "生成结果为空，未得到可用模块文档。";
    const INVALID_DOC_ERROR: &str = "生成结果不是合法模块文档 JSON。";

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EMPTY_DOC_ERROR.to_string());
    }

    if let Some(doc) = parse_module_doc_from_text_candidate(trimmed, theme)? {
        return Ok(doc);
    }

    for value in vw_shared::json::extract_json_values(trimmed) {
        if let Some(doc) = parse_module_doc_from_value(value.clone(), theme)? {
            return Ok(doc);
        }
        for text in collect_plan_json_text_candidates(&value) {
            if let Some(doc) = parse_module_doc_from_text_candidate(&text, theme)? {
                return Ok(doc);
            }
        }
    }

    let streamed_raw = logs.join("\n");
    if !streamed_raw.trim().is_empty() {
        if let Some(doc) = parse_module_doc_from_text_candidate(&streamed_raw, theme)? {
            return Ok(doc);
        }
        for value in vw_shared::json::extract_json_values(&streamed_raw) {
            if let Some(doc) = parse_module_doc_from_value(value.clone(), theme)? {
                return Ok(doc);
            }
            for text in collect_plan_json_text_candidates(&value) {
                if let Some(doc) = parse_module_doc_from_text_candidate(&text, theme)? {
                    return Ok(doc);
                }
            }
        }
    }

    Err(INVALID_DOC_ERROR.to_string())
}
