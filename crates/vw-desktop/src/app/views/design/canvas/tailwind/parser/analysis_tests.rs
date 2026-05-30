#[test]
fn variant_classification_distinguishes_flattened_and_export_only() {
    assert!(super::is_flattenable_variant("hover"));
    assert_eq!(
        super::classify_variant_chain(&["hover"]).map(|item| item.0),
        Some(super::TailwindTokenSupport::FlattenedVariant)
    );
    assert_eq!(
        super::classify_variant_chain(&["dark"]).map(|item| item.0),
        Some(super::TailwindTokenSupport::ExportOnly)
    );
}

#[test]
fn analyze_classes_reports_export_only_animation() {
    let analysis = super::analyze_classes("animate-spin hover:bg-blue-500");
    assert!(analysis.issues.iter().any(|issue| issue.original_class == "animate-spin"));
    assert!(
        analysis
            .issues
            .iter()
            .any(|issue| issue.support == super::TailwindTokenSupport::FlattenedVariant)
    );
}
