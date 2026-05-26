use crate::app::Message;
use crate::apps::mindmap::canvas::dash_segments_px;
use crate::apps::mindmap::state::EdgeStyle;
use iced::widget::canvas::{self, Frame, Geometry, LineCap, LineDash, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 线条样式预览器
///
/// 在画布上渲染一条水平线，用于预览不同的线条样式（实线、虚线、点线等）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct LineStylePreview {
    /// 要预览的线条样式
    pub(crate) style: EdgeStyle,
    /// 线条颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for LineStylePreview {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let y = bounds.height / 2.0;
        let stroke_width = (bounds.height * 0.18).clamp(1.6, 2.4);
        let inset = (stroke_width / 2.0).ceil() + 1.0;
        let start = Point::new(inset, y);
        let end = Point::new((bounds.width - inset).max(inset), y);

        let mut stroke =
            Stroke { width: stroke_width, style: self.color.into(), ..Stroke::default() };

        if let Some(segments) = dash_segments_px(self.style, 1.0) {
            stroke.line_dash = LineDash { segments, offset: 0 };
            if self.style == EdgeStyle::Dotted {
                stroke.line_cap = LineCap::Round;
            }
        }

        frame.stroke(&Path::line(start, end), stroke);

        vec![frame.into_geometry()]
    }
}

/// 边框样式预览器
///
/// 在画布上渲染一个圆角矩形边框，用于预览不同的边框样式。
#[derive(Debug, Clone, Copy)]
pub(crate) struct BorderStylePreview {
    /// 要预览的边框样式
    pub(crate) style: EdgeStyle,
    /// 边框颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for BorderStylePreview {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let stroke_width = (bounds.height * 0.18).clamp(1.6, 2.4);
        let inset = (stroke_width / 2.0).ceil() + 1.0;
        let w = (bounds.width - inset * 2.0).max(0.0);
        let h = (bounds.height - inset * 2.0).max(0.0);

        if w <= 0.0 || h <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let radius = (h / 2.0).max(0.0);
        let path =
            Path::rounded_rectangle(Point::new(inset, inset), Size::new(w, h), radius.into());

        let mut stroke =
            Stroke { width: stroke_width, style: self.color.into(), ..Stroke::default() };

        if let Some(segments) = dash_segments_px(self.style, 1.0) {
            stroke.line_dash = LineDash { segments, offset: 0 };
            if self.style == EdgeStyle::Dotted {
                stroke.line_cap = LineCap::Round;
            }
        }

        frame.stroke(&path, stroke);

        vec![frame.into_geometry()]
    }
}
