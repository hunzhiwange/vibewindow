//! 设计属性消息处理模块
//!
//! 为降低 `property.rs` 的体积，内部实现按职责拆到 `property/` 子模块，
//! 对外仍保持 `property::update` 这一入口不变。

use crate::app::message::DesignMessage;
use crate::app::{App, Message};
use iced::Task;

#[path = "property/common.rs"]
mod common;
#[path = "property/editors.rs"]
mod editors;
#[path = "property/groups.rs"]
mod groups;
#[path = "property/pickers.rs"]
mod pickers;
#[path = "property/tailwind.rs"]
mod tailwind;
#[path = "property/updates.rs"]
mod updates;

pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        DesignMessage::SetActiveGroup(group_id) => groups::set_active_group(app, group_id),
        DesignMessage::NewGroupNameChanged(value) => groups::new_group_name_changed(app, value),
        DesignMessage::CreateGroup => groups::create_group(app),
        DesignMessage::PageMenuToggle(group_id, x, y) => {
            groups::toggle_page_menu(app, group_id, x, y)
        }
        DesignMessage::PageMenuClose => groups::close_page_menu(app),
        DesignMessage::PageActionSelected(
            group_id,
            crate::app::message::design::PageAction::Rename,
        ) => groups::rename_page_requested(app, group_id),
        DesignMessage::PageActionSelected(
            group_id,
            crate::app::message::design::PageAction::Duplicate,
        ) => groups::duplicate_page(app, group_id),
        DesignMessage::PageActionSelected(
            group_id,
            crate::app::message::design::PageAction::Delete,
        ) => groups::delete_page(app, group_id),
        DesignMessage::PageActionSelected(
            group_id,
            crate::app::message::design::PageAction::MoveUp,
        ) => groups::move_page_up(app, group_id),
        DesignMessage::PageActionSelected(
            group_id,
            crate::app::message::design::PageAction::MoveDown,
        ) => groups::move_page_down(app, group_id),
        DesignMessage::PageRenameChanged(value) => groups::page_rename_changed(app, value),
        DesignMessage::PageRenameSubmit => groups::submit_page_rename(app),
        DesignMessage::PageRenameCancel => groups::cancel_page_rename(app),
        DesignMessage::SetTailwindFilter(query) => tailwind::set_filter(app, query),
        DesignMessage::OpenTailwindClassPicker(element_id, position_opt) => {
            tailwind::open_class_picker(app, element_id, position_opt)
        }
        DesignMessage::CloseTailwindClassPicker => tailwind::close_class_picker(app),
        DesignMessage::TailwindInspectorHover(hovered) => {
            tailwind::set_inspector_hover(app, hovered)
        }
        DesignMessage::TailwindClassInputChanged(element_id, input) => {
            tailwind::class_input_changed(app, element_id, input)
        }
        DesignMessage::TailwindClassInputSubmit(element_id) => {
            tailwind::class_input_submit(app, element_id)
        }
        DesignMessage::TailwindNodeClassInputChanged(element_id, path, input) => {
            tailwind::node_class_input_changed(app, element_id, path, input)
        }
        DesignMessage::TailwindNodeClassDropdownClose(element_id, path) => {
            tailwind::close_node_class_dropdown(app, element_id, path)
        }
        DesignMessage::TailwindNodeClassInputSubmit(element_id, path) => {
            tailwind::node_class_input_submit(app, element_id, path)
        }
        DesignMessage::AddTailwindClassToken(element_id, token) => {
            tailwind::add_class_token(app, element_id, token)
        }
        DesignMessage::SetFontFilter(query) => pickers::set_font_filter(app, query),
        DesignMessage::OpenFontPicker(element_id, position_opt) => {
            pickers::open_font_picker(app, element_id, position_opt)
        }
        DesignMessage::CloseFontPicker => pickers::close_font_picker(app),
        DesignMessage::FontPickerSelect(element_id, family) => {
            pickers::select_font(app, element_id, family)
        }
        DesignMessage::SetIconPickerFilter(query) => pickers::set_icon_picker_filter(app, query),
        DesignMessage::OpenIconPicker(element_id, position_opt) => {
            pickers::open_icon_picker(app, element_id, position_opt)
        }
        DesignMessage::CloseIconPicker => pickers::close_icon_picker(app),
        DesignMessage::SetIconPickerFamilyTab(family) => {
            pickers::set_icon_picker_family_tab(app, family)
        }
        DesignMessage::IconFamilySelected { element_id, family } => {
            pickers::select_icon_family(app, element_id, family)
        }
        DesignMessage::IconPickerSelect { element_id, family, name } => {
            pickers::select_icon(app, element_id, family, name)
        }
        DesignMessage::ShowHelpModal(text) => pickers::show_help_modal(app, text),
        DesignMessage::CloseHelpModal => pickers::close_help_modal(app),
        DesignMessage::OpenColorPicker(color, target, position_opt) => {
            pickers::open_color_picker(app, color, target, position_opt)
        }
        DesignMessage::OpenFillPicker(element_id, fill_index, position_opt) => {
            pickers::open_fill_picker(app, element_id, fill_index, position_opt)
        }
        DesignMessage::CloseFillPicker => pickers::close_fill_picker(app),
        DesignMessage::OpenEffectPicker(element_id, effect_index, position_opt) => {
            pickers::open_effect_picker(app, element_id, effect_index, position_opt)
        }
        DesignMessage::CloseEffectPicker => pickers::close_effect_picker(app),
        DesignMessage::FillPickerEyedropper => pickers::toggle_fill_picker_eyedropper(app),
        DesignMessage::FillPickerFormatChange(format) => {
            pickers::change_fill_picker_format(app, format)
        }
        DesignMessage::FillPickerColorChange(color) => {
            pickers::change_fill_picker_color(app, color)
        }
        DesignMessage::ColorPickerEyedropper => pickers::toggle_color_picker_eyedropper(app),
        DesignMessage::PickColor(point) => pickers::pick_color(app, point),
        DesignMessage::CloseColorPicker => pickers::close_color_picker(app),
        DesignMessage::ColorPickerFormatChange(format) => {
            pickers::change_color_picker_format(app, format)
        }
        DesignMessage::ColorPickerChange(color) => pickers::change_color_picker_color(app, color),
        DesignMessage::PropertyUpdate(id, key, value) => {
            updates::property_update(app, id, key, value)
        }
        DesignMessage::PropertiesUpdate(id, props) => updates::properties_update(app, id, props),
        DesignMessage::BatchPropertiesUpdate(all_updates) => {
            updates::batch_properties_update(app, all_updates)
        }
        DesignMessage::PropertyUpdateTransient(id, key, value) => {
            update(app, DesignMessage::PropertyUpdate(id, key, value))
        }
        DesignMessage::PropertiesUpdateTransient(id, props) => {
            update(app, DesignMessage::PropertiesUpdate(id, props))
        }
        DesignMessage::BatchPropertiesUpdateTransient(all_updates) => {
            update(app, DesignMessage::BatchPropertiesUpdate(all_updates))
        }
        DesignMessage::ContextEditorAction(action) => editors::context_editor_action(app, action),
        DesignMessage::ToggleContextEditor => editors::toggle_context_editor(app),
        DesignMessage::ContentEditorAction(action) => editors::content_editor_action(app, action),
        DesignMessage::TailwindHtmlEditorAction(action) => {
            editors::tailwind_html_editor_action(app, action)
        }
        DesignMessage::TailwindNodeClassEditorAction(action) => {
            editors::tailwind_node_class_editor_action(app, action)
        }
        DesignMessage::TailwindNodeTextEditorAction(action) => {
            editors::tailwind_node_text_editor_action(app, action)
        }
        DesignMessage::SelectFill(index) => pickers::select_fill(app, index),
        DesignMessage::SelectEffect(index) => pickers::select_effect(app, index),
        _ => Task::none(),
    }
}

#[cfg(test)]
#[path = "property/common_tests.rs"]
mod common_tests;

#[cfg(test)]
#[path = "property/editors_tests.rs"]
mod editors_tests;

#[cfg(test)]
#[path = "property/groups_tests.rs"]
mod groups_tests;

#[cfg(test)]
#[path = "property/pickers_tests.rs"]
mod pickers_tests;

#[cfg(test)]
#[path = "property/tailwind_tests.rs"]
mod tailwind_tests;

#[cfg(test)]
#[path = "property/updates_tests.rs"]
mod updates_tests;
