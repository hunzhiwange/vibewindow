use super::*;

#[test]
fn size_and_status_debug_are_stable() {
    let size = Size { cols: 80, rows: 24 };
    assert_eq!(size.cols, 80);
    assert!(format!("{:?}", Status::Running).contains("Running"));
}
