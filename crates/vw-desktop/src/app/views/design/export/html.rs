//! 设计导出模块，负责把内部设计文档转换为 HTML、SVG 或共享的 CSS/尺寸表示。

use super::util::{parse_fills_to_css, parse_size_to_css, parse_size_val_opt, process_color_value,
    resolve_variable_value};
use crate::app::views::design::models::{DesignDoc, DesignElement};

/// 根据设计文档或元素生成外部表示。
///
/// 返回生成后的内容；找不到指定元素时返回 `None`，避免导出不完整结果。
pub fn generate_html(doc: &DesignDoc) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<style>\n");
    html.push_str("body { margin: 0; padding: 0; background: #f0f0f0; overflow: auto; }\n");

    let (width, height, offset_x, offset_y) =
        if let Some((min_x, min_y, max_x, max_y)) = doc.get_bounds() {
            let padding = 50.0;
            let w = (max_x - min_x) + padding * 2.0;
            let h = (max_y - min_y) + padding * 2.0;
            (w.max(800.0), h.max(600.0), min_x - padding, min_y - padding)
        } else {
            (1920.0, 1080.0, 0.0, 0.0)
        };

    html.push_str(&format!(".artboard {{ position: relative; background: white; overflow: visible; width: {}px; height: {}px; }}\n", width, height));
    html.push_str(".element { position: absolute; box-sizing: border-box; }\n");

    html.push_str(":root {\n");
    let current_mode = doc.theme.as_ref().map(|t| t.mode.as_str());

    for (name, def) in &doc.variables {
        if let Some(val) = resolve_variable_value(def, current_mode) {
            let var_name =
                if name.starts_with("--") { name.clone() } else { format!("--{}", name) };
            html.push_str(&format!("  {}: {};\n", var_name, process_color_value(&val)));
        }
    }
    html.push_str("}\n");

    html.push_str("</style>\n");
    html.push_str(
        "<script src=\"https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4\"></script>\n",
    );
    html.push_str("</head>\n<body>\n");
    html.push_str("<div id=\"canvas-root\" class=\"artboard\">\n");

    for child in &doc.children {
        render_element(&mut html, child, doc, offset_x, offset_y);
    }

    html.push_str("</div>\n");
    html.push_str("</body>\n</html>");

    html
}

/// 根据设计文档或元素生成外部表示。
///
/// 返回生成后的内容；找不到指定元素时返回 `None`，避免导出不完整结果。
pub fn generate_element_html(doc: &DesignDoc, element_id: &str) -> Option<String> {
    let element = doc.find_element(element_id)?;

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<style>\n");
    html.push_str("body { margin: 0; padding: 0; background: #f0f0f0; overflow: auto; }\n");

    let w = parse_size_val_opt(&element.width);
    let h = parse_size_val_opt(&element.height);
    let padding = 50.0;

    let width = if w > 0.0 { w + padding * 2.0 } else { 800.0 };
    let height = if h > 0.0 { h + padding * 2.0 } else { 600.0 };

    let offset_x = element.x - padding;
    let offset_y = element.y - padding;

    html.push_str(&format!(".artboard {{ position: relative; background: white; overflow: visible; width: {}px; height: {}px; }}\n", width, height));
    html.push_str(".element { position: absolute; box-sizing: border-box; }\n");

    html.push_str(":root {\n");
    let current_mode = doc.theme.as_ref().map(|t| t.mode.as_str());

    for (name, def) in &doc.variables {
        if let Some(val) = resolve_variable_value(def, current_mode) {
            let var_name =
                if name.starts_with("--") { name.clone() } else { format!("--{}", name) };
            html.push_str(&format!("  {}: {};\n", var_name, process_color_value(&val)));
        }
    }
    html.push_str("}\n");

    html.push_str("</style>\n");
    html.push_str(
        "<script src=\"https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4\"></script>\n",
    );
    html.push_str("</head>\n<body>\n");
    html.push_str("<div id=\"canvas-root\" class=\"artboard\">\n");

    render_element(&mut html, element, doc, offset_x, offset_y);

    html.push_str("</div>\n");
    html.push_str("</body>\n</html>");

    Some(html)
}

fn render_element(
    html: &mut String,
    element: &DesignElement,
    doc: &DesignDoc,
    offset_x: f32,
    offset_y: f32,
) {
    let style = generate_style(element, doc, offset_x, offset_y);
    let classes = element.class.as_deref().unwrap_or("");

    html.push_str(&format!("<div class=\"element {}\" style=\"{}\">\n", classes, style));

    if element.kind.eq_ignore_ascii_case("text") || element.content.is_some() {
        if let Some(text) = &element.content {
            html.push_str(text);
        } else if element.kind.eq_ignore_ascii_case("text") {
            html.push_str(&element.name.clone().unwrap_or_default());
        }
    }

    for child in &element.children {
        render_element(html, child, doc, 0.0, 0.0);
    }

    html.push_str("</div>\n");
}

fn generate_style(
    element: &DesignElement,
    _doc: &DesignDoc,
    offset_x: f32,
    offset_y: f32,
) -> String {
    let mut style = String::new();

    let x = element.x - offset_x;
    let y = element.y - offset_y;
    let w = parse_size_to_css(&element.width);
    let h = parse_size_to_css(&element.height);

    style.push_str(&format!("left: {}px; top: {}px; width: {}; height: {}; ", x, y, w, h));

    let fills = parse_fills_to_css(&element.fill);
    if !fills.is_empty() {
        if element.kind.eq_ignore_ascii_case("text") {
            if fills.contains("gradient") {
                style.push_str(&format!("background: {}; -webkit-background-clip: text; background-clip: text; color: transparent; ", fills));
            } else {
                style.push_str(&format!("color: {}; ", fills));
            }
        } else {
            style.push_str(&format!("background: {}; ", fills));
        }
    }

    if let Some(stroke) = &element.stroke {
        let color = stroke.fill.as_deref().unwrap_or("black");
        style.push_str(&format!("border: 1px solid {}; ", process_color_value(color)));
    }

    if element.kind.eq_ignore_ascii_case("text") || element.kind.eq_ignore_ascii_case("typography")
    {
        if let Some(v) = &element.text_align_vertical {
            let ai = match v.as_str() {
                "center" | "middle" => "center",
                "bottom" | "end" => "flex-end",
                _ => "flex-start",
            };
            style.push_str(&format!("display: flex; align-items: {}; ", ai));
        }
        if let Some(align) = &element.text_align {
            let jc = match align.as_str() {
                "center" => "center",
                "right" => "flex-end",
                _ => "flex-start",
            };
            style.push_str(&format!("justify-content: {}; ", jc));
        }
        if let Some(family) = &element.font_family {
            style.push_str(&format!("font-family: '{}'; ", family));
        }
        if let Some(weight) = &element.font_weight
            && let Some(w) = weight.as_str() {
                style.push_str(&format!("font-weight: {}; ", w));
            }
        if let Some(s) = &element.font_style {
            style.push_str(&format!("font-style: {}; ", s));
        }
        if let Some(d) = &element.text_decoration {
            style.push_str(&format!("text-decoration: {}; ", d));
        }
        if let Some(align) = &element.text_align {
            style.push_str(&format!("text-align: {}; ", align));
        }
        if let Some(line_height) = &element.line_height {
            if let Some(n) = line_height.as_f64() {
                style.push_str(&format!("line-height: {}; ", n));
            } else if let Some(s) = line_height.as_str() {
                style.push_str(&format!("line-height: {}; ", s));
            }
        }
    }

    style
}

#[cfg(test)]
#[path = "html_tests.rs"]
mod html_tests;
