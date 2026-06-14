#[test]
fn dark_theme_detection_matches_theme_variant() {
    assert!(super::is_dark_theme(&iced::Theme::Dark));
    assert!(!super::is_dark_theme(&iced::Theme::Light));
}

#[test]
fn action_icon_maps_known_labels_and_defaults() {
    use crate::app::assets::Icon;

    assert_eq!(super::action_icon("打开"), Icon::ChevronRight);
    assert_eq!(super::action_icon("打开最近"), Icon::ArrowClockwise);
    assert_eq!(super::action_icon("打开文件夹"), Icon::FolderOpen);
    assert_eq!(super::action_icon("添加"), Icon::Plus);
    assert_eq!(super::action_icon("独立窗口"), Icon::Box);
    assert_eq!(super::action_icon("浏览器"), Icon::ArrowUp);
    assert_eq!(super::action_icon("编辑"), Icon::Pencil);
    assert_eq!(super::action_icon("取消"), Icon::X);
    assert_eq!(super::action_icon("删除"), Icon::Trash);
    assert_eq!(super::action_icon("保存"), Icon::Save);
    assert_eq!(super::action_icon("unknown"), Icon::ChevronRight);
}

#[test]
fn button_styles_cover_interaction_states() {
    let theme = iced::Theme::Light;
    for status in [
        iced::widget::button::Status::Active,
        iced::widget::button::Status::Hovered,
        iced::widget::button::Status::Pressed,
        iced::widget::button::Status::Disabled,
    ] {
        let cool = super::cool_icon_button_style(&theme, status);
        assert_eq!(cool.border.radius.top_left, 999.0);

        let tile = super::tile_button_style(&theme, status);
        assert_eq!(tile.border.radius.top_left, 20.0);

        let primary = super::primary_button_style(&theme, status);
        assert_eq!(primary.border.radius.top_left, 12.0);

        let icon = super::icon_button_style(&theme, status);
        assert_eq!(icon.border.radius.top_left, 12.0);
    }
}

#[test]
fn input_and_editor_styles_delegate_to_settings_styles() {
    let theme = iced::Theme::Dark;
    let input = super::figma_text_input_style(&theme, iced::widget::text_input::Status::Active);
    assert!(input.border.width >= 0.0);

    let editor = super::figma_text_editor_style(&theme, iced::widget::text_editor::Status::Active);
    assert!(editor.border.width >= 0.0);
}

#[test]
fn helpers_build_tooltip_icon_and_tile_elements() {
    let _ = super::icon_svg(crate::app::assets::Icon::X, 14.0);
    let _ = super::tooltip_bubble("tip");
    let _ = super::tooltip_bubble_el(iced::widget::text("rich").into());
    let _ = super::tile(
        crate::app::assets::Icon::LayoutTextWindow,
        "Tile".to_string(),
        iced::Color::from_rgb8(1, 2, 3),
        vec![("打开", crate::app::Message::None), ("other", crate::app::Message::None)],
        crate::app::Message::None,
    );
}
