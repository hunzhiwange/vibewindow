use super::{TuiPill, TuiTone};

#[test]
fn tui_pill_new_preserves_label_and_tone() {
    let pill = TuiPill::new("Ready", TuiTone::Success);
    assert_eq!(pill.label, "Ready");
    assert_eq!(pill.tone, TuiTone::Success);
}
