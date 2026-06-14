use super::*;

#[test]
fn get_image_returns_all_png_backed_assets() {
    for icon in [
        Icon::Logo,
        Icon::AppFinder,
        Icon::AppTerminal,
        Icon::AppTextMate,
        Icon::AppXcode,
        Icon::AppWindsurf,
    ] {
        std::hint::black_box(get_image(icon));
    }
}

#[test]
#[should_panic(expected = "Image missing in assets map")]
fn get_image_panics_for_non_png_icon() {
    let _ = get_image(Icon::Gear);
}
