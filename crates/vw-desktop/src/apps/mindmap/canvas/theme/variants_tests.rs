use super::variants::{BUSINESS_VARIANTS, CLASSIC_VARIANTS};

#[test]
fn variant_groups_expose_non_empty_theme_sets() {
    assert!(!CLASSIC_VARIANTS.is_empty());
    assert!(!BUSINESS_VARIANTS.is_empty());
}
