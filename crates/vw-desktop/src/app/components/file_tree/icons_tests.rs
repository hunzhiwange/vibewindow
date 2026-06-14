use super::icons::{file_icon_for, static_icon_svg, themed_icon_svg};
use crate::app::assets::Icon;

#[test]
fn file_icon_for_uses_extension_mapping() {
    assert_eq!(file_icon_for("main.rs"), Icon::Rust);
    assert_eq!(file_icon_for("README.MD"), Icon::Markdown);
    assert_eq!(file_icon_for("config.yml"), Icon::Yaml);
    assert_eq!(file_icon_for("unknown.bin"), Icon::Document);
}

#[test]
fn icon_svg_builders_accept_known_icons() {
    let _ = static_icon_svg(Icon::File);
    let _ = themed_icon_svg(Icon::FolderOpen);
}
