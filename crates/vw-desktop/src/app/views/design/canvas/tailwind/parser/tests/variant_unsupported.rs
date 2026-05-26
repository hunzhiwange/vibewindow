//! Tailwind 类名解析测试模块，验证布局、间距、尺寸与不支持变体的解析结果保持稳定。

use super::*;

fn find_issue<'a>(
    analysis: &'a TailwindParseAnalysis,
    original_class: &str,
) -> &'a TailwindTokenIssue {
    analysis
        .issues
        .iter()
        .find(|issue| issue.original_class == original_class)
        .unwrap_or_else(|| panic!("missing issue for {original_class}"))
}

#[test]
fn test_variant_prefixes_degrade_to_base_utilities() {
    let analysis = TailwindParser::analyze("p-4 md:p-8 lg:text-center hover:bg-red-500");

    assert_eq!(analysis.style.padding, Some(32.0));
    assert_eq!(analysis.style.text_align, Some("center".to_string()));
    assert_eq!(analysis.style.background_color, Some(TailwindColors::RED_500));

    assert_eq!(find_issue(&analysis, "md:p-8").support, TailwindTokenSupport::FlattenedVariant);
    assert_eq!(
        find_issue(&analysis, "lg:text-center").support,
        TailwindTokenSupport::FlattenedVariant
    );
    assert_eq!(
        find_issue(&analysis, "hover:bg-red-500").support,
        TailwindTokenSupport::FlattenedVariant
    );
}

#[test]
fn test_dark_variants_are_export_only_and_do_not_override_static_snapshot() {
    let baseline = TailwindParser::parse("bg-white text-black");
    let analysis = TailwindParser::analyze("bg-white dark:bg-black text-black dark:text-white");

    assert_eq!(analysis.style.background_color, baseline.background_color);
    assert_eq!(analysis.style.text_color, baseline.text_color);
    assert_eq!(find_issue(&analysis, "dark:bg-black").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(find_issue(&analysis, "dark:text-white").support, TailwindTokenSupport::ExportOnly);
}

#[test]
fn test_export_only_effect_families_are_classified_explicitly() {
    let analysis = TailwindParser::analyze(
        "shadow-md shadow-xl animate-spin ring-2 backdrop-blur-sm mask-linear-gradient",
    );

    assert_eq!(analysis.style.shadow_offset_y, Some(4.0));
    assert_eq!(analysis.style.shadow_spread, Some(3.0));
    assert_eq!(find_issue(&analysis, "shadow-xl").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(find_issue(&analysis, "animate-spin").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(find_issue(&analysis, "ring-2").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(find_issue(&analysis, "backdrop-blur-sm").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(
        find_issue(&analysis, "mask-linear-gradient").support,
        TailwindTokenSupport::ExportOnly
    );
}

#[test]
fn test_reject_unsupported_or_incomplete_arbitrary_utilities() {
    let analysis = TailwindParser::analyze(
        "w-[50%] h-[calc(100%-1rem)] top-[12] bg-[rgb(1,2,3)] p-[12px] w-[320px unknown-token",
    );

    assert_eq!(analysis.style.width, None);
    assert_eq!(analysis.style.height, None);
    assert_eq!(analysis.style.top, None);
    assert_eq!(analysis.style.background_color, None);
    assert_eq!(analysis.style.padding, None);

    assert_eq!(find_issue(&analysis, "w-[50%]").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(find_issue(&analysis, "bg-[rgb(1,2,3)]").support, TailwindTokenSupport::ExportOnly);
    assert_eq!(find_issue(&analysis, "unknown-token").support, TailwindTokenSupport::Unsupported);
}

#[test]
fn test_unknown_variants_stop_flattening_and_fall_back_to_export_only() {
    let analysis = TailwindParser::analyze("supports-[display:grid]:grid [&>*]:p-4");

    assert_eq!(analysis.style.display, None);
    assert_eq!(analysis.style.padding, None);
    assert_eq!(
        find_issue(&analysis, "supports-[display:grid]:grid").support,
        TailwindTokenSupport::ExportOnly
    );
    assert_eq!(find_issue(&analysis, "[&>*]:p-4").support, TailwindTokenSupport::ExportOnly);
}
