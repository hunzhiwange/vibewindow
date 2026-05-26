//! 设计画布 Tailwind 支持模块。
//!
//! 该模块负责把 Tailwind 风格类名转换为画布渲染可用的结构化样式，供布局、形状和文本渲染路径复用。

use super::types::{
    ParsedStyle, TailwindParseAnalysis, TailwindTokenIssue, TailwindTokenSupport,
};
use super::utilities::apply_supported_utility;

fn is_flattenable_variant(prefix: &str) -> bool {
    matches!(
        prefix,
        "sm"
            | "md"
            | "lg"
            | "xl"
            | "2xl"
            | "hover"
            | "focus"
            | "focus-visible"
            | "focus-within"
            | "active"
            | "visited"
            | "disabled"
            | "enabled"
            | "checked"
            | "selected"
            | "open"
    ) || prefix.starts_with("group-")
        || prefix.starts_with("peer-")
}

fn classify_variant_chain(prefixes: &[&str]) -> Option<(TailwindTokenSupport, &'static str)> {
    if prefixes.is_empty() {
        return None;
    }

    if prefixes.contains(&"dark") {
        return Some((
            TailwindTokenSupport::ExportOnly,
            "dark variant is export-only on the static canvas",
        ));
    }

    if prefixes.iter().all(|prefix| is_flattenable_variant(prefix)) {
        return Some((
            TailwindTokenSupport::FlattenedVariant,
            "variant prefix was flattened into a static canvas snapshot",
        ));
    }

    Some((
        TailwindTokenSupport::ExportOnly,
        "unsupported variant prefix is export-only on the static canvas",
    ))
}

fn classify_export_only_token(class_name: &str) -> Option<&'static str> {
    if class_name.starts_with("animate-") {
        return Some("animation utilities are export-only on the static canvas");
    }
    if class_name == "transition"
        || class_name.starts_with("transition-")
        || class_name.starts_with("duration-")
        || class_name.starts_with("ease-")
        || class_name.starts_with("delay-")
    {
        return Some("transition utilities are export-only on the static canvas");
    }
    if class_name == "filter"
        || class_name == "filter-none"
        || class_name.starts_with("blur-")
        || class_name.starts_with("brightness-")
        || class_name.starts_with("contrast-")
        || class_name.starts_with("drop-shadow")
        || class_name == "grayscale"
        || class_name.starts_with("hue-rotate-")
        || class_name == "invert"
        || class_name.starts_with("saturate-")
        || class_name == "sepia"
    {
        return Some("filter utilities are export-only on the static canvas");
    }
    if class_name.starts_with("backdrop-") {
        return Some("backdrop utilities are export-only on the static canvas");
    }
    if class_name == "mask" || class_name.starts_with("mask-") {
        return Some("mask utilities are export-only on the static canvas");
    }
    if class_name == "ring"
        || class_name == "ring-inset"
        || class_name.starts_with("ring-")
        || class_name.starts_with("ring-offset-")
    {
        return Some("ring utilities are export-only on the static canvas");
    }
    if class_name.starts_with("shadow-")
        && !matches!(
            class_name,
            "shadow-none" | "shadow-sm" | "shadow" | "shadow-md" | "shadow-lg"
        )
    {
        return Some("complex shadow utilities are export-only on the static canvas");
    }
    if class_name.contains('[') && class_name.ends_with(']') {
        return Some("unsupported arbitrary value is export-only on the static canvas");
    }

    None
}

fn push_issue(
    issues: &mut Vec<TailwindTokenIssue>,
    original_class: &str,
    normalized_class: &str,
    support: TailwindTokenSupport,
    reason: &'static str,
) {
    issues.push(TailwindTokenIssue {
        original_class: original_class.to_string(),
        normalized_class: Some(normalized_class.to_string()),
        support,
        reason,
    });
}

/// 模块内部可见的 analyze_class_token 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn analyze_class_token(
    style: &mut ParsedStyle,
    issues: &mut Vec<TailwindTokenIssue>,
    class: &str,
) {
    let parts: Vec<&str> = class.split(':').collect();
    let (prefixes, token) = if parts.len() > 1 {
        (&parts[..parts.len() - 1], parts[parts.len() - 1])
    } else {
        (&[][..], class)
    };
    let class_clean = token.split('/').next().unwrap_or(token);

    if let Some((support, reason)) = classify_variant_chain(prefixes)
        && support != TailwindTokenSupport::FlattenedVariant
    {
        push_issue(issues, class, class_clean, support, reason);
        return;
    }

    if apply_supported_utility(style, class_clean) {
        if let Some((TailwindTokenSupport::FlattenedVariant, reason)) =
            classify_variant_chain(prefixes)
        {
            push_issue(
                issues,
                class,
                class_clean,
                TailwindTokenSupport::FlattenedVariant,
                reason,
            );
        }
        return;
    }

    if let Some(reason) = classify_export_only_token(class_clean) {
        push_issue(
            issues,
            class,
            class_clean,
            TailwindTokenSupport::ExportOnly,
            reason,
        );
        return;
    }

    let reason = if !prefixes.is_empty() {
        "variant resolved to an unsupported base utility on the static canvas"
    } else {
        "unmatched tailwind token on the static canvas"
    };
    push_issue(
        issues,
        class,
        class_clean,
        TailwindTokenSupport::Unsupported,
        reason,
    );
}

/// 模块内部可见的 analyze_classes 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub(super) fn analyze_classes(key: &str) -> TailwindParseAnalysis {
    let mut analysis = TailwindParseAnalysis::default();

    for class in key.split_whitespace() {
        analyze_class_token(&mut analysis.style, &mut analysis.issues, class);
    }

    analysis
}

#[cfg(test)]
#[path = "analysis_tests.rs"]
mod analysis_tests;
