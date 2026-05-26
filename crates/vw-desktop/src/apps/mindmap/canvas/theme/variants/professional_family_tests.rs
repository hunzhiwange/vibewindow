use super::professional_family::{BUSINESS_VARIANTS, SOFT_VARIANTS};

#[test]
fn professional_family_variant_sets_are_complete() {
    assert_eq!(BUSINESS_VARIANTS.len(), 6);
    assert_eq!(SOFT_VARIANTS.len(), 8);
    assert!(BUSINESS_VARIANTS.iter().all(|theme| theme.line_color.is_some()));
}
