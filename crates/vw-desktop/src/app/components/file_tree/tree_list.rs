//! 文件树列表渲染模块
//!
//! 本模块提供文件树的 UI 渲染功能，包括：
//! - 项目文件树列表：显示项目的目录结构和文件
//! - Git 变更列表：显示当前 Git 仓库中已修改的文件
//!
//! # 核心功能
//!
//! - 构建层级目录结构的可展开/折叠树形视图
//! - 支持文件和目录的右键菜单
//! - 支持文件拖拽操作
//! - 支持文件/目录的点击选中与预览

use iced::widget::{button, column, container, row, text};
use iced::{Background, Color, Element, Length, Theme};

use crate::app::assets::Icon;
use crate::app::components::file_tree::icons::{file_icon_for, static_icon_svg, themed_icon_svg};
use crate::app::components::file_tree::menu::build_file_tree_menu;
use crate::app::components::file_tree::model::FileTreeNode;
use crate::app::components::file_tree::widgets::RightClickArea;
use crate::app::components::overlays::PointBelowOverlay;
use crate::app::message::project::ProjectMessage;
use crate::app::{App, Message, message};

/// 构建项目文件树列表
///
/// 根据应用状态渲染项目目录的树形结构视图。支持：
/// - 目录的展开/折叠
/// - 文件的点击预览
/// - 右键菜单
/// - 拖拽操作
///
/// # 参数
///
/// * `app` - 应用状态引用，包含项目路径、文件索引、展开状态等信息
///
/// # 返回值
///
/// 返回渲染好的文件树 UI 元素
///
/// # 示例
///
/// ```ignore
/// let tree_element = build_file_tree_list(&app);
/// ```
pub fn build_file_tree_list(app: &App) -> Element<'_, Message> {
    /// 递归渲染目录节点
    ///
    /// 将目录树结构渲染为 UI 元素，包括目录项和文件项。
    /// 支持目录的展开/折叠、右键菜单、拖拽等交互。
    ///
    /// # 参数
    ///
    /// * `app` - 应用状态引用
    /// * `node` - 当前要渲染的目录节点
    /// * `prefix` - 当前目录的相对路径前缀
    /// * `depth` - 当前目录的层级深度（用于缩进）
    /// * `project_root` - 项目根目录的绝对路径
    ///
    /// # 返回值
    ///
    /// 返回渲染好的目录内容 UI 元素
    fn render_dir<'a>(
        app: &'a App,
        node: &FileTreeNode,
        prefix: String,
        depth: usize,
        project_root: &'a str,
    ) -> Element<'a, Message> {
        let mut col = column![].spacing(0).width(Length::Fill);
        // 根据深度计算缩进（每层两个空格）
        let indent = "  ".repeat(depth);

        // 遍历并渲染所有子目录
        for (name, child_node) in &node.children {
            // 构建完整的相对路径
            let full =
                if prefix.is_empty() { name.clone() } else { format!("{}/{}", prefix, name) };
            // 检查当前目录是否处于展开状态
            let expanded = app.is_file_tree_dir_expanded(&full);
            // 根据展开状态选择箭头图标
            let chevron_icon = if expanded { Icon::ChevronDown } else { Icon::ChevronRight };
            let folder_icon = Icon::FolderOpen;
            // 构建目录头部行：缩进 + 箭头 + 文件夹图标 + 目录名
            let header_row = row![
                text(indent.clone()).size(13),
                themed_icon_svg(chevron_icon),
                container(themed_icon_svg(folder_icon)).width(Length::Fixed(16.0)),
                text(full.split('/').next_back().unwrap_or(&full).to_string()).size(13)
            ]
            .spacing(4);
            // 构建可点击的目录按钮，点击时切换展开/折叠状态
            let header_btn = button(container(header_row).width(Length::Fill))
                .on_press(Message::Project(message::ProjectMessage::ToggleTreeDir(full.clone())))
                .style(move |theme: &Theme, status| {
                    let p = theme.palette().primary;
                    // 悬停时的背景色（主色的 10% 透明度）
                    let hover_bg = Color::from_rgba(p.r, p.g, p.b, 0.10);
                    // 按下时的背景色（主色的 18% 透明度）
                    let pressed_bg = Color::from_rgba(p.r, p.g, p.b, 0.18);
                    // 根据按钮状态选择背景色
                    let bg = match status {
                        iced::widget::button::Status::Hovered => hover_bg,
                        iced::widget::button::Status::Pressed => pressed_bg,
                        _ => Color::TRANSPARENT,
                    };
                    iced::widget::button::Style {
                        background: if bg != Color::TRANSPARENT {
                            Some(Background::Color(bg))
                        } else {
                            None
                        },
                        text_color: theme.palette().text,
                        border: iced::Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .width(Length::Fill);

            // 构建目录的绝对路径
            let abs_path =
                std::path::Path::new(project_root).join(&full).to_string_lossy().to_string();
            let abs_path_for_click = abs_path.clone();
            // 包装为支持右键菜单和拖拽的区域
            let right_click = Element::new(RightClickArea::new(
                header_btn.into(),
                Box::new(move |pos| {
                    Message::Project(ProjectMessage::FileTreeRightClicked(
                        abs_path_for_click.clone(),
                        "tree-dir".to_string(),
                        pos.x,
                        pos.y,
                    ))
                }),
                Some(Message::Project(ProjectMessage::FileTreeDragStart(abs_path.clone(), None))),
                Some(Message::Project(ProjectMessage::FileTreeDragEnd)),
            ));

            // 检查是否需要显示右键菜单（当前路径与菜单路径匹配）
            let item = if app.file_tree_menu_path.as_deref()
                == Some(
                    std::path::Path::new(project_root).join(&full).to_string_lossy().as_ref(),
                )
                && app.file_tree_menu_source.as_deref() == Some("tree-dir")
            {
                // 显示右键菜单覆盖层
                PointBelowOverlay::new(right_click, build_file_tree_menu(app, true))
                    .show(true)
                    .anchor(app.file_tree_menu_anchor.unwrap_or(iced::Point::ORIGIN))
                    .on_close(Message::Project(ProjectMessage::FileTreeMenuClose))
                    .into()
            } else {
                right_click
            };

            col = col.push(item);
            // 如果目录已展开，递归渲染子目录内容
            if expanded {
                col = col.push(render_dir(app, child_node, full, depth + 1, project_root));
            }
        }

        // 遍历并渲染所有文件
        for rel in &node.files {
            // 提取文件名（路径的最后一部分）
            let file_name = std::path::Path::new(rel)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(rel)
                .to_string();
            // 构建文件的绝对路径
            let abs = std::path::Path::new(project_root).join(rel).to_string_lossy().to_string();
            // 检查文件是否被选中（当前预览路径）
            let selected = app.active_preview_path.as_deref() == Some(&abs);
            let abs_clone = abs.clone();
            // 构建文件行：缩进 + 文件图标 + 文件名
            let content_row: Element<'_, Message, Theme> = row![
                text(indent.clone()).size(13),
                container(static_icon_svg(file_icon_for(&file_name))).width(Length::Fixed(16.0)),
                text(file_name).size(13)
            ]
            .spacing(4)
            .into();
            // 构建可点击的文件按钮，点击时打开文件预览
            let btn = button(container(content_row).width(Length::Fill))
                .on_press(Message::Preview(message::PreviewMessage::Open(abs_clone.clone())))
                .style(move |theme: &Theme, status| {
                    let p = theme.palette().primary;
                    // 悬停时的背景色
                    let hover_bg = Color::from_rgba(p.r, p.g, p.b, 0.10);
                    // 选中状态的背景色
                    let selected_bg = Color::from_rgba(p.r, p.g, p.b, 0.18);

                    // 基础背景色：如果已选中则显示选中背景
                    let base_bg =
                        if selected { Some(Background::Color(selected_bg)) } else { None };
                    // 根据按钮状态选择背景色
                    let bg = match status {
                        iced::widget::button::Status::Hovered => Some(Background::Color(hover_bg)),
                        iced::widget::button::Status::Pressed => {
                            Some(Background::Color(selected_bg))
                        }
                        _ => base_bg,
                    };
                    iced::widget::button::Style {
                        background: bg,
                        text_color: theme.palette().text,
                        border: iced::Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .width(Length::Fill);

            let abs_for_right_click = abs.clone();
            // 包装为支持右键菜单和拖拽的区域
            let right_click = Element::new(RightClickArea::new(
                btn.into(),
                Box::new(move |pos| {
                    Message::Project(ProjectMessage::FileTreeRightClicked(
                        abs_for_right_click.clone(),
                        "tree-file".to_string(),
                        pos.x,
                        pos.y,
                    ))
                }),
                Some(Message::Project(ProjectMessage::FileTreeDragStart(abs.clone(), None))),
                Some(Message::Project(ProjectMessage::FileTreeDragEnd)),
            ));

            // 检查是否需要显示右键菜单
            let item = if app.file_tree_menu_path.as_deref() == Some(&abs)
                && app.file_tree_menu_source.as_deref() == Some("tree-file")
            {
                PointBelowOverlay::new(right_click, build_file_tree_menu(app, true))
                    .show(true)
                    .anchor(app.file_tree_menu_anchor.unwrap_or(iced::Point::ORIGIN))
                    .on_close(Message::Project(ProjectMessage::FileTreeMenuClose))
                    .into()
            } else {
                right_click
            };

            col = col.push(item);
        }

        col.into()
    }

    let root = app.project_path.as_deref().unwrap_or("");
    let tree = app
        .current_file_tree_model()
        .filter(|tree| tree.has_entries())
        .cloned()
        .or_else(|| {
            (!root.is_empty() && !app.current_file_index().is_empty()).then(|| {
                crate::app::components::file_tree::model::build_file_tree_model(
                    root,
                    app.current_file_index(),
                )
            })
        })
        .filter(|tree| tree.has_entries());

    if let Some(tree) = tree {
        render_dir(app, &tree, String::new(), 0, root)
    } else {
        column![text("暂无文件").size(13).color(Color::from_rgb(0.5, 0.5, 0.5)),]
            .spacing(10)
            .padding(10)
            .align_x(iced::Alignment::Center)
            .into()
    }
}

/// 构建 Git 变更文件列表
///
/// 根据 Git 仓库状态渲染已修改文件的树形结构视图。支持：
/// - 目录的展开/折叠
/// - 变更文件的点击查看
/// - 右键菜单
/// - 拖拽操作
///
/// # 参数
///
/// * `app` - 应用状态引用，包含 Git 变更文件列表、展开状态等信息
///
/// # 返回值
///
/// 返回渲染好的变更文件列表 UI 元素
///
/// # 示例
///
/// ```ignore
/// let changes_element = build_changes_list(&app);
/// ```
pub fn build_changes_list(app: &App) -> Element<'_, Message> {
    let files = &app.git_changed_files;
    // 处理空列表情况
    if files.is_empty() {
        if app.git_changed_files_loading {
            // 显示加载中状态
            return column![text("加载中")].padding(10).into();
        }
        // 显示无变更状态
        return column![text("无变更")].padding(10).into();
    }

    use std::collections::BTreeMap;

    /// 目录节点结构体
    ///
    /// 用于构建变更文件树的层级数据结构。每个节点代表一个目录，
    /// 包含子目录映射和当前目录下的变更文件列表。
    #[derive(Default)]
    struct DirNode {
        /// 子目录映射：目录名 -> 子目录节点
        children: BTreeMap<String, DirNode>,
        /// 当前目录下的变更文件路径列表（相对路径）
        files: Vec<String>,
    }

    /// 将相对路径插入到目录树结构中
    ///
    /// 根据路径的分段信息，递归地在树结构中创建对应的节点。
    /// 路径的最后一段作为文件名添加到对应节点的 files 列表中。
    ///
    /// # 参数
    ///
    /// * `tree` - 目录树根节点
    /// * `rel` - 相对于项目根目录的文件路径
    fn insert_path(tree: &mut DirNode, rel: &str) {
        // 按路径分隔符拆分，过滤空字符串
        let parts = rel.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>();
        if parts.is_empty() {
            return;
        }
        // 如果只有一个部分，说明是根目录下的文件
        if parts.len() == 1 {
            tree.files.push(rel.to_string());
            return;
        }
        let mut node = tree;
        // 遍历路径分段，构建目录层级
        for (i, part) in parts.iter().enumerate() {
            // 路径的最后一段是文件名
            if i == parts.len() - 1 {
                node.files.push(rel.to_string());
                return;
            }
            // 否则进入或创建对应的子目录节点
            node = node.children.entry(part.to_string()).or_default();
        }
    }

    /// 递归渲染变更目录节点
    ///
    /// 将变更文件树结构渲染为 UI 元素，包括目录项和文件项。
    /// 支持目录的展开/折叠、右键菜单、拖拽等交互。
    ///
    /// # 参数
    ///
    /// * `app` - 应用状态引用
    /// * `node` - 当前要渲染的目录节点
    /// * `prefix` - 当前目录的相对路径前缀
    /// * `depth` - 当前目录的层级深度（用于缩进）
    /// * `project_root` - 项目根目录的绝对路径
    ///
    /// # 返回值
    ///
    /// 返回渲染好的目录内容 UI 元素
    fn render_dir<'a>(
        app: &'a App,
        node: DirNode,
        prefix: String,
        depth: usize,
        project_root: &'a str,
    ) -> Element<'a, Message> {
        let mut col = column![].spacing(0).width(Length::Fill);
        // 根据深度计算缩进（每层两个空格）
        let indent = "  ".repeat(depth);

        // 遍历并渲染所有子目录
        for (name, child) in node.children.into_iter() {
            // 构建完整的相对路径
            let full = if prefix.is_empty() { name } else { format!("{}/{}", prefix, name) };
            // 变更列表使用 "chg:" 前缀作为展开状态的键
            let key = format!("chg:{}", full);
            // 检查当前目录是否处于展开状态
            let expanded = app.is_file_tree_dir_expanded(&key);
            // 根据展开状态选择箭头图标
            let chevron_icon = if expanded { Icon::ChevronDown } else { Icon::ChevronRight };
            // 构建目录头部行：缩进 + 箭头 + 文件夹图标 + 目录名
            let header_row = row![
                text(indent.clone()).size(13),
                themed_icon_svg(chevron_icon),
                container(themed_icon_svg(Icon::FolderOpen)).width(Length::Fixed(16.0)),
                text(full.split('/').next_back().unwrap_or(&full).to_string()).size(13)
            ]
            .spacing(4);
            // 构建可点击的目录按钮，点击时切换展开/折叠状态
            let header_btn = button(container(header_row).width(Length::Fill))
                .on_press(Message::Project(message::ProjectMessage::ToggleTreeDir(key.clone())))
                .style(move |theme: &Theme, status| {
                    let p = theme.palette().primary;
                    // 悬停时的背景色（主色的 10% 透明度）
                    let hover_bg = Color::from_rgba(p.r, p.g, p.b, 0.10);
                    // 根据按钮状态选择背景色
                    let bg = match status {
                        iced::widget::button::Status::Hovered => hover_bg,
                        iced::widget::button::Status::Pressed => hover_bg,
                        _ => Color::TRANSPARENT,
                    };
                    iced::widget::button::Style {
                        background: if bg != Color::TRANSPARENT {
                            Some(Background::Color(bg))
                        } else {
                            None
                        },
                        text_color: theme.palette().text,
                        border: iced::Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .width(Length::Fill);

            // 构建目录的绝对路径
            let abs_path =
                std::path::Path::new(project_root).join(&full).to_string_lossy().to_string();
            let abs_path_for_click = abs_path.clone();
            // 包装为支持右键菜单和拖拽的区域
            let right_click = Element::new(RightClickArea::new(
                header_btn.into(),
                Box::new(move |pos| {
                    Message::Project(ProjectMessage::FileTreeRightClicked(
                        abs_path_for_click.clone(),
                        "changes-dir".to_string(),
                        pos.x,
                        pos.y,
                    ))
                }),
                Some(Message::Project(ProjectMessage::FileTreeDragStart(abs_path.clone(), None))),
                Some(Message::Project(ProjectMessage::FileTreeDragEnd)),
            ));

            // 检查是否需要显示右键菜单（当前路径与菜单路径匹配）
            let item = if app.file_tree_menu_path.as_deref() == Some(&abs_path)
                && app.file_tree_menu_source.as_deref() == Some("changes-dir")
            {
                // 显示右键菜单覆盖层
                PointBelowOverlay::new(right_click, build_file_tree_menu(app, true))
                    .show(true)
                    .anchor(app.file_tree_menu_anchor.unwrap_or(iced::Point::ORIGIN))
                    .on_close(Message::Project(ProjectMessage::FileTreeMenuClose))
                    .into()
            } else {
                right_click
            };

            col = col.push(item);
            // 如果目录已展开，递归渲染子目录内容
            if expanded {
                col = col.push(render_dir(app, child, full, depth + 1, project_root));
            }
        }

        // 遍历并渲染所有变更文件
        for rel in node.files.into_iter() {
            // 提取文件名（路径的最后一部分）
            let file_name = std::path::Path::new(&rel)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&rel)
                .to_string();
            // 检查文件是否被选中（当前聚焦的变更文件）
            let selected = app.git_focused_file.as_deref() == Some(rel.as_str());
            // 构建文件行：缩进 + 文件图标 + 文件名
            let content_row = row![
                text(indent.clone()).size(13),
                container(static_icon_svg(file_icon_for(&file_name))).width(Length::Fixed(16.0)),
                text(file_name).size(13)
            ]
            .spacing(4);
            // 构建文件的绝对路径
            let abs = std::path::Path::new(project_root).join(&rel).to_string_lossy().to_string();
            let rel_for_open = rel.clone();
            // 构建可点击的文件按钮，点击时打开变更文件查看
            let btn = button(container(content_row).width(Length::Fill))
                .on_press(Message::Project(message::ProjectMessage::OpenChangedFile(rel_for_open)))
                .style(move |theme: &Theme, status| {
                    let p = theme.palette().primary;
                    // 悬停时的背景色
                    let hover_bg = Color::from_rgba(p.r, p.g, p.b, 0.10);

                    // 基础背景色：如果已选中则显示悬停背景
                    let base_bg = if selected { Some(Background::Color(hover_bg)) } else { None };
                    // 根据按钮状态选择背景色
                    let bg = match status {
                        iced::widget::button::Status::Hovered => Some(Background::Color(hover_bg)),
                        iced::widget::button::Status::Pressed => Some(Background::Color(hover_bg)),
                        _ => base_bg,
                    };
                    iced::widget::button::Style {
                        background: bg,
                        text_color: theme.palette().text,
                        border: iced::Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .width(Length::Fill);

            let abs_for_click = abs.clone();
            // 包装为支持右键菜单和拖拽的区域
            let right_click = Element::new(RightClickArea::new(
                btn.into(),
                Box::new(move |pos| {
                    Message::Project(ProjectMessage::FileTreeRightClicked(
                        abs_for_click.clone(),
                        "changes-file".to_string(),
                        pos.x,
                        pos.y,
                    ))
                }),
                Some(Message::Project(ProjectMessage::FileTreeDragStart(abs.clone(), None))),
                Some(Message::Project(ProjectMessage::FileTreeDragEnd)),
            ));

            // 检查是否需要显示右键菜单
            let item = if app.file_tree_menu_path.as_deref() == Some(&abs)
                && app.file_tree_menu_source.as_deref() == Some("changes-file")
            {
                PointBelowOverlay::new(right_click, build_file_tree_menu(app, true))
                    .show(true)
                    .anchor(app.file_tree_menu_anchor.unwrap_or(iced::Point::ORIGIN))
                    .on_close(Message::Project(ProjectMessage::FileTreeMenuClose))
                    .into()
            } else {
                right_click
            };

            col = col.push(item);
        }

        col.into()
    }

    // 获取项目根路径
    let root = app.project_path.as_deref().unwrap_or("");
    // 初始化目录树根节点
    let mut tree = DirNode::default();
    // 将所有变更文件路径插入目录树
    for f in files {
        insert_path(&mut tree, f);
    }

    // 渲染变更文件树
    render_dir(app, tree, String::new(), 0, root)
}
