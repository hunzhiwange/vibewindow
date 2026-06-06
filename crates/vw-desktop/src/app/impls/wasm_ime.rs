//! wasm 聊天输入 IME 事件桥。
//!
//! winit 的 Web 后端不会把浏览器 IME 文本完整送进 iced 文本编辑器。
//! 这里用一个真实 textarea 接收浏览器文本输入，再复用现有输入编辑器状态更新路径。

use iced::Subscription;
use iced::widget::text_editor;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure};

use crate::app::{Message, message};

const TEXTAREA_ID: &str = "vibe-window-chat-ime-bridge";

pub(crate) fn subscription() -> Subscription<Message> {
    Subscription::run(ime_events)
}

pub(crate) fn focus_textarea() {
    if let Ok(textarea) = ensure_textarea() {
        textarea.set_value("");
        focus_now(&textarea);
        focus_next_tick(textarea);
    }
}

fn ime_events() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(32, |output: iced::futures::channel::mpsc::Sender<Message>| async move {
        let Ok(textarea) = ensure_textarea() else {
            iced::futures::future::pending::<()>().await;
            return;
        };
        let last_composition_commit = Rc::new(RefCell::new(None::<String>));
        let active = Rc::new(Cell::new(false));

        let input_target = textarea.clone();
        let input_last_commit = Rc::clone(&last_composition_commit);
        let mut input_output = output.clone();
        let input_closure =
            Closure::<dyn FnMut(web_sys::InputEvent)>::new(move |event: web_sys::InputEvent| {
                if event.is_composing() {
                    return;
                }

                let content = input_target.value();
                if content.is_empty() {
                    return;
                }

                input_target.set_value("");
                if input_last_commit.borrow_mut().take().as_deref() == Some(content.as_str()) {
                    return;
                }

                let _ = input_output
                    .try_send(Message::Chat(message::ChatMessage::WasmImeCommit(content)));
            });

        let _ = textarea
            .add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref());
        input_closure.forget();

        let composition_target = textarea.clone();
        let composition_last_commit = Rc::clone(&last_composition_commit);
        let mut composition_output = output.clone();
        let composition_closure = Closure::<dyn FnMut(web_sys::CompositionEvent)>::new(
            move |event: web_sys::CompositionEvent| {
                let content = event.data().unwrap_or_else(|| composition_target.value());
                if content.is_empty() {
                    return;
                }

                composition_target.set_value("");
                *composition_last_commit.borrow_mut() = Some(content.clone());
                let _ = composition_output
                    .try_send(Message::Chat(message::ChatMessage::WasmImeCommit(content)));
            },
        );

        let _ = textarea.add_event_listener_with_callback(
            "compositionend",
            composition_closure.as_ref().unchecked_ref(),
        );
        composition_closure.forget();

        let mut key_output = output.clone();
        let key_closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |event: web_sys::KeyboardEvent| {
                if event.is_composing() {
                    return;
                }

                let message = match event.key().as_str() {
                    "Backspace" => {
                        event.prevent_default();
                        message::ChatMessage::WasmImeBackspace
                    }
                    "Delete" => {
                        event.prevent_default();
                        message::ChatMessage::WasmImeDelete
                    }
                    "Enter" if event.ctrl_key() || event.meta_key() => {
                        event.prevent_default();
                        message::ChatMessage::SendPressed
                    }
                    "Enter" => {
                        event.prevent_default();
                        message::ChatMessage::WasmImeCommit("\n".to_string())
                    }
                    "Tab" => {
                        event.prevent_default();
                        message::ChatMessage::FileSearchSelectCurrent
                    }
                    "Escape" => {
                        event.prevent_default();
                        message::ChatMessage::FileSearchInputChanged(String::new())
                    }
                    "a" | "A" if event.ctrl_key() || event.meta_key() => {
                        event.prevent_default();
                        message::ChatMessage::SelectAllInput
                    }
                    key => {
                        let Some(mut motion) = motion_for_key(key) else {
                            return;
                        };
                        event.prevent_default();

                        if event.ctrl_key() || event.meta_key() {
                            motion = motion.widen();
                        }

                        message::ChatMessage::WasmImeMove { motion, select: event.shift_key() }
                    }
                };

                let _ = key_output.try_send(Message::Chat(message));
            },
        );

        let _ = textarea
            .add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref());
        key_closure.forget();

        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let mouse_target = textarea.clone();
            let mouse_active = Rc::clone(&active);
            let mouse_closure = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(
                move |event: web_sys::MouseEvent| {
                    let is_input_click = is_bottom_input_click(event.client_y());
                    mouse_active.set(is_input_click);
                    if !is_input_click {
                        return;
                    }

                    mouse_target.set_value("");
                    focus_now(&mouse_target);
                    focus_next_tick(mouse_target.clone());
                },
            );

            let _ = document.add_event_listener_with_callback_and_bool(
                "mousedown",
                mouse_closure.as_ref().unchecked_ref(),
                true,
            );
            mouse_closure.forget();

            let paste_active = Rc::clone(&active);
            let paste_target = textarea.clone();
            let mut paste_output = output.clone();
            let paste_closure = Closure::<dyn FnMut(web_sys::ClipboardEvent)>::new(
                move |event: web_sys::ClipboardEvent| {
                    if !paste_active.get() {
                        return;
                    }

                    let text = event
                        .clipboard_data()
                        .and_then(|data| data.get_data("text/plain").ok())
                        .unwrap_or_default();
                    if text.is_empty() {
                        return;
                    }

                    event.prevent_default();
                    paste_target.set_value("");
                    focus_now(&paste_target);
                    focus_next_tick(paste_target.clone());

                    let _ = paste_output
                        .try_send(Message::Chat(message::ChatMessage::WasmImeCommit(text)));
                },
            );

            let _ = document.add_event_listener_with_callback_and_bool(
                "paste",
                paste_closure.as_ref().unchecked_ref(),
                true,
            );
            paste_closure.forget();
        }

        iced::futures::future::pending::<()>().await;
    })
}

fn focus_now(textarea: &web_sys::HtmlTextAreaElement) {
    let element = textarea.unchecked_ref::<web_sys::HtmlElement>();
    let _ = element.focus();
}

fn focus_next_tick(textarea: web_sys::HtmlTextAreaElement) {
    let closure = Closure::<dyn FnMut()>::new(move || {
        focus_now(&textarea);
    });

    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            0,
        );
    }
    closure.forget();
}

fn ensure_textarea() -> Result<web_sys::HtmlTextAreaElement, wasm_bindgen::JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("document unavailable"))?;

    if let Some(existing) = document.get_element_by_id(TEXTAREA_ID) {
        return existing.dyn_into::<web_sys::HtmlTextAreaElement>().map_err(Into::into);
    }

    let textarea =
        document.create_element("textarea")?.dyn_into::<web_sys::HtmlTextAreaElement>()?;

    textarea.set_id(TEXTAREA_ID);
    textarea.set_attribute("aria-hidden", "true")?;
    textarea.set_attribute("autocomplete", "off")?;
    textarea.set_attribute("autocorrect", "off")?;
    textarea.set_attribute("autocapitalize", "off")?;
    textarea.set_attribute("spellcheck", "false")?;
    textarea.set_attribute(
        "style",
        concat!(
            "position:fixed;",
            "left:24px;",
            "bottom:120px;",
            "width:1px;",
            "height:1px;",
            "opacity:0.01;",
            "z-index:1;",
            "resize:none;",
            "border:0;",
            "padding:0;",
            "margin:0;",
            "outline:none;",
            "background:transparent;",
            "color:transparent;",
            "caret-color:transparent;",
            "overflow:hidden;"
        ),
    )?;

    let body =
        document.body().ok_or_else(|| wasm_bindgen::JsValue::from_str("body unavailable"))?;
    body.append_child(&textarea)?;

    Ok(textarea)
}

fn motion_for_key(key: &str) -> Option<text_editor::Motion> {
    match key {
        "ArrowLeft" => Some(text_editor::Motion::Left),
        "ArrowRight" => Some(text_editor::Motion::Right),
        "ArrowUp" => Some(text_editor::Motion::Up),
        "ArrowDown" => Some(text_editor::Motion::Down),
        "Home" => Some(text_editor::Motion::Home),
        "End" => Some(text_editor::Motion::End),
        "PageUp" => Some(text_editor::Motion::PageUp),
        "PageDown" => Some(text_editor::Motion::PageDown),
        _ => None,
    }
}

fn is_bottom_input_click(client_y: i32) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let Some(height) = window.inner_height().ok().and_then(|value| value.as_f64()) else {
        return false;
    };

    f64::from(client_y) >= height - 260.0
}
