//! Mesh 填充属性模块，负责渲染网格填充控制项并协调局部编辑状态。

use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::properties::fill::types::{FillItem, MeshFill};
use crate::app::views::design::properties::utils::prop_section;

use super::actions::{
    apply_mesh_color_to_all, clear_mesh_selection, regenerate_mesh_colors, reset_mesh_positions,
    reset_selected_mesh_curve, reset_selected_mesh_position, shuffle_mesh_colors,
    update_mesh_grid_cells, update_mesh_mirroring, update_mesh_outline, update_mesh_selected_color,
    update_mesh_selection,
};
use super::utils::mirroring_flags;

/// 渲染对应的设计界面片段。
///
/// 返回 Iced 元素；输入为空或不支持时由调用方保留现有界面兜底。
pub fn render(
    mesh: MeshFill,
    index: usize,
    fills: Vec<FillItem>,
    id: String,
    pan: iced::Vector,
    zoom: f32,
) -> Element<'static, Message> {
    let grid_cols = mesh.columns.saturating_sub(1).clamp(1, 5);
    let grid_rows = mesh.rows.saturating_sub(1).clamp(1, 5);
    let (mirror_x, mirror_y) = mirroring_flags(mesh.mirroring.as_deref());

    let grid_picker: Element<'static, Message> = column![
        row![
            text("网格大小")
                .size(11)
                .line_height(iced::widget::text::LineHeight::Relative(1.2))
                .style(iced::widget::text::secondary),
            Space::new().width(Length::Fill),
            text(format!("{} × {}", grid_cols, grid_rows)).size(12)
        ]
        .align_y(iced::Alignment::Center),
        grid_size_matrix(grid_cols, grid_rows, id.clone(), fills.clone(), index)
    ]
    .spacing(8)
    .into();

    let icon_btn = |icon: Icon, selected: bool, on_press: Message| {
        let icon = svg(assets::get_icon(icon))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .style(|theme: &Theme, _| iced::widget::svg::Style {
                color: Some(theme.palette().text),
            });

        button(icon)
            .padding(6)
            .style(move |theme: &Theme, status| {
                let p = theme.extended_palette();
                let bg = if selected {
                    Some(Background::Color(p.background.weak.color))
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => {
                            Some(Background::Color(p.background.weak.color.scale_alpha(0.60)))
                        }
                        iced::widget::button::Status::Pressed => {
                            Some(Background::Color(p.background.strong.color))
                        }
                        _ => None,
                    }
                };
                iced::widget::button::Style {
                    background: bg,
                    border: Border {
                        width: 1.0,
                        color: p.background.strong.color,
                        radius: 8.0.into(),
                    },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .on_press(on_press)
    };

    let align_is_x = if mirror_x && !mirror_y {
        true
    } else { !(mirror_y && !mirror_x) };

    let mirroring_controls = prop_section(
        "对齐",
        row![
            icon_btn(
                Icon::SymmetryHorizontal,
                align_is_x,
                update_mesh_mirroring(id.clone(), fills.clone(), index, Some("x".to_string())),
            )
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(28.0)),
            icon_btn(
                Icon::SymmetryVertical,
                !align_is_x,
                update_mesh_mirroring(id.clone(), fills.clone(), index, Some("y".to_string())),
            )
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(28.0)),
        ]
        .spacing(6),
    );

    let outline_btn = |label: &'static str, selected: bool, on_press: Message| {
        button(text(label).size(12))
            .padding([5, 10])
            .style(move |theme: &Theme, status| {
                let p = theme.extended_palette();
                let bg = if selected {
                    Some(Background::Color(p.background.weak.color))
                } else {
                    match status {
                        iced::widget::button::Status::Hovered => {
                            Some(Background::Color(p.background.weak.color.scale_alpha(0.60)))
                        }
                        iced::widget::button::Status::Pressed => {
                            Some(Background::Color(p.background.strong.color))
                        }
                        _ => None,
                    }
                };
                iced::widget::button::Style {
                    background: bg,
                    border: Border {
                        width: 1.0,
                        color: p.background.strong.color,
                        radius: 8.0.into(),
                    },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .on_press(on_press)
    };

    let outline_controls = prop_section(
        "轮廓",
        row![
            outline_btn(
                "关闭",
                !mesh.outline,
                update_mesh_outline(id.clone(), fills.clone(), index, false),
            ),
            outline_btn(
                "开启",
                mesh.outline,
                update_mesh_outline(id.clone(), fills.clone(), index, true),
            ),
        ]
        .spacing(6),
    );

    let selected_point_idx_opt = mesh.selected_point_index.filter(|i| *i < mesh.colors.len());
    let selected_point_idx = selected_point_idx_opt.unwrap_or(0);
    let selected_color =
        mesh.colors.get(selected_point_idx).cloned().unwrap_or_else(|| "#ffffff".to_string());

    let mut color_button =
        button(text(selected_color.clone()).size(12)).padding([5, 10]).style(button::secondary);
    if selected_point_idx_opt.is_some() {
        color_button = color_button.on_press({
            let rgba = super::super::solid::parse_hex_to_rgba(&selected_color);
            let c = Color::from_rgba(rgba.0, rgba.1, rgba.2, rgba.3);
            Message::Design(DesignMessage::OpenColorPicker(
                c,
                crate::app::views::design::models::ColorPickerTarget::MeshPoint {
                    element_id: id.clone(),
                    fill_index: index,
                    point_index: selected_point_idx,
                },
                None,
            ))
        });
    }

    let cols_to_show = mesh.columns.clamp(2, 6);
    let rows_to_show = mesh.rows.clamp(2, 6);

    let mut colors_grid = column![].spacing(4);
    for r in 0..rows_to_show {
        let mut row_widget = row![].spacing(4);
        for c in 0..cols_to_show {
            let idx = r * mesh.columns + c;
            let color_val = mesh.colors.get(idx).cloned().unwrap_or_else(|| "#ffffff".to_string());
            let is_selected = selected_point_idx_opt == Some(idx);
            let preview = button(Space::new().width(18).height(18))
                .style(move |theme: &Theme, status| {
                    let p = theme.extended_palette();
                    let hover = matches!(status, iced::widget::button::Status::Hovered);
                    let bg = super::super::solid::parse_color(&color_val)
                        .map(Into::into)
                        .or(Some(Background::Color(p.background.base.color)));
                    iced::widget::button::Style {
                        background: bg,
                        border: Border {
                            width: if is_selected { 2.0 } else { 1.0 },
                            color: if is_selected {
                                theme.palette().primary
                            } else if hover {
                                theme.palette().primary.scale_alpha(0.45)
                            } else {
                                p.background.strong.color
                            },
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .padding(0)
                .on_press(update_mesh_selection(id.clone(), fills.clone(), index, idx));
            row_widget = row_widget.push(preview);
        }
        colors_grid = colors_grid.push(row_widget);
    }

    let shuffle_btn = {
        let mut btn = button(text("打乱").size(12)).padding([6, 10]).style(button::secondary);
        if mesh.selected_point_index.is_none() {
            btn = btn.on_press(shuffle_mesh_colors(id.clone(), fills.clone(), index));
        }
        btn
    };

    let regenerate_btn = {
        let mut btn = button(text("重新生成").size(12)).padding([6, 10]).style(button::secondary);
        if mesh.selected_point_index.is_none() {
            btn = btn.on_press(regenerate_mesh_colors(id.clone(), fills.clone(), index));
        }
        btn
    };

    let mut random_selected_btn =
        button(text("随机此点").size(12)).padding([6, 10]).style(button::secondary);
    if selected_point_idx_opt.is_some() {
        random_selected_btn = random_selected_btn.on_press(update_mesh_selected_color(
            id.clone(),
            fills.clone(),
            index,
            selected_point_idx,
        ));
    }

    let mut apply_to_all_btn =
        button(text("应用到全部").size(12)).padding([6, 10]).style(button::secondary);
    if selected_point_idx_opt.is_some() {
        apply_to_all_btn = apply_to_all_btn.on_press(apply_mesh_color_to_all(
            id.clone(),
            fills.clone(),
            index,
            selected_color.clone(),
        ));
    }

    let reset_section = {
        let reset_all = button(text("重置全部").size(12))
            .padding([6, 10])
            .on_press(reset_mesh_positions(id.clone(), fills.clone(), index))
            .style(button::secondary);

        let mut reset_point =
            button(text("重置此点").size(12)).padding([6, 10]).style(button::secondary);
        if mesh.selected_point_index.is_some() {
            reset_point = reset_point.on_press(reset_selected_mesh_position(
                id.clone(),
                fills.clone(),
                index,
            ));
        }

        let mut reset_curve =
            button(text("重置曲线").size(12)).padding([6, 10]).style(button::secondary);
        if mesh.selected_point_index.is_some() {
            reset_curve =
                reset_curve.on_press(reset_selected_mesh_curve(id.clone(), fills.clone(), index));
        }

        let mut clear_sel =
            button(text("取消选中").size(12)).padding([6, 10]).style(button::secondary);
        if mesh.selected_point_index.is_some() {
            clear_sel = clear_sel.on_press(clear_mesh_selection(id.clone(), fills.clone(), index));
        }

        prop_section("位置", row![reset_all, reset_point, reset_curve, clear_sel].spacing(6))
    };

    let top = row![
        container(grid_picker).width(Length::FillPortion(3)),
        container(column![mirroring_controls, outline_controls].spacing(12).width(Length::Fill),)
            .width(Length::FillPortion(2))
    ]
    .spacing(14)
    .align_y(iced::Alignment::Start);

    let selected_col = selected_point_idx % mesh.columns.max(1);
    let selected_row = selected_point_idx / mesh.columns.max(1);
    let selected_info = prop_section(
        "选中",
        column![
            row![
                text(if selected_point_idx_opt.is_some() {
                    format!(
                        "点 {} / {}（第 {} 行，第 {} 列）",
                        selected_point_idx + 1,
                        mesh.columns * mesh.rows,
                        selected_row + 1,
                        selected_col + 1
                    )
                } else {
                    "未选中".to_string()
                })
                .size(12)
            ],
            row![color_button, random_selected_btn, apply_to_all_btn]
                .spacing(6)
                .align_y(iced::Alignment::Center),
        ]
        .spacing(8),
    );

    let colors_section = prop_section(
        "颜色",
        column![colors_grid, selected_info, row![regenerate_btn, shuffle_btn].spacing(6)]
            .spacing(10),
    );

    let preview_section = prop_section(
        "预览位置",
        row![
            text(format!("x {:.0}", pan.x)).size(12),
            text(format!("y {:.0}", pan.y)).size(12),
            text(format!("{}%", (zoom * 100.0).round() as i32)).size(12)
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    );

    column![top, colors_section, reset_section, preview_section].spacing(14).into()
}

fn grid_size_matrix(
    selected_cols: usize,
    selected_rows: usize,
    id: String,
    fills: Vec<FillItem>,
    index: usize,
) -> Element<'static, Message> {
    let mut matrix = column![].spacing(3);
    for r in 1..=5 {
        let mut row_widget = row![].spacing(3);
        for c in 1..=5 {
            let active = c <= selected_cols && r <= selected_rows;
            let btn = button(Space::new().width(16).height(16))
                .padding(0)
                .style(move |theme: &Theme, status| {
                    let p = theme.extended_palette();
                    let hover = matches!(status, iced::widget::button::Status::Hovered);
                    let bg = if active {
                        Some(Background::Color(p.background.weak.color))
                    } else if hover {
                        Some(Background::Color(p.background.weak.color.scale_alpha(0.35)))
                    } else {
                        None
                    };
                    iced::widget::button::Style {
                        background: bg,
                        border: Border {
                            width: 1.0,
                            color: p.background.strong.color,
                            radius: 3.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .on_press(update_mesh_grid_cells(id.clone(), fills.clone(), index, c, r));
            row_widget = row_widget.push(btn);
        }
        matrix = matrix.push(row_widget);
    }

    container(matrix)
        .padding(6)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border { width: 1.0, color: p.background.strong.color, radius: 8.0.into() },
                ..Default::default()
            }
        })
        .into()
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod ui_tests;
