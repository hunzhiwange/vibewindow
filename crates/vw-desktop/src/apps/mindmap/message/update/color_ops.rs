//! 思维导图颜色操作更新逻辑，处理节点、边线和主题颜色变更。

use crate::app::views::design::models::ColorFormat;
use crate::app::{App, Message};
use crate::apps::mindmap::canvas::theme::{
    CUSTOM_THEME_GROUP_ID, MindMapCustomTheme, resolve_theme, theme_group_variant_count,
};
use crate::apps::mindmap::state::{EdgeStyle, MindMapColorPicker, MindMapColorTarget};
use iced::{Color, Task};

use super::super::persist::persist;

pub(super) fn rgba_u32_from_color(color: Color) -> u32 {
    let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

/// 构建或更新 open color picker 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn open_color_picker(
    app: &mut App,
    target: MindMapColorTarget,
    color: Color,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let format = tab.active_color_picker.as_ref().map(|p| p.format).unwrap_or(ColorFormat::Hex);
        tab.active_color_picker =
            Some(MindMapColorPicker { color, format, target, picking: false });
        tab.show_diagram_type_picker = false;
        tab.show_markdown_import = false;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_action_menu = false;
        tab.show_theme_panel = false;
    }
    Task::none()
}

/// 构建或更新 color picker changed 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn color_picker_changed(app: &mut App, color: Color) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let Some(active) = tab.active_color_picker.clone() else {
            return Task::none();
        };
        tab.active_color_picker = Some(MindMapColorPicker { color, ..active.clone() });

        let rgba = rgba_u32_from_color(color);
        match active.target {
            MindMapColorTarget::NodeFill => {
                if let Some(path) = tab.selected_path.clone() {
                    tab.node_fills.insert(path, rgba);
                }
            }
            MindMapColorTarget::NodeText => {
                if let Some(path) = tab.selected_path.clone() {
                    tab.node_text_colors.insert(path, rgba);
                }
            }
            MindMapColorTarget::NodeBorder => {
                if let Some(path) = tab.selected_path.clone() {
                    tab.node_border_colors.insert(path, rgba);
                }
            }
            MindMapColorTarget::EdgeStroke => {
                if let Some(path) = tab.selected_path.clone()
                    && !path.is_empty() {
                        tab.edge_colors.insert(path, rgba);
                    }
            }
            MindMapColorTarget::Background => {
                tab.background = Some(rgba);
            }
        }
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 color picker format changed 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn color_picker_format_changed(app: &mut App, format: ColorFormat) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut()
        && let Some(active) = tab.active_color_picker.clone() {
            tab.active_color_picker = Some(MindMapColorPicker { format, ..active });
        }
    Task::none()
}

/// 构建或更新 reset color target 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn reset_color_target(app: &mut App, target: MindMapColorTarget) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let default_color = match target {
            MindMapColorTarget::NodeFill => Color::from_rgba8(255, 255, 255, 1.0),
            MindMapColorTarget::NodeText => Color::from_rgba8(17, 24, 39, 1.0),
            MindMapColorTarget::NodeBorder | MindMapColorTarget::EdgeStroke => {
                Color::from_rgba8(208, 215, 222, 1.0)
            }
            MindMapColorTarget::Background => Color::from_rgba8(255, 255, 255, 1.0),
        };

        match target {
            MindMapColorTarget::NodeFill => {
                if let Some(path) = tab.selected_path.clone() {
                    tab.node_fills.remove(&path);
                }
            }
            MindMapColorTarget::NodeText => {
                if let Some(path) = tab.selected_path.clone() {
                    tab.node_text_colors.remove(&path);
                }
            }
            MindMapColorTarget::NodeBorder => {
                if let Some(path) = tab.selected_path.clone() {
                    tab.node_border_colors.remove(&path);
                }
            }
            MindMapColorTarget::EdgeStroke => {
                if let Some(path) = tab.selected_path.clone()
                    && !path.is_empty() {
                        tab.edge_colors.remove(&path);
                    }
            }
            MindMapColorTarget::Background => {
                tab.background = None;
            }
        }

        if let Some(active) = tab.active_color_picker.as_mut()
            && active.target == target
        {
            active.color = default_color;
        }

        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set background 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_background(app: &mut App, bg: Option<u32>) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.background = bg;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set theme group 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_theme_group(app: &mut App, group_id: String) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.theme_group = group_id;
        tab.theme_variant = 0;
        tab.follow_theme_background = true;
        tab.background = None;
        tab.edge_colors.clear();
        tab.edge_styles.clear();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set theme variant 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_theme_variant(app: &mut App, group_id: String, variant: usize) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let n = if group_id == CUSTOM_THEME_GROUP_ID {
            tab.custom_themes.len().max(1)
        } else {
            theme_group_variant_count(&group_id).max(1)
        };
        tab.theme_group = group_id;
        tab.theme_variant = variant % n;
        tab.follow_theme_background = true;
        tab.background = None;
        tab.edge_colors.clear();
        tab.edge_styles.clear();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 save theme to custom 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn save_theme_to_custom(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let view = resolve_theme(&tab.theme_group, tab.theme_variant, &tab.custom_themes);
        let custom = MindMapCustomTheme {
            background_color: view.background_color,
            root_fill: view.root_fill,
            root_text: view.root_text,
            branch_fills: view.branch_fills.to_vec(),
            branch_text: view.branch_text,
            leaf_fill: view.leaf_fill,
            leaf_text: view.leaf_text,
            line_color: view.line_color,
            is_dark: view.is_dark,
        };
        tab.custom_themes.push(custom);
        tab.theme_group = CUSTOM_THEME_GROUP_ID.to_string();
        tab.theme_variant = tab.custom_themes.len().saturating_sub(1);
        tab.follow_theme_background = true;
        tab.background = None;
        tab.edge_colors.clear();
        tab.edge_styles.clear();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 delete custom theme 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn delete_custom_theme(app: &mut App, index: usize) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if tab.custom_themes.is_empty() {
            return Task::none();
        }
        let idx = index.min(tab.custom_themes.len().saturating_sub(1));
        tab.custom_themes.remove(idx);

        if tab.custom_themes.is_empty() {
            tab.theme_group = "classic".to_string();
            tab.theme_variant = 0;
        } else if tab.theme_group == CUSTOM_THEME_GROUP_ID {
            tab.theme_variant = tab.theme_variant.min(tab.custom_themes.len().saturating_sub(1));
        }

        tab.background = None;
        tab.edge_colors.clear();
        tab.edge_styles.clear();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 cancel theme background 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn cancel_theme_background(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.follow_theme_background = false;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set edge style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_edge_style(app: &mut App, style: EdgeStyle) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if let Some(path) = tab.selected_path.clone() {
            if !path.is_empty() {
                tab.edge_styles.insert(path, style);
            } else {
                tab.edge_style = style;
            }
        } else {
            tab.edge_style = style;
        }
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set node border style 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_node_border_style(app: &mut App, style: EdgeStyle) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if let Some(path) = tab.selected_path.clone() {
            if !path.is_empty() {
                tab.node_border_styles.insert(path, style);
            } else {
                tab.node_border_style = style;
            }
        } else {
            tab.node_border_style = style;
        }
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}
