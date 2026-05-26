//! 位置属性面板模块
//!
//! 本模块提供设计元素位置属性的渲染功能，包括 X 坐标、Y 坐标和旋转角度的编辑控件。
//! 这些控件允许用户在属性面板中精确调整设计元素的位置和旋转状态。

use super::utils::{prop_section, prop_text_input_style};
use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::DesignElement;
use iced::widget::{column, container, row, text, text_input};
use iced::{Element, Length};

/// 渲染位置属性面板
///
/// 为指定的设计元素创建位置属性编辑界面，包括：
/// - X 坐标输入框
/// - Y 坐标输入框
/// - 旋转角度输入框
///
/// # 参数
///
/// - `element`: 要编辑的设计元素引用，包含当前的位置和旋转信息
///
/// # 返回值
///
/// 返回一个 Iced Element，包含完整的位置属性编辑界面
///
/// # 示例
///
/// ```ignore
/// let element = DesignElement {
///     id: "elem-1".to_string(),
///     x: 100.0,
///     y: 200.0,
///     rotation: Some(45.0),
///     // ...其他字段
/// };
/// let view = render(&element);
/// ```
pub fn render(element: &DesignElement) -> Element<'_, Message> {
    let id = element.id.clone();

    let x_input = text_input("", &element.x.to_string())
        .on_input({
            let id = id.clone();
            move |s| {
                Message::Design(DesignMessage::PropertyUpdate(
                    id.clone(),
                    "x".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(s.parse().unwrap_or(0.0)).unwrap(),
                    ),
                ))
            }
        })
        .style(prop_text_input_style)
        .padding(6)
        .size(12)
        .width(Length::Fill);

    let y_input = text_input("", &element.y.to_string())
        .on_input({
            let id = id.clone();
            move |s| {
                Message::Design(DesignMessage::PropertyUpdate(
                    id.clone(),
                    "y".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(s.parse().unwrap_or(0.0)).unwrap(),
                    ),
                ))
            }
        })
        .style(prop_text_input_style)
        .padding(6)
        .size(12)
        .width(Length::Fill);

    let rotation_input = text_input("", &element.rotation.unwrap_or(0.0).to_string())
        .on_input({
            let id = id.clone();
            move |s| {
                Message::Design(DesignMessage::PropertyUpdate(
                    id.clone(),
                    "rotation".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(s.parse().unwrap_or(0.0)).unwrap(),
                    ),
                ))
            }
        })
        .style(prop_text_input_style)
        .padding(6)
        .size(12)
        .width(Length::Fill);

    column![
        text("位置")
            .size(12)
            .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        row![
            container(prop_section("X", x_input)).width(Length::Fill),
            container(prop_section("Y", y_input)).width(Length::Fill),
        ]
        .spacing(10),
        container(prop_section("旋转", rotation_input)).width(Length::Fill),
    ]
    .spacing(8)
    .into()
}

#[cfg(test)]
#[path = "position_tests.rs"]
mod position_tests;
