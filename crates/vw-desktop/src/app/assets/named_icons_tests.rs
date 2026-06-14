use super::*;
use serde_json::json;

#[test]
fn canonical_named_icon_family_normalizes_safe_tokens() {
    assert_eq!(
        canonical_named_icon_family(" Feather_Icons.svg").as_deref(),
        Some("feather-icons")
    );
    assert_eq!(canonical_named_icon_family("").as_deref(), None);
    assert_eq!(canonical_named_icon_family("bad/path").as_deref(), None);
}

#[test]
fn named_icon_family_label_formats_normalized_names() {
    assert_eq!(named_icon_family_label("phosphor-icons"), "Phosphor Icons");
    assert_eq!(named_icon_family_label(" Feather_icons.svg "), "Feather Icons");
}

#[test]
fn named_icon_svg_text_finds_flat_and_weighted_assets() {
    let thin_weight = json!(100);

    assert!(named_icon_svg_text("feather", "arrow_up", None).is_some());
    assert!(named_icon_svg_text("phosphor", "columns", Some(&thin_weight)).is_some());
}

#[test]
fn named_icon_catalog_and_family_json_include_generated_families() {
    let catalog = named_icon_catalog();

    assert!(catalog.iter().any(|entry| entry.family == "feather"));
    assert!(catalog.iter().any(|entry| entry.family == "phosphor"));
    assert!(named_icon_family_json("Feather").is_some());
    assert!(named_icon_family_json("missing-family").is_none());
}
