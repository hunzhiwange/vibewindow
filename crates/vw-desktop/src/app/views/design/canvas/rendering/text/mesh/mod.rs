use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{Frame, Path};
use iced::{Color, Point};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::app::views::design::{canvas::parse::parse_color, models::VariableDef};

pub(super) struct MeshSpec {
    pub(super) columns: usize,
    pub(super) rows: usize,
    pub(super) colors: Vec<Color>,
    pub(super) points: Vec<Point>,
}

pub(super) fn extract_mesh_fill(
    fill: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<MeshSpec> {
    match fill {
        Some(serde_json::Value::Object(map)) => parse_mesh_object(map, variables, theme_mode),
        Some(serde_json::Value::Array(arr)) => {
            for item in arr {
                if let serde_json::Value::Object(map) = item
                    && let Some(mesh) = parse_mesh_object(map, variables, theme_mode)
                {
                    return Some(mesh);
                }
            }
            None
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum CurveType {
    Line,
    Quadratic,
    Cubic,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) struct CurveVertex {
    pub(super) position: [f32; 2],
    pub(super) curve_type: CurveType,
    pub(super) control_0: [f32; 2],
    pub(super) control_1: [f32; 2],
    pub(super) uv_offset: [f32; 2],
}

/*
曲线渲染缺口梳理（现状 vs 目标）

现状（本模块当前在文本渲染里做的事情）：
- 只负责 mesh gradient 的解析与采样：extract_mesh_fill / sample_mesh_color
- 文本本身仍由 iced 的 Frame::fill_text 绘制（GPU 文本图集），因此无法访问：
  - 字形轮廓（TrueType/OTF 的 quad/cubic Bézier 段）
  - 曲线路径采样（tessellation / subdivision）
  - 法线/距离场（SDF/MSDF）所需的几何属性

缺失模块（要实现“曲线渲染文本”至少需要）：
- 轮廓解析：从字体文件（TTF/OTF）读出 glyph 的 contour 段（line/quad/cubic）
- 路径采样：将曲线转换为可渲染的几何（细分成线段/三角形）或保留曲线段并在 GPU 中求距离
- 法线/距离：SDF 抗锯齿通常需要在片段着色器里计算到曲线的有符号距离（或 MSDF）

在当前代码库（iced::widget::canvas）约束下：
- 没有现成的自定义 fragment shader 与自定义顶点属性通路（顶点扩展、单 draw call 混合 line+curve）
- 因此这里提供一个“可工作”的 CPU 曲线轮廓路径渲染（outline -> Path），并保留 CurveVertex 作为未来 GPU 路径的结构入口。
*/

#[derive(Debug, Clone, Copy)]
enum OutlineOp {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

struct OutlineCollector {
    ops: Vec<OutlineOp>,
}

impl OutlineCollector {
    fn new() -> Self {
        Self { ops: Vec::new() }
    }
}

impl ttf_parser::OutlineBuilder for OutlineCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.ops.push(OutlineOp::MoveTo(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.ops.push(OutlineOp::LineTo(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.ops.push(OutlineOp::QuadTo(x1, y1, x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.ops.push(OutlineOp::CubicTo(x1, y1, x2, y2, x, y));
    }

    fn close(&mut self) {
        self.ops.push(OutlineOp::Close);
    }
}

static FONT_BYTES_CACHE: Lazy<Mutex<HashMap<String, Arc<Vec<u8>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static GLYPH_OUTLINE_CACHE: Lazy<Mutex<HashMap<(String, u16), Arc<Vec<OutlineOp>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const CURVE_TESS_TOLERANCE_PX: f32 = 0.25;
const CURVE_TESS_MAX_DEPTH: u8 = 12;

#[cfg(not(target_arch = "wasm32"))]
fn find_font_file_for_family(family: &str) -> Option<std::path::PathBuf> {
    use std::path::{Path, PathBuf};

    let family = family.trim();
    if family.is_empty() {
        return None;
    }
    let needle = family
        .to_ascii_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut dirs: Vec<&'static str> = Vec::new();
    #[cfg(target_os = "macos")]
    {
        dirs.push("/System/Library/Fonts");
        dirs.push("/Library/Fonts");
        if let Some(home) = std::env::var_os("HOME").and_then(|s| s.into_string().ok()) {
            dirs.push(Box::leak(format!("{home}/Library/Fonts").into_boxed_str()));
        }
    }
    #[cfg(target_os = "linux")]
    {
        dirs.push("/usr/share/fonts");
        dirs.push("/usr/local/share/fonts");
        if let Some(home) = std::env::var_os("HOME").and_then(|s| s.into_string().ok()) {
            dirs.push(Box::leak(format!("{home}/.fonts").into_boxed_str()));
            dirs.push(Box::leak(format!("{home}/.local/share/fonts").into_boxed_str()));
        }
    }
    #[cfg(target_os = "windows")]
    {
        dirs.push("C:\\Windows\\Fonts");
    }

    fn walk(dir: &Path, depth: usize, needle: &str) -> Option<PathBuf> {
        if depth == 0 {
            return None;
        }
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(p) = walk(&path, depth.saturating_sub(1), needle) {
                    return Some(p);
                }
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            if !["ttf", "otf", "ttc", "otc"].contains(&ext.as_str()) {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
                .replace(['_', '-'], " ");
            if stem.contains(needle) {
                return Some(path);
            }
        }
        None
    }

    for d in dirs {
        let p = Path::new(d);
        if !p.exists() {
            continue;
        }
        if let Some(found) = walk(p, 3, &needle) {
            return Some(found);
        }
    }
    None
}

fn resolve_font_bytes(family: &str) -> Option<Arc<Vec<u8>>> {
    let family_key = family.trim().to_string();
    if family_key.is_empty() {
        return None;
    }

    if let Some(existing) = FONT_BYTES_CACHE.lock().ok().and_then(|m| m.get(&family_key).cloned()) {
        return Some(existing);
    }

    #[cfg(target_arch = "wasm32")]
    let bytes: Option<Vec<u8>> = {
        let fallback = crate::fonts::JETBRAINS_MONO_REGULAR;
        Some(fallback.to_vec())
    };

    #[cfg(not(target_arch = "wasm32"))]
    let bytes: Option<Vec<u8>> = {
        if let Some(path) = find_font_file_for_family(&family_key) {
            std::fs::read(path).ok()
        } else {
            Some(crate::fonts::JETBRAINS_MONO_REGULAR.to_vec())
        }
    };

    let bytes = Arc::new(bytes?);
    let mut cache = FONT_BYTES_CACHE.lock().ok()?;
    cache.insert(family_key, bytes.clone());
    Some(bytes)
}

fn with_face<R>(family: &str, f: impl FnOnce(&ttf_parser::Face<'_>) -> R) -> Option<R> {
    let bytes = resolve_font_bytes(family)?;
    let face = ttf_parser::Face::parse(bytes.as_slice(), 0).ok()?;
    Some(f(&face))
}

fn glyph_outline_ops(family: &str, glyph_id: ttf_parser::GlyphId) -> Option<Arc<Vec<OutlineOp>>> {
    let gid = glyph_id.0;
    if let Some(existing) =
        GLYPH_OUTLINE_CACHE.lock().ok().and_then(|m| m.get(&(family.to_string(), gid)).cloned())
    {
        return Some(existing);
    }

    let ops = with_face(family, |face| {
        let mut collector = OutlineCollector::new();
        let ok = face.outline_glyph(glyph_id, &mut collector);
        ok.is_some().then_some(collector.ops)
    })??;

    let ops = Arc::new(ops);
    let mut cache = GLYPH_OUTLINE_CACHE.lock().ok()?;
    cache.insert((family.to_string(), gid), ops.clone());
    Some(ops)
}

fn build_iced_path_from_ops(
    ops: &[OutlineOp],
    builder: &mut Builder,
    pen_x: f32,
    baseline_y: f32,
    scale: f32,
) {
    let map = |x: f32, y: f32| -> Point { Point::new(pen_x + x * scale, baseline_y - y * scale) };

    fn point_line_distance(p: Point, a: Point, b: Point) -> f32 {
        let abx = b.x - a.x;
        let aby = b.y - a.y;
        let apx = p.x - a.x;
        let apy = p.y - a.y;
        let ab2 = abx * abx + aby * aby;
        if ab2 <= 1e-6 {
            return (apx * apx + apy * apy).sqrt();
        }
        let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
        let cx = a.x + abx * t;
        let cy = a.y + aby * t;
        ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt()
    }

    fn flatten_quad(builder: &mut Builder, p0: Point, p1: Point, p2: Point, tol: f32, depth: u8) {
        let flat = point_line_distance(p1, p0, p2) <= tol;
        if flat || depth >= CURVE_TESS_MAX_DEPTH {
            builder.line_to(p2);
            return;
        }
        let p01 = Point::new((p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5);
        let p12 = Point::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
        let p012 = Point::new((p01.x + p12.x) * 0.5, (p01.y + p12.y) * 0.5);
        flatten_quad(builder, p0, p01, p012, tol, depth + 1);
        flatten_quad(builder, p012, p12, p2, tol, depth + 1);
    }

    fn flatten_cubic(
        builder: &mut Builder,
        p0: Point,
        p1: Point,
        p2: Point,
        p3: Point,
        tol: f32,
        depth: u8,
    ) {
        let d1 = point_line_distance(p1, p0, p3);
        let d2 = point_line_distance(p2, p0, p3);
        let flat = d1.max(d2) <= tol;
        if flat || depth >= CURVE_TESS_MAX_DEPTH {
            builder.line_to(p3);
            return;
        }
        let p01 = Point::new((p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5);
        let p12 = Point::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
        let p23 = Point::new((p2.x + p3.x) * 0.5, (p2.y + p3.y) * 0.5);
        let p012 = Point::new((p01.x + p12.x) * 0.5, (p01.y + p12.y) * 0.5);
        let p123 = Point::new((p12.x + p23.x) * 0.5, (p12.y + p23.y) * 0.5);
        let p0123 = Point::new((p012.x + p123.x) * 0.5, (p012.y + p123.y) * 0.5);
        flatten_cubic(builder, p0, p01, p012, p0123, tol, depth + 1);
        flatten_cubic(builder, p0123, p123, p23, p3, tol, depth + 1);
    }

    let tol_px = CURVE_TESS_TOLERANCE_PX;
    let mut current = Point::new(0.0, 0.0);
    let mut have_current = false;

    for op in ops {
        match *op {
            OutlineOp::MoveTo(x, y) => {
                current = map(x, y);
                have_current = true;
                builder.move_to(current);
            }
            OutlineOp::LineTo(x, y) => {
                let p = map(x, y);
                if have_current {
                    builder.line_to(p);
                } else {
                    builder.move_to(p);
                    have_current = true;
                }
                current = p;
            }
            OutlineOp::QuadTo(x1, y1, x, y) => {
                let p1 = map(x1, y1);
                let p2 = map(x, y);
                if !have_current {
                    builder.move_to(p2);
                    current = p2;
                    have_current = true;
                } else {
                    flatten_quad(builder, current, p1, p2, tol_px, 0);
                    current = p2;
                }
            }
            OutlineOp::CubicTo(x1, y1, x2, y2, x, y) => {
                let p1 = map(x1, y1);
                let p2 = map(x2, y2);
                let p3 = map(x, y);
                if !have_current {
                    builder.move_to(p3);
                    current = p3;
                    have_current = true;
                } else {
                    flatten_cubic(builder, current, p1, p2, p3, tol_px, 0);
                    current = p3;
                }
            }
            OutlineOp::Close => builder.close(),
        }
    }
}

#[allow(dead_code)]
pub(super) fn measure_text_width(
    text: &str,
    font_family: &str,
    font_size_px: f32,
    letter_spacing_px: f32,
) -> f32 {
    if font_size_px <= 0.0 {
        return 0.0;
    }
    let Some(width) = with_face(font_family, |face| {
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
                .unwrap_or_else(|| if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 });
            if !first {
                w += letter_spacing_px.max(0.0);
            }
            first = false;
            w += advance;
        }
        w
    }) else {
        return text
            .chars()
            .map(|ch| if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 })
            .sum::<f32>()
            + letter_spacing_px.max(0.0) * (text.chars().count().saturating_sub(1) as f32);
    };
    width
}

#[allow(dead_code)]
pub(super) fn wrap_text_lines_with_font(
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
    let letter_spacing = letter_spacing_px.max(0.0);

    if let Some(lines) = with_face(font_family, |face| {
        let units_per_em = match face.units_per_em() {
            0 => 1000.0,
            n => n as f32,
        };
        let scale = font_size_px / units_per_em;

        let mut out = Vec::new();
        for line in content.lines() {
            if line.is_empty() {
                out.push(String::new());
                continue;
            }

            let mut current = String::new();
            let mut current_w = 0.0;

            for ch in line.chars() {
                let advance = face
                    .glyph_index(ch)
                    .and_then(|gid| face.glyph_hor_advance(gid))
                    .map(|a| a as f32 * scale)
                    .unwrap_or_else(|| {
                        if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 }
                    });
                let extra = if current.is_empty() { 0.0 } else { letter_spacing };
                if current_w + extra + advance > max_width + eps && !current.is_empty() {
                    out.push(current);
                    current = String::new();
                    current_w = 0.0;
                }

                if !current.is_empty() {
                    current_w += letter_spacing;
                }
                current.push(ch);
                current_w += advance;
            }

            if !current.is_empty() {
                out.push(current);
            }
        }
        out
    }) {
        return lines;
    }

    let mut out = Vec::new();
    for line in content.lines() {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }

        let mut current = String::new();
        let mut current_width = 0.0;
        for ch in line.chars() {
            let ch_width = if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 };
            let extra = if current.is_empty() { 0.0 } else { letter_spacing };
            if current_width + extra + ch_width > max_width + eps && !current.is_empty() {
                out.push(current);
                current = String::new();
                current_width = 0.0;
            }
            if !current.is_empty() {
                current_width += letter_spacing;
            }
            current.push(ch);
            current_width += ch_width;
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}

pub(super) fn draw_char_outline(
    frame: &mut Frame,
    ch: char,
    pen_top_left: Point,
    font_family: &str,
    font_size_px: f32,
    color: Color,
) -> f32 {
    if font_size_px <= 0.0 {
        return 0.0;
    }

    let Some((advance_px, ascent_px, ops)) = with_face(font_family, |face| {
        let units_per_em = match face.units_per_em() {
            0 => 1000.0,
            n => n as f32,
        };
        let scale = font_size_px / units_per_em;
        let ascent = face.ascender() as f32 * scale;
        let gid = face.glyph_index(ch)?;
        let advance = face.glyph_hor_advance(gid).unwrap_or(0) as f32 * scale;
        let ops = glyph_outline_ops(font_family, gid)?;
        Some((advance, ascent, ops))
    })
    .flatten() else {
        let fallback = if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 };
        return fallback;
    };

    if !ops.is_empty() && !ch.is_whitespace() {
        let baseline_y = pen_top_left.y + ascent_px;
        let scale = with_face(font_family, |face| {
            let units_per_em = match face.units_per_em() {
                0 => 1000.0,
                n => n as f32,
            };
            font_size_px / units_per_em
        })
        .unwrap_or(font_size_px / 1000.0);

        let path = Path::new(|builder| {
            build_iced_path_from_ops(ops.as_slice(), builder, pen_top_left.x, baseline_y, scale);
        });
        frame.fill(&path, color);
    }

    if advance_px > 0.0 {
        advance_px
    } else if ch as u32 > 127 {
        font_size_px
    } else {
        font_size_px * 0.6
    }
}

pub(super) fn measure_char_advance(ch: char, font_family: &str, font_size_px: f32) -> f32 {
    if font_size_px <= 0.0 {
        return 0.0;
    }
    let Some(advance_px) = with_face(font_family, |face| {
        let units_per_em = match face.units_per_em() {
            0 => 1000.0,
            n => n as f32,
        };
        let scale = font_size_px / units_per_em;
        face.glyph_index(ch).and_then(|gid| face.glyph_hor_advance(gid)).map(|a| a as f32 * scale)
    })
    .flatten() else {
        return if ch as u32 > 127 { font_size_px } else { font_size_px * 0.6 };
    };

    if advance_px > 0.0 {
        advance_px
    } else if ch as u32 > 127 {
        font_size_px
    } else {
        font_size_px * 0.6
    }
}

fn parse_mesh_object(
    map: &serde_json::Map<String, serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<MeshSpec> {
    if let Some(serde_json::Value::String(kind)) = map.get("type") {
        if kind != "mesh_gradient" {
            return None;
        }
    } else {
        return None;
    }
    if let Some(serde_json::Value::Bool(false)) = map.get("enabled") {
        return None;
    }
    let columns = map.get("columns").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let rows = map.get("rows").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    if columns == 0 || rows == 0 {
        return None;
    }
    let mut colors = Vec::new();
    if let Some(serde_json::Value::Array(arr)) = map.get("colors") {
        for color in arr {
            if let serde_json::Value::String(s) = color {
                colors.push(parse_color(s, variables, theme_mode));
            }
        }
    }
    if colors.is_empty() {
        return None;
    }
    let expected = columns.saturating_mul(rows);
    if expected > 0 {
        if colors.len() < expected {
            let last = colors.last().copied().unwrap_or(Color::WHITE);
            colors.resize(expected, last);
        } else if colors.len() > expected {
            colors.truncate(expected);
        }
    }

    let mut points = default_points(columns, rows);
    if let Some(serde_json::Value::Array(arr)) = map.get("points") {
        let copy = arr.len().min(expected);
        for i in 0..copy {
            if let Some(serde_json::Value::Array(p)) = arr.get(i) {
                let x = p.first().and_then(|v| v.as_f64()).unwrap_or(points[i].x as f64) as f32;
                let y = p.get(1).and_then(|v| v.as_f64()).unwrap_or(points[i].y as f64) as f32;
                points[i] = Point::new(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
            }
        }
    }
    Some(MeshSpec { columns, rows, colors, points })
}

fn default_points(columns: usize, rows: usize) -> Vec<Point> {
    let columns = columns.max(2);
    let rows = rows.max(2);
    let mut out = Vec::with_capacity(columns.saturating_mul(rows));
    for r in 0..rows {
        for c in 0..columns {
            let u = if columns > 1 { c as f32 / (columns - 1) as f32 } else { 0.0 };
            let v = if rows > 1 { r as f32 / (rows - 1) as f32 } else { 0.0 };
            out.push(Point::new(u, v));
        }
    }
    out
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color { r: lerp(a.r, b.r, t), g: lerp(a.g, b.g, t), b: lerp(a.b, b.b, t), a: lerp(a.a, b.a, t) }
}

pub(super) fn sample_mesh_color(mesh: &MeshSpec, u: f32, v: f32, fallback: Color) -> Color {
    if mesh.columns < 2 || mesh.rows < 2 {
        return mesh.colors.first().copied().unwrap_or(fallback);
    }
    let query = Point::new(u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));

    let mut best: Option<(usize, f32, f32, f32)> = None;
    for r in 0..mesh.rows.saturating_sub(1) {
        for c in 0..mesh.columns.saturating_sub(1) {
            let i00 = r * mesh.columns + c;
            let i10 = r * mesh.columns + (c + 1);
            let i01 = (r + 1) * mesh.columns + c;
            let i11 = (r + 1) * mesh.columns + (c + 1);

            let p00 = mesh.points.get(i00).copied().unwrap_or(Point::new(0.0, 0.0));
            let p10 = mesh.points.get(i10).copied().unwrap_or(Point::new(1.0, 0.0));
            let p01 = mesh.points.get(i01).copied().unwrap_or(Point::new(0.0, 1.0));
            let p11 = mesh.points.get(i11).copied().unwrap_or(Point::new(1.0, 1.0));

            let min_x = p00.x.min(p10.x).min(p01.x).min(p11.x);
            let max_x = p00.x.max(p10.x).max(p01.x).max(p11.x);
            let min_y = p00.y.min(p10.y).min(p01.y).min(p11.y);
            let max_y = p00.y.max(p10.y).max(p01.y).max(p11.y);
            if query.x < min_x || query.x > max_x || query.y < min_y || query.y > max_y {
                continue;
            }

            let initial_s = ((query.x * (mesh.columns - 1) as f32) - c as f32).clamp(0.0, 1.0);
            let initial_t = ((query.y * (mesh.rows - 1) as f32) - r as f32).clamp(0.0, 1.0);
            if let Some((s, t, err2)) =
                invert_bilerp(p00, p10, p01, p11, query, initial_s, initial_t)
            {
                best = match best {
                    Some((_, _, _, best_err2)) if best_err2 <= err2 => best,
                    _ => Some((i00, s, t, err2)),
                };
            }
        }
    }

    if let Some((i00, s, t, _)) = best {
        let c0 = i00 % mesh.columns;
        let r0 = i00 / mesh.columns;
        let c1 = (c0 + 1).min(mesh.columns - 1);
        let r1 = (r0 + 1).min(mesh.rows - 1);

        let idx = |r: usize, c: usize| -> Option<Color> {
            let i = r * mesh.columns + c;
            mesh.colors.get(i).copied()
        };
        let c00 = idx(r0, c0).unwrap_or(fallback);
        let c10 = idx(r0, c1).unwrap_or(fallback);
        let c01 = idx(r1, c0).unwrap_or(fallback);
        let c11 = idx(r1, c1).unwrap_or(fallback);

        let top = lerp_color(c00, c10, s);
        let bottom = lerp_color(c01, c11, s);
        return lerp_color(top, bottom, t);
    }

    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let col_pos = u * (mesh.columns - 1) as f32;
    let row_pos = v * (mesh.rows - 1) as f32;
    let c0 = col_pos.floor() as usize;
    let r0 = row_pos.floor() as usize;
    let c1 = (c0 + 1).min(mesh.columns - 1);
    let r1 = (r0 + 1).min(mesh.rows - 1);
    let tx = col_pos - c0 as f32;
    let ty = row_pos - r0 as f32;
    let idx = |r: usize, c: usize| -> Option<Color> {
        let i = r * mesh.columns + c;
        mesh.colors.get(i).copied()
    };
    let c00 = idx(r0, c0).unwrap_or(fallback);
    let c10 = idx(r0, c1).unwrap_or(fallback);
    let c01 = idx(r1, c0).unwrap_or(fallback);
    let c11 = idx(r1, c1).unwrap_or(fallback);
    let top = lerp_color(c00, c10, tx);
    let bottom = lerp_color(c01, c11, tx);
    lerp_color(top, bottom, ty)
}

fn invert_bilerp(
    p00: Point,
    p10: Point,
    p01: Point,
    p11: Point,
    target: Point,
    initial_s: f32,
    initial_t: f32,
) -> Option<(f32, f32, f32)> {
    let ax = p00.x;
    let ay = p00.y;
    let bx = p10.x - p00.x;
    let by = p10.y - p00.y;
    let cx = p01.x - p00.x;
    let cy = p01.y - p00.y;
    let dx = p11.x + p00.x - p10.x - p01.x;
    let dy = p11.y + p00.y - p10.y - p01.y;

    let mut s = initial_s;
    let mut t = initial_t;
    for _ in 0..8 {
        let px = ax + bx * s + cx * t + dx * s * t;
        let py = ay + by * s + cy * t + dy * s * t;
        let fx = px - target.x;
        let fy = py - target.y;

        let dsdx = bx + dx * t;
        let dsdy = by + dy * t;
        let dtdx = cx + dx * s;
        let dtdy = cy + dy * s;

        let det = dsdx * dtdy - dsdy * dtdx;
        if det.abs() < 1e-8 {
            break;
        }

        let ds = (-fx * dtdy + fy * dtdx) / det;
        let dt = (-fy * dsdx + fx * dsdy) / det;

        s = (s + ds).clamp(-0.25, 1.25);
        t = (t + dt).clamp(-0.25, 1.25);
    }

    let px = ax + bx * s + cx * t + dx * s * t;
    let py = ay + by * s + cy * t + dy * s * t;
    let err2 = (px - target.x).powi(2) + (py - target.y).powi(2);

    if (-0.001..=1.001).contains(&s) && (-0.001..=1.001).contains(&t) {
        Some((s.clamp(0.0, 1.0), t.clamp(0.0, 1.0), err2))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
