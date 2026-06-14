use super::find_results::{find_match_highlight_bg, to_relative_path};
use iced::Theme;

#[test]
fn to_relative_path_strips_project_root() {
    assert_eq!(to_relative_path("/tmp/project", "/tmp/project/src/main.rs"), "src/main.rs");
    assert_eq!(to_relative_path("/tmp/project", "/other/main.rs"), "/other/main.rs");
}

#[test]
fn find_match_highlight_bg_is_visible() {
    assert!(find_match_highlight_bg(&Theme::Dark).a > 0.0);
    assert!(find_match_highlight_bg(&Theme::Light).a > 0.0);
}

#[test]
fn to_relative_path_normalizes_windows_separators() {
    assert_eq!(
        to_relative_path("C:/work/project", "C:\\work\\project\\src\\main.rs"),
        "src/main.rs"
    );
}
