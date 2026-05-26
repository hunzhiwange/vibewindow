//! 技能配置设置界面组件。

mod browser;
mod catalog;
mod help;

use crate::app::components::system_settings_common::{
    SETTINGS_LABEL_WIDTH, settings_checkbox_style, settings_error_banner, settings_help_button,
    settings_muted_text_style, settings_page_intro, settings_panel,
    settings_section_card, settings_segment_button_style, settings_text_input_style,
};
use crate::app::state::SkillsSettingsTab;
use crate::app::{App, Message, message};
use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};
use vw_config_types::skills::SkillsPromptInjectionMode;

fn field_row<'a>(
    label: &'static str,
    description: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text(label).size(13),
                text(description).size(11).style(settings_muted_text_style),
            ]
            .spacing(4)
            .width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
            container(control.into()).width(Length::Fill),
        ]
        .spacing(22)
        .align_y(Alignment::Center),
    )
    .padding([14, 0])
    .width(Length::Fill)
    .into()
}

fn settings_tab_button(
    label: &'static str,
    tab: SkillsSettingsTab,
    active_tab: SkillsSettingsTab,
) -> Element<'static, Message> {
    let is_active = tab == active_tab;

    button(text(label).size(13))
        .padding([8, 14])
        .on_press(Message::Settings(message::SettingsMessage::SkillsTabChanged(tab)))
        .style(move |theme: &iced::Theme, status| {
            settings_segment_button_style(theme, status, is_active)
        })
        .into()
}

fn plugins_placeholder() -> Element<'static, Message> {
    column![
        settings_section_card("插件", "预留占位，后续在这里接入插件浏览与管理能力。"),
        settings_panel(
            column![
                text("插件内容暂未开放。"),
                text("当前仅保留占位，不展示具体内容。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(6)
        ),
    ]
    .spacing(16)
    .width(Length::Fill)
    .into()
}

fn system_config_view(app: &App) -> Element<'_, Message> {
    let s = &app.skills_settings;

    let discovery_row = field_row(
        "Discovery",
        "读取顺序参考 Claude Code，并保持当前项目优先覆盖。",
        text("当前项目 .vibewindow/skills -> 当前项目 skills -> 父级 .vibewindow/skills -> ~/.vibewindow/skills")
            .size(12)
            .style(settings_muted_text_style),
    );

    let open_enabled_row = field_row(
        "Community sync",
        "控制是否启用 open-skills 仓库同步。",
        checkbox(s.open_skills_enabled).label("启用 open-skills 仓库同步").on_toggle(|v| {
            Message::Settings(message::SettingsMessage::SkillsOpenEnabledToggled(v))
        })
        .style(settings_checkbox_style),
    );

    let open_dir_row = field_row(
        "Repository path",
        "为空时默认使用 $HOME/open-skills。",
        text_input("默认: $HOME/open-skills", &s.open_skills_dir_input)
            .on_input(|v| Message::Settings(message::SettingsMessage::SkillsOpenDirChanged(v)))
            .padding([10, 12])
            .size(13)
            .style(settings_text_input_style)
            .width(Length::Fill),
    );

    let mode_full = matches!(s.prompt_injection_mode, SkillsPromptInjectionMode::Full);
    let mode_row = field_row(
        "Injection mode",
        "compact 更省上下文，full 会注入完整技能内容。",
        checkbox(mode_full).label("使用 full（关闭时为 compact）").on_toggle(
            |v| Message::Settings(message::SettingsMessage::SkillsPromptInjectionModeChanged(
                if v {
                    SkillsPromptInjectionMode::Full
                } else {
                    SkillsPromptInjectionMode::Compact
                }
            ))
        )
        .style(settings_checkbox_style),
    );

    let mut config_card = column![
        settings_section_card(
            "系统配置",
            "目录读取按项目、父级和全局技能目录分层发现；这里保留社区同步与 prompt injection 配置。",
        ),
        settings_panel(column![discovery_row, open_enabled_row, open_dir_row, mode_row].spacing(0)),
    ]
    .spacing(16)
    .width(Length::Fill);

    if let Some(err) = &s.save_error {
        config_card = config_card.push(settings_error_banner(err));
    }

    config_card.into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let s = &app.skills_settings;
    let help_btn =
        settings_help_button(Message::Settings(message::SettingsMessage::SkillsHelpOpen));

    let content = match s.active_tab {
        SkillsSettingsTab::Skills => browser::view(app),
        SkillsSettingsTab::Plugins => plugins_placeholder(),
        SkillsSettingsTab::SystemConfig => system_config_view(app),
    };

    column![
        row![
            container(settings_page_intro(
                "技能配置",
                "技能、插件与系统配置分开展示；技能页支持项目目录与全部目录切换。",
            ))
            .width(Length::Fill),
            help_btn,
        ]
        .align_y(Alignment::Start),
        row![
            settings_tab_button("技能", SkillsSettingsTab::Skills, s.active_tab),
            settings_tab_button("插件", SkillsSettingsTab::Plugins, s.active_tab),
            settings_tab_button("系统配置", SkillsSettingsTab::SystemConfig, s.active_tab),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        content,
    ]
    .spacing(16)
    .width(Length::Fill)
    .into()
}

pub fn view_overlays<'a>(app: &'a App, dialog: Element<'a, Message>) -> Element<'a, Message> {
    let dialog = browser::view_overlays(app, dialog);
    help::view_overlays(app, dialog)
}

#[cfg(test)]
mod browser_tests;
#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod help_tests;
