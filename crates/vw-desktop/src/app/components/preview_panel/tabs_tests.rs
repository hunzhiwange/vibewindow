use super::tabs::{DisplayLanguage, all_languages, build_preview_tab_menu};
use crate::app::Message;
use iced_code_editor::i18n::Language;

#[test]
fn all_languages_keep_stable_order_and_labels() {
    let labels = all_languages().iter().map(ToString::to_string).collect::<Vec<_>>();

    assert_eq!(labels.first().map(String::as_str), Some("English"));
    assert_eq!(
        labels,
        vec![
            "English",
            "简体中文",
            "Français",
            "Español",
            "Deutsch",
            "Italiano",
            "Português (BR)",
            "Português (PT)",
        ]
    );
}

#[test]
fn display_language_labels_match_editor_languages() {
    let cases = [
        (Language::English, "English"),
        (Language::ChineseSimplified, "简体中文"),
        (Language::French, "Français"),
        (Language::Spanish, "Español"),
        (Language::German, "Deutsch"),
        (Language::Italian, "Italiano"),
        (Language::PortugueseBR, "Português (BR)"),
        (Language::PortuguesePT, "Português (PT)"),
    ];

    for (language, label) in cases {
        assert_eq!(DisplayLanguage(language).to_string(), label);
    }
}

#[test]
fn display_language_is_copyable_and_comparable() {
    let language = DisplayLanguage(Language::English);
    let copied = language;

    assert_eq!(copied, language);
    assert!(format!("{copied:?}").contains("English"));
}

#[test]
fn preview_tab_menu_builds_for_owned_path_messages() {
    let menu = build_preview_tab_menu("/tmp/src/main.rs");
    let element: iced::Element<'_, Message> = menu;

    std::hint::black_box(element);
}
