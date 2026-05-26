use super::*;

impl<'a> DesignCanvas<'a> {
    /// 处理键盘按键事件
    ///
    /// 根据按下的键执行相应操作：
    /// - **Escape**：取消当前的网格拖拽状态，清除选中的控制点，归一化网格
    /// - **Delete/Backspace**：删除选中的网格控制点或重置控制点手柄
    ///
    /// # 参数
    ///
    /// - `state`：可变的画布状态引用，包含拖拽状态和选中信息
    /// - `key`：按下的键
    ///
    /// # 返回值
    ///
    /// 返回 `Some(Action<Message>)` 表示需要执行的操作（重绘或发布消息），
    /// 返回 `None` 表示该键未处理。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// if let Some(action) = canvas.update_key_pressed(&mut state, &key) {
    ///     // 处理返回的 action
    /// }
    /// ```
    pub(in super::super) fn update_key_pressed(
        &self,
        state: &mut DesignCanvasState,
        key: &iced::keyboard::Key,
    ) -> Option<Action<Message>> {
        match key {
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                state.mesh_drag = None;
                state.selected_mesh_handle = None;
                state.tool_preview_start = None;
                state.tool_preview_current = None;
                state.tool_preview_parent_id = None;
                state.brush_points_world.clear();
                state.brush_erasing = false;
                state.brush_erase_dirty = false;

                let Some(element_id) = self.selected_id else {
                    return Some(Action::request_redraw());
                };
                let Some(el) = find_element_by_id(&self.doc.children, element_id) else {
                    return Some(Action::request_redraw());
                };

                let mut fills = mesh::parse_fill_items(&el.fill);
                let Some(fill_index) =
                    mesh::choose_mesh_fill_index(&fills, self.selected_fill_index)
                else {
                    return Some(Action::request_redraw());
                };

                if let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get_mut(fill_index) {
                    m.normalize();
                    if m.selected_point_index.is_some() {
                        m.selected_point_index = None;
                        return Some(Action::publish(Message::Design(
                            DesignMessage::PropertyUpdateTransient(
                                element_id.to_string(),
                                "fill".to_string(),
                                serde_json::json!(fills),
                            ),
                        )));
                    }
                }

                Some(Action::request_redraw())
            }
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete)
            | iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => {
                let Some(element_id) = self.selected_id else {
                    return None;
                };
                let Some(el) = find_element_by_id(&self.doc.children, element_id) else {
                    return None;
                };

                let mut fills = mesh::parse_fill_items(&el.fill);
                let Some(fill_index) =
                    mesh::choose_mesh_fill_index(&fills, self.selected_fill_index)
                else {
                    return None;
                };

                let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get_mut(fill_index) else {
                    return None;
                };

                m.normalize();

                let point_index = state
                    .selected_mesh_handle
                    .as_ref()
                    .filter(|h| h.element_id == element_id && h.fill_index == fill_index)
                    .map(|h| h.point_index)
                    .or(m.selected_point_index)?;

                if point_index >= m.points.len() {
                    return None;
                }

                let handle_index = state
                    .selected_mesh_handle
                    .as_ref()
                    .filter(|h| {
                        h.element_id == element_id
                            && h.fill_index == fill_index
                            && h.point_index == point_index
                            && h.handle_index < 4
                    })
                    .map(|h| h.handle_index);

                let px = m.points[point_index].first().copied().unwrap_or(0.0).clamp(0.0, 1.0);
                let py = m.points[point_index].get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0);

                let mut next = mesh::mesh_point_handles(m, point_index);
                if let Some(hi) = handle_index {
                    let base = hi.saturating_mul(2);
                    if base + 1 < 8 {
                        next[base] = px;
                        next[base + 1] = py;
                    }
                } else {
                    next = [px, py, px, py, px, py, px, py];
                }
                if point_index < m.handles.len() {
                    m.handles[point_index] = next.to_vec();
                }

                state.selected_mesh_handle = None;

                Some(Action::publish(Message::Design(DesignMessage::PropertyUpdate(
                    element_id.to_string(),
                    "fill".to_string(),
                    serde_json::json!(fills),
                ))))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "keyboard_tests.rs"]
mod keyboard_tests;
