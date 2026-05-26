//! 渲染应用视图中的模态窗口。
//! 本模块只描述模态内容和交互消息，不直接承担持久化策略。

use iced::Element;

use super::{App, Message, message};

/// 模块内可见函数，执行 with_question_modal 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(crate) fn with_question_modal<'a>(
    app: &App,
    mut root_content: Element<'a, Message>,
) -> Element<'a, Message> {
    if let Some(req) = app.question_modal_request.clone() {
        use crate::app::components::system_settings_common::{
            primary_action_btn_style, rounded_action_btn_style, settings_modal_card,
            settings_modal_overlay, settings_text_input_style,
        };
        use iced::widget::{Space, button, column, container, row, text, text_input};
        use iced::{Color, Length, Theme};

        let header = req
            .questions
            .first()
            .map(|question| question.header.clone())
            .unwrap_or_else(|| "需要确认".to_string());

        let mut content = column![text(header).size(16)].spacing(12);
        for (question_index, question) in req.questions.iter().enumerate() {
            let mut question_column = column![text(question.question.clone()).size(14)].spacing(8);
            let selected_answers = app
                .question_modal_answers
                .get(question_index)
                .cloned()
                .unwrap_or_default();

            let mut option_column = column![].spacing(6);
            for option in &question.options {
                let label = option.label.clone();
                let message_label = label.clone();
                let selected = selected_answers.iter().any(|value| value == &label);

                let button_label = column![
                    text(label.clone()).size(13),
                    text(option.description.clone()).size(11).style(|theme: &Theme| {
                        iced::widget::text::Style {
                            color: Some(theme.extended_palette().secondary.base.text),
                        }
                    })
                ]
                .spacing(2);

                let button = button(container(button_label).width(Length::Fill).padding([6, 10]))
                    .width(Length::Fill)
                    .on_press(Message::Chat(message::ChatMessage::QuestionOptionToggled(
                        question_index,
                        message_label,
                    )))
                    .style(move |theme: &Theme, status| {
                        let mut style = iced::widget::button::secondary(theme, status);
                        if selected {
                            let palette = theme.extended_palette();
                            style.background = Some(palette.primary.base.color.into());
                            style.text_color = Color::WHITE;
                        }
                        style
                    });

                option_column = option_column.push(button);
            }
            if !question.options.is_empty() {
                question_column = question_column.push(option_column);
            }

            if !question.multiple.unwrap_or(false)
                && let Some(selected_answer) = selected_answers.first()
                && let Some(preview) = question
                    .options
                    .iter()
                    .find(|option| option.label == *selected_answer)
                    .and_then(|option| option.preview.as_deref())
                && !preview.trim().is_empty()
            {
                question_column = question_column.push(
                    column![
                        text("预览").size(11).style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.extended_palette().secondary.base.text),
                        }),
                        text(preview.to_string()).size(11),
                    ]
                    .spacing(4),
                );
            }

            if question.custom.unwrap_or(false) {
                let value = app
                    .question_modal_custom
                    .get(question_index)
                    .cloned()
                    .unwrap_or_default();
                let input = text_input("输入你的答案", &value)
                    .on_input(move |value| {
                        Message::Chat(message::ChatMessage::QuestionCustomChanged(
                            question_index,
                            value,
                        ))
                    })
                    .padding([10, 12])
                    .style(settings_text_input_style);
                question_column = question_column.push(input);
            }

            content = content.push(question_column);
        }

        let cancel = button(text("取消").size(13))
            .on_press(Message::Chat(message::ChatMessage::QuestionReject))
            .padding([6, 12])
            .style(rounded_action_btn_style);
        let submit = button(text("确定").size(13))
            .on_press(Message::Chat(message::ChatMessage::QuestionSubmit))
            .padding([6, 12])
            .style(primary_action_btn_style);

        content = content.push(row![Space::new().width(Length::Fill), cancel, submit].spacing(8));

        let card = settings_modal_card(content).width(Length::Fixed(420.0));

        root_content = settings_modal_overlay(
            Some(root_content),
            Message::Chat(message::ChatMessage::QuestionReject),
            card,
        );
    }

    root_content
}
#[cfg(test)]
#[path = "question_modal_tests.rs"]
mod question_modal_tests;
