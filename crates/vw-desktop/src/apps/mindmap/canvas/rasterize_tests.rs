#[cfg(not(target_arch = "wasm32"))]
#[test]
fn render_svg_to_png_rejects_invalid_svg() {
    assert!(super::rasterize::render_svg_to_png("not svg").is_none());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn render_svg_to_png_encodes_valid_svg() {
    let png = super::rasterize::render_svg_to_png(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2">
  <rect width="2" height="2" fill="#ff0000"/>
</svg>"##,
    )
    .expect("valid SVG should render");

    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
}
