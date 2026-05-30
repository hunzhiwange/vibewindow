//! Tailwind utility 解析模块，负责把受支持的类名映射到设计画布的结构化样式字段。

use super::super::TailwindColors;
use super::types::ParsedStyle;
use iced::Color;

fn parse_spacing_scale_value(token: &str) -> Option<f32> {
    match token {
        "0" => Some(0.0),
        "px" => Some(1.0),
        "1" => Some(4.0),
        "2" => Some(8.0),
        "3" => Some(12.0),
        "4" => Some(16.0),
        "5" => Some(20.0),
        "6" => Some(TailwindColors::SPACING_6),
        "7" => Some(TailwindColors::SPACING_7),
        "8" => Some(TailwindColors::SPACING_8),
        "10" => Some(TailwindColors::SPACING_10),
        "12" => Some(TailwindColors::SPACING_12),
        "16" => Some(TailwindColors::SPACING_16),
        "20" => Some(80.0),
        "24" => Some(96.0),
        "32" => Some(128.0),
        "40" => Some(160.0),
        "48" => Some(192.0),
        "56" => Some(224.0),
        "64" => Some(256.0),
        _ => None,
    }
}

fn parse_prefixed_spacing_value(class_name: &str, prefix: &str) -> Option<f32> {
    class_name
        .strip_prefix(prefix)
        .and_then(|suffix| suffix.strip_prefix('-'))
        .and_then(parse_spacing_scale_value)
}

fn parse_arbitrary_bracket_value<'a>(class_name: &'a str, prefix: &str) -> Option<&'a str> {
    class_name
        .strip_prefix(prefix)
        .and_then(|suffix| suffix.strip_prefix("-["))
        .and_then(|suffix| suffix.strip_suffix(']'))
}

fn parse_arbitrary_px_value(class_name: &str, prefix: &str) -> Option<f32> {
    parse_arbitrary_bracket_value(class_name, prefix)?.strip_suffix("px")?.parse::<f32>().ok()
}

fn parse_arbitrary_hex_color(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::from_rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::from_rgba(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ))
        }
        _ => None,
    }
}

fn parse_spacing_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    if let Some(value) = parse_prefixed_spacing_value(class_name, "p") {
        style.padding = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "px") {
        style.padding_left = Some(value);
        style.padding_right = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "py") {
        style.padding_top = Some(value);
        style.padding_bottom = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "pt") {
        style.padding_top = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "pr") {
        style.padding_right = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "pb") {
        style.padding_bottom = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "pl") {
        style.padding_left = Some(value);
        return true;
    }

    if let Some(value) = parse_prefixed_spacing_value(class_name, "m") {
        style.margin = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "mx") {
        style.margin_left = Some(value);
        style.margin_right = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "my") {
        style.margin_top = Some(value);
        style.margin_bottom = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "mt") {
        style.margin_top = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "mr") {
        style.margin_right = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "mb") {
        style.margin_bottom = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "ml") {
        style.margin_left = Some(value);
        return true;
    }

    if let Some(value) = parse_prefixed_spacing_value(class_name, "gap") {
        style.gap_x = Some(value);
        style.gap_y = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "gap-x") {
        style.gap_x = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "gap-y") {
        style.gap_y = Some(value);
        return true;
    }

    false
}

fn parse_offset_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    if let Some(value) = parse_prefixed_spacing_value(class_name, "top") {
        style.top = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "right") {
        style.right = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "bottom") {
        style.bottom = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "left") {
        style.left = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "-top") {
        style.top = Some(-value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "-right") {
        style.right = Some(-value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "-bottom") {
        style.bottom = Some(-value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "-left") {
        style.left = Some(-value);
        return true;
    }

    false
}

fn parse_size_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    match class_name {
        "w-full" | "w-screen" => {
            style.width = Some(-1.0);
            return true;
        }
        "h-full" | "h-screen" => {
            style.height = Some(-1.0);
            return true;
        }
        "w-auto" => {
            style.width = None;
            return true;
        }
        "h-auto" => {
            style.height = None;
            return true;
        }
        "max-w-screen-sm" => {
            style.max_width = Some(640.0);
            return true;
        }
        "max-w-screen-md" => {
            style.max_width = Some(768.0);
            return true;
        }
        "max-w-screen-lg" => {
            style.max_width = Some(1024.0);
            return true;
        }
        "max-w-screen-xl" => {
            style.max_width = Some(1280.0);
            return true;
        }
        "max-w-screen-2xl" => {
            style.max_width = Some(1536.0);
            return true;
        }
        _ => {}
    }

    if let Some(value) = parse_prefixed_spacing_value(class_name, "w") {
        style.width = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "h") {
        style.height = Some(value);
        return true;
    }

    false
}

fn parse_flex_item_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    match class_name {
        "grow" => {
            style.flex_grow = Some(1.0);
            return true;
        }
        "grow-0" => {
            style.flex_grow = Some(0.0);
            return true;
        }
        "shrink" => {
            style.flex_shrink = Some(1.0);
            return true;
        }
        "shrink-0" => {
            style.flex_shrink = Some(0.0);
            return true;
        }
        "flex-1" => {
            style.flex_grow = Some(1.0);
            style.flex_shrink = Some(1.0);
            style.flex_basis = Some(0.0);
            return true;
        }
        "flex-auto" => {
            style.flex_grow = Some(1.0);
            style.flex_shrink = Some(1.0);
            return true;
        }
        "flex-none" => {
            style.flex_grow = Some(0.0);
            style.flex_shrink = Some(0.0);
            return true;
        }
        _ => {}
    }

    if let Some(value) =
        class_name.strip_prefix("grow-").and_then(|token| token.parse::<f32>().ok())
    {
        style.flex_grow = Some(value);
        return true;
    }
    if let Some(value) =
        class_name.strip_prefix("shrink-").and_then(|token| token.parse::<f32>().ok())
    {
        style.flex_shrink = Some(value);
        return true;
    }
    if let Some(value) = parse_prefixed_spacing_value(class_name, "basis") {
        style.flex_basis = Some(value);
        return true;
    }

    false
}

fn parse_color_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    if let Some(token) = class_name.strip_prefix("bg-")
        && let Some(color) = TailwindColors::resolve_background_color(token)
    {
        style.background_color = Some(color);
        return true;
    }

    if let Some(token) = class_name.strip_prefix("text-")
        && let Some(color) = TailwindColors::resolve_text_color(token)
    {
        style.text_color = Some(color);
        return true;
    }

    if let Some(token) = class_name.strip_prefix("border-")
        && let Some(color) = TailwindColors::resolve_border_color(token)
    {
        style.border_color = Some(color);
        return true;
    }

    false
}

fn parse_opacity_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    let Some(token) = class_name.strip_prefix("opacity-") else {
        return false;
    };

    let Some(value) = token.parse::<f32>().ok() else {
        return false;
    };

    style.opacity = Some((value / 100.0).clamp(0.0, 1.0));
    true
}

fn parse_translate_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    if let Some(value) = parse_prefixed_spacing_value(class_name, "translate-x") {
        style.translate_x = Some(value);
        return true;
    }

    if let Some(token) = class_name.strip_prefix("-translate-x-")
        && let Some(value) = parse_spacing_scale_value(token)
    {
        style.translate_x = Some(-value);
        return true;
    }

    if let Some(value) = parse_prefixed_spacing_value(class_name, "translate-y") {
        style.translate_y = Some(value);
        return true;
    }

    if let Some(token) = class_name.strip_prefix("-translate-y-")
        && let Some(value) = parse_spacing_scale_value(token)
    {
        style.translate_y = Some(-value);
        return true;
    }

    false
}

fn set_shadow_preset(style: &mut ParsedStyle, offset_y: f32, spread: f32, alpha: f32) {
    style.shadow_color = Some(Color::from_rgba(0.0, 0.0, 0.0, alpha.clamp(0.0, 1.0)));
    style.shadow_offset_x = Some(0.0);
    style.shadow_offset_y = Some(offset_y);
    style.shadow_spread = Some(spread.max(0.0));
}

fn parse_effect_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    match class_name {
        "shadow-none" => {
            style.shadow_color = None;
            style.shadow_offset_x = None;
            style.shadow_offset_y = None;
            style.shadow_spread = None;
            return true;
        }
        "shadow-sm" => {
            set_shadow_preset(style, 1.0, 1.0, 0.12);
            return true;
        }
        "shadow" => {
            set_shadow_preset(style, 2.0, 2.0, 0.14);
            return true;
        }
        "shadow-md" => {
            set_shadow_preset(style, 4.0, 3.0, 0.16);
            return true;
        }
        "shadow-lg" => {
            set_shadow_preset(style, 8.0, 4.0, 0.18);
            return true;
        }
        "outline" | "outline-1" => {
            style.outline_width = Some(1.0);
            return true;
        }
        "outline-0" => {
            style.outline_width = Some(0.0);
            return true;
        }
        "outline-2" => {
            style.outline_width = Some(2.0);
            return true;
        }
        "outline-4" => {
            style.outline_width = Some(4.0);
            return true;
        }
        "outline-8" => {
            style.outline_width = Some(8.0);
            return true;
        }
        "outline-none" => {
            style.outline_width = Some(0.0);
            style.outline_style = Some("none".to_string());
            return true;
        }
        "outline-solid" => {
            style.outline_style = Some("solid".to_string());
            return true;
        }
        "outline-dashed" => {
            style.outline_style = Some("dashed".to_string());
            return true;
        }
        "outline-dotted" => {
            style.outline_style = Some("dotted".to_string());
            return true;
        }
        "outline-double" => {
            style.outline_style = Some("double".to_string());
            return true;
        }
        _ => {}
    }

    if let Some(value) =
        class_name.strip_prefix("outline-offset-").and_then(|token| token.parse::<f32>().ok())
    {
        style.outline_offset = Some(value);
        return true;
    }

    if let Some(value) =
        class_name.strip_prefix("-outline-offset-").and_then(|token| token.parse::<f32>().ok())
    {
        style.outline_offset = Some(-value);
        return true;
    }

    if let Some(token) = class_name.strip_prefix("outline-")
        && let Some(color) = TailwindColors::resolve_border_color(token)
    {
        style.outline_color = Some(color);
        return true;
    }

    false
}

fn parse_arbitrary_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    if let Some(value) = parse_arbitrary_px_value(class_name, "w") {
        style.width = Some(value);
        return true;
    }
    if let Some(value) = parse_arbitrary_px_value(class_name, "h") {
        style.height = Some(value);
        return true;
    }
    if let Some(value) = parse_arbitrary_px_value(class_name, "top") {
        style.top = Some(value);
        return true;
    }
    if let Some(value) = parse_arbitrary_px_value(class_name, "right") {
        style.right = Some(value);
        return true;
    }
    if let Some(value) = parse_arbitrary_px_value(class_name, "bottom") {
        style.bottom = Some(value);
        return true;
    }
    if let Some(value) = parse_arbitrary_px_value(class_name, "left") {
        style.left = Some(value);
        return true;
    }
    if let Some(value) = parse_arbitrary_bracket_value(class_name, "bg")
        && let Some(color) = parse_arbitrary_hex_color(value)
    {
        style.background_color = Some(color);
        return true;
    }

    false
}

/// 执行 apply_supported_utility 对应的设计辅助逻辑。
///
/// 返回值直接交给调用方继续渲染、导入或属性更新。
pub(super) fn apply_supported_utility(style: &mut ParsedStyle, class_name: &str) -> bool {
    if parse_spacing_utility(style, class_name)
        || parse_offset_utility(style, class_name)
        || parse_size_utility(style, class_name)
        || parse_flex_item_utility(style, class_name)
        || parse_color_utility(style, class_name)
        || parse_opacity_utility(style, class_name)
        || parse_translate_utility(style, class_name)
        || parse_effect_utility(style, class_name)
        || parse_arbitrary_utility(style, class_name)
    {
        return true;
    }

    match class_name {
        "border-solid" => style.border_style = Some("solid".to_string()),
        "border-dashed" => style.border_style = Some("dashed".to_string()),
        "border-dotted" => style.border_style = Some("dotted".to_string()),
        "border-double" => style.border_style = Some("double".to_string()),
        "border-hidden" => style.border_style = Some("hidden".to_string()),
        "border-none" => style.border_style = Some("none".to_string()),

        "flex" => style.display = Some("flex".to_string()),
        "flex-row" => style.flex_direction = Some("row".to_string()),
        "flex-row-reverse" => style.flex_direction = Some("row-reverse".to_string()),
        "flex-col" => style.flex_direction = Some("column".to_string()),
        "flex-col-reverse" => style.flex_direction = Some("column-reverse".to_string()),
        "items-center" => style.align_items = Some("center".to_string()),
        "items-start" => style.align_items = Some("flex-start".to_string()),
        "items-end" => style.align_items = Some("flex-end".to_string()),
        "justify-center" => style.justify_content = Some("center".to_string()),
        "justify-between" => style.justify_content = Some("space-between".to_string()),
        "justify-start" => style.justify_content = Some("flex-start".to_string()),
        "justify-end" => style.justify_content = Some("flex-end".to_string()),
        "relative" => style.position = Some("relative".to_string()),
        "absolute" => style.position = Some("absolute".to_string()),
        "hidden" => style.display = Some("none".to_string()),
        "block" => style.display = Some("block".to_string()),
        "inline-block" => style.display = Some("inline-block".to_string()),
        "inline" => style.display = Some("inline".to_string()),
        "inline-flex" => style.display = Some("inline-flex".to_string()),
        "grid" => style.display = Some("grid".to_string()),

        "grid-cols-1" => style.grid_cols = Some(1),
        "grid-cols-2" => style.grid_cols = Some(2),
        "grid-cols-3" => style.grid_cols = Some(3),
        "grid-cols-4" => style.grid_cols = Some(4),
        "grid-cols-5" => style.grid_cols = Some(5),
        "grid-cols-6" => style.grid_cols = Some(6),
        "grid-cols-12" => style.grid_cols = Some(12),

        "mx-auto" => {
            style.margin_left = Some(-1.0);
            style.margin_right = Some(-1.0);
        }
        "my-auto" => {
            style.margin_top = Some(-1.0);
            style.margin_bottom = Some(-1.0);
        }

        "text-sm" => style.font_size = Some(TailwindColors::TEXT_SM),
        "text-base" => style.font_size = Some(TailwindColors::TEXT_BASE),
        "text-lg" => style.font_size = Some(TailwindColors::TEXT_LG),
        "text-xl" => style.font_size = Some(TailwindColors::TEXT_XL),
        "text-2xl" => style.font_size = Some(TailwindColors::TEXT_2XL),
        "text-3xl" => style.font_size = Some(TailwindColors::TEXT_3XL),

        "font-bold" => style.font_weight = Some(700),
        "font-semibold" => style.font_weight = Some(600),
        "font-normal" => style.font_weight = Some(400),
        "font-light" => style.font_weight = Some(300),

        "text-center" => style.text_align = Some("center".to_string()),
        "text-start" | "text-left" => style.text_align = Some("left".to_string()),
        "text-end" | "text-right" => style.text_align = Some("right".to_string()),
        "text-justify" => style.text_align = Some("justify".to_string()),

        "italic" => style.font_style = Some("italic".to_string()),
        "not-italic" => style.font_style = Some("normal".to_string()),

        "underline" => style.text_decoration = Some("underline".to_string()),
        "line-through" => style.text_decoration = Some("line-through".to_string()),
        "no-underline" => style.text_decoration = Some("none".to_string()),

        "uppercase" => style.text_transform = Some("uppercase".to_string()),
        "lowercase" => style.text_transform = Some("lowercase".to_string()),
        "capitalize" => style.text_transform = Some("capitalize".to_string()),

        "tracking-tighter" => style.letter_spacing = Some(-0.8),
        "tracking-tight" => style.letter_spacing = Some(-0.4),
        "tracking-normal" => style.letter_spacing = Some(0.0),
        "tracking-wide" => style.letter_spacing = Some(0.4),
        "tracking-wider" => style.letter_spacing = Some(0.8),
        "tracking-widest" => style.letter_spacing = Some(1.6),

        "leading-none" => style.line_height = Some(1.0),
        "leading-tight" => style.line_height = Some(1.25),
        "leading-snug" => style.line_height = Some(1.375),
        "leading-normal" => style.line_height = Some(1.5),
        "leading-relaxed" => style.line_height = Some(1.625),
        "leading-loose" => style.line_height = Some(2.0),

        "rounded" => style.border_radius = Some(TailwindColors::ROUNDED_BASE),
        "rounded-xs" => style.border_radius = Some(TailwindColors::ROUNDED_XS),
        "rounded-sm" => style.border_radius = Some(TailwindColors::ROUNDED_SM),
        "rounded-md" => style.border_radius = Some(TailwindColors::ROUNDED_MD),
        "rounded-lg" => style.border_radius = Some(TailwindColors::ROUNDED_LG),
        "rounded-xl" => style.border_radius = Some(TailwindColors::ROUNDED_XL),
        "rounded-2xl" => style.border_radius = Some(TailwindColors::ROUNDED_2XL),
        "rounded-3xl" => style.border_radius = Some(TailwindColors::ROUNDED_3XL),
        "rounded-4xl" => style.border_radius = Some(TailwindColors::ROUNDED_4XL),
        "rounded-none" => style.border_radius = Some(0.0),
        "rounded-full" => style.border_radius = Some(TailwindColors::ROUNDED_FULL),

        "border" => style.border_width = Some(1.0),
        "border-0" => style.border_width = Some(0.0),
        "border-2" => style.border_width = Some(2.0),
        "border-4" => style.border_width = Some(4.0),
        "border-8" => style.border_width = Some(8.0),
        "border-t" => style.border_top_width = Some(1.0),
        "border-r" => style.border_right_width = Some(1.0),
        "border-b" => style.border_bottom_width = Some(1.0),
        "border-l" => style.border_left_width = Some(1.0),
        "border-t-0" => style.border_top_width = Some(0.0),
        "border-r-0" => style.border_right_width = Some(0.0),
        "border-b-0" => style.border_bottom_width = Some(0.0),
        "border-l-0" => style.border_left_width = Some(0.0),
        "border-t-2" => style.border_top_width = Some(2.0),
        "border-r-2" => style.border_right_width = Some(2.0),
        "border-b-2" => style.border_bottom_width = Some(2.0),
        "border-l-2" => style.border_left_width = Some(2.0),
        "border-t-4" => style.border_top_width = Some(4.0),
        "border-r-4" => style.border_right_width = Some(4.0),
        "border-b-4" => style.border_bottom_width = Some(4.0),
        "border-l-4" => style.border_left_width = Some(4.0),
        "border-t-8" => style.border_top_width = Some(8.0),
        "border-r-8" => style.border_right_width = Some(8.0),
        "border-b-8" => style.border_bottom_width = Some(8.0),
        "border-l-8" => style.border_left_width = Some(8.0),
        "border-x" => {
            style.border_left_width = Some(1.0);
            style.border_right_width = Some(1.0);
        }
        "border-y" => {
            style.border_top_width = Some(1.0);
            style.border_bottom_width = Some(1.0);
        }
        "border-x-2" => {
            style.border_left_width = Some(2.0);
            style.border_right_width = Some(2.0);
        }
        "border-y-2" => {
            style.border_top_width = Some(2.0);
            style.border_bottom_width = Some(2.0);
        }
        "border-x-4" => {
            style.border_left_width = Some(4.0);
            style.border_right_width = Some(4.0);
        }
        "border-y-4" => {
            style.border_top_width = Some(4.0);
            style.border_bottom_width = Some(4.0);
        }
        "border-s" => style.border_inline_start_width = Some(1.0),
        "border-e" => style.border_inline_end_width = Some(1.0),
        "border-s-2" => style.border_inline_start_width = Some(2.0),
        "border-e-2" => style.border_inline_end_width = Some(2.0),
        "border-s-4" => style.border_inline_start_width = Some(4.0),
        "border-e-4" => style.border_inline_end_width = Some(4.0),
        "divide-x" => style.divide_x_width = Some(1.0),
        "divide-y" => style.divide_y_width = Some(1.0),
        "divide-x-2" => style.divide_x_width = Some(2.0),
        "divide-y-2" => style.divide_y_width = Some(2.0),
        "divide-x-4" => style.divide_x_width = Some(4.0),
        "divide-y-4" => style.divide_y_width = Some(4.0),
        "divide-x-reverse" => style.divide_x_reverse = true,
        "divide-y-reverse" => style.divide_y_reverse = true,

        _ => return false,
    }

    true
}

#[cfg(test)]
#[path = "utilities_tests.rs"]
mod utilities_tests;
