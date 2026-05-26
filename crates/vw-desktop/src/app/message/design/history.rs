//! 设计历史管理模块
//!
//! 本模块提供设计文档的撤销/重做（Undo/Redo）功能实现。
//! 通过维护历史状态栈，用户可以在编辑过程中回退到之前的状态或重做已撤销的操作。
//!
//! # 核心功能
//!
//! - **撤销操作**：回退到上一个设计状态
//! - **重做操作**：恢复已撤销的设计状态
//! - **快照保存**：将当前设计状态保存到历史记录中
//!
//! # 历史记录管理策略
//!
//! - 历史栈最多保留 50 个状态
//! - 当超过限制时，自动移除最旧的记录
//! - 执行新操作时，会清除"未来"的历史记录（即重做栈）

use crate::app::message::DesignMessage;
use crate::app::views::design::models::compute_tree_metrics;
use crate::app::{App, Message};
use iced::Task;

/// 处理设计历史相关的消息
///
/// 该函数负责管理设计文档的历史状态，实现撤销、重做和快照功能。
///
/// # 参数
///
/// - `app` - 应用程序的可变引用，用于访问和修改设计状态
/// - `message` - 设计消息枚举，指定要执行的历史操作类型
///
/// # 返回值
///
/// 返回一个 `Task<Message>`，表示可能需要执行的后续任务。
/// 当前实现中，所有历史操作都不产生额外的异步任务，因此返回 `Task::none()`。
///
/// # 支持的操作
///
/// - `DesignMessage::Undo`：撤销到上一个状态
/// - `DesignMessage::Redo`：重做到下一个状态
/// - `DesignMessage::Snapshot`：保存当前状态到历史记录
///
/// # 副作用
///
/// 操作成功后，会触发以下状态更新：
/// - 设计文档（`doc`）恢复到目标历史状态
/// - 图层树指标（`layer_tree_metrics`）重新计算
/// - 画布缓存（`canvas_cache`）被清空
/// - 所有选中状态被重置
///
/// # 示例
///
/// ```ignore
/// // 在消息处理循环中
/// let task = update(&mut app, DesignMessage::Undo);
/// // 执行撤销操作后，设计状态已回退到上一版本
/// ```
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    // 尝试获取当前激活的设计状态，如果不存在则直接返回
    if let Some(state) = app.active_design_state_mut() {
        match message {
            // 撤销操作：回退到上一个历史状态
            DesignMessage::Undo => {
                // 检查是否还有可撤销的历史记录
                if state.history_index > 0 {
                    // 回退历史索引
                    state.history_index -= 1;

                    // 从历史栈中获取上一个状态
                    if let Some(prev_doc) = state.history.get(state.history_index) {
                        state.doc = prev_doc.clone();
                        state.ensure_valid_group();
                        state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                        state.canvas_cache.clear();
                        state.selected_element_id = None;
                        state.selected_element_ids.clear();
                    }
                }
            }

            // 重做操作：前进到下一个历史状态
            DesignMessage::Redo => {
                // 检查是否还有可重做的历史记录
                if state.history_index + 1 < state.history.len() {
                    // 前进历史索引
                    state.history_index += 1;

                    // 从历史栈中获取下一个状态
                    if let Some(next_doc) = state.history.get(state.history_index) {
                        state.doc = next_doc.clone();
                        state.ensure_valid_group();
                        state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                        state.canvas_cache.clear();
                        state.selected_element_id = None;
                        state.selected_element_ids.clear();
                    }
                }
            }

            // 快照操作：保存当前状态到历史记录
            DesignMessage::Snapshot => {
                // 截断"未来"的历史记录
                // 当用户在撤销后执行新操作时，之前的重做路径将失效
                if state.history_index + 1 < state.history.len() {
                    state.history.truncate(state.history_index + 1);
                }

                state.doc.normalize_groups();
                state.history.push(state.doc.clone());
                // 更新历史索引到最新位置
                state.history_index = state.history.len() - 1;

                // 限制历史记录大小，防止内存占用过大
                // 保留最近 50 个状态，超过时移除最旧的记录
                if state.history.len() > 50 {
                    state.history.remove(0);
                    // 移除最旧记录后，索引需要相应减一
                    state.history_index -= 1;
                }
            }

            // 其他设计消息不在本模块处理范围内
            _ => {}
        }
    }

    // 历史操作不需要产生额外的异步任务
    Task::none()
}

