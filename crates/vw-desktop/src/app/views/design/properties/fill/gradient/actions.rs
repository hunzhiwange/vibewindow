//! 渐变填充操作模块，负责把属性面板输入转换为可回放的设计消息。

use crate::app::Message;
use crate::app::message::DesignMessage;
use crate::app::views::design::properties::fill::types::{
    FillItem, FillObject, GradientCenter, GradientFill, GradientSize, GradientStop,
};

/// 执行 update_gradient_type 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_type(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    new_type: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if new_type == "mesh_gradient" {
        if let Some(item) = new_fills.get_mut(index) {
            *item = FillItem::Object(FillObject::Mesh(
                crate::app::views::design::properties::fill::types::MeshFill::new_random(3, 3),
            ));
        }
    } else if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index) {
        g.gradient_type = new_type;
    } else if let Some(item) = new_fills.get_mut(index) {
        *item = FillItem::Object(FillObject::Gradient(GradientFill {
            gradient_type: new_type,
            enabled: true,
            rotation: 0.0,
            colors: vec![],
            center: None,
            size: None,
            size_h: None,
        }));
    }

    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_rotation 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_rotation(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    rot: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Ok(val) = rot.parse::<f64>()
    {
        g.rotation = val;
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_center_x 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_center_x(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Ok(v) = val.parse::<f64>()
    {
        let mut center = g.center.clone().unwrap_or(GradientCenter { x: 50.0, y: 50.0 });
        center.x = v;
        g.center = Some(center);
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_center_y 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_center_y(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Ok(v) = val.parse::<f64>()
    {
        let mut center = g.center.clone().unwrap_or(GradientCenter { x: 50.0, y: 50.0 });
        center.y = v;
        g.center = Some(center);
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

#[allow(dead_code)]
/// 执行 update_gradient_size_w 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_size_w(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Ok(v) = val.parse::<f64>()
    {
        let mut size =
            g.size.clone().unwrap_or(GradientSize { width: Some(100.0), height: Some(100.0) });
        size.width = Some(v);
        g.size = Some(size);
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_size_h 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_size_h(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    let cleaned = val.trim().trim_end_matches('%');
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Ok(v) = cleaned.parse::<f64>()
    {
        g.size_h = Some(v.clamp(0.0, 200.0));
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_size_v 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_size_v(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    val: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Ok(v) = val.parse::<f64>()
    {
        let mut size =
            g.size.clone().unwrap_or(GradientSize { width: Some(100.0), height: Some(100.0) });
        size.height = Some(v);
        g.size = Some(size);
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 处理颜色值的解析、格式化或空间转换。
///
/// 无法识别的颜色返回空结果，避免把错误颜色静默写入设计元素。
pub(super) fn update_gradient_stop_color(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    stop_idx: usize,
    color: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && let Some(stop) = g.colors.get_mut(stop_idx)
    {
        stop.color = color;
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_stop_position 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_stop_position(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    stop_idx: usize,
    pos: String,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index) {
        let cleaned = pos.trim().trim_end_matches('%');
        if let Ok(val) = cleaned.parse::<f64>() {
            let clamped = val.clamp(0.0, 100.0);
            if let Some(stop) = g.colors.get_mut(stop_idx) {
                stop.position = clamped / 100.0;
            }
        }
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 remove_gradient_stop 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn remove_gradient_stop(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    stop_idx: usize,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index)
        && stop_idx < g.colors.len()
    {
        g.colors.remove(stop_idx);
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 add_gradient_stop 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn add_gradient_stop(id: String, fills: Vec<FillItem>, index: usize) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index) {
        g.colors.push(GradientStop { color: "#ffffff".to_string(), position: 0.5 });
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

/// 执行 update_gradient_stops 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn update_gradient_stops(
    id: String,
    fills: Vec<FillItem>,
    index: usize,
    stops: Vec<GradientStop>,
) -> Message {
    use serde_json::json;
    let mut new_fills = fills;
    if let Some(FillItem::Object(FillObject::Gradient(g))) = new_fills.get_mut(index) {
        g.colors = stops;
    }
    Message::Design(DesignMessage::PropertyUpdate(id, "fill".to_string(), json!(new_fills)))
}

#[cfg(test)]
#[path = "actions_tests.rs"]
mod actions_tests;
