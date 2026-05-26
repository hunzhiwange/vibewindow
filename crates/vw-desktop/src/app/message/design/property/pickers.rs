//! 处理设计属性选择器消息，包括颜色、尺寸和资源选择。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::common::{parse_fills, upsert_variable_value};
use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::find_element_by_id;
use crate::app::views::design::canvas::hit::hit_test;
use crate::app::views::design::canvas::parse::parse_fill;
use crate::app::views::design::models::{ColorFormat, ColorPickerTarget, Stroke};
use crate::app::views::design::properties::appearance::ActiveEffectPicker;
use crate::app::views::design::properties::color_picker::{ActiveColorPicker, format_rgba_to_hex};
use crate::app::views::design::properties::fill::ActiveFillPicker;
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};
use crate::app::views::design::properties::icon::{
    ActiveIconPicker, icon_weight_label, icon_weight_options_for_family,
    icon_weight_value_from_label,
};
use crate::app::views::design::properties::typography::{
    ActiveFontPicker, available_weights_for_font, font_weight_label, font_weight_value_from_label,
};
use crate::app::{App, Message};
use iced::{Color, Point, Task};

/// set_font_filter 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn set_font_filter(app: &mut App, query: String) -> Task<Message> {
    app.font_filter_query = query;
    Task::none()
}

/// open_font_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn open_font_picker(
    app: &mut App,
    element_id: String,
    position_opt: Option<Point>,
) -> Task<Message> {
    let position = position_opt.unwrap_or(app.cursor_position);
    app.active_color_picker = None;
    app.active_fill_picker = None;
    app.active_effect_picker = None;
    app.active_icon_picker = None;
    app.design_help_text = None;
    app.font_filter_query.clear();
    app.active_font_picker = Some(ActiveFontPicker { element_id, position });
    Task::none()
}

/// close_font_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_font_picker(app: &mut App) -> Task<Message> {
    app.active_font_picker = None;
    Task::none()
}

/// select_font 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn select_font(app: &mut App, element_id: String, family: String) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        let current_weight =
            doc.find_element(&element_id).map(|el| font_weight_label(&el.font_weight));
        let current_weight = current_weight.unwrap_or_else(|| "Regular".to_string());
        let opts = available_weights_for_font(&family);
        let target_label = if opts.contains(&current_weight) {
            current_weight
        } else if opts.contains(&"Regular".to_string()) {
            "Regular".to_string()
        } else {
            opts.first().cloned().unwrap_or("Regular".to_string())
        };
        let weight_value = font_weight_value_from_label(&target_label);

        doc.update_property(
            &element_id,
            "fontFamily",
            serde_json::Value::String(family),
        );
        doc.update_property(
            &element_id,
            "fontWeight",
            serde_json::Value::String(weight_value),
        );
        state.canvas_cache.clear();
    }
    app.active_font_picker = None;
    Task::none()
}

/// set_icon_picker_filter 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn set_icon_picker_filter(app: &mut App, query: String) -> Task<Message> {
    app.icon_picker_filter_query = query;
    Task::none()
}

/// open_icon_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn open_icon_picker(
    app: &mut App,
    element_id: String,
    position_opt: Option<Point>,
) -> Task<Message> {
    let position = position_opt.unwrap_or(app.cursor_position);
    app.active_color_picker = None;
    app.active_fill_picker = None;
    app.active_effect_picker = None;
    app.active_font_picker = None;
    app.design_help_text = None;
    app.icon_picker_filter_query.clear();
    let current_family = app
        .active_design_state()
        .and_then(|state| state.doc.find_element(&element_id))
        .and_then(|element| element.icon_font_family.as_deref())
        .and_then(crate::app::assets::canonical_named_icon_family)
        .unwrap_or_else(|| "lucide".to_string());
    app.icon_picker_family_tab = current_family;
    app.active_icon_picker = Some(ActiveIconPicker { element_id, position });
    Task::none()
}

/// close_icon_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_icon_picker(app: &mut App) -> Task<Message> {
    app.active_icon_picker = None;
    Task::none()
}

/// set_icon_picker_family_tab 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn set_icon_picker_family_tab(app: &mut App, family: String) -> Task<Message> {
    app.icon_picker_family_tab = family;
    Task::none()
}

/// select_icon_family 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn select_icon_family(
    app: &mut App,
    element_id: String,
    family: String,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        let current_weight = doc.find_element(&element_id).map(|el| icon_weight_label(&el.weight));
        doc.update_property(
            &element_id,
            "iconFontFamily",
            serde_json::Value::String(family.clone()),
        );
        let options = icon_weight_options_for_family(&family);
        if !options.is_empty() {
            let target_label = current_weight
                .filter(|label| options.contains(label))
                .unwrap_or_else(|| "Regular".to_string());
            doc.update_property(
                &element_id,
                "weight",
                icon_weight_value_from_label(&target_label),
            );
        }
        state.canvas_cache.clear();
    }
    app.icon_picker_family_tab = family;
    app.active_icon_picker = None;
    Task::none()
}

/// select_icon 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn select_icon(
    app: &mut App,
    element_id: String,
    family: String,
    name: String,
) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        let current_weight = doc.find_element(&element_id).map(|el| icon_weight_label(&el.weight));
        doc.update_property(
            &element_id,
            "iconFontFamily",
            serde_json::Value::String(family.clone()),
        );
        doc.update_property(
            &element_id,
            "iconFontName",
            serde_json::Value::String(name),
        );
        let options = icon_weight_options_for_family(&family);
        if !options.is_empty() {
            let target_label = current_weight
                .filter(|label| options.contains(label))
                .unwrap_or_else(|| "Regular".to_string());
            doc.update_property(
                &element_id,
                "weight",
                icon_weight_value_from_label(&target_label),
            );
        }
        state.canvas_cache.clear();
    }
    app.icon_picker_family_tab = family;
    app.active_icon_picker = None;
    Task::none()
}

/// show_help_modal 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn show_help_modal(app: &mut App, text: String) -> Task<Message> {
    app.design_help_text = Some(text);
    Task::none()
}

/// close_help_modal 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_help_modal(app: &mut App) -> Task<Message> {
    app.design_help_text = None;
    Task::none()
}

/// open_color_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn open_color_picker(
    app: &mut App,
    color: Color,
    target: ColorPickerTarget,
    position_opt: Option<Point>,
) -> Task<Message> {
    let position = position_opt.unwrap_or(app.cursor_position);
    if !matches!(
        target,
        ColorPickerTarget::Fill { .. }
            | ColorPickerTarget::GradientStop { .. }
            | ColorPickerTarget::MeshPoint { .. }
    ) {
        app.active_fill_picker = None;
    }
    if !matches!(target, ColorPickerTarget::Effect { .. }) {
        app.active_effect_picker = None;
    }
    app.active_font_picker = None;
    app.active_icon_picker = None;
    app.active_color_picker = Some(ActiveColorPicker {
        color,
        format: ColorFormat::Hex,
        target,
        position,
        picking: false,
    });
    Task::none()
}

/// open_fill_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn open_fill_picker(
    app: &mut App,
    element_id: String,
    fill_index: usize,
    position_opt: Option<Point>,
) -> Task<Message> {
    let position = position_opt.unwrap_or(app.cursor_position);
    app.active_color_picker = None;
    app.active_effect_picker = None;
    app.active_font_picker = None;
    app.active_icon_picker = None;
    app.active_fill_picker = Some(ActiveFillPicker {
        element_id: element_id.clone(),
        fill_index,
        position,
        format: ColorFormat::Hex,
        picking: false,
    });
    if let Some(state) = app.active_design_state_mut() {
        state.selected_fill_index = Some(fill_index);
    }
    Task::none()
}

/// close_fill_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_fill_picker(app: &mut App) -> Task<Message> {
    app.active_fill_picker = None;
    Task::none()
}

/// open_effect_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn open_effect_picker(
    app: &mut App,
    element_id: String,
    effect_index: usize,
    position_opt: Option<Point>,
) -> Task<Message> {
    let position = position_opt.unwrap_or(app.cursor_position);
    app.active_color_picker = None;
    app.active_fill_picker = None;
    app.active_font_picker = None;
    app.active_icon_picker = None;
    app.design_help_text = None;
    app.active_effect_picker =
        Some(ActiveEffectPicker { element_id: element_id.clone(), effect_index, position });
    if let Some(state) = app.active_design_state_mut() {
        state.selected_effect_index = Some(effect_index);
    }
    Task::none()
}

/// close_effect_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_effect_picker(app: &mut App) -> Task<Message> {
    app.active_effect_picker = None;
    Task::none()
}

/// toggle_fill_picker_eyedropper 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_fill_picker_eyedropper(app: &mut App) -> Task<Message> {
    if let Some(picker) = &mut app.active_fill_picker {
        picker.picking = !picker.picking;
    }
    Task::none()
}

/// change_fill_picker_format 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn change_fill_picker_format(app: &mut App, format: ColorFormat) -> Task<Message> {
    if let Some(picker) = &mut app.active_fill_picker {
        picker.format = format;
    }
    Task::none()
}

/// change_fill_picker_color 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn change_fill_picker_color(app: &mut App, color: Color) -> Task<Message> {
    let (element_id, fill_index) = if let Some(picker) = &app.active_fill_picker {
        (picker.element_id.clone(), picker.fill_index)
    } else {
        return Task::none();
    };

    if let Some(picker) = &mut app.active_fill_picker {
        picker.picking = false;
    }

    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        let fills_json = if let Some(el) = doc.find_element(&element_id) {
            el.fill.clone()
        } else {
            None
        };

        if let Some(fills_val) = fills_json {
            let mut fills = parse_fills(&fills_val);
            let is_empty_input =
                fills_val.is_null() || fills_val.as_array().map(|a| a.is_empty()).unwrap_or(false);
            if fills.is_empty() && !is_empty_input {
                return Task::none();
            }

            if let Some(item) = fills.get_mut(fill_index) {
                let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                match item {
                    FillItem::Color(value) => *value = hex,
                    FillItem::Object(FillObject::Solid { color: value, .. }) => *value = hex,
                    FillItem::Object(FillObject::Color { color: value, .. }) => *value = hex,
                    _ => {}
                }
            }

            doc.update_property(&element_id, "fill", serde_json::json!(fills));
            state.canvas_cache.clear();
        }
    }

    Task::none()
}

/// toggle_color_picker_eyedropper 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn toggle_color_picker_eyedropper(app: &mut App) -> Task<Message> {
    if let Some(picker) = &mut app.active_color_picker {
        picker.picking = !picker.picking;
    }
    Task::none()
}

#[allow(dead_code)]
fn pick_color_from_canvas(app: &App) -> Option<Color> {
    let picker = app.active_color_picker.as_ref()?;
    if !picker.picking {
        return None;
    }
    let state = app.active_design_state()?;
    let point = app.cursor_position;
    let logical_x = (point.x - state.pan.x) / state.zoom;
    let logical_y = (point.y - state.pan.y) / state.zoom;
    let id = hit_test(&state.doc.children, &state.doc, logical_x, logical_y)?;
    let el = find_element_by_id(&state.doc.children, &id)?;
    let theme_mode = state.doc.theme.as_ref().map(|theme| theme.mode.as_str());
    Some(parse_fill(&el.fill, &state.doc.variables, theme_mode))
}

#[allow(dead_code)]
fn pick_fill_color_from_canvas(app: &App, point: Point) -> Option<Color> {
    let picker = app.active_fill_picker.as_ref()?;
    if !picker.picking {
        return None;
    }
    let state = app.active_design_state()?;
    let logical_x = (point.x - state.pan.x) / state.zoom;
    let logical_y = (point.y - state.pan.y) / state.zoom;
    let id = hit_test(&state.doc.children, &state.doc, logical_x, logical_y)?;
    let el = find_element_by_id(&state.doc.children, &id)?;
    let theme_mode = state.doc.theme.as_ref().map(|theme| theme.mode.as_str());
    Some(parse_fill(&el.fill, &state.doc.variables, theme_mode))
}

/// pick_color 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn pick_color(app: &mut App, point: Point) -> Task<Message> {
    let mut picked_color = None;
    if let Some(picker) = &app.active_color_picker
        && picker.picking
        && let Some(state) = app.active_design_state()
    {
        let logical_x = (point.x - state.pan.x) / state.zoom;
        let logical_y = (point.y - state.pan.y) / state.zoom;
        if let Some(id) = hit_test(&state.doc.children, &state.doc, logical_x, logical_y)
            && let Some(el) = find_element_by_id(&state.doc.children, &id)
        {
            let theme_mode = state.doc.theme.as_ref().map(|theme| theme.mode.as_str());
            picked_color = Some(parse_fill(&el.fill, &state.doc.variables, theme_mode));
        }
    }
    if let Some(color) = picked_color {
        if let Some(picker) = &mut app.active_color_picker {
            picker.picking = false;
        }
        return Task::done(Message::Design(DesignMessage::ColorPickerChange(color)));
    }

    let mut picked_color = None;
    if let Some(picker) = &app.active_fill_picker
        && picker.picking
        && let Some(state) = app.active_design_state()
    {
        let logical_x = (point.x - state.pan.x) / state.zoom;
        let logical_y = (point.y - state.pan.y) / state.zoom;
        if let Some(id) = hit_test(&state.doc.children, &state.doc, logical_x, logical_y)
            && let Some(el) = find_element_by_id(&state.doc.children, &id)
        {
            let theme_mode = state.doc.theme.as_ref().map(|theme| theme.mode.as_str());
            picked_color = Some(parse_fill(&el.fill, &state.doc.variables, theme_mode));
        }
    }
    if let Some(color) = picked_color {
        if let Some(picker) = &mut app.active_fill_picker {
            picker.picking = false;
        }
        return Task::done(Message::Design(DesignMessage::FillPickerColorChange(color)));
    }

    Task::none()
}

/// close_color_picker 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn close_color_picker(app: &mut App) -> Task<Message> {
    app.active_color_picker = None;
    Task::none()
}

/// change_color_picker_format 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn change_color_picker_format(app: &mut App, format: ColorFormat) -> Task<Message> {
    if let Some(picker) = &mut app.active_color_picker {
        picker.format = format;
    }
    Task::none()
}

/// change_color_picker_color 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn change_color_picker_color(app: &mut App, color: Color) -> Task<Message> {
    let target_opt = app.active_color_picker.as_ref().map(|picker| picker.target.clone());
    if let Some(picker) = &mut app.active_color_picker {
        picker.color = color;
    }

    if let Some(target) = target_opt {
        let mut bail = false;
        if let Some(state) = app.active_design_state_mut() {
            let doc = &mut state.doc;
            match target {
                ColorPickerTarget::Fill { element_id, fill_index } => {
                    let fills_json = if let Some(el) = doc.find_element(&element_id) {
                        el.fill.clone()
                    } else {
                        None
                    };

                    if let Some(fills_val) = fills_json {
                        let mut fills = parse_fills(&fills_val);
                        let is_empty_input = fills_val.is_null()
                            || fills_val.as_array().map(|a| a.is_empty()).unwrap_or(false);
                        if fills.is_empty() && !is_empty_input {
                            eprintln!(
                                "Design: Failed to parse fills, skipping update. Value: {:?}",
                                fills_val
                            );
                            bail = true;
                        }

                        if let Some(item) = fills.get_mut(fill_index) {
                            let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                            match item {
                                FillItem::Color(value) => *value = hex,
                                FillItem::Object(FillObject::Solid { color: value, .. }) => {
                                    *value = hex
                                }
                                _ => {}
                            }
                        }

                        doc.update_property(&element_id, "fill", serde_json::json!(fills));
                        state.canvas_cache.clear();
                    }
                }
                ColorPickerTarget::GradientStop { element_id, fill_index, stop_index } => {
                    let fills_json = if let Some(el) = doc.find_element(&element_id) {
                        el.fill.clone()
                    } else {
                        None
                    };

                    if let Some(fills_val) = fills_json {
                        let mut fills = parse_fills(&fills_val);
                        let is_empty_input = fills_val.is_null()
                            || fills_val.as_array().map(|a| a.is_empty()).unwrap_or(false);
                        if fills.is_empty() && !is_empty_input {
                            bail = true;
                        }

                        if let Some(FillItem::Object(FillObject::Gradient(gradient))) =
                            fills.get_mut(fill_index)
                            && let Some(stop) = gradient.colors.get_mut(stop_index)
                        {
                            stop.color = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                        }

                        doc.update_property(&element_id, "fill", serde_json::json!(fills));
                        state.canvas_cache.clear();
                    }
                }
                ColorPickerTarget::MeshPoint { element_id, fill_index, point_index } => {
                    let fills_json = if let Some(el) = doc.find_element(&element_id) {
                        el.fill.clone()
                    } else {
                        None
                    };

                    if let Some(fills_val) = fills_json {
                        let mut fills = parse_fills(&fills_val);
                        let is_empty_input = fills_val.is_null()
                            || fills_val.as_array().map(|a| a.is_empty()).unwrap_or(false);
                        if fills.is_empty() && !is_empty_input {
                            bail = true;
                        }

                        if let Some(FillItem::Object(FillObject::Mesh(mesh))) =
                            fills.get_mut(fill_index)
                            && let Some(mesh_color) = mesh.colors.get_mut(point_index)
                        {
                            *mesh_color = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                        }

                        doc.update_property(&element_id, "fill", serde_json::json!(fills));
                        state.canvas_cache.clear();
                    }
                }
                ColorPickerTarget::Effect { element_id, effect_index } => {
                    let effects_json = if let Some(el) = doc.find_element(&element_id) {
                        el.effect.clone()
                    } else {
                        None
                    };

                    if let Some(effects_val) = effects_json {
                        let mut effects: Vec<crate::app::views::design::models::Effect> =
                            if let Ok(list) = serde_json::from_value(effects_val.clone()) {
                                list
                            } else if let Ok(item) = serde_json::from_value(effects_val.clone()) {
                                vec![item]
                            } else {
                                vec![]
                            };

                        if let Some(effect) = effects.get_mut(effect_index) {
                            effect.color =
                                Some(format_rgba_to_hex(color.r, color.g, color.b, color.a));
                        }

                        doc.update_property(&element_id, "effect", serde_json::json!(effects));
                        state.canvas_cache.clear();
                    }
                }
                ColorPickerTarget::ContextFill { element_id } => {
                    let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                    let fills = vec![FillItem::Object(FillObject::Solid {
                        color: hex,
                        enabled: true,
                    })];
                    doc.update_property(&element_id, "fill", serde_json::json!(fills));
                    state.canvas_cache.clear();
                }
                ColorPickerTarget::ContextBorder { element_id } => {
                    let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                    let is_dashed = doc
                        .find_element(&element_id)
                        .and_then(|el| el.stroke.as_ref())
                        .and_then(|stroke| stroke.fill.as_ref())
                        .is_some_and(|fill| fill.contains("dashArray"));
                    let fill = if is_dashed {
                        format!(
                            "[{{\"type\":\"solid\",\"color\":\"{}\",\"opacity\":1.0,\"dashArray\":[4,4]}}]",
                            hex
                        )
                    } else {
                        format!(
                            "[{{\"type\":\"solid\",\"color\":\"{}\",\"opacity\":1.0}}]",
                            hex
                        )
                    };
                    let stroke = Stroke {
                        align: Some("inside".to_string()),
                        thickness: Some(serde_json::json!(1.0)),
                        fill: Some(fill),
                    };
                    doc.update_property(&element_id, "stroke", serde_json::json!(stroke));
                    state.canvas_cache.clear();
                }
                ColorPickerTarget::ContextText { element_id } => {
                    let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                    doc.update_property(&element_id, "color", serde_json::Value::String(hex));
                    state.canvas_cache.clear();
                }
                ColorPickerTarget::VariableValue { variable_name, mode } => {
                    let hex = format_rgba_to_hex(color.r, color.g, color.b, color.a);
                    if let Some(def) = doc.variables.get_mut(&variable_name) {
                        upsert_variable_value(&mut def.value, mode.as_deref(), hex);
                        state.canvas_cache.clear();
                    }
                }
            }
        }
        if bail {
            return Task::none();
        }
    }

    if let Some(picker) = &mut app.active_color_picker {
        picker.color = color;
    }
    Task::none()
}

/// select_fill 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn select_fill(app: &mut App, index: Option<usize>) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.selected_fill_index = index;
    }
    Task::none()
}

/// select_effect 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn select_effect(app: &mut App, index: Option<usize>) -> Task<Message> {
    if let Some(state) = app.active_design_state_mut() {
        state.selected_effect_index = index;
    }
    Task::none()
}

