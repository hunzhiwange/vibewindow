//! 工作流应用编辑器视图，负责应用名称、图标、描述和请求限制的表单渲染。

use super::*;
use iced::widget::{column, row};

/// 构建或更新 build app editor modal 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn build_app_editor_modal(state: &WorkflowState) -> Element<'_, Message> {
    let Some(editor) = state.app_editor.as_ref() else {
        return container(Space::new().width(1).height(1)).into();
    };

    let title = match &editor.mode {
        super::state::WorkflowAppEditorMode::Create => "新增应用",
        super::state::WorkflowAppEditorMode::Edit(_) => "编辑应用信息",
    };

    let subtitle = match &editor.mode {
        super::state::WorkflowAppEditorMode::Create => {
            "创建一个新的 workflow 应用，后续可以单独保存成 yml 文件。"
        }
        super::state::WorkflowAppEditorMode::Edit(_) => {
            "修改应用元数据，不会丢失原始 Dify yml 里的节点业务字段。"
        }
    };

    let content = column![
        row![
            column![
                text(title).size(24),
                text(subtitle).size(12).style(settings_muted_text_style),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            settings_close_button(Message::WorkflowTool(WorkflowMessage::CloseAppEditor)),
        ]
        .align_y(Alignment::Center),
        section_card("应用基础信息", "表单结构参考 Dify 应用设置页，先整理名称、图标和描述。"),
        build_editor_field(
            "应用名称",
            workflow_text_input("例如：客服分流工作流", &editor.name, |value| {
                Message::WorkflowTool(WorkflowMessage::AppEditorNameChanged(value))
            }),
        ),
        build_editor_field(
            "应用图标",
            workflow_text_input("例如：🤖", &editor.icon, |value| {
                Message::WorkflowTool(WorkflowMessage::AppEditorIconChanged(value))
            }),
        ),
        build_editor_field(
            "应用描述",
            workflow_text_input("描述这个 workflow 用来做什么", &editor.description, |value| {
                Message::WorkflowTool(WorkflowMessage::AppEditorDescriptionChanged(value))
            }),
        ),
        build_editor_field(
            "使用 web app 图标替换 🤖",
            row![
                toggler(editor.use_icon_as_answer_icon).on_toggle(|value| {
                    Message::WorkflowTool(WorkflowMessage::AppEditorUseIconAsAnswerIconChanged(value))
                }),
                text("在分享和 Explore 场景里用应用图标替换默认机器人图标")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into(),
        ),
        build_editor_field(
            "最大活跃请求数",
            workflow_text_input("0 表示不限制", &editor.max_active_requests_input, |value| {
                Message::WorkflowTool(WorkflowMessage::AppEditorMaxActiveRequestsChanged(value))
            }),
        ),
        row![
            Space::new().width(Length::Fill),
            button(text("取消"))
                .style(rounded_action_btn_style)
                .padding([9, 14])
                .on_press(Message::WorkflowTool(WorkflowMessage::CloseAppEditor)),
            button(text("保存应用信息"))
                .style(primary_action_btn_style)
                .padding([9, 14])
                .on_press(Message::WorkflowTool(WorkflowMessage::SubmitAppEditor)),
        ]
        .spacing(10),
    ]
    .spacing(16);

    container(
        container(scrollable(content).height(Length::Fixed(620.0)))
            .width(Length::Fixed(680.0))
            .padding([24, 26])
            .style(modal_card_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

#[cfg(test)]
#[path = "app_editor_tests.rs"]
mod app_editor_tests;
