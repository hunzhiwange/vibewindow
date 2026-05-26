use super::styles::{file_icon_for, truncate_title};
use crate::app::assets::Icon;

#[test]
fn file_icon_for_known_extensions() {
    assert_eq!(file_icon_for("main.rs"), Icon::Rust);
    assert_eq!(file_icon_for("README.md"), Icon::Markdown);
    assert_eq!(file_icon_for("unknown.bin"), Icon::Document);
}

#[test]
fn truncate_title_keeps_short_titles_and_shortens_long_titles() {
    assert_eq!(truncate_title("short", 8), "short");
    assert_eq!(truncate_title("preview-panel", 7), "preview…");
}
