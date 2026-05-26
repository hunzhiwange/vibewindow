pub mod gradient;
pub mod image;
pub mod mesh;
pub mod solid;
pub mod types;

use iced::widget::{Space, button, column, container, row, svg, text, text_input};
use iced::{Background, Color, Element, Length, Point, Theme};
use serde_json::{Value, json};

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{ColorFormat, DesignElement};

use self::types::{FillItem, FillObject, GradientFill, GradientStop, ImageFill, MeshFill};

#[derive(Debug, Clone)]
pub struct ActiveFillPicker {
    pub element_id: String,
    pub fill_index: usize,
    pub position: Point,
    pub format: ColorFormat,
    pub picking: bool,
}

pub fn render<'a>(
    element: &'a DesignElement,
    selected_index: Option<usize>,
) -> Element<'a, Message> {
    let fills = parse_fills(&element.fill);
    let id = element.id.clone();

    let list = render_fill_list(&fills, &id, selected_index);

    column![
        row![
            text("填充")
                .size(12)
                .font(iced::font::Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            Space::new().width(Length::Fill),
            button(text("+").size(12))
                .on_press(add_fill(id.clone(), &fills))
                .style(button::text)
                .padding(4)
        ]
        .align_y(iced::Alignment::Center),
        list
    ]
    .spacing(10)
    .into()
}

pub fn render_popover<'a>(
    element: &'a DesignElement,
    fill_index: usize,
    format: ColorFormat,
    picking: bool,
    pan: iced::Vector,
    zoom: f32,
) -> Element<'a, Message> {
    let fills = parse_fills(&element.fill);
    if let Some(item) = fills.get(fill_index) {
        render_picker(
            item.clone(),
            fill_index,
            fills,
            element.id.clone(),
            format,
            picking,
            pan,
            zoom,
        )
    } else {
        column![].into()
    }
}

fn parse_fills(value: &Option<Value>) -> Vec<FillItem> {
    let value = match value {
        Some(v) => v,
        None => return vec![],
    };
    let mut fills = if let Ok(fills) = serde_json::from_value::<Vec<FillItem>>(value.clone()) {
        fills
    } else if let Ok(fill) = serde_json::from_value::<FillItem>(value.clone()) {
        vec![fill]
    } else {
        vec![]
    };

    for item in &mut fills {
        if let FillItem::Object(FillObject::Mesh(m)) = item {
            m.normalize();
        }
    }

    fills
}

fn render_fill_list(
    fills: &[FillItem],
    id: &str,
    selected_index: Option<usize>,
) -> Element<'static, Message> {
    let mut col = column![].spacing(5);

    let action_btn = |icon: Icon, on_press: Message| {
        let content = container(svg(assets::get_icon(icon)).width(12).height(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        button(content)
            .on_press(on_press)
            .style(|theme: &Theme, status| {
                let ext = theme.extended_palette();
                let background = match status {
                    button::Status::Hovered => Some(ext.background.weak.color),
                    button::Status::Pressed => Some(ext.background.strong.color),
                    _ => None,
                };
                button::Style {
                    background: background.map(Into::into),
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    ..button::Style::default()
                }
            })
            .padding(0)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
    };

    // Iterate in reverse order to show Top layer (last in array) at the top of the list
    for (i, item) in fills.iter().enumerate().rev() {
        let is_selected = selected_index == Some(i);
        let preview_color = match item {
            FillItem::Color(c) => Some(c.clone()),
            FillItem::Object(FillObject::Solid { color, .. }) => Some(color.clone()),
            FillItem::Object(FillObject::Color { color, .. }) => Some(color.clone()),
            _ => None,
        };

        let preview: Element<_> = if let Some(c) = preview_color {
            container(Space::new().width(16).height(16))
                .style(move |_: &Theme| {
                    // solid::parse_hex_to_rgba returns tuple, need to convert to Color
                    let (r, g, b, a) = solid::parse_hex_to_rgba(&c);
                    let color = Color::from_rgba(r, g, b, a);

                    container::Style {
                        background: Some(color.into()),
                        border: iced::Border {
                            color: Color::from_rgb(0.8, 0.8, 0.8),
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .into()
        } else {
            svg(assets::get_icon(match item {
                FillItem::Object(FillObject::Gradient(_)) => Icon::Sliders,
                FillItem::Object(FillObject::Mesh(_)) => Icon::LayoutTextWindow, // Use LayoutTextWindow for Mesh
                FillItem::Object(FillObject::Image(_)) => Icon::Image,
                _ => Icon::Circle,
            }))
            .width(16)
            .height(16)
            .into()
        };

        let label = match item {
            FillItem::Color(c) => c.clone(),
            FillItem::Object(FillObject::Solid { color, .. }) => color.clone(),
            FillItem::Object(FillObject::Color { color, .. }) => color.clone(),
            FillItem::Object(FillObject::Gradient(g)) => {
                gradient_type_label(&g.gradient_type).to_string()
            }
            FillItem::Object(FillObject::Mesh(_)) => "网格".to_string(),
            FillItem::Object(FillObject::Image(_)) => "图片".to_string(),
        };

        let preview_btn = button(preview)
            .on_press(Message::Design(DesignMessage::OpenFillPicker(id.to_string(), i, None)))
            .style(button::text)
            .padding(0)
            .width(16)
            .height(16);

        let detail: Element<_> = match item {
            FillItem::Color(c)
            | FillItem::Object(FillObject::Solid { color: c, .. })
            | FillItem::Object(FillObject::Color { color: c, .. }) => {
                let id = id.to_string();
                let fills_vec = fills.to_vec();
                let val = c.clone();
                text_input("十六进制", &val)
                    .on_input(move |s| update_fill_color(id.clone(), fills_vec.clone(), i, s))
                    .style(super::utils::prop_text_input_style)
                    .width(Length::Fill)
                    .into()
            }
            _ => text(label).size(12).into(),
        };

        let row_content = row![
            preview_btn,
            detail,
            Space::new().width(Length::Fill),
            action_btn(
                if item.is_enabled() { Icon::Eye } else { Icon::EyeSlash },
                toggle_fill(id.to_string(), fills, i),
            ),
            action_btn(Icon::Trash, remove_fill(id.to_string(), fills, i))
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let row_container = container(row_content)
            .style(move |theme: &Theme| {
                let p = theme.palette();
                container::Style {
                    background: None,
                    border: iced::Border {
                        width: if is_selected { 1.0 } else { 0.0 },
                        color: if is_selected { p.primary } else { Color::TRANSPARENT },
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding(6)
            .height(Length::Fixed(36.0))
            .width(Length::Fill);

        col = col.push(
            iced::widget::mouse_area(row_container)
                .on_press(Message::Design(DesignMessage::SelectFill(Some(i)))),
        );
    }

    col.into()
}

fn gradient_type_label(gradient_type: &str) -> &str {
    match gradient_type {
        "linear" => "线性",
        "radial" => "径向",
        "angular" => "角向",
        "mesh_gradient" => "网格",
        _ => gradient_type,
    }
}

fn update_fill_color(id: String, fills: Vec<FillItem>, index: usize, color: String) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(item) = new_fills.get_mut(index) {
        match item {
            FillItem::Color(c) => *c = color,
            FillItem::Object(FillObject::Solid { color: c, .. }) => *c = color,
            FillItem::Object(FillObject::Color { color: c, .. }) => *c = color,
            _ => {}
        }
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FillTab {
    Color,
    Gradient,
    Mesh,
    Image,
}

fn render_picker(
    item: FillItem,
    index: usize,
    fills: Vec<FillItem>,
    id: String,
    format: ColorFormat,
    picking: bool,
    pan: iced::Vector,
    zoom: f32,
) -> Element<'static, Message> {
    let current_tab = match &item {
        FillItem::Color(_)
        | FillItem::Object(FillObject::Solid { .. })
        | FillItem::Object(FillObject::Color { .. }) => FillTab::Color,
        FillItem::Object(FillObject::Gradient(_)) => FillTab::Gradient,
        FillItem::Object(FillObject::Mesh(_)) => FillTab::Mesh,
        FillItem::Object(FillObject::Image(_)) => FillTab::Image,
    };

    let id_clone = id.clone();
    let fills_clone = fills.clone();

    let tab_btn = move |tab: FillTab, icon: Icon, is_active: bool| {
        let id = id_clone.clone();
        let fills = fills_clone.clone();
        button(svg(assets::get_icon(icon)).width(16).height(16))
            .style(move |theme: &Theme, _| {
                let p = theme.palette();
                let extended = theme.extended_palette();
                let bg = if is_active {
                    Some(Background::Color(extended.background.base.color))
                } else {
                    None
                };
                button::Style {
                    background: bg,
                    text_color: p.text,
                    border: iced::Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: if is_active { p.primary } else { extended.background.strong.color },
                    },
                    ..button::Style::default()
                }
            })
            .on_press(change_fill_type(id, fills, index, tab))
            .padding(8)
            .width(Length::Fill)
    };

    let tabs = container(
        row![
            tab_btn(FillTab::Color, Icon::Square, current_tab == FillTab::Color),
            tab_btn(FillTab::Gradient, Icon::Sliders, current_tab == FillTab::Gradient),
            tab_btn(FillTab::Mesh, Icon::LayoutTextWindow, current_tab == FillTab::Mesh),
            tab_btn(FillTab::Image, Icon::Image, current_tab == FillTab::Image),
        ]
        .spacing(6)
        .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill);

    let content = match item {
        FillItem::Color(c) => solid::render(c, index, fills, id, format, picking),
        FillItem::Object(FillObject::Solid { color, .. }) => {
            solid::render(color, index, fills, id, format, picking)
        }
        FillItem::Object(FillObject::Color { color, .. }) => {
            solid::render(color, index, fills, id, format, picking)
        }
        FillItem::Object(FillObject::Gradient(g)) => gradient::render(g, index, fills, id),
        FillItem::Object(FillObject::Mesh(m)) => mesh::render(m, index, fills, id, pan, zoom),
        FillItem::Object(FillObject::Image(img)) => image::render(img, index, fills, id),
    };

    column![tabs, content].spacing(10).into()
}

// Helpers for actions

fn add_fill(id: String, fills: &[FillItem]) -> Message {
    let mut new_fills = fills.to_vec();
    new_fills.push(FillItem::Color("#000000ff".to_string()));
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

fn remove_fill(id: String, fills: &[FillItem], index: usize) -> Message {
    let mut new_fills = fills.to_vec();
    if index < new_fills.len() {
        new_fills.remove(index);
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

fn toggle_fill(id: String, fills: &[FillItem], index: usize) -> Message {
    let mut new_fills = fills.to_vec();
    if let Some(item) = new_fills.get_mut(index) {
        item.set_enabled(!item.is_enabled());
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

fn change_fill_type(id: String, fills: Vec<FillItem>, index: usize, new_type: FillTab) -> Message {
    let mut new_fills = fills;
    if let Some(item) = new_fills.get_mut(index) {
        *item = match new_type {
            FillTab::Color => FillItem::Color("#000000ff".to_string()),
            FillTab::Gradient => FillItem::Object(FillObject::Gradient(GradientFill {
                gradient_type: "linear".to_string(),
                enabled: true,
                rotation: 0.0,
                colors: vec![
                    GradientStop { color: "#000000ff".to_string(), position: 0.0 },
                    GradientStop { color: "#ffffffff".to_string(), position: 1.0 },
                ],
                center: None,
                size: None,
                size_h: None,
            })),
            FillTab::Mesh => FillItem::Object(FillObject::Mesh(MeshFill::new_random(3, 3))),
            FillTab::Image => FillItem::Object(FillObject::Image(ImageFill {
                enabled: true,
                url: "".to_string(),
                mode: "fill_width".to_string(),
            })),
        };
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

#[cfg(test)]
mod tests;
