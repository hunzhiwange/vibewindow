//! Redis 工具详情模块，负责连接、命令、键空间和运行时信息面板。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    settings_muted_text_style, settings_panel, settings_text_input_style, settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::{App, Message};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};
use std::collections::{BTreeMap, HashSet};

use super::super::common::{build_detail_action_button, redis_scroll_direction, themed_icon_svg};

#[derive(Debug, Default, Clone)]
struct RedisKeyTreeNode {
    label: String,
    full_key: Option<String>,
    children: BTreeMap<String, RedisKeyTreeNode>,
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `is_busy`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_key_tree_panel<'a>(app: &'a App, is_busy: bool) -> Element<'a, Message> {
    let has_selected_connection = app.redis_tool.selected_connection_id.is_some();
    let mut content = column![
        row![
            column![
                text("键树浏览").size(14),
                text("通过网关执行 SCAN + MATCH，按页加载后在本地按冒号层级归并。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            settings_value_badge(format!("已加载 {} 项", app.redis_tool.key_browser_items.len())),
            settings_value_badge(if app.redis_tool.key_browser_has_more {
                "可继续加载"
            } else {
                "当前批次已到底"
            }),
            if let Some(selected_key) = &app.redis_tool.selected_key {
                settings_value_badge(truncate_key_label(selected_key))
            } else {
                Space::new().width(Length::Shrink).into()
            },
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        row![
            text_input("键模式，例如 * 或 order:*", &app.redis_tool.key_browser_pattern)
                .on_input(|value| Message::RedisTool(RedisToolMessage::KeyBrowserPatternChanged(
                    value
                )))
                .on_submit(Message::RedisTool(RedisToolMessage::ReloadSelectedKeys))
                .padding([8, 10])
                .size(12)
                .width(Length::Fill)
                .style(settings_text_input_style),
            build_detail_action_button(
                "重载键树",
                Message::RedisTool(RedisToolMessage::ReloadSelectedKeys),
                false,
                has_selected_connection && !is_busy,
            ),
            build_detail_action_button(
                "新增 Key",
                Message::RedisTool(RedisToolMessage::OpenCreateKeyModal),
                true,
                has_selected_connection && !is_busy,
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    if app.redis_tool.key_browser_items.is_empty() {
        content = content.push(
            text(
                "当前还没有已加载的键。选择连接后可直接重载键树，认证失败时会在顶部显示网关错误。",
            )
            .size(12)
            .style(settings_muted_text_style),
        );
    } else {
        let tree = build_key_tree_root(&app.redis_tool.key_browser_items);
        let mut rows = column![].spacing(6).width(Length::Fill);
        for (segment, node) in tree.children {
            rows = rows.push(render_key_tree_node(
                node,
                segment,
                0,
                &app.redis_tool.key_tree_expanded_paths,
                app.redis_tool.selected_key.as_deref(),
            ));
        }

        content = content.push(
            container(scrollable(rows).direction(redis_scroll_direction()).height(Length::Fill))
                .padding([4, 0])
                .width(Length::Fill)
                .height(Length::Fill),
        );
    }

    if app.redis_tool.key_browser_has_more {
        content = content.push(
            row![
                Space::new().width(Length::Fill),
                build_detail_action_button(
                    "加载更多",
                    Message::RedisTool(RedisToolMessage::LoadMoreKeys),
                    true,
                    has_selected_connection && !is_busy,
                ),
            ]
            .align_y(Alignment::Center),
        );
    }

    settings_panel(content.height(Length::Fill)).height(Length::Fill).into()
}

fn build_key_tree_root(keys: &[String]) -> RedisKeyTreeNode {
    let mut root = RedisKeyTreeNode::default();
    for key in keys {
        insert_key_tree(&mut root, key);
    }
    root
}

fn insert_key_tree(root: &mut RedisKeyTreeNode, key: &str) {
    let mut node = root;
    for segment in key.split(':').filter(|segment| !segment.trim().is_empty()) {
        node = node.children.entry(segment.to_string()).or_insert_with(|| RedisKeyTreeNode {
            label: segment.to_string(),
            full_key: None,
            children: BTreeMap::new(),
        });
    }
    node.full_key = Some(key.to_string());
}

fn render_key_tree_node(
    node: RedisKeyTreeNode,
    path: String,
    depth: usize,
    expanded_paths: &HashSet<String>,
    selected_key: Option<&str>,
) -> Element<'static, Message> {
    if node.children.is_empty() {
        let label = node.full_key.unwrap_or(node.label);
        let selected = selected_key == Some(label.as_str());
        return build_key_tree_leaf_row(label, depth, false, selected);
    }

    let is_expanded = expanded_paths.contains(&path);
    let terminal_key = node.full_key.clone();
    let mut content = column![build_key_tree_branch_row(
        node.label.clone(),
        path.clone(),
        depth,
        is_expanded,
        count_terminal_keys(&node),
    )]
    .spacing(6)
    .width(Length::Fill);

    if is_expanded {
        if let Some(full_key) = terminal_key {
            let selected = selected_key == Some(full_key.as_str());
            content = content.push(build_key_tree_leaf_row(full_key, depth + 1, true, selected));
        }
        for (segment, child) in node.children {
            let child_path = format!("{path}:{segment}");
            content = content.push(render_key_tree_node(
                child,
                child_path,
                depth + 1,
                expanded_paths,
                selected_key,
            ));
        }
    }

    content.into()
}

fn build_key_tree_branch_row(
    label: String,
    path: String,
    depth: usize,
    is_expanded: bool,
    item_count: usize,
) -> Element<'static, Message> {
    let indent = depth as f32 * 18.0;
    let icon = if is_expanded { Icon::ChevronDown } else { Icon::ChevronRight };

    button(
        container(
            row![
                Space::new().width(Length::Fixed(indent)),
                themed_icon_svg(icon, 12.0),
                themed_icon_svg(Icon::FolderOpen, 14.0),
                text(label).size(12),
                Space::new().width(Length::Fill),
                settings_value_badge(format!("{} 项", item_count)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 10])
        .width(Length::Fill)
        .style(branch_row_container_style),
    )
    .on_press(Message::RedisTool(RedisToolMessage::ToggleKeyTreePath(path)))
    .style(button::text)
    .width(Length::Fill)
    .into()
}

fn build_key_tree_leaf_row(
    label: String,
    depth: usize,
    exact_key_child: bool,
    selected: bool,
) -> Element<'static, Message> {
    let indent = depth as f32 * 18.0 + 28.0;
    button(
        container(
            row![
                Space::new().width(Length::Fixed(indent)),
                themed_icon_svg(Icon::FileText, 12.0),
                text(label.clone()).size(11).style(move |theme: &Theme| {
                    leaf_row_text_style(theme, selected, exact_key_child)
                }),
                Space::new().width(Length::Fill),
                if selected {
                    settings_value_badge("已选中")
                } else {
                    Space::new().width(Length::Shrink).into()
                },
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme| leaf_row_container_style(theme, selected)),
    )
    .on_press(Message::RedisTool(RedisToolMessage::SelectKey(label)))
    .style(button::text)
    .width(Length::Fill)
    .into()
}

fn branch_row_container_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(Background::Color(palette.background.base.color.scale_alpha(0.42))),
        border: Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.18),
            radius: 14.0.into(),
        },
        ..Default::default()
    }
}

fn leaf_row_text_style(
    theme: &Theme,
    selected: bool,
    exact_key_child: bool,
) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(if selected {
            theme.palette().primary
        } else if exact_key_child {
            theme.palette().text.scale_alpha(0.96)
        } else {
            theme.palette().text.scale_alpha(0.88)
        }),
    }
}

fn leaf_row_container_style(theme: &Theme, selected: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(Background::Color(if selected {
            theme.palette().primary.scale_alpha(0.12)
        } else {
            Color::TRANSPARENT
        })),
        border: Border {
            width: 1.0,
            color: if selected {
                theme.palette().primary.scale_alpha(0.22)
            } else {
                palette.background.strong.color.scale_alpha(0.0)
            },
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

fn count_terminal_keys(node: &RedisKeyTreeNode) -> usize {
    usize::from(node.full_key.is_some())
        + node.children.values().map(count_terminal_keys).sum::<usize>()
}

fn truncate_key_label(key: &str) -> String {
    const LIMIT: usize = 20;
    if key.chars().count() <= LIMIT {
        return format!("选中 {key}");
    }

    let preview = key.chars().take(LIMIT).collect::<String>();
    format!("选中 {preview}...")
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod keys_tests;
