//! 思维导图应用头部视图模块
//!
//! 本模块负责渲染思维导图应用的顶部工具栏界面，包括：
//! - 文件操作按钮（新建、打开、保存、另存为等）
//! - 导出功能按钮（PNG、JPEG、SVG 格式）
//! - 缩放控制按钮
//! - 图表类型选择器
//! - Markdown 大纲导入功能
//!
//! 该模块是思维导图视图层的一部分，通过 Iced 框架构建声明式 UI。

use crate::app::Message;
use crate::app::components::overlays::BelowOverlay;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapDiagramType, MindMapTab};
use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Alignment, Border, Element, Length, Theme};

use super::common::picker_style;

/// 渲染思维导图应用的头部工具栏
///
/// 该函数构建并返回一个包含完整工具栏的 UI 元素，工具栏布局为：
/// - 左侧：当前标签页标题
/// - 右侧：操作按钮组（文件操作、导出、图表类型、Markdown 导入、缩放控制）
///
/// # 参数
///
/// * `tab_opt` - 当前激活的思维导图标签页的可选引用
///   - `Some(tab)`: 存在激活的标签页，从标签页获取标题、缩放比例、图表类型等状态
///   - `None`: 无激活标签页，使用默认值渲染工具栏
///
/// # 返回值
///
/// 返回一个 `Element<Message>` 类型的 UI 元素，代表完整的头部工具栏界面
///
/// # UI 结构
///
/// 工具栏包含以下几个主要部分：
/// 1. **标题区域** - 显示当前标签页的标题，默认为"思维导图"
/// 2. **文件操作区** - 新建、打开、保存、另存为、另存为 JSON
/// 3. **导出操作区** - 导出 PNG、JPEG、SVG（仅在标签页存在时可点击）
/// 4. **图表类型选择器** - 切换不同的图表展示形式（思维导图、组织架构图等）
/// 5. **Markdown 导入** - 支持 Markdown 大纲导入功能
/// 6. **缩放控制** - 放大、缩小及显示当前缩放比例
///
/// # 示例
///
/// ```ignore
/// let tab = MindMapTab::new("我的项目".to_string());
/// let header = render(Some(&tab));
/// // 返回包含工具栏的 UI 元素
/// ```
#[allow(dead_code)]
pub(super) fn render<'a>(tab_opt: Option<&'a MindMapTab>) -> Element<'a, Message> {
    // 提取标题：从标签页获取标题，若不存在则使用默认值"思维导图"
    let title = tab_opt.map(|t| t.title.clone()).unwrap_or_else(|| "思维导图".to_string());

    // 构建左侧标题区域：包含标题文本，垂直居中对齐
    let left = row![text(title).size(18)].spacing(10).align_y(Alignment::Center);

    // 构建文件操作按钮行：新建、打开、保存、另存为、另存为 JSON
    let file_actions_row = row![
        button(text("新建"))
            .style(button::primary)
            .on_press(Message::MindMapTool(MindMapMessage::New))
            .padding([6, 12]),
        button(text("打开"))
            .style(button::secondary)
            .on_press(Message::MindMapTool(MindMapMessage::Open))
            .padding([6, 12]),
        button(text("保存"))
            .style(button::secondary)
            .on_press(Message::MindMapTool(MindMapMessage::Save))
            .padding([6, 12]),
        button(text("另存为"))
            .style(button::secondary)
            .on_press(Message::MindMapTool(MindMapMessage::SaveAs))
            .padding([6, 12]),
        button(text("另存为JSON"))
            .style(button::secondary)
            .on_press(Message::MindMapTool(MindMapMessage::SaveAsJson))
            .padding([6, 12]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // 构建导出操作按钮行：PNG、JPEG、SVG
    // 当标签页存在时，按钮可点击并触发导出消息；否则按钮处于禁用状态
    let export_actions_row: Element<'_, Message> = if tab_opt.is_some() {
        row![
            button(text("导出 PNG"))
                .style(button::secondary)
                .on_press(Message::MindMapTool(MindMapMessage::ExportPng))
                .padding([6, 12]),
            button(text("导出 JPEG"))
                .style(button::secondary)
                .on_press(Message::MindMapTool(MindMapMessage::ExportJpeg))
                .padding([6, 12]),
            button(text("导出 SVG"))
                .style(button::secondary)
                .on_press(Message::MindMapTool(MindMapMessage::ExportSvg))
                .padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    } else {
        // 无标签页时，导出按钮保持显示但不可点击
        row![
            button(text("导出 PNG")).style(button::secondary).padding([6, 12]),
            button(text("导出 JPEG")).style(button::secondary).padding([6, 12]),
            button(text("导出 SVG")).style(button::secondary).padding([6, 12]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    // 将文件操作和导出操作组合成一个垂直布局
    let file_actions = column![file_actions_row, export_actions_row]
        .spacing(6)
        .align_x(iced::alignment::Horizontal::Left);

    // 构建缩放控制组件：缩小按钮、缩放比例显示、放大按钮
    let zoom_controls = {
        // 获取当前缩放比例标签：从标签页读取缩放值并格式化为百分比
        let zoom_label = tab_opt
            .map(|t| format!("{:.0}%", t.zoom * 100.0))
            .unwrap_or_else(|| "100%".to_string());

        row![
            // 缩小按钮：缩放因子为 1/1.10（约 0.91）
            button(text("−"))
                .style(button::secondary)
                .on_press(Message::MindMapTool(MindMapMessage::Zoom(1.0 / 1.10, None)))
                .padding([6, 10]),
            // 显示当前缩放比例
            text(zoom_label).size(13),
            // 放大按钮：缩放因子为 1.10
            button(text("+"))
                .style(button::secondary)
                .on_press(Message::MindMapTool(MindMapMessage::Zoom(1.10, None)))
                .padding([6, 10]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    };

    // 获取图表类型选择器的显示状态
    let show_diagram_type_picker = tab_opt.map(|t| t.show_diagram_type_picker).unwrap_or(false);

    // 构建图表类型选择器：支持多种图表布局形式
    let diagram_type_picker: Element<'_, Message> = if let Some(tab) = tab_opt {
        // 构建下拉菜单内容
        let overlay = {
            // 定义所有支持的图表类型
            let types = [
                MindMapDiagramType::MindMap,  // 思维导图
                MindMapDiagramType::OrgChart, // 组织架构图
                MindMapDiagramType::Fishbone, // 鱼骨图
                MindMapDiagramType::Timeline, // 时间线
                MindMapDiagramType::Tree,     // 树形图
                MindMapDiagramType::Bracket,  // 括号图
            ];

            // 创建单个图表类型按钮的辅助函数
            // 当前选中的类型使用 primary 样式，其他使用 secondary 样式
            let type_btn = |t: MindMapDiagramType| {
                let active = tab.diagram_type == t;
                let b = if active {
                    button(text(t.label())).style(button::primary)
                } else {
                    button(text(t.label())).style(button::secondary)
                };
                b.on_press(Message::MindMapTool(MindMapMessage::SelectDiagramType(t)))
                    .padding([6, 10])
            };

            // 构建类型列表：依次添加所有图表类型按钮
            let mut type_list = column![].spacing(6).width(Length::Fixed(130.0));
            for t in types {
                type_list = type_list.push(type_btn(t));
            }

            // 将类型列表包装在容器中，添加标题和样式
            container(column![text("图表类型").size(13), type_list,].spacing(10).padding(12))
                .style(picker_style)
        };

        // 创建当前选中的图表类型标签
        let label = format!("类型: {}", tab.diagram_type.label());
        // 使用 BelowOverlay 组件创建下拉菜单效果
        BelowOverlay::new(
            button(text(label))
                .style(button::secondary)
                .padding([6, 12])
                .on_press(Message::MindMapTool(MindMapMessage::ToggleDiagramTypePicker)),
            overlay,
        )
        .show(show_diagram_type_picker) // 控制下拉菜单的显示/隐藏
        .gap(6.0) // 按钮与下拉菜单之间的间距
        .on_close(Message::MindMapTool(MindMapMessage::ClosePickers)) // 关闭菜单的消息
        .into()
    } else {
        // 无标签页时，显示默认类型按钮（不可点击）
        button(text("类型: 思维导图")).style(button::secondary).padding([6, 12]).into()
    };

    // 获取 Markdown 导入面板的显示状态
    let show_markdown_import = tab_opt.map(|t| t.show_markdown_import).unwrap_or(false);

    // 构建 Markdown 导入面板：支持将 Markdown 大纲转换为思维导图
    let md_import: Element<'_, Message> = if let Some(tab) = tab_opt {
        // 构建 Markdown 编辑器的下拉面板
        let overlay = container(
            column![
                text("Markdown 大纲").size(13),
                // 文本编辑器：用于输入 Markdown 内容
                container(
                    text_editor(&tab.markdown_import_editor)
                        .height(Length::Fixed(220.0))
                        .on_action(|a| {
                            Message::MindMapTool(MindMapMessage::MarkdownImportEditorAction(a))
                        })
                        .padding(10)
                )
                .style(|theme: &Theme| iced::widget::container::Style {
                    background: Some(theme.extended_palette().background.base.color.into()),
                    border: Border {
                        width: 1.0,
                        color: theme.extended_palette().background.weak.color,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
                }),
                // 应用按钮：将 Markdown 内容转换为思维导图结构
                row![
                    button(text("应用"))
                        .style(button::primary)
                        .on_press(Message::MindMapTool(MindMapMessage::ApplyMarkdownImport))
                        .padding([6, 12]),
                ]
                .align_y(Alignment::Center)
            ]
            .spacing(10)
            .padding(12),
        )
        .style(picker_style);

        // 使用 BelowOverlay 组件创建下拉面板效果
        BelowOverlay::new(
            button(text("Markdown 大纲"))
                .style(button::secondary)
                .padding([6, 12])
                .on_press(Message::MindMapTool(MindMapMessage::ToggleMarkdownImport)),
            overlay,
        )
        .show(show_markdown_import) // 控制面板的显示/隐藏
        .gap(6.0) // 按钮与面板之间的间距
        .on_close(Message::MindMapTool(MindMapMessage::ClosePickers)) // 关闭面板的消息
        .into()
    } else {
        // 无标签页时，显示默认按钮（不可点击）
        button(text("Markdown 大纲")).style(button::secondary).padding([6, 12]).into()
    };

    // 构建右侧操作区域：将所有操作组件水平排列
    let actions = row![
        file_actions,                            // 文件操作按钮组
        diagram_type_picker,                     // 图表类型选择器
        md_import,                               // Markdown 导入面板
        Space::new().width(Length::Fixed(16.0)), // 间隔空间
        zoom_controls,                           // 缩放控制
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    // 构建最终布局：左侧标题 + 弹性空间 + 右侧操作区
    row![left, Space::new().width(Length::Fill), actions]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
