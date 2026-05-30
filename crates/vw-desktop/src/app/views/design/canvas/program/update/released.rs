use super::*;

impl<'a> DesignCanvas<'a> {
    /// 处理鼠标左键释放事件
    pub(in super::super) fn handle_left_released(
        &self,
        state: &mut DesignCanvasState,
        cursor_pos: Point,
    ) -> Option<Action<Message>> {
        if self.active_tool == DesignTool::Pen && !state.brush_points_world.is_empty() {
            let action = create_brush_path_element(
                &state.brush_points_world,
                self.brush_color_hex,
                self.brush_width_px,
            )
            .map(|element| {
                Action::publish(Message::Design(DesignMessage::CreateElement {
                    element,
                    parent_id: None,
                    start_editing: false,
                }))
            });

            state.brush_points_world.clear();
            state.brush_erasing = false;
            state.brush_erase_dirty = false;
            return action.or_else(|| Some(Action::request_redraw()));
        }

        if self.active_tool == DesignTool::Eraser && state.brush_erasing {
            state.brush_erasing = false;
            let changed = state.brush_erase_dirty;
            state.brush_erase_dirty = false;
            return Some(if changed {
                Action::publish(Message::Design(DesignMessage::Snapshot))
            } else {
                Action::request_redraw()
            });
        }

        if tool_supports_drag_preview(self.active_tool) && state.tool_preview_start.is_some() {
            let created = build_created_element(self, state, cursor_pos).map(
                |(element, parent_id, start_editing)| {
                    Action::publish(Message::Design(DesignMessage::CreateElement {
                        element,
                        parent_id,
                        start_editing,
                    }))
                },
            );

            state.tool_preview_start = None;
            state.tool_preview_current = None;
            state.tool_preview_parent_id = None;

            return created.or_else(|| Some(Action::request_redraw()));
        }

        if let Some(drag) = state.mesh_drag.take() {
            if drag.has_moved {
                let mut payload = serde_json::Value::Null;
                if let Some(el) = find_element_by_id(&self.doc.children, &drag.element_id) {
                    let mut fills = mesh::parse_fill_items(&el.fill);
                    if let Some(FillItem::Object(FillObject::Mesh(m))) =
                        fills.get_mut(drag.fill_index)
                    {
                        m.normalize();
                        payload = mesh::mesh_curve_change_payload(m, drag.point_index, drag.kind);
                    }
                }

                return Some(Action::publish(Message::Design(DesignMessage::MeshCurveChanged(
                    drag.element_id,
                    drag.fill_index,
                    payload,
                ))));
            }
            return Some(Action::request_redraw());
        }

        if let Some(start) = state.selection_box_start {
            let end = cursor_pos;
            let x = start.x.min(end.x);
            let y = start.y.min(end.y);
            let w = (end.x - start.x).abs();
            let h = (end.y - start.y).abs();

            let selection_rect = Rectangle::new(Point::new(x, y), Size::new(w, h));

            let selected = selection::find_intersecting_ids(
                &self.doc.children,
                self.doc.as_ref(),
                self.pan,
                self.zoom,
                selection_rect,
            );

            state.selection_box_start = None;
            return Some(Action::publish(Message::Design(DesignMessage::MultiSelect(selected))));
        }

        if let Some((items, _, has_moved)) = state.moving_elements.take() {
            if !has_moved {
                state.drop_target_frame_id = None;
                return Some(Action::request_redraw());
            }
            let ids: Vec<String> = items.into_iter().map(|(id, _)| id).collect();
            let parent = state.drop_target_frame_id.take();
            return Some(Action::publish(Message::Design(DesignMessage::ReparentElements(
                ids, parent,
            ))));
        }

        if state.resizing.is_some() || state.rotating.is_some() {
            state.resizing = None;
            state.rotating = None;
            state.drag_start = None;
            return Some(Action::publish(Message::Design(DesignMessage::Snapshot)));
        }

        if self.active_tool == DesignTool::Hand {
            state.is_panning = false;
            state.last_cursor_pos = None;
            return Some(Action::request_redraw());
        }

        None
    }
}

#[cfg(test)]
#[path = "released_tests.rs"]
mod released_tests;
