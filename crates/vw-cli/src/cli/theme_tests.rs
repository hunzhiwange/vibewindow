use super::theme::{SCROLLBAR_THUMB, SCROLLBAR_TRACK, TEXT_MUTED, TEXT_PRIMARY};

#[test]
fn theme_tokens_keep_dark_surface_contrast_explicit() {
    assert_ne!(TEXT_PRIMARY, TEXT_MUTED);
    assert_ne!(SCROLLBAR_TRACK, SCROLLBAR_THUMB);
}
