//! 设计视图剪贴板操作模块
//!
//! 本模块提供设计视图中元素的剪贴板操作功能，包括：
//! - 复制（Copy）：将选中的设计元素序列化为 JSON 并写入系统剪贴板
//! - 剪切（Cut）：复制选中的设计元素后将其从画布中删除
//! - 粘贴（Paste）：从系统剪贴板读取 JSON 数据，反序列化为设计元素并添加到画布
//!
//! ## 功能特性
//!
//! - **深拷贝粘贴**：粘贴时为元素及其所有子元素重新生成唯一标识符，避免 ID 冲突
//! - **自动偏移**：粘贴的元素会自动偏移位置（默认 20 像素），便于用户识别新粘贴的元素
//! - **历史记录集成**：粘贴操作会自动触发历史快照，支持撤销/重做

use crate::app::message::design::{DesignMessage, LayerAction};
use crate::app::views::design::models::DesignElement;
use crate::app::{App, Message};
use iced::Task;

/// 处理设计视图的剪贴板相关消息
///
/// 该函数是设计视图中所有剪贴板操作的统一入口点，根据消息类型执行相应的操作。
///
/// # 参数
///
/// - `app`: 应用程序状态的可变引用，用于访问和修改设计文档状态
/// - `message`: 剪贴板相关的设计消息，可能是复制、剪切、粘贴或剪贴板内容接收通知
///
/// # 返回值
///
/// 返回一个 Iced `Task<Message>`，可能包含以下副作用：
/// - 写入系统剪贴板的任务
/// - 从系统剪贴板读取的任务
/// - 触发图层操作的任务（如删除元素）
/// - 触发历史快照的任务
/// - 无操作（`Task::none()`）
///
/// # 消息处理逻辑
///
/// - **Copy**: 将当前选中的元素序列化为 JSON 并写入剪贴板
/// - **Cut**: 先执行复制操作，然后删除选中的元素
/// - **Paste**: 请求从系统剪贴板读取内容
/// - **ClipboardContentReceived**: 处理剪贴板读取结果，反序列化并添加元素到画布
///
/// # 示例
///
/// ```ignore
/// // 在消息处理循环中调用
/// let task = update(&mut app, DesignMessage::Copy);
/// // 执行返回的任务
/// ```
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        // 处理复制操作：将选中的设计元素序列化后写入系统剪贴板
        DesignMessage::Copy => {
            // 尝试获取当前活动的设计视图状态
            if let Some(state) = app.active_design_state() {
                // 检查是否有选中的元素
                if let Some(id) = &state.selected_element_id {
                    // 在文档中查找该元素
                    if let Some(el) = state.doc.find_element(id) {
                        // 将元素序列化为 JSON 字符串
                        let json = serde_json::to_string(el).unwrap_or_default();
                        // 返回写入剪贴板的任务
                        return iced::clipboard::write(json);
                    }
                }
            }
            // 如果没有活动设计状态、没有选中元素或查找失败，返回空任务
            Task::none()
        }

        // 处理剪切操作：复制选中的元素后将其删除
        DesignMessage::Cut => {
            // 首先执行复制操作，获取复制任务
            let copy_task = update(app, DesignMessage::Copy);

            // 然后尝试删除选中的元素
            if let Some(state) = app.active_design_state() {
                // 克隆选中的元素 ID（因为后续需要可变借用 state）
                if let Some(id) = &state.selected_element_id.clone() {
                    // 批量执行复制任务和删除任务
                    // 使用 LayerAction::Delete 删除选中的元素
                    return Task::batch(vec![
                        copy_task,
                        Task::done(Message::Design(DesignMessage::LayerActionSelected(
                            id.clone(),
                            LayerAction::Delete,
                        ))),
                    ]);
                }
            }
            // 如果没有选中的元素可删除，仅返回复制任务
            copy_task
        }

        // 处理粘贴操作：请求从系统剪贴板读取内容
        // 读取成功后会发送 ClipboardContentReceived 消息
        DesignMessage::Paste => iced::clipboard::read()
            .map(|content| Message::Design(DesignMessage::ClipboardContentReceived(content))),

        // 处理剪贴板内容接收：将 JSON 反序列化为设计元素并添加到画布
        DesignMessage::ClipboardContentReceived(Some(content)) => {
            let cursor_position = app.cursor_position;
            // 尝试获取当前活动设计视图的可变状态
            if let Some(state) = app.active_design_state_mut() {
                // 尝试将剪贴板内容反序列化为设计元素
                if let Ok(mut el) = serde_json::from_str::<DesignElement>(&content) {
                    // 为元素及其所有子元素重新生成唯一标识符
                    // 这样可以避免粘贴后的元素与原始元素 ID 冲突
                    let mut counter = 0;
                    regenerate_ids(&mut el, &mut counter);

                    if let Some(anchor) = state.paste_anchor.take().or_else(|| {
                        if cursor_position == iced::Point::ORIGIN {
                            None
                        } else {
                            Some(cursor_position)
                        }
                    }) {
                        el.x = (anchor.x - state.pan.x) / state.zoom;
                        el.y = (anchor.y - state.pan.y) / state.zoom;
                    } else {
                        el.x += 20.0;
                        el.y += 20.0;
                    }

                    // 将元素添加到文档根节点的子元素列表中
                    state.doc.children.push(el);

                    // 触发历史快照，以便用户可以撤销此粘贴操作
                    return Task::done(Message::Design(DesignMessage::Snapshot));
                }
            }
            // 如果反序列化失败或没有活动设计状态，返回空任务
            Task::none()
        }

        // 其他未处理的消息类型，返回空任务
        _ => Task::none(),
    }
}

/// 为设计元素及其所有子元素重新生成唯一标识符
///
/// 当粘贴元素时，需要为新元素及其子元素分配新的唯一 ID，
/// 以避免与原始元素或其他已存在元素的 ID 冲突。
///
/// # 参数
///
/// - `el`: 要重新生成 ID 的设计元素的可变引用
/// - `counter`: 计数器，用于确保同一粘贴批次中的元素 ID 唯一性
///
/// # ID 生成策略
///
/// ID 格式为 `paste_{timestamp}_{counter}`，其中：
/// - `timestamp`: 当前 Unix 时间戳（纳秒级）
/// - `counter`: 递增计数器，确保同一时刻粘贴的多个元素具有不同的 ID
///
/// # 递归行为
///
/// 该函数会递归地处理元素的所有子元素，确保整个元素树都获得新的 ID。
///
/// # 示例
///
/// ```ignore
/// let mut element = deserialize_element_from_clipboard();
/// let mut counter = 0;
/// regenerate_ids(&mut element, &mut counter);
/// // 现在 element 及其所有子元素都有新的唯一 ID
/// ```
fn regenerate_ids(el: &mut DesignElement, counter: &mut u64) {
    // 递增计数器，确保每个元素获得不同的序号
    *counter += 1;

    // 获取当前 Unix 时间戳（纳秒），用于生成时间唯一的 ID 前缀
    let now = crate::app::time::now_ms();

    // 生成新的 ID，格式：paste_{timestamp}_{counter}
    el.id = format!("paste_{}_{}", now, counter);

    // 递归处理所有子元素
    for child in &mut el.children {
        regenerate_ids(child, counter);
    }
}

