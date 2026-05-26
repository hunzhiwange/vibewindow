//! 思维导图视图入口，负责根据当前状态组织整体界面。

mod active;
mod common;
mod empty;
mod header;

#[cfg(test)]
mod common_tests;
#[cfg(test)]
mod empty_tests;
#[cfg(test)]
mod header_tests;
#[cfg(test)]
#[path = "view_tests.rs"]
mod view_tests;

use crate::app::{App, Message};
use iced::widget::{column, container};
use iced::{Element, Length, Theme};

/// 构建或更新 view 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn view(app: &App) -> Element<'_, Message> {
    let tab_opt = app.active_mindmap_tab();

    let body: Element<'_, Message> =
        if let Some(tab) = tab_opt { active::render(tab) } else { empty::render() };

    let body = container(body).width(Length::Fill).height(Length::Fill);

    container(column![body].spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| iced::widget::container::Style {
            background: Some(theme.extended_palette().background.base.color.into()),
            ..Default::default()
        })
        .into()
}
