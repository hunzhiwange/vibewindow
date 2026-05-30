use super::icons::{file_icon_for, static_icon_svg, themed_icon_svg};
use crate::app::assets::Icon;

#[test]
fn file_icon_for_uses_extension_mapping() {
    assert_ne!(
        format!("{:?}", file_icon_for("main.rs")),
        format!("{:?}", file_icon_for("README.md"))
    );
}

#[test]
fn icon_svg_builders_accept_known_icons() {
    let _ = static_icon_svg(Icon::File);
    let _ = themed_icon_svg(Icon::FolderOpen);
}
