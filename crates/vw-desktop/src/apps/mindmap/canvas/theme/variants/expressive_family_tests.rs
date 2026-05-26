use super::expressive_family::{CLASH_VARIANTS, ROSE_VARIANTS};

#[test]
fn expressive_family_variant_sets_are_complete() {
    assert_eq!(ROSE_VARIANTS.len(), 8);
    assert_eq!(CLASH_VARIANTS.len(), 8);
    assert!(ROSE_VARIANTS.iter().all(|theme| !theme.branch_fills.is_empty()));
}
