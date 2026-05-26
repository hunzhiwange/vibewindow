//! 桌面应用顶部栏的按钮、菜单与窗口交互控件。
//!
//! 本模块主要负责把应用状态转换为桌面端可渲染的 Iced 控件，并把用户操作映射回上层消息。
//! 注释聚焦调用边界、返回值和错误传播方式，便于后续维护设置页与工具栏行为时快速定位职责。

use super::widgets::{menu_btn, menu_container, menu_item_btn, menu_separator};
use crate::app::components::overlays::BelowOverlay;
use crate::app::message::view::MenuType;
use crate::app::{App, Message, Screen, message};
use crate::apps::mindmap::MindMapMessage;
use iced::Element;
use iced::widget::column;

/// 构建或处理 `file_menu` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn file_menu(app: &App) -> Element<'_, Message> {
    let is_design = matches!(app.screen, Screen::Design);
    let is_mindmap = matches!(app.screen, Screen::MindMapTool);
    let is_project = matches!(app.screen, Screen::Project);
    let file_btn = menu_btn("文件", MenuType::File, app.active_menu);

    let file_content = menu_container(if is_design {
        column![
            menu_item_btn(
                "打开文件...",
                Some("⌘O"),
                Some(Message::Design(crate::app::message::DesignMessage::Open))
            ),
            menu_item_btn(
                "导入 Figma",
                None,
                Some(Message::Design(crate::app::message::DesignMessage::ToolSelected(
                    crate::app::views::design::models::DesignTool::ImportFigma
                )))
            ),
            menu_item_btn(
                "解析 Figma",
                None,
                Some(Message::Design(crate::app::message::DesignMessage::ParseFigma))
            ),
            menu_item_btn(
                "保存文件",
                Some("⌘S"),
                Some(Message::Design(crate::app::message::DesignMessage::Save))
            ),
            menu_item_btn(
                "另存为...",
                Some("⇧⌘S"),
                Some(Message::Design(crate::app::message::DesignMessage::SaveAs))
            ),
            menu_item_btn(
                "导出 HTML",
                None,
                Some(Message::Design(crate::app::message::DesignMessage::ExportHtml))
            ),
            menu_item_btn("返回首页", None, Some(Message::View(message::ViewMessage::GoHome))),
        ]
        .into()
    } else if is_mindmap {
        let can_save = app.active_mindmap_tab().is_some();
        column![
            menu_item_btn("新建", Some("⌘N"), Some(Message::MindMapTool(MindMapMessage::New))),
            menu_item_btn("打开", Some("⌘O"), Some(Message::MindMapTool(MindMapMessage::Open))),
            menu_item_btn(
                "保存",
                Some("⌘S"),
                can_save.then_some(Message::MindMapTool(MindMapMessage::Save))
            ),
            menu_item_btn(
                "另存为",
                Some("⇧⌘S"),
                can_save.then_some(Message::MindMapTool(MindMapMessage::SaveAs))
            ),
            menu_item_btn(
                "另存为JSON",
                None,
                can_save.then_some(Message::MindMapTool(MindMapMessage::SaveAsJson))
            ),
            menu_item_btn("返回首页", None, Some(Message::View(message::ViewMessage::GoHome))),
        ]
        .into()
    } else if is_project {
        column![
            menu_item_btn(
                "新建会话",
                None,
                Some(Message::View(message::ViewMessage::ProjectFileNewSession))
            ),
            menu_item_btn(
                "新建项目",
                None,
                Some(Message::View(message::ViewMessage::ProjectFileNewProject))
            ),
            menu_item_btn(
                "查看会话",
                None,
                Some(Message::View(message::ViewMessage::ProjectFileShowSessions))
            ),
            menu_item_btn(
                "查看项目",
                None,
                Some(Message::View(message::ViewMessage::ProjectFileShowProjects))
            ),
            menu_separator(),
            menu_item_btn(
                "保存文件",
                Some("⌘S"),
                Some(Message::Preview(crate::app::message::PreviewMessage::SaveFile))
            ),
            menu_item_btn(
                "保存所有文件",
                Some("⌥⌘S"),
                Some(Message::View(message::ViewMessage::ProjectFileSaveAll))
            ),
        ]
        .into()
    } else {
        column![
            menu_item_btn(
                "保存代码",
                Some("⌘S"),
                Some(Message::Preview(crate::app::message::PreviewMessage::SaveFile))
            ),
            menu_item_btn("返回首页", None, Some(Message::View(message::ViewMessage::GoHome))),
        ]
        .into()
    });

    BelowOverlay::new(file_btn, file_content)
        .show(app.active_menu == Some(MenuType::File))
        .on_close(Message::View(message::ViewMessage::ToggleMenu(None)))
        .into()
}

/// 构建或处理 `edit_menu` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn edit_menu(app: &App) -> Element<'_, Message> {
    let is_design = matches!(app.screen, Screen::Design);
    let is_mindmap = matches!(app.screen, Screen::MindMapTool);
    let is_project = matches!(app.screen, Screen::Project);
    let edit_btn = menu_btn("编辑", MenuType::Edit, app.active_menu);

    let edit_content = menu_container(if is_design {
        column![
            menu_item_btn(
                "撤销",
                Some("⌘Z"),
                Some(Message::Design(crate::app::message::DesignMessage::Undo))
            ),
            menu_item_btn(
                "恢复",
                Some("⇧⌘Z"),
                Some(Message::Design(crate::app::message::DesignMessage::Redo))
            ),
            menu_item_btn(
                "剪切",
                Some("⌘X"),
                Some(Message::Design(crate::app::message::DesignMessage::Cut))
            ),
            menu_item_btn(
                "复制",
                Some("⌘C"),
                Some(Message::Design(crate::app::message::DesignMessage::Copy))
            ),
            menu_item_btn(
                "粘贴",
                Some("⌘V"),
                Some(Message::Design(crate::app::message::DesignMessage::Paste))
            ),
        ]
        .into()
    } else if is_mindmap {
        let (can_undo, can_redo, can_cut, can_copy, can_paste, can_delete) =
            if let Some(tab) = app.active_mindmap_tab() {
                (
                    !tab.undo_stack.is_empty(),
                    !tab.redo_stack.is_empty(),
                    tab.selected_path.as_deref().is_some_and(|path| !path.is_empty()),
                    tab.selected_path.is_some(),
                    tab.selected_path.is_some() && tab.clipboard_node.is_some(),
                    tab.selected_path.as_deref().is_some_and(|path| !path.is_empty()),
                )
            } else {
                (false, false, false, false, false, false)
            };

        column![
            menu_item_btn(
                "撤销",
                Some("⌘Z"),
                can_undo.then_some(Message::MindMapTool(MindMapMessage::Undo))
            ),
            menu_item_btn(
                "恢复",
                Some("⇧⌘Z"),
                can_redo.then_some(Message::MindMapTool(MindMapMessage::Redo))
            ),
            menu_item_btn(
                "剪切",
                Some("⌘X"),
                can_cut.then_some(Message::MindMapTool(MindMapMessage::CutNode))
            ),
            menu_item_btn(
                "复制",
                Some("⌘C"),
                can_copy.then_some(Message::MindMapTool(MindMapMessage::CopyNode))
            ),
            menu_item_btn(
                "粘贴",
                Some("⌘V"),
                can_paste.then_some(Message::MindMapTool(MindMapMessage::PasteNode))
            ),
            menu_item_btn(
                "删除",
                Some("Delete"),
                can_delete.then_some(Message::MindMapTool(MindMapMessage::DeleteNode))
            ),
        ]
        .into()
    } else if is_project {
        column![
            menu_item_btn(
                "撤销",
                Some("⌘Z"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::Undo))
            ),
            menu_item_btn(
                "重做",
                Some("⇧⌘Z"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::Redo))
            ),
            menu_separator(),
            menu_item_btn(
                "剪切",
                Some("⌘X"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::Cut))
            ),
            menu_item_btn(
                "复制",
                Some("⌘C"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::Copy))
            ),
            menu_item_btn(
                "粘贴",
                Some("⌘V"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::Paste))
            ),
            menu_separator(),
            menu_item_btn(
                "搜索",
                Some("⌘F"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::OpenSearch))
            ),
            menu_item_btn(
                "替换",
                Some("⌥⌘F"),
                Some(Message::Editor(crate::app::message::editor::EditorMessage::OpenReplace))
            ),
            menu_separator(),
            menu_item_btn(
                "在文件中搜索",
                None,
                Some(Message::Project(message::ProjectMessage::FileTreeFindInProject))
            ),
            menu_item_btn(
                "在文件中替换",
                None,
                Some(Message::Project(message::ProjectMessage::FileTreeReplaceInProject))
            ),
        ]
        .into()
    } else {
        column![].into()
    });

    BelowOverlay::new(edit_btn, edit_content)
        .show(app.active_menu == Some(MenuType::Edit))
        .on_close(Message::View(message::ViewMessage::ToggleMenu(None)))
        .into()
}

/// 构建或处理 `view_menu` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn view_menu(app: &App) -> Element<'_, Message> {
    let view_btn = menu_btn("视图", MenuType::View, app.active_menu);
    let file_manager_panel_toggle_label =
        if app.show_file_manager { "隐藏右侧面板" } else { "显示右侧面板" };
    let file_manager_panel_toggle = menu_item_btn(
        file_manager_panel_toggle_label,
        None,
        Some(Message::View(message::ViewMessage::FileManagerPanelVisible(!app.show_file_manager))),
    );

    let file_manager_mode_toggle_label =
        if app.file_manager_show_changes { "切换到全部文件" } else { "切换到更改" };
    let file_manager_mode_toggle = menu_item_btn(
        file_manager_mode_toggle_label,
        None,
        Some(Message::Project(message::ProjectMessage::FileManagerShowChanges(
            !app.file_manager_show_changes,
        ))),
    );

    let view_content = menu_container(
        column![
            menu_item_btn(
                "切换左侧面板",
                None,
                Some(Message::View(message::ViewMessage::ToggleSettingsPanel))
            ),
            file_manager_panel_toggle,
            file_manager_mode_toggle,
            menu_item_btn(
                "切换 Diff 面板",
                None,
                Some(Message::View(message::ViewMessage::ToggleDiffPanel))
            ),
            menu_item_btn(
                "切换终端面板",
                None,
                Some(Message::View(message::ViewMessage::ToggleTerminalPanel))
            ),
            menu_item_btn(
                "缩小",
                Some("⌘-"),
                Some(Message::Design(message::DesignMessage::ZoomOut))
            ),
            menu_item_btn(
                "适应屏幕",
                Some("⌘0"),
                Some(Message::Design(message::DesignMessage::ZoomFit))
            ),
        ]
        .into(),
    );

    BelowOverlay::new(view_btn, view_content)
        .show(app.active_menu == Some(MenuType::View))
        .on_close(Message::View(message::ViewMessage::ToggleMenu(None)))
        .into()
}

/// 构建或处理 `help_menu` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回可交给 Iced 渲染树使用的 `Element`，其中已绑定必要的消息回调。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub(super) fn help_menu(app: &App) -> Element<'_, Message> {
    let help_btn = menu_btn("帮助", MenuType::Help, app.active_menu);
    let help_content = menu_container(
        column![
            menu_item_btn(
                "关于",
                None,
                Some(Message::View(message::ViewMessage::ToggleAboutModal))
            ),
            menu_separator(),
            menu_item_btn("重启", None, Some(Message::View(message::ViewMessage::RestartApp))),
            menu_item_btn(
                "安装 CLI 命令工具",
                None,
                Some(Message::View(message::ViewMessage::InstallCliTool))
            ),
            menu_item_btn(
                "检测更新",
                None,
                Some(Message::View(message::ViewMessage::OpenAppUpdateModal))
            ),
        ]
        .into(),
    );

    BelowOverlay::new(help_btn, help_content)
        .show(app.active_menu == Some(MenuType::Help))
        .on_close(Message::View(message::ViewMessage::ToggleMenu(None)))
        .into()
}
