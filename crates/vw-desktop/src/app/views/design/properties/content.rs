//! 内容属性面板模块
//!
//! 本模块提供设计视图中元素内容属性的编辑界面组件，包括：
//! - 图层名称编辑器
//! - 上下文内容编辑器
//! - 文本内容编辑器
//!
//! 这些组件用于在属性面板中显示和编辑设计元素的内容相关属性。

use super::utils::{prop_section, prop_text_editor_style, prop_text_input_style};
use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::DesignElement;
use iced::widget::{Space, button, column, row, text, text_editor, text_input};
use iced::{Element, Length};

/// 渲染节点标题输入组件
///
/// 创建一个文本输入框，用于编辑设计元素的图层名称。
/// 当用户输入时，会触发 `PropertyUpdate` 消息更新元素的 `name` 属性。
///
/// # 参数
///
/// * `element` - 要编辑的设计元素引用，从中获取元素的 ID 和当前名称
///
/// # 返回值
///
/// 返回一个包含标签和输入框的属性区域 Element
///
/// # 示例
///
/// ```ignore
/// let element = DesignElement { id: "node1", name: Some("我的节点".to_string()), ... };
/// let title_ui = render_node_title(&element);
/// ```
pub fn render_node_title<'a>(element: &'a DesignElement) -> Element<'a, Message> {
    // 获取元素 ID，用于后续消息中标识目标元素
    let id = element.id.clone();
    // 获取当前标题，如果不存在则使用空字符串作为默认值
    let title = element.name.clone().unwrap_or_default();

    // 创建属性区域，包含"图层名称"标签和文本输入框
    prop_section(
        "图层名称",
        text_input("输入图层名称...", &title)
            // 监听输入变化，发送属性更新消息
            .on_input(move |s| {
                Message::Design(DesignMessage::PropertyUpdate(
                    id.clone(),
                    "name".to_string(),
                    serde_json::Value::String(s),
                ))
            })
            .style(prop_text_input_style)
            .padding(6)
            .size(12),
    )
}

/// 渲染上下文内容编辑器
///
/// 创建一个可折叠的上下文编辑区域，包含标题栏和可选的文本编辑器。
/// 用户可以通过点击展开/折叠按钮来显示或隐藏编辑器。
///
/// # 参数
///
/// * `_element` - 设计元素引用（当前未使用，保留用于未来扩展）
/// * `context_editor` - 文本编辑器的内容对象，存储上下文文本
/// * `context_expanded` - 布尔值，指示编辑器是否处于展开状态
///
/// # 返回值
///
/// 返回一个包含标题栏和可选编辑器的 Element
///
/// # 消息
///
/// - 点击折叠按钮时发送 `ToggleContextEditor` 消息
/// - 编辑器内容变化时发送 `ContextEditorAction` 消息
pub fn render_context<'a>(
    _element: &'a DesignElement,
    context_editor: &'a text_editor::Content,
    context_expanded: bool,
) -> Element<'a, Message> {
    // 根据展开状态选择不同的图标：展开时显示向下箭头，折叠时显示向右箭头
    let icon_code = if context_expanded {
        crate::app::assets::Icon::ChevronDown
    } else {
        crate::app::assets::Icon::ChevronRight
    };

    // 创建折叠/展开切换按钮，使用 SVG 图标
    let toggle_btn = button(
        iced::widget::svg(crate::app::assets::get_icon(icon_code)).width(12).height(12).style(
            |theme: &iced::Theme, _| iced::widget::svg::Style { color: Some(theme.palette().text) },
        ),
    )
    .on_press(Message::Design(DesignMessage::ToggleContextEditor))
    .style(button::text)
    .padding(0);

    // 构建标题栏：包含"上下文"标签和切换按钮
    let header = row![
        text("上下文")
            .size(11)
            .line_height(iced::widget::text::LineHeight::Relative(1.2))
            .style(iced::widget::text::secondary),
        Space::new().width(Length::Fill), // 填充空间，将按钮推到右侧
        toggle_btn
    ]
    .align_y(iced::Alignment::Center);

    // 如果编辑器已展开，显示编辑器内容区域
    if context_expanded {
        // 创建文本编辑器，绑定内容并监听编辑动作
        let body = text_editor(context_editor)
            .on_action(|a| Message::Design(DesignMessage::ContextEditorAction(a)))
            .size(12)
            .height(Length::Fixed(40.0))
            .style(prop_text_editor_style);

        // 返回包含标题和编辑器的完整布局
        return column![header, body].spacing(8).into();
    }

    // 如果编辑器已折叠，仅返回标题栏
    column![header].spacing(8).into()
}

/// 渲染文本内容编辑器
///
/// 创建一个固定高度的文本编辑器区域，用于编辑设计元素的文本内容。
/// 编辑器始终可见，不可折叠。
///
/// # 参数
///
/// * `_element` - 设计元素引用（当前未使用，保留用于未来扩展）
/// * `content_editor` - 文本编辑器的内容对象，存储文本内容
///
/// # 返回值
///
/// 返回一个包含"文本内容"标签和编辑器的 Element
///
/// # 消息
///
/// - 编辑器内容变化时发送 `ContentEditorAction` 消息
///
/// # 示例
///
/// ```ignore
/// let content = text_editor::Content::new();
/// let ui = render_text_content(&element, &content);
/// ```
pub fn render_text_content<'a>(
    _element: &'a DesignElement,
    content_editor: &'a text_editor::Content,
) -> Element<'a, Message> {
    // 创建包含标签和文本编辑器的属性区域
    column![prop_section(
        "文本内容",
        text_editor(content_editor)
            // 监听编辑器动作，发送内容更新消息
            .on_action(|a| Message::Design(DesignMessage::ContentEditorAction(a)))
            .size(12)
            .height(Length::Fixed(60.0)) // 固定高度 60 像素
            .style(prop_text_editor_style),
    ),]
    .spacing(10)
    .into()
}

#[cfg(test)]
#[path = "content_tests.rs"]
mod content_tests;
