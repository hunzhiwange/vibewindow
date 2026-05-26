#[cfg(not(target_arch = "wasm32"))]
#[test]
fn render_svg_to_png_rejects_invalid_svg() {
    assert!(super::rasterize::render_svg_to_png("not svg").is_none());
}
