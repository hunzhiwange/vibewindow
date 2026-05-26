use super::logo::logo_text_lines;

#[test]
fn logo_text_lines_keep_brand_text_visible() {
    let lines = logo_text_lines(1);
    assert_eq!(lines.len(), 2);
    assert!(format!("{:?}", lines[1]).contains("VibeWindow"));
}
