use super::status_animation::{STATUS_SPINNER_FRAMES, spinner_frame};

#[test]
fn spinner_frame_wraps_by_frame_count() {
    assert_eq!(spinner_frame(0), STATUS_SPINNER_FRAMES[0]);
    assert_eq!(spinner_frame(STATUS_SPINNER_FRAMES.len()), STATUS_SPINNER_FRAMES[0]);
}
