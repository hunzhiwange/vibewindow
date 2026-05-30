//! 设计生成画布同步辅助。
//!
//! 本模块负责：
//! - 页面与模块规划节点查找
//! - 目标 frame 归一化与候选构建
//! - 生成结果回填到当前画布

use super::parser::default_target_frame_options;
use super::prompts::compact_multiline;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{DesignGenerationPage, DesignGenerationStatus};

pub(super) fn find_generation_page_mut<'a>(
    pages: &'a mut [DesignGenerationPage],
    page_frame_id: &str,
) -> Option<&'a mut DesignGenerationPage> {
    pages.iter_mut().find(|page| page.frame_id == page_frame_id)
}

pub(super) fn find_generation_page_index(
    pages: &[DesignGenerationPage],
    page_frame_id: &str,
) -> Option<usize> {
    if let Some(index) = pages.iter().position(|page| page.frame_id == page_frame_id) {
        return Some(index);
    }
    let normalized = normalize_target_frame_id(page_frame_id);
    pages.iter().position(|page| {
        page.frame_id == normalized
            || page.frame_id.eq_ignore_ascii_case(page_frame_id)
            || page.frame_id.eq_ignore_ascii_case(&normalized)
    })
}

pub(super) fn find_generation_module_index(
    page: &DesignGenerationPage,
    module_id: &str,
) -> Option<usize> {
    if let Some(index) = page.modules.iter().position(|module| module.module_id == module_id) {
        return Some(index);
    }
    let normalized = normalize_target_frame_id(module_id);
    page.modules.iter().position(|module| {
        module.module_id == normalized
            || module.target_frame_id == module_id
            || module.target_frame_id == normalized
            || module.module_id.eq_ignore_ascii_case(module_id)
            || module.module_id.eq_ignore_ascii_case(&normalized)
            || module.target_frame_id.eq_ignore_ascii_case(module_id)
            || module.target_frame_id.eq_ignore_ascii_case(&normalized)
    })
}

pub(super) fn collect_retry_error_context(page: &DesignGenerationPage) -> String {
    let mut lines = Vec::new();
    for module in &page.modules {
        if !matches!(module.status, DesignGenerationStatus::Failed) {
            continue;
        }
        let recent = module
            .logs
            .iter()
            .rev()
            .find(|line| line.contains("failed") || line.contains("error") || line.contains("失败"))
            .cloned()
            .unwrap_or_else(|| "无详细错误日志".to_string());
        lines.push(format!("模块 {}: {}", module.title, compact_multiline(&recent)));
    }
    lines.join("\n")
}

fn collect_element_ids(elements: &[DesignElement], ids: &mut Vec<String>) {
    for element in elements {
        ids.push(element.id.clone());
        collect_element_ids(&element.children, ids);
    }
}

fn extract_embedded_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if let Some(marker_index) = trimmed.find("\"id\"") {
        let tail = &trimmed[marker_index + 4..];
        if let Some(colon_index) = tail.find(':') {
            let tail = tail[colon_index + 1..].trim_start();
            if let Some(start_quote) = tail.find('"') {
                let after_start = &tail[start_quote + 1..];
                if let Some(end_quote) = after_start.find('"') {
                    let candidate = after_start[..end_quote].trim();
                    if !candidate.is_empty() {
                        return Some(candidate.to_string());
                    }
                }
            }
        }
    }
    None
}

pub(super) fn normalize_target_frame_id(raw_target: &str) -> String {
    let trimmed = raw_target.trim();
    if let Some(extracted) = extract_embedded_id(trimmed) {
        extracted
    } else {
        trimmed.trim_matches('"').trim_end_matches(',').to_string()
    }
}

pub(super) fn build_target_frame_options(
    doc: &DesignDoc,
    page_index: usize,
    module_index: usize,
    current_target: Option<&str>,
) -> Vec<String> {
    let mut options = default_target_frame_options(page_index, module_index);
    let mut canvas_ids = Vec::new();
    collect_element_ids(&doc.children, &mut canvas_ids);
    for id in canvas_ids {
        if !options.iter().any(|option| option == &id) {
            options.push(id);
        }
    }
    if let Some(target) = current_target {
        let normalized = normalize_target_frame_id(target);
        if !normalized.is_empty() && !options.iter().any(|option| option == &normalized) {
            options.push(normalized);
        }
    }
    options
}

fn resolve_canvas_target_id(doc: &DesignDoc, raw_target: &str) -> Option<String> {
    let normalized = normalize_target_frame_id(raw_target);
    if normalized.is_empty() {
        return None;
    }
    if doc.find_element(&normalized).is_some() {
        return Some(normalized);
    }
    let mut ids = Vec::new();
    collect_element_ids(&doc.children, &mut ids);
    ids.into_iter().find(|id| id.eq_ignore_ascii_case(&normalized))
}

pub(super) fn sync_module_placeholder_status(
    state: &mut crate::app::views::design::state::DesignState,
    target_frame_id: &str,
    status: DesignGenerationStatus,
) {
    let Some(target_id) = resolve_canvas_target_id(&state.doc, target_frame_id) else {
        return;
    };
    let anim_phase = (state.design_generation_anim_frame / 2) % 3;
    let status_text = if status == DesignGenerationStatus::Running {
        let dots = match anim_phase {
            0 => ".",
            1 => "..",
            _ => "...",
        };
        format!("{}{} · 实时预览占位", status.label(), dots)
    } else {
        format!("{} · 实时预览占位", status.label())
    };
    let badge_color = match status {
        DesignGenerationStatus::Queued => "#F59E0B",
        DesignGenerationStatus::Running => match anim_phase {
            0 => "#7C3AED",
            1 => "#9333EA",
            _ => "#A855F7",
        },
        DesignGenerationStatus::Filled => "#0891B2",
        DesignGenerationStatus::Failed => "#DC2626",
        DesignGenerationStatus::Generated => "#059669",
        DesignGenerationStatus::Aggregated => "#0D9488",
        _ => "$--secondary",
    };
    let badge_text = match status {
        DesignGenerationStatus::Queued => "queued",
        DesignGenerationStatus::Running => "running",
        DesignGenerationStatus::Filled => "filled",
        DesignGenerationStatus::Failed => "failed",
        DesignGenerationStatus::Generated => "generated",
        DesignGenerationStatus::Aggregated => "aggregated",
        _ => "preview",
    };
    let slot_hint = match status {
        DesignGenerationStatus::Running => "占位模块正在生成中，完成后会替换为新结构",
        DesignGenerationStatus::Queued => "占位模块已排队，等待生成并回填",
        DesignGenerationStatus::Failed => "占位模块生成失败，可重试当前模块",
        _ => "占位模块，可继续生成并汇总到当前页面",
    };
    state.doc.update_property(
        &format!("{}-status", target_id),
        "content",
        serde_json::json!(status_text),
    );
    state.doc.update_property(
        &format!("{}-badge", target_id),
        "fill",
        serde_json::json!(badge_color),
    );
    state.doc.update_property(
        &format!("{}-badge-text", target_id),
        "content",
        serde_json::json!(badge_text),
    );
    state.doc.update_property(
        &format!("{}-slot-hint", target_id),
        "content",
        serde_json::json!(slot_hint),
    );
    state.doc.update_property(
        &format!("{}-status-id", target_id),
        "content",
        serde_json::json!(format!("id: {} · {}", target_id, status.label())),
    );
    if status == DesignGenerationStatus::Running {
        let badge_width = match anim_phase {
            0 => 88.0,
            1 => 92.0,
            _ => 96.0,
        };
        state.doc.update_property(
            &format!("{}-badge", target_id),
            "width",
            serde_json::json!(badge_width),
        );
    } else {
        state.doc.update_property(
            &format!("{}-badge", target_id),
            "width",
            serde_json::json!(88.0),
        );
    }
    state.doc.update_property(&format!("{}-status", target_id), "visible", serde_json::json!(true));
    state.doc.update_property(&format!("{}-badge", target_id), "visible", serde_json::json!(true));
    state.doc.update_property(
        &format!("{}-badge-text", target_id),
        "visible",
        serde_json::json!(true),
    );
    state.doc.update_property(
        &format!("{}-slot-hint", target_id),
        "visible",
        serde_json::json!(true),
    );
    state.doc.update_property(
        &format!("{}-status-id", target_id),
        "visible",
        serde_json::json!(true),
    );
}

pub(super) fn apply_module_doc_to_canvas(
    state: &mut crate::app::views::design::state::DesignState,
    target_frame_id: &str,
    generated_doc: &DesignDoc,
) -> Result<(), String> {
    fn find_mut<'a>(elements: &'a mut [DesignElement], id: &str) -> Option<&'a mut DesignElement> {
        for element in elements {
            if element.id == id {
                return Some(element);
            }
            if let Some(found) = find_mut(&mut element.children, id) {
                return Some(found);
            }
        }
        None
    }

    let resolved_target_id = resolve_canvas_target_id(&state.doc, target_frame_id);
    let Some(target_id) = resolved_target_id else {
        let normalized = normalize_target_frame_id(target_frame_id);
        let mut available = Vec::new();
        collect_element_ids(&state.doc.children, &mut available);
        let candidates = available
            .into_iter()
            .filter(|id| id.contains(&normalized) || normalized.contains(id))
            .take(8)
            .collect::<Vec<_>>();
        let candidates_text =
            if candidates.is_empty() { "无".to_string() } else { candidates.join(", ") };
        return Err(format!(
            "未找到目标模块占位 frame: {}（规范化后: {}，可用相近 id: {}）",
            target_frame_id, normalized, candidates_text
        ));
    };

    let target = find_mut(&mut state.doc.children, &target_id)
        .ok_or_else(|| format!("未找到目标模块占位 frame: {}", target_id))?;

    if generated_doc.children.len() == 1 && generated_doc.children[0].kind == "frame" {
        let generated_frame = &generated_doc.children[0];
        target.name = generated_frame.name.clone().or_else(|| target.name.clone());
        target.context = generated_frame.context.clone().or_else(|| target.context.clone());
        target.width = generated_frame.width.clone();
        target.height = generated_frame.height.clone();
        target.fill = generated_frame.fill.clone();
        target.geometry = generated_frame.geometry.clone();
        target.layout = generated_frame.layout.clone();
        target.gap = generated_frame.gap.clone();
        target.padding = generated_frame.padding.clone();
        target.slot = generated_frame.slot.clone();
        target.align_items = generated_frame.align_items.clone();
        target.justify_content = generated_frame.justify_content.clone();
        target.corner_radius = generated_frame.corner_radius.clone();
        target.stroke = generated_frame.stroke.clone();
        target.effect = generated_frame.effect.clone();
        target.class = generated_frame.class.clone();
        target.rotation = generated_frame.rotation;
        target.opacity = generated_frame.opacity;
        target.enabled = generated_frame.enabled;
        target.clip_content = generated_frame.clip_content;
        target.fill_width = generated_frame.fill_width;
        target.hug_width = generated_frame.hug_width;
        target.fill_height = generated_frame.fill_height;
        target.hug_height = generated_frame.hug_height;
        target.reusable = generated_frame.reusable;
        target.visible = generated_frame.visible;
        target.theme = generated_frame.theme.clone();
        target.export = generated_frame.export.clone();
        target.children = generated_frame.children.clone();
    } else {
        target.children = generated_doc.children.clone();
    }

    target.clip = Some(true);
    state.doc.update_property(&format!("{}-status", target_id), "content", serde_json::json!(""));
    state.doc.update_property(
        &format!("{}-badge-text", target_id),
        "content",
        serde_json::json!(""),
    );
    state.doc.update_property(
        &format!("{}-slot-hint", target_id),
        "content",
        serde_json::json!(""),
    );
    state.doc.update_property(
        &format!("{}-status-id", target_id),
        "content",
        serde_json::json!(""),
    );
    state.doc.update_property(
        &format!("{}-status", target_id),
        "visible",
        serde_json::json!(false),
    );
    state.doc.update_property(&format!("{}-badge", target_id), "visible", serde_json::json!(false));
    state.doc.update_property(
        &format!("{}-badge-text", target_id),
        "visible",
        serde_json::json!(false),
    );
    state.doc.update_property(
        &format!("{}-slot-hint", target_id),
        "visible",
        serde_json::json!(false),
    );
    state.doc.update_property(
        &format!("{}-status-id", target_id),
        "visible",
        serde_json::json!(false),
    );
    Ok(())
}

pub(super) fn apply_page_doc_to_canvas(
    state: &mut crate::app::views::design::state::DesignState,
    page_frame_id: &str,
    generated_doc: &DesignDoc,
) -> Result<(), String> {
    fn find_mut<'a>(elements: &'a mut [DesignElement], id: &str) -> Option<&'a mut DesignElement> {
        for element in elements {
            if element.id == id {
                return Some(element);
            }
            if let Some(found) = find_mut(&mut element.children, id) {
                return Some(found);
            }
        }
        None
    }

    let target = find_mut(&mut state.doc.children, page_frame_id)
        .ok_or_else(|| format!("未找到目标页面 frame: {}", page_frame_id))?;
    if generated_doc.children.len() == 1 && generated_doc.children[0].kind == "frame" {
        let generated_frame = &generated_doc.children[0];
        target.name = generated_frame.name.clone().or_else(|| target.name.clone());
        target.context = generated_frame.context.clone().or_else(|| target.context.clone());
        target.width = generated_frame.width.clone();
        target.height = generated_frame.height.clone();
        target.fill = generated_frame.fill.clone();
        target.geometry = generated_frame.geometry.clone();
        target.layout = generated_frame.layout.clone();
        target.gap = generated_frame.gap.clone();
        target.padding = generated_frame.padding.clone();
        target.slot = generated_frame.slot.clone();
        target.align_items = generated_frame.align_items.clone();
        target.justify_content = generated_frame.justify_content.clone();
        target.corner_radius = generated_frame.corner_radius.clone();
        target.stroke = generated_frame.stroke.clone();
        target.effect = generated_frame.effect.clone();
        target.class = generated_frame.class.clone();
        target.rotation = generated_frame.rotation;
        target.opacity = generated_frame.opacity;
        target.enabled = generated_frame.enabled;
        target.clip_content = generated_frame.clip_content;
        target.fill_width = generated_frame.fill_width;
        target.hug_width = generated_frame.hug_width;
        target.fill_height = generated_frame.fill_height;
        target.hug_height = generated_frame.hug_height;
        target.reusable = generated_frame.reusable;
        target.visible = generated_frame.visible;
        target.theme = generated_frame.theme.clone();
        target.export = generated_frame.export.clone();
        target.children = generated_frame.children.clone();
    } else {
        target.children = generated_doc.children.clone();
    }
    target.clip = Some(true);
    Ok(())
}
