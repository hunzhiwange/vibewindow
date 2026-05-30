use super::types::ShadowSpec;
use crate::app::views::design::models::VariableDef;
use iced::Color;
use iced::Vector;
use iced::border::Radius;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use ttf_parser::Face;

static FONT_FAMILY_INTERN: Lazy<Mutex<HashMap<String, &'static str>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static FONT_FACE_CACHE: Lazy<Mutex<HashMap<&'static str, Face<'static>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static CHAR_WIDTH_CACHE: Lazy<Mutex<HashMap<(u32, &'static str, u32), f32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static TEXT_WIDTH_CACHE: Lazy<Mutex<HashMap<(String, &'static str, u32, u32), f32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn intern_font_family_name(family: &str) -> &'static str {
    let key = family.trim().to_string();
    if key.is_empty() {
        return "JetBrains Mono";
    }

    if let Ok(cache) = FONT_FAMILY_INTERN.lock()
        && let Some(v) = cache.get(&key)
    {
        return v;
    }

    let leaked: &'static str = Box::leak(key.clone().into_boxed_str());
    if let Ok(mut cache) = FONT_FAMILY_INTERN.lock() {
        cache.insert(key, leaked);
    }
    leaked
}

fn fallback_char_advance_px(ch: char, font_size_px: f32) -> f32 {
    if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 }
}

fn cached_font_face_static(font_family: &'static str) -> Option<Face<'static>> {
    if let Ok(cache) = FONT_FACE_CACHE.lock()
        && let Some(face) = cache.get(&font_family)
    {
        return Some(face.clone());
    }

    let bytes = embedded_font_bytes_for_family(font_family)?;
    let face = Face::parse(bytes, 0).ok()?;

    if let Ok(mut cache) = FONT_FACE_CACHE.lock() {
        cache.insert(font_family, face.clone());
    }

    Some(face)
}

#[allow(dead_code)]
fn cached_font_face(font_family: &str) -> Option<Face<'static>> {
    let family = intern_font_family_name(font_family);
    cached_font_face_static(family)
}

pub fn measure_font_vertical_metrics_with_font(
    font_family: &str,
    font_size_px: f32,
) -> Option<(f32, f32)> {
    if font_size_px <= 0.0 {
        return None;
    }
    let family = intern_font_family_name(font_family);
    let face = cached_font_face_static(family)?;
    let units_per_em = match face.units_per_em() {
        0 => 1000.0,
        n => n as f32,
    };
    let scale = font_size_px / units_per_em;
    let ascent = (face.ascender() as f32 * scale).max(0.0);
    let descent = ((face.descender() as f32).abs() * scale).max(0.0);
    Some((ascent, descent))
}

fn measure_char_width_with_font(ch: char, font_family: &str, font_size_px: f32) -> f32 {
    if font_size_px <= 0.0 {
        return 0.0;
    }

    let family = intern_font_family_name(font_family);
    let key = (ch as u32, family, font_size_px.to_bits());
    if let Ok(cache) = CHAR_WIDTH_CACHE.lock()
        && let Some(v) = cache.get(&key)
    {
        return *v;
    };

    let val = if let Some(face) = cached_font_face_static(family) {
        let units_per_em = match face.units_per_em() {
            0 => 1000.0,
            n => n as f32,
        };
        let scale = font_size_px / units_per_em;

        face.glyph_index(ch)
            .and_then(|gid| face.glyph_hor_advance(gid))
            .map(|a| a as f32 * scale)
            .unwrap_or_else(|| fallback_char_advance_px(ch, font_size_px))
    } else {
        fallback_char_advance_px(ch, font_size_px)
    };

    if let Ok(mut cache) = CHAR_WIDTH_CACHE.lock() {
        if cache.len() > 8192 {
            cache.clear();
        }
        cache.insert(key, val);
    }
    val
}

pub fn measure_text_width_with_font(
    text: &str,
    font_family: &str,
    font_size_px: f32,
    letter_spacing_px: f32,
) -> f32 {
    if font_size_px <= 0.0 {
        return 0.0;
    }

    let family = intern_font_family_name(font_family);
    let can_cache_text = text.len() <= 128;
    let text_key = if can_cache_text { Some(text.to_string()) } else { None };

    if let Some(text_key) = text_key.as_ref() {
        let key = (text_key.clone(), family, font_size_px.to_bits(), letter_spacing_px.to_bits());
        if let Ok(cache) = TEXT_WIDTH_CACHE.lock()
            && let Some(v) = cache.get(&key)
        {
            return *v;
        }
    }

    let val = if let Some(face) = cached_font_face_static(family) {
        let units_per_em = match face.units_per_em() {
            0 => 1000.0,
            n => n as f32,
        };
        let scale = font_size_px / units_per_em;

        let mut w = 0.0;
        let mut first = true;
        for ch in text.chars() {
            let advance = face
                .glyph_index(ch)
                .and_then(|gid| face.glyph_hor_advance(gid))
                .map(|a| a as f32 * scale)
                .unwrap_or_else(|| fallback_char_advance_px(ch, font_size_px));
            if !first {
                w += letter_spacing_px.max(0.0);
            }
            first = false;
            w += advance;
        }
        w
    } else {
        text.chars().map(|ch| fallback_char_advance_px(ch, font_size_px)).sum::<f32>()
            + letter_spacing_px.max(0.0) * (text.chars().count().saturating_sub(1) as f32)
    };

    if let Some(text_key) = text_key {
        let key = (text_key, family, font_size_px.to_bits(), letter_spacing_px.to_bits());
        if let Ok(mut cache) = TEXT_WIDTH_CACHE.lock() {
            if cache.len() > 4096 {
                cache.clear();
            }
            cache.insert(key, val);
        }
    }
    val
}

pub fn wrap_text_lines_with_font(
    content: &str,
    max_width: f32,
    font_family: &str,
    font_size_px: f32,
    letter_spacing_px: f32,
) -> Vec<String> {
    if max_width <= 0.0 || font_size_px <= 0.0 {
        return Vec::new();
    }

    let eps = (font_size_px * 0.03).max(0.5).min(1.0);
    let space_w = measure_text_width_with_font(" ", font_family, font_size_px, 0.0);

    let mut out: Vec<String> = Vec::new();
    for raw_line in content.lines() {
        if raw_line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut current_w = 0.0;

        for word in raw_line.split_whitespace() {
            let word_w = measure_text_width_with_font(word, font_family, font_size_px, 0.0);

            if current.is_empty() && word_w <= max_width + eps {
                current.push_str(word);
                current_w = word_w;
                continue;
            }

            if !current.is_empty() && current_w + space_w + word_w <= max_width + eps {
                current.push(' ');
                current.push_str(word);
                current_w += space_w + word_w;
                continue;
            }

            if !current.is_empty() {
                out.push(current);
            }
            current = String::new();
            current_w = 0.0;

            let mut chunk = String::new();
            let mut chunk_w = 0.0;
            for ch in word.chars() {
                let ch_w = measure_char_width_with_font(ch, font_family, font_size_px);
                let extra = if chunk.is_empty() { 0.0 } else { letter_spacing_px.max(0.0) };
                if chunk_w + extra + ch_w > max_width + eps && !chunk.is_empty() {
                    out.push(chunk);
                    chunk = String::new();
                    chunk_w = 0.0;
                }
                if !chunk.is_empty() {
                    chunk_w += letter_spacing_px.max(0.0);
                }
                chunk.push(ch);
                chunk_w += ch_w;
            }
            if !chunk.is_empty() {
                current = chunk;
                current_w = chunk_w;
            }
        }

        if !current.is_empty() {
            out.push(current);
        }
    }

    out
}

pub fn resolve_variable<'a>(
    name: &str,
    variables: &'a HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<&'a String> {
    if let Some(def) = variables.get(name) {
        // Try to find a value for the current theme mode
        let val = if let Some(mode) = theme_mode {
            def.value.iter().find(|v| v.theme.as_ref().map(|t| t.mode == mode).unwrap_or(false))
        } else {
            None
        };

        // Fallback to default (no theme or first one)
        let val = val
            .or_else(|| def.value.iter().find(|v| v.theme.is_none()))
            .or_else(|| def.value.first());

        if let Some(v) = val {
            return Some(&v.value);
        }
    }
    None
}

pub fn parse_size(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<f32> {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f as f32),
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_size(
                        &Some(serde_json::Value::String(val_str.clone())),
                        variables,
                        theme_mode,
                    );
                }
            }

            if s.starts_with("fill_container") { None } else { s.parse::<f32>().ok() }
        }
        _ => None,
    }
}

pub fn parse_fill(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Color {
    match v {
        Some(serde_json::Value::Array(arr)) => {
            for item in arr {
                if let Some(color) = parse_fill_value(item, variables, theme_mode) {
                    return color;
                }
            }
            Color::TRANSPARENT
        }
        Some(serde_json::Value::String(s)) => parse_color(s, variables, theme_mode),
        Some(serde_json::Value::Object(map)) => {
            if let Some(serde_json::Value::Bool(false)) = map.get("enabled") {
                return Color::TRANSPARENT;
            }
            let mut color = if let Some(serde_json::Value::String(c)) = map.get("color") {
                parse_color(c, variables, theme_mode)
            } else {
                Color::TRANSPARENT
            };
            if let Some(serde_json::Value::Number(op)) =
                map.get("opacity").or_else(|| map.get("alpha"))
                && let Some(op_f) = op.as_f64()
            {
                color.a *= op_f as f32;
            }
            color
        }
        _ => Color::TRANSPARENT,
    }
}

pub fn parse_fills(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Vec<Color> {
    match v {
        Some(serde_json::Value::Array(arr)) => {
            arr.iter().filter_map(|item| parse_fill_value(item, variables, theme_mode)).collect()
        }
        Some(_) => {
            if let Some(val) = v {
                if let Some(color) = parse_fill_value(val, variables, theme_mode) {
                    vec![color]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
        None => vec![],
    }
}

fn parse_fill_value(
    value: &serde_json::Value,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<Color> {
    match value {
        serde_json::Value::String(s) => Some(parse_color(s, variables, theme_mode)),
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Bool(false)) = map.get("enabled") {
                return None;
            }
            if let Some(serde_json::Value::String(c)) = map.get("color") {
                return Some(parse_color(c, variables, theme_mode));
            }
            if let Some(serde_json::Value::Array(colors)) = map.get("colors") {
                for color in colors {
                    if let serde_json::Value::String(s) = color {
                        return Some(parse_color(s, variables, theme_mode));
                    }
                    if let serde_json::Value::Object(color_map) = color
                        && let Some(serde_json::Value::String(c)) = color_map.get("color")
                    {
                        return Some(parse_color(c, variables, theme_mode));
                    }
                }
            }
            None
        }
        _ => None,
    }
}

pub fn parse_radius(
    v: &Option<serde_json::Value>,
    h: f32,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> f32 {
    match v {
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_radius(
                        &Some(serde_json::Value::String(val_str.clone())),
                        h,
                        variables,
                        theme_mode,
                    );
                }
            } else if s.ends_with('%') {
                let pct = s.trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
                return h * (pct / 100.0);
            } else {
                let cleaned = s.trim().trim_end_matches("px").trim();
                return cleaned.parse::<f32>().unwrap_or(0.0);
            }
            0.0
        }
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f as f32).unwrap_or(0.0),
        Some(serde_json::Value::Array(arr)) => {
            if let Some(first) = arr.first() {
                parse_radius(&Some(first.clone()), h, variables, theme_mode)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

pub fn parse_corner_radii(
    v: &Option<serde_json::Value>,
    w: f32,
    h: f32,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Radius {
    let max_r = (w.min(h) / 2.0).max(0.0);
    let clamp = |r: f32| r.clamp(0.0, max_r);

    match v {
        Some(serde_json::Value::Array(arr)) => match arr.len() {
            0 => Radius::from(0.0),
            1 => clamp(parse_radius(&Some(arr[0].clone()), h, variables, theme_mode)).into(),
            2 => {
                let a = clamp(parse_radius(&Some(arr[0].clone()), h, variables, theme_mode));
                let b = clamp(parse_radius(&Some(arr[1].clone()), h, variables, theme_mode));
                Radius { top_left: a, top_right: b, bottom_right: a, bottom_left: b }
            }
            3 => {
                let tl = clamp(parse_radius(&Some(arr[0].clone()), h, variables, theme_mode));
                let tr = clamp(parse_radius(&Some(arr[1].clone()), h, variables, theme_mode));
                let br = clamp(parse_radius(&Some(arr[2].clone()), h, variables, theme_mode));
                Radius { top_left: tl, top_right: tr, bottom_right: br, bottom_left: tr }
            }
            _ => {
                let tl = clamp(parse_radius(&Some(arr[0].clone()), h, variables, theme_mode));
                let tr = clamp(parse_radius(&Some(arr[1].clone()), h, variables, theme_mode));
                let br = clamp(parse_radius(&Some(arr[2].clone()), h, variables, theme_mode));
                let bl = clamp(parse_radius(&Some(arr[3].clone()), h, variables, theme_mode));
                Radius { top_left: tl, top_right: tr, bottom_right: br, bottom_left: bl }
            }
        },
        _ => clamp(parse_radius(v, h, variables, theme_mode)).into(),
    }
}

pub fn parse_thickness(
    v: &Option<&serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> f32 {
    match v {
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    // Need to convert string to Value to recurse
                    let val = serde_json::Value::String(val_str.clone());
                    return parse_thickness(&Some(&val), variables, theme_mode);
                }
            }
            1.0
        }
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f as f32).unwrap_or(1.0),
        Some(serde_json::Value::Object(map)) => {
            let keys = ["top", "bottom", "left", "right"];
            for k in keys {
                if let Some(serde_json::Value::Number(n)) = map.get(k) {
                    return n.as_f64().map(|f| f as f32).unwrap_or(1.0);
                }
            }
            1.0
        }
        _ => 1.0,
    }
}

pub fn parse_font_size(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> f32 {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f as f32).unwrap_or(16.0),
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_font_size(
                        &Some(serde_json::Value::String(val_str.clone())),
                        variables,
                        theme_mode,
                    );
                }
            }
            s.parse::<f32>().unwrap_or(16.0)
        }
        _ => 16.0,
    }
}

pub fn resolve_font_family(
    v: &Option<String>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> String {
    let mut resolved = v.as_deref().unwrap_or("").trim().to_string();
    while resolved.starts_with("$-") {
        let var_name = resolved.strip_prefix("$").unwrap_or(&resolved);
        if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
            resolved = val_str.clone();
        } else {
            break;
        }
    }
    if resolved.trim().is_empty() {
        resolved = "JetBrains Mono".to_string();
    }
    let key = resolved
        .to_ascii_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if key == "geist" {
        return "Noto Sans CJK SC".to_string();
    }
    resolved
}

fn embedded_font_bytes_for_family(font_family: &str) -> Option<&'static [u8]> {
    let key = font_family
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if key == "jetbrains mono" {
        return Some(crate::fonts::JETBRAINS_MONO_REGULAR);
    }
    if key == "noto sans cjk sc" {
        return Some(crate::fonts::NOTO_SANS_CJK_SC_REGULAR);
    }
    if key == "geist" {
        return Some(crate::fonts::NOTO_SANS_CJK_SC_REGULAR);
    }
    None
}

pub fn parse_color(
    s: &str,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Color {
    let s = s.trim();
    if s.is_empty() {
        return Color::TRANSPARENT;
    }
    if s.starts_with("$-") {
        let var_name = s.strip_prefix("$").unwrap_or(s);
        if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
            return parse_color(val_str, variables, theme_mode);
        }

        // Fallback to hardcoded values if not found in variables (for backward compatibility or missing vars)
        match s {
            "$--background" => return Color::from_rgb8(20, 20, 20),
            "$--popover" => return Color::from_rgb8(40, 40, 40),
            "$--border" => return Color::from_rgb8(60, 60, 60),
            "$--text" => return Color::WHITE,
            "$--color-info" => return Color::from_rgba8(40, 40, 80, 1.0),
            "$--color-info-foreground" => return Color::from_rgb8(220, 220, 255),
            "$--color-warning-foreground" => return Color::from_rgb8(255, 200, 0),
            "$--primary" => return Color::from_rgb8(255, 165, 0),
            "$--primary-foreground" => return Color::BLACK,
            "$--secondary" => return Color::from_rgb8(40, 40, 40),
            "$--secondary-foreground" => return Color::WHITE,
            "$--foreground" => return Color::WHITE,
            "$--muted-foreground" => return Color::from_rgb8(160, 160, 160),
            "$--destructive" => return Color::from_rgb8(255, 80, 80),
            _ => return Color::from_rgb8(100, 100, 100),
        }
    }
    if let Some(hex) = s.strip_prefix('#') {
        let hex = hex.trim();
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
                return Color::from_rgb8(r * 17, g * 17, b * 17);
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[3..4], 16).unwrap_or(15);
                return Color::from_rgba8(r * 17, g * 17, b * 17, (a * 17) as f32 / 255.0);
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                return Color::from_rgb8(r, g, b);
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
                return Color::from_rgba8(r, g, b, a as f32 / 255.0);
            }
            _ => return Color::from_rgb(1.0, 0.0, 1.0),
        }
    }
    if s.starts_with("rgb") {
        let content = s
            .trim_start_matches("rgba")
            .trim_start_matches("rgb")
            .trim_start_matches('(')
            .trim_end_matches(')');
        let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
        if parts.len() >= 3 {
            let r = parts[0].parse::<u8>().unwrap_or(0);
            let g = parts[1].parse::<u8>().unwrap_or(0);
            let b = parts[2].parse::<u8>().unwrap_or(0);
            let a = if parts.len() > 3 { parts[3].parse::<f32>().unwrap_or(1.0) } else { 1.0 };
            return Color::from_rgba8(r, g, b, a);
        }
    }
    if let Some(c) = parse_named_color(s) {
        return c;
    }
    Color::TRANSPARENT
}

pub fn parse_named_color(name: &str) -> Option<Color> {
    match name.to_lowercase().as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::from_rgb8(255, 0, 0)),
        "green" => Some(Color::from_rgb8(0, 128, 0)),
        "blue" => Some(Color::from_rgb8(0, 0, 255)),
        "yellow" => Some(Color::from_rgb8(255, 255, 0)),
        "cyan" => Some(Color::from_rgb8(0, 255, 255)),
        "magenta" => Some(Color::from_rgb8(255, 0, 255)),
        "gray" | "grey" => Some(Color::from_rgb8(128, 128, 128)),
        "transparent" => Some(Color::TRANSPARENT),
        "orange" => Some(Color::from_rgb8(255, 165, 0)),
        "purple" => Some(Color::from_rgb8(128, 0, 128)),
        "pink" => Some(Color::from_rgb8(255, 192, 203)),
        "brown" => Some(Color::from_rgb8(165, 42, 42)),
        "silver" => Some(Color::from_rgb8(192, 192, 192)),
        "lime" => Some(Color::from_rgb8(0, 255, 0)),
        "maroon" => Some(Color::from_rgb8(128, 0, 0)),
        "olive" => Some(Color::from_rgb8(128, 128, 0)),
        "navy" => Some(Color::from_rgb8(0, 0, 128)),
        "teal" => Some(Color::from_rgb8(0, 128, 128)),
        _ => None,
    }
}

pub fn parse_line_height(
    v: &Option<serde_json::Value>,
    font_size: f32,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> f32 {
    match v {
        Some(serde_json::Value::Number(n)) => {
            let val = n.as_f64().map(|f| f as f32).unwrap_or(1.2);
            if val < 4.0 { val * font_size } else { val }
        }
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_line_height(
                        &Some(serde_json::Value::String(val_str.clone())),
                        font_size,
                        variables,
                        theme_mode,
                    );
                }
            }

            if s.ends_with('%') {
                let pct = s.trim_end_matches('%').parse::<f32>().unwrap_or(120.0);
                font_size * (pct / 100.0)
            } else {
                s.parse::<f32>().unwrap_or(font_size * 1.2)
            }
        }
        _ => font_size * 1.2,
    }
}

pub fn resolve_stroke_color(
    stroke_fill: Option<&str>,
    _element_fill: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Color {
    if let Some(s) = stroke_fill {
        parse_color(s, variables, theme_mode)
    } else {
        Color::TRANSPARENT
    }
}

pub fn parse_shadow(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<ShadowSpec> {
    match v {
        Some(serde_json::Value::Object(map)) => {
            let color = if let Some(serde_json::Value::String(c)) = map.get("color") {
                parse_color(c, variables, theme_mode)
            } else {
                Color::BLACK
            };
            let offset_x =
                map.get("offset_x").and_then(|v| v.as_f64()).map(|f| f as f32).unwrap_or(0.0);
            let offset_y =
                map.get("offset_y").and_then(|v| v.as_f64()).map(|f| f as f32).unwrap_or(4.0);
            let blur = map.get("blur").and_then(|v| v.as_f64()).map(|f| f as f32).unwrap_or(10.0);
            let spread =
                map.get("spread").and_then(|v| v.as_f64()).map(|f| f as f32).unwrap_or(0.0);

            Some(ShadowSpec { color, offset: Vector::new(offset_x, offset_y), blur, spread })
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "parse_tests.rs"]
mod parse_tests;
