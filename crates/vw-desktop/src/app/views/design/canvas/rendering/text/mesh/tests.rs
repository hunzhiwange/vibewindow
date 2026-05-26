//! 设计画布文本渲染模块。
//!
//! 该模块处理文本节点的排版、网格、树结构或便签绘制逻辑，确保 DOM 风格输入能够稳定映射到画布中的可见文本。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn measures_and_wraps_with_font() {
        let w = measure_text_width("Hello", "JetBrains Mono", 16.0, 0.0);
        assert!(w > 0.0);

        let lines = wrap_text_lines_with_font("Hello world", w * 0.6, "JetBrains Mono", 16.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn wrap_is_tolerant_to_subpixel_width() {
        let font = "JetBrains Mono";
        let size = 14.0;
        let w = measure_text_width("Tab Item", font, size, 0.0);

        let lines = wrap_text_lines_with_font("Tab Item", (w - 0.25).max(0.0), font, size, 0.0);
        assert_eq!(lines, vec!["Tab Item".to_string()]);

        let lines = wrap_text_lines_with_font("Tab Item", (w - 2.0).max(0.0), font, size, 0.0);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn builds_glyph_outline_path() {
        let Some(ops) = with_face("JetBrains Mono", |face| {
            let gid = face.glyph_index('A')?;
            glyph_outline_ops("JetBrains Mono", gid)
        })
        .flatten() else {
            return;
        };

        let _path = Path::new(|builder| {
            build_iced_path_from_ops(ops.as_slice(), builder, 0.0, 0.0, 1.0);
        });
    }
}
