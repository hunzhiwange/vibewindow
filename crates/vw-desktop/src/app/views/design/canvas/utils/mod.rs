//! 设计画布工具模块，集中处理元素查找、样式应用、主题模式和实例引用解析。

use super::super::models::{DesignDoc, DesignElement, Stroke};
use super::tailwind::TailwindParser;
use iced::Color;
use serde_json::Value;

fn color_to_hex(c: Color) -> String {
    let r = (c.r * 255.0) as u8;
    let g = (c.g * 255.0) as u8;
    let b = (c.b * 255.0) as u8;
    let a = (c.a * 255.0) as u8;
    if a == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }
}

/// 执行 apply_tailwind_classes 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub fn apply_tailwind_classes(element: &mut DesignElement) {
    if let Some(class_str) = &element.class {
        let style = TailwindParser::parse(class_str);

        if let Some(w) = style.width {
            if w < 0.0 {
                element.fill_width = Some(true);
            } else {
                element.width =
                    Some(Value::Number(serde_json::Number::from_f64(w as f64).unwrap()));
            }
        }

        if let Some(h) = style.height {
            if h < 0.0 {
                element.fill_height = Some(true);
            } else {
                element.height =
                    Some(Value::Number(serde_json::Number::from_f64(h as f64).unwrap()));
            }
        }

        if let Some(p) = style.padding {
            element.padding = Some(Value::Number(serde_json::Number::from_f64(p as f64).unwrap()));
        }

        if let Some(g) = style.gap_x.or(style.gap_y) {
            element.gap = Some(Value::Number(serde_json::Number::from_f64(g as f64).unwrap()));
        }

        if let Some(c) = style.text_color {
            element.color = Some(color_to_hex(c));
        }

        if let Some(c) = style.background_color {
            element.fill = Some(Value::String(color_to_hex(c)));
        }

        if let Some(r) = style.border_radius {
            element.corner_radius =
                Some(Value::Number(serde_json::Number::from_f64(r as f64).unwrap()));
        }

        if style.border_width.is_some() || style.border_color.is_some() {
            let mut stroke = element.stroke.clone().unwrap_or(Stroke {
                align: Some("inside".to_string()),
                thickness: None,
                fill: None,
            });
            if let Some(w) = style.border_width {
                stroke.thickness =
                    Some(Value::Number(serde_json::Number::from_f64(w as f64).unwrap()));
            }
            if let Some(c) = style.border_color {
                stroke.fill = Some(color_to_hex(c));
            }
            element.stroke = Some(stroke);
        }

        if let Some(size) = style.font_size {
            element.font_size =
                Some(Value::Number(serde_json::Number::from_f64(size as f64).unwrap()));
        }

        if let Some(align) = style.align_items {
            element.align_items = Some(align);
        }

        if let Some(justify) = style.justify_content {
            element.justify_content = Some(justify);
        }
    }
}

/// 执行 theme_mode_for_element 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub fn theme_mode_for_element<'a>(
    doc: &'a DesignDoc,
    element: &'a DesignElement,
    inherited: Option<&'a str>,
) -> Option<&'a str> {
    element
        .theme
        .as_ref()
        .and_then(|v| v.get("Mode"))
        .and_then(|v| v.as_str())
        .or(inherited)
        .or_else(|| doc.theme.as_ref().map(|t| t.mode.as_str()))
}

/// 执行 find_element_by_id 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub fn find_element_by_id<'a>(
    elements: &'a [DesignElement],
    id: &str,
) -> Option<&'a DesignElement> {
    for el in elements {
        if el.id == id {
            return Some(el);
        }
        if let Some(found) = find_element_by_id(&el.children, id) {
            return Some(found);
        }
    }
    None
}

/// 执行 clone_with_overrides 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub fn clone_with_overrides(
    node: &DesignElement,
    overrides: &serde_json::Map<String, Value>,
) -> Option<DesignElement> {
    if let Some(Value::Object(ov)) = overrides.get(&node.id)
        && let Some(Value::Bool(enabled)) = ov.get("enabled")
        && !enabled
    {
        return None;
    }

    if let Some(Value::Object(ov)) = overrides.get(&node.id)
        && ov.contains_key("type")
        && ov.contains_key("id")
        && let Ok(mut replacement) =
            serde_json::from_value::<DesignElement>(Value::Object(ov.clone()))
    {
        replacement.children = replacement
            .children
            .iter()
            .filter_map(|c| clone_with_overrides(c, overrides))
            .collect();
        return Some(replacement);
    }
    let mut new_node = node.clone();
    let mut children_overridden = false;
    if let Some(Value::Object(ov)) = overrides.get(&node.id) {
        if let Some(Value::String(s)) = ov.get("type") {
            if s != &node.kind {
                if !ov.contains_key("fill") {
                    new_node.fill = None;
                }
                if !ov.contains_key("content") {
                    new_node.content = None;
                }
                if !ov.contains_key("geometry") {
                    new_node.geometry = None;
                }
                if !ov.contains_key("stroke") {
                    new_node.stroke = None;
                }
            }
            new_node.kind = s.clone();
        }
        if let Some(Value::Bool(v)) = ov.get("enabled") {
            new_node.enabled = Some(*v);
        }
        if let Some(Value::Bool(v)) = ov.get("visible") {
            new_node.visible = Some(*v);
        }
        if let Some(Value::Number(n)) = ov.get("x")
            && let Some(v) = n.as_f64() {
                new_node.x = v as f32;
            }
        if let Some(Value::Number(n)) = ov.get("y")
            && let Some(v) = n.as_f64() {
                new_node.y = v as f32;
            }
        if let Some(Value::String(s)) = ov.get("name") {
            new_node.name = Some(s.clone());
        }
        if let Some(v) = ov.get("width") {
            new_node.width = Some(v.clone());
        }
        if let Some(v) = ov.get("height") {
            new_node.height = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("content") {
            new_node.content = Some(s.clone());
        }
        if let Some(v) = ov.get("fill") {
            new_node.fill = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("geometry") {
            new_node.geometry = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("layout") {
            new_node.layout = Some(s.clone());
        }
        if let Some(v) = ov.get("gap") {
            new_node.gap = Some(v.clone());
        }
        if let Some(v) = ov.get("padding") {
            new_node.padding = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("alignItems") {
            new_node.align_items = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("justifyContent") {
            new_node.justify_content = Some(s.clone());
        }
        if let Some(v) = ov.get("cornerRadius") {
            new_node.corner_radius = Some(v.clone());
        }
        if let Some(v) = ov.get("stroke")
            && let Ok(stroke) = serde_json::from_value::<Stroke>(v.clone()) {
                new_node.stroke = Some(stroke);
            }
        if let Some(v) = ov.get("effect") {
            new_node.effect = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("class") {
            new_node.class = Some(s.clone());
        }
        if let Some(Value::Number(n)) = ov.get("rotation")
            && let Some(v) = n.as_f64() {
                new_node.rotation = Some(v as f32);
            }
        if let Some(Value::Number(n)) = ov.get("opacity")
            && let Some(v) = n.as_f64() {
                new_node.opacity = Some(v as f32);
            }
        if let Some(Value::Bool(v)) = ov.get("clip") {
            new_node.clip = Some(*v);
        }
        if let Some(Value::Bool(v)) = ov.get("clipContent") {
            new_node.clip_content = Some(*v);
        }
        if let Some(Value::Bool(v)) = ov.get("fillWidth") {
            new_node.fill_width = Some(*v);
        }
        if let Some(Value::Bool(v)) = ov.get("fillHeight") {
            new_node.fill_height = Some(*v);
        }
        if let Some(Value::Bool(v)) = ov.get("hugWidth") {
            new_node.hug_width = Some(*v);
        }
        if let Some(Value::Bool(v)) = ov.get("hugHeight") {
            new_node.hug_height = Some(*v);
        }
        if let Some(Value::String(s)) = ov.get("context") {
            new_node.context = Some(s.clone());
        }
        if let Some(v) = ov.get("fontSize") {
            new_node.font_size = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("fontFamily") {
            new_node.font_family = Some(s.clone());
        }
        if let Some(v) = ov.get("fontWeight") {
            new_node.font_weight = Some(v.clone());
        }
        if let Some(v) = ov.get("weight") {
            new_node.weight = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("fontStyle") {
            new_node.font_style = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("textDecoration") {
            new_node.text_decoration = Some(s.clone());
        }
        if let Some(v) = ov.get("lineHeight") {
            new_node.line_height = Some(v.clone());
        }
        if let Some(v) = ov.get("letterSpacing") {
            new_node.letter_spacing = Some(v.clone());
        }
        if let Some(Value::String(s)) = ov.get("textAlignVertical") {
            new_node.text_align_vertical = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("textAlign") {
            new_node.text_align = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("textGrowth") {
            new_node.text_growth = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("color") {
            new_node.color = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("iconFontName") {
            new_node.icon_font_name = Some(s.clone());
        }
        if let Some(Value::String(s)) = ov.get("iconFontFamily") {
            new_node.icon_font_family = Some(s.clone());
        }
        if let Some(children_val) = ov.get("children")
            && let Ok(children) = serde_json::from_value(children_val.clone()) {
                new_node.children = children;
                children_overridden = true;
            }
    }
    if !children_overridden {
        new_node.children =
            node.children.iter().filter_map(|c| clone_with_overrides(c, overrides)).collect();
    }
    Some(new_node)
}

/// 执行 resolve_ref_instance 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub fn resolve_ref_instance(
    element: &DesignElement,
    roots: &[DesignElement],
    overrides: Option<&serde_json::Map<String, Value>>,
) -> Option<DesignElement> {
    let ref_id = element.reference.as_deref()?;
    let target = find_element_by_id(roots, ref_id)?;
    let mut inst = target.clone();
    inst.x = 0.0;
    inst.y = 0.0;
    if element.width.is_some() {
        inst.width = element.width.clone();
    }
    if element.height.is_some() {
        inst.height = element.height.clone();
    }
    if element.fill.is_some() {
        inst.fill = element.fill.clone();
    }
    if element.icon_font_name.is_some() {
        inst.icon_font_name = element.icon_font_name.clone();
    }
    if element.icon_font_family.is_some() {
        inst.icon_font_family = element.icon_font_family.clone();
    }
    if element.weight.is_some() {
        inst.weight = element.weight.clone();
    }
    if element.effect.is_some() {
        inst.effect = element.effect.clone();
    }
    if element.stroke.is_some() {
        inst.stroke = element.stroke.clone();
    }
    if element.layout.is_some() {
        inst.layout = element.layout.clone();
    }
    if element.padding.is_some() {
        inst.padding = element.padding.clone();
    }
    if element.gap.is_some() {
        inst.gap = element.gap.clone();
    }
    if element.align_items.is_some() {
        inst.align_items = element.align_items.clone();
    }
    if element.justify_content.is_some() {
        inst.justify_content = element.justify_content.clone();
    }
    if element.corner_radius.is_some() {
        inst.corner_radius = element.corner_radius.clone();
    }
    if element.slot.is_some() {
        inst.slot = element.slot.clone();
    }
    if !element.children.is_empty() {
        inst.children = element.children.clone();
    }
    if element.clip.is_some() {
        inst.clip = element.clip;
    }
    if element.clip_content.is_some() {
        inst.clip_content = element.clip_content;
    }
    if element.rotation.is_some() {
        inst.rotation = element.rotation;
    }
    if element.opacity.is_some() {
        inst.opacity = element.opacity;
    }
    if element.enabled.is_some() {
        inst.enabled = element.enabled;
    }
    if element.visible.is_some() {
        inst.visible = element.visible;
    }

    let target_map = inst.descendants.as_ref().and_then(|v| v.as_object());
    let self_map = element.descendants.as_ref().and_then(|v| v.as_object());

    if inst.kind == "ref" {
        let mut merged: serde_json::Map<String, Value> = serde_json::Map::new();
        if let Some(m) = target_map {
            for (k, v) in m.iter() {
                merged.insert(k.clone(), v.clone());
            }
        }
        if let Some(m) = overrides {
            for (k, v) in m.iter() {
                merged.insert(k.clone(), v.clone());
            }
        }
        if let Some(m) = self_map {
            for (k, v) in m.iter() {
                merged.insert(k.clone(), v.clone());
            }
        }
        if merged.is_empty() {
            inst.descendants = None;
        } else {
            inst.descendants = Some(Value::Object(merged));
        }
        return Some(inst);
    }

    match (overrides, self_map) {
        (None, None) => Some(inst),
        (Some(m), None) => clone_with_overrides(&inst, m),
        (None, Some(m)) => clone_with_overrides(&inst, m),
        (Some(parent), Some(local)) => {
            let mut merged = parent.clone();
            for (k, v) in local.iter() {
                merged.insert(k.clone(), v.clone());
            }
            clone_with_overrides(&inst, &merged)
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
