use super::classic_family::{CLASSIC_VARIANTS, RETRO_VARIANTS, VITALITY_VARIANTS};

#[test]
fn classic_family_variant_sets_are_complete() {
    assert_eq!(CLASSIC_VARIANTS.len(), 8);
    assert_eq!(RETRO_VARIANTS.len(), 8);
    assert_eq!(VITALITY_VARIANTS.len(), 8);
    assert!(CLASSIC_VARIANTS.iter().all(|theme| !theme.id.is_empty()));
}
