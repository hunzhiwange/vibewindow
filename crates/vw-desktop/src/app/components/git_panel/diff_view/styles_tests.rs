use iced::widget::{container, text};
use iced::{Color, Element};

use crate::app::Message;

#[test]
fn task_715_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("styles_tests.rs"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn content(label: &'static str) -> Element<'static, Message> {
    container(text(label)).into()
}

#[test]
fn diff_highlight_is_enabled_for_default_app() {
    let app = app();

    assert!(super::styles::diff_highlight_enabled(&app));
}

#[test]
fn row_and_pane_builders_accept_all_tones_and_emphasis_states() {
    for tone in [
        super::DiffSplitPaneTone::Neutral,
        super::DiffSplitPaneTone::Empty,
        super::DiffSplitPaneTone::Add,
        super::DiffSplitPaneTone::Delete,
    ] {
        let _: Element<'static, Message> =
            super::styles::merge_diff_row(content("row"), tone, false);
        let _: Element<'static, Message> =
            super::styles::merge_diff_row(content("row"), tone, true);
        let _: Element<'static, Message> =
            super::styles::diff_split_pane(content("pane"), tone, false);
        let _: Element<'static, Message> =
            super::styles::diff_split_pane(content("pane"), tone, true);
    }
}

#[test]
fn explicit_background_builders_cover_plain_and_emphasized_paths() {
    let bg = Color::from_rgba8(12, 34, 56, 0.5);

    let _: Element<'static, Message> =
        super::styles::merge_diff_row_with_background(content("row"), bg, false);
    let _: Element<'static, Message> =
        super::styles::merge_diff_row_with_background(content("row"), bg, true);
    let _: Element<'static, Message> =
        super::styles::diff_split_pane_with_background(content("pane"), bg, false);
    let _: Element<'static, Message> =
        super::styles::diff_split_pane_with_background(content("pane"), bg, true);
    let _: Element<'static, Message> =
        super::styles::diff_line_number_with_background(content("line"), bg);
}

#[test]
fn divider_and_line_number_area_build_empty_and_interactive_variants() {
    let _: Element<'static, Message> = super::styles::diff_split_divider();
    let _: Element<'static, Message> = super::styles::split_line_number_area(
        "src/lib.rs",
        None,
        "",
        super::markers::LineNumberTone::Neutral,
    );
    let _: Element<'static, Message> = super::styles::split_line_number_area(
        "src/lib.rs",
        Some((0, false)),
        "added",
        super::markers::LineNumberTone::Add,
    );
    let _: Element<'static, Message> = super::styles::split_line_number_area(
        "src/lib.rs",
        Some((3, true)),
        "deleted",
        super::markers::LineNumberTone::Delete,
    );
}
