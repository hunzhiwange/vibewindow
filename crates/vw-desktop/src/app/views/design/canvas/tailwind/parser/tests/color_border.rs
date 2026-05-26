//! Tailwind 解析器的行为测试。
//!
//! 该测试模块覆盖颜色、边框等类名解析场景，帮助渲染层在重构时保持 CSS 语义转换稳定。

use super::*;

#[test]
fn test_parse_typography() {
    let style = TailwindParser::parse(
        "text-lg font-bold italic underline tracking-wide leading-relaxed text-center text-blue-500 uppercase opacity-75",
    );

    assert_eq!(style.font_size, Some(TailwindColors::TEXT_LG));
    assert_eq!(style.font_weight, Some(700));
    assert_eq!(style.font_style, Some("italic".to_string()));
    assert_eq!(style.text_decoration, Some("underline".to_string()));
    assert_eq!(style.letter_spacing, Some(0.4));
    assert_eq!(style.line_height, Some(1.625));
    assert_eq!(style.text_align, Some("center".to_string()));
    assert_eq!(style.text_color, Some(TailwindColors::BLUE_500));
    assert_eq!(style.text_transform, Some("uppercase".to_string()));
    assert_eq!(style.opacity, Some(0.75));
}

#[test]
fn test_parse_opacity_utilities() {
    assert_eq!(TailwindParser::parse("opacity-0").opacity, Some(0.0));
    assert_eq!(TailwindParser::parse("opacity-25").opacity, Some(0.25));
    assert_eq!(TailwindParser::parse("opacity-50").opacity, Some(0.5));
    assert_eq!(TailwindParser::parse("opacity-100").opacity, Some(1.0));
}

#[test]
fn test_parse_translate_utilities() {
    let style = TailwindParser::parse("translate-x-4 -translate-y-2");

    assert_eq!(style.translate_x, Some(16.0));
    assert_eq!(style.translate_y, Some(-8.0));
}

#[test]
fn test_parse_effect_utilities() {
    let style = TailwindParser::parse(
        "shadow-md outline-2 outline-dashed -outline-offset-1 outline-blue-500",
    );

    assert_eq!(style.shadow_offset_x, Some(0.0));
    assert_eq!(style.shadow_offset_y, Some(4.0));
    assert_eq!(style.shadow_spread, Some(3.0));
    assert_eq!(style.outline_width, Some(2.0));
    assert_eq!(style.outline_style, Some("dashed".to_string()));
    assert_eq!(style.outline_offset, Some(-1.0));
    assert_eq!(style.outline_color, Some(TailwindColors::BLUE_500));
}

#[test]
fn test_parse_shared_named_color_utilities() {
    for token in TailwindColors::TEXT_COLOR_TOKENS {
        let style = TailwindParser::parse(&format!("text-{}", token));
        assert_eq!(style.text_color, TailwindColors::resolve_text_color(token));
    }

    for token in TailwindColors::BACKGROUND_COLOR_TOKENS {
        let style = TailwindParser::parse(&format!("bg-{}", token));
        assert_eq!(style.background_color, TailwindColors::resolve_background_color(token));
    }

    for token in TailwindColors::BORDER_COLOR_TOKENS {
        let style = TailwindParser::parse(&format!("border-{}", token));
        assert_eq!(style.border_color, TailwindColors::resolve_border_color(token));
    }
}
