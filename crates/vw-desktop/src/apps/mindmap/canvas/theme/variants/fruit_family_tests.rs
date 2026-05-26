use super::fruit_family::{CHERRY_VARIANTS, PURPLE_VARIANTS};

#[test]
fn fruit_family_variant_sets_are_complete() {
    assert_eq!(CHERRY_VARIANTS.len(), 8);
    assert_eq!(PURPLE_VARIANTS.len(), 8);
    assert!(PURPLE_VARIANTS.iter().all(|theme| !theme.name.is_empty()));
}
