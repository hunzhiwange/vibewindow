//! 系统设置中技能管理页面的浏览、目录或帮助视图。
//!
//! 本模块主要负责把应用状态转换为桌面端可渲染的 Iced 控件，并把用户操作映射回上层消息。
//! 注释聚焦调用边界、返回值和错误传播方式，便于后续维护设置页与工具栏行为时快速定位职责。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    icon_svg, settings_panel_style, settings_value_badge,
};
use crate::app::state::{
    SkillsCatalogItem as CatalogSkillMeta, SkillsCatalogKind as CatalogSkillKind,
};
use crate::app::{Message, message};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length};
use std::path::Path;

/// 构建或处理 `catalog_matches_query` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回本函数生成的状态、样式或辅助值，供同一流程继续组合使用。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn catalog_matches_query(skill: &CatalogSkillMeta, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let q = query.to_ascii_lowercase();
    skill.id.to_ascii_lowercase().contains(&q)
        || skill.title.to_ascii_lowercase().contains(&q)
        || skill.description.to_ascii_lowercase().contains(&q)
        || skill.source.to_ascii_lowercase().contains(&q)
        || skill.source_path.as_ref().is_some_and(|path| path.to_ascii_lowercase().contains(&q))
}

/// 构建或处理 `section_card_style` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回本函数生成的状态、样式或辅助值，供同一流程继续组合使用。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn section_card_style(theme: &iced::Theme) -> iced::widget::container::Style {
    let mut style = settings_panel_style(theme);
    style.shadow = iced::Shadow::default();
    style.border.radius = 14.0.into();
    style
}

/// 构建或处理 `skill_badge` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回本函数生成的状态、样式或辅助值，供同一流程继续组合使用。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn skill_badge(label: impl Into<String>, emphasized: bool) -> Element<'static, Message> {
    let label = label.into();
    if !emphasized {
        return settings_value_badge(label);
    }

    container(text(label).size(11))
        .padding([4, 8])
        .style(move |theme: &iced::Theme| {
            let palette = theme.extended_palette();
            let (background, border, text_color) = (
                palette.primary.weak.color.scale_alpha(0.42),
                palette.primary.base.color.scale_alpha(0.6),
                palette.primary.base.color,
            );

            iced::widget::container::Style {
                text_color: Some(text_color),
                background: Some(Background::Color(background)),
                border: Border { width: 1.0, color: border, radius: 999.0.into() },
                ..Default::default()
            }
        })
        .into()
}

fn catalog_skill_initials(skill: &CatalogSkillMeta) -> String {
    let initials = skill
        .title
        .split(|ch: char| ch.is_whitespace() || ch == '-' || ch == '_')
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>();

    if initials.is_empty() {
        skill.id.chars().take(2).collect::<String>().to_ascii_uppercase()
    } else {
        initials.to_ascii_uppercase()
    }
}

fn section_copy(source: &str) -> (Icon, &'static str, &'static str) {
    match source {
        "workspace" => {
            (Icon::FolderOpen, "项目目录", "当前项目的 .vibewindow/skills 与 skills 目录。")
        }
        "ancestor" => (Icon::FolderOpen, "父级目录", "从上级目录逐层发现的 .vibewindow/skills。"),
        "global" => (Icon::FolderOpen, "全局目录", "当前用户的 ~/.vibewindow/skills。"),
        "bundled" => (Icon::Sliders, "内置技能", "产品内置技能，可按需安装到当前项目。"),
        _ => (Icon::Grid1x2, "其他来源", "未归类的技能来源。"),
    }
}

fn source_label(source: &str) -> &'static str {
    match source {
        "workspace" => "Workspace",
        "ancestor" => "Parent",
        "global" => "Global",
        "bundled" => "Built-in",
        _ => "Source",
    }
}

fn compact_source_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut display = trimmed.to_string();
    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy();
        if display.starts_with(home.as_ref()) {
            display = format!("~{}", &display[home.len()..]);
        }
    }

    let path = Path::new(&display);
    let parts = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    if parts.len() <= 4 {
        return display;
    }

    let tail = &parts[parts.len().saturating_sub(4)..];
    if display.starts_with("~/") {
        format!("~/.../{}", tail.join("/"))
    } else if display.starts_with('/') {
        format!("/.../{}", tail.join("/"))
    } else {
        format!(".../{}", tail.join("/"))
    }
}

fn source_path_text(path: Option<String>) -> Element<'static, Message> {
    let display = path.as_deref().map(compact_source_path).unwrap_or_default();

    container(
        text(display)
            .size(11)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::Word)
            .style(|theme: &iced::Theme| iced::widget::text::Style {
                color: Some(theme.palette().text.scale_alpha(0.58)),
            }),
    )
    .width(Length::Fill)
    .into()
}

fn catalog_item(skill: CatalogSkillMeta, is_selected: bool) -> Element<'static, Message> {
    let initials = catalog_skill_initials(&skill);
    let title = skill.title.clone();
    let description = skill.description.clone();
    let skill_id = skill.id.clone();
    let resource_text = if skill.resource_count == 0 {
        "仅 SKILL.md".to_string()
    } else {
        format!("{} 个附加资源", skill.resource_count)
    };
    let kind = skill.kind;
    let source = skill.source.clone();
    let source_path = skill.source_path.clone();

    let status_badge = if skill.installed {
        skill_badge(if skill.enabled { "已启用" } else { "已禁用" }, skill.enabled)
    } else {
        skill_badge("未安装", false)
    };

    let content = container(
        row![
            container(text(initials).size(16))
                .width(52)
                .height(52)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(move |theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    let background = match kind {
                        CatalogSkillKind::Recommended => {
                            palette.primary.weak.color.scale_alpha(0.7)
                        }
                        CatalogSkillKind::System => palette.secondary.weak.color.scale_alpha(0.55),
                        CatalogSkillKind::Personal => palette.success.weak.color.scale_alpha(0.55),
                    };
                    let text_color = match kind {
                        CatalogSkillKind::Recommended => palette.primary.base.color,
                        CatalogSkillKind::System => palette.secondary.base.color,
                        CatalogSkillKind::Personal => palette.success.base.color,
                    };

                    iced::widget::container::Style {
                        text_color: Some(text_color),
                        background: Some(Background::Color(background)),
                        border: Border { radius: 16.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                }),
            column![
                row![
                    text(title)
                        .size(17)
                        .width(Length::Fill)
                        .wrapping(iced::widget::text::Wrapping::Word),
                    skill_badge(
                        match kind {
                            CatalogSkillKind::Recommended => "Featured",
                            CatalogSkillKind::System => "Built-in",
                            CatalogSkillKind::Personal => "Local",
                        },
                        matches!(kind, CatalogSkillKind::Recommended),
                    ),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(description)
                    .size(13)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::Word)
                    .style(|theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(theme.palette().text.scale_alpha(0.7)),
                    }),
                row![
                    skill_badge(skill_id, false),
                    skill_badge(resource_text, false),
                    skill_badge(source_label(&source), false),
                    status_badge,
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                source_path_text(source_path),
            ]
            .spacing(10)
            .width(Length::Fill),
            skill_badge("点击查看", is_selected),
        ]
        .spacing(16)
        .align_y(Alignment::Start),
    )
    .padding([18, 20])
    .width(Length::Fill)
    .style(move |theme: &iced::Theme| {
        let mut style = settings_panel_style(theme);
        let palette = theme.extended_palette();
        let base_border = match kind {
            CatalogSkillKind::Recommended => palette.primary.base.color.scale_alpha(0.28),
            CatalogSkillKind::System => palette.background.strong.color.scale_alpha(0.55),
            CatalogSkillKind::Personal => palette.success.base.color.scale_alpha(0.24),
        };
        let base_background = match kind {
            CatalogSkillKind::Recommended => palette.primary.weak.color.scale_alpha(0.06),
            CatalogSkillKind::System => palette.background.weak.color.scale_alpha(0.08),
            CatalogSkillKind::Personal => palette.success.weak.color.scale_alpha(0.06),
        };

        style.background = Some(Background::Color(if is_selected {
            palette.primary.weak.color.scale_alpha(0.14)
        } else {
            base_background
        }));
        style.border.color =
            if is_selected { palette.primary.base.color.scale_alpha(0.54) } else { base_border };
        style.border.radius = 18.0.into();
        style.shadow = iced::Shadow::default();
        style
    });

    button(content)
        .width(Length::Fill)
        .padding(0)
        .on_press(Message::Settings(message::SettingsMessage::SkillsDetailRequested(skill.id)))
        .style(|_theme: &iced::Theme, _status| iced::widget::button::Style::default())
        .into()
}

/// 构建或处理 `catalog_group_section` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回本函数生成的状态、样式或辅助值，供同一流程继续组合使用。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn catalog_group_section<'a>(
    source: &'static str,
    skills: Vec<CatalogSkillMeta>,
    selected_skill_id: Option<&str>,
) -> Element<'a, Message> {
    let (icon, title, subtitle) = section_copy(source);
    let skill_count = skills.len();
    let rows = skills.into_iter().fold(column![].spacing(12), |column, skill| {
        let is_selected = selected_skill_id == Some(skill.id.as_str());
        column.push(catalog_item(skill, is_selected))
    });

    column![
        row![
            row![
                container(icon_svg(icon, 14.0)).padding([8, 8]).style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::container::Style {
                        text_color: Some(theme.palette().text.scale_alpha(0.76)),
                        background: Some(Background::Color(
                            palette.background.weak.color.scale_alpha(0.24),
                        )),
                        border: Border { radius: 10.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                }),
                column![
                    text(title).size(16),
                    text(subtitle).size(12).style(|theme: &iced::Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.palette().text.scale_alpha(0.62)),
                        }
                    }),
                ]
                .spacing(2),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .width(Length::Fill),
            skill_badge(format!("{skill_count}"), source == "workspace"),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        rows,
    ]
    .spacing(14)
    .into()
}
