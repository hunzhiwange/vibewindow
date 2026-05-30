//! 设计导出模块，负责把内部设计文档转换为 HTML、SVG 或共享的 CSS/尺寸表示。

use super::util::{
    color_to_hex, parse_fills_to_css, parse_size_val_opt, process_color_value,
    resolve_variable_value,
};
use crate::app::views::design::canvas::tailwind::{ParsedStyle, TailwindParser};
use crate::app::views::design::models::{DesignDoc, DesignElement};

/// 根据设计文档或元素生成外部表示。
///
/// 返回生成后的内容；找不到指定元素时返回 `None`，避免导出不完整结果。
pub fn generate_element_svg(doc: &DesignDoc, element_id: &str) -> Option<String> {
    let element = doc.find_element(element_id)?;

    let w = parse_size_val_opt(&element.width);
    let h = parse_size_val_opt(&element.height);

    let width = if w > 0.0 { w } else { 100.0 };
    let height = if h > 0.0 { h } else { 100.0 };

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        width, height, width, height
    ));
    svg.push('\n');

    svg.push_str("<style>\n");
    svg.push_str(":root {\n");
    let current_mode = doc.theme.as_ref().map(|t| t.mode.as_str());
    for (name, def) in &doc.variables {
        if let Some(val) = resolve_variable_value(def, current_mode) {
            let var_name =
                if name.starts_with("--") { name.clone() } else { format!("--{}", name) };
            svg.push_str(&format!("  {}: {};\n", var_name, process_color_value(&val)));
        }
    }
    svg.push_str("}\n");
    svg.push_str("</style>\n");

    render_element_svg(&mut svg, element, doc, element.x, element.y);

    svg.push_str("</svg>");

    Some(svg)
}

fn render_element_svg(
    svg: &mut String,
    element: &DesignElement,
    doc: &DesignDoc,
    offset_x: f32,
    offset_y: f32,
) {
    let _ = doc;
    let x = element.x - offset_x;
    let y = element.y - offset_y;
    let w_raw = parse_size_val_opt(&element.width);
    let h_raw = parse_size_val_opt(&element.height);

    let mut tw_style = ParsedStyle::default();
    if let Some(class) = &element.class {
        tw_style = TailwindParser::parse(class);
    }

    let w = if w_raw > 0.0 { w_raw } else { tw_style.width.unwrap_or(0.0) };
    let h = if h_raw > 0.0 { h_raw } else { tw_style.height.unwrap_or(0.0) };

    let fill = if let Some(_f) = &element.fill {
        parse_fills_to_css(&element.fill)
    } else if let Some(bg) = tw_style.background_color {
        color_to_hex(bg)
    } else {
        "none".to_string()
    };

    let stroke = if let Some(s) = &element.stroke {
        s.fill.clone().unwrap_or("none".to_string())
    } else {
        "none".to_string()
    };
    let stroke_width = if element.stroke.is_some() { 1.0 } else { 0.0 };

    let corner_radius = tw_style.border_radius.unwrap_or(0.0);

    svg.push_str(&format!(r#"<g transform="translate({}, {})">"#, x, y));

    if element.kind.eq_ignore_ascii_case("text") {
        let content = element.content.as_deref().unwrap_or(element.name.as_deref().unwrap_or(""));
        let font_size = parse_size_val_opt(&element.font_size);
        let fs = if font_size > 0.0 { font_size } else { tw_style.font_size.unwrap_or(16.0) };

        let text_color = if let Some(c) = &element.color {
            c.clone()
        } else if let Some(tc) = tw_style.text_color {
            color_to_hex(tc)
        } else {
            "black".to_string()
        };

        svg.push_str(&format!(
            r#"<text x="0" y="{}" font-size="{}" fill="{}" font-family="sans-serif">{}</text>"#,
            fs,
            fs,
            process_color_value(&text_color),
            content
        ));
    } else if fill != "none" || stroke != "none" {
        svg.push_str(&format!(r#"<rect width="{}" height="{}" fill="{}" stroke="{}" stroke-width="{}" rx="{}" ry="{}" />"#,
            w, h,
            process_color_value(&fill),
            process_color_value(&stroke),
            stroke_width,
            corner_radius, corner_radius
        ));
    }

    for child in &element.children {
        render_element_svg(svg, child, doc, 0.0, 0.0);
    }

    svg.push_str("</g>\n");
}

#[cfg(test)]
#[path = "svg_tests.rs"]
mod svg_tests;
