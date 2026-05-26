use crate::app::Message;
use crate::apps::mindmap::state::TimelineLayoutFormat;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 时间线布局格式预览器
///
/// 在画布上渲染时间线布局的缩略预览，展示时间轴上节点的排列方式。
#[derive(Debug, Clone, Copy)]
pub(crate) struct TimelineLayoutFormatPreview {
    /// 要预览的时间线布局格式
    pub(crate) format: TimelineLayoutFormat,
    /// 预览颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for TimelineLayoutFormatPreview {
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
        let w = bounds.width.max(1.0);
        let h = bounds.height.max(1.0);
        let node_h = (h * 0.22).clamp(10.0, 14.0);
        let node_w = (w * 0.30).clamp(18.0, 28.0);
        let radius = (node_h / 2.0).max(0.0);
        let stroke_w = (h * 0.06).clamp(1.0, 1.6);
        let baseline_y = h * 0.55;
        let root = Point::new(w * 0.20, baseline_y);
        let x_gap = w * 0.24;
        let y_gap = h * 0.26;
        let child_dx = w * 0.10;
        let axis = [
            Point::new(root.x + x_gap, baseline_y),
            Point::new(root.x + x_gap * 2.0, baseline_y),
            Point::new(root.x + x_gap * 3.0, baseline_y),
        ];

        let to_rect = |center: Point, ww: f32, hh: f32| {
            Rectangle::new(Point::new(center.x - ww / 2.0, center.y - hh / 2.0), Size::new(ww, hh))
        };

        let root_rect = to_rect(root, node_w, node_h);
        let root_path =
            Path::rounded_rectangle(root_rect.position(), root_rect.size(), radius.into());
        frame.fill(&root_path, self.color.scale_alpha(0.22));
        frame.stroke(
            &root_path,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.8)),
        );

        for (i, c) in axis.iter().copied().enumerate() {
            let axis_rect = to_rect(c, node_w * 0.92, node_h * 0.90);
            let axis_path =
                Path::rounded_rectangle(axis_rect.position(), axis_rect.size(), radius.into());
            let start = Point::new(root_rect.x + root_rect.width, root.y);
            let end = Point::new(axis_rect.x, c.y);
            let dx = (end.x - start.x).abs();
            let control = (dx * 0.5).clamp(10.0, 60.0);
            let c1 = Point::new(start.x + control, start.y);
            let c2 = Point::new(end.x - control, end.y);

            let edge = Path::new(|b| {
                b.move_to(start);
                b.bezier_curve_to(c1, c2, end);
            });

            frame.stroke(
                &edge,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
            );
            frame.fill(&axis_path, self.color.scale_alpha(0.16));
            frame.stroke(
                &axis_path,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.7)),
            );

            let sign = match self.format {
                TimelineLayoutFormat::UpDown => {
                    if i % 2 == 0 {
                        -1.0
                    } else {
                        1.0
                    }
                }
                TimelineLayoutFormat::AllUp => -1.0,
                TimelineLayoutFormat::AllDown => 1.0,
            };

            let sub = Point::new(c.x + child_dx, c.y + sign * y_gap);
            let sub_rect = to_rect(sub, node_w * 0.86, node_h * 0.86);
            let sub_path =
                Path::rounded_rectangle(sub_rect.position(), sub_rect.size(), radius.into());
            let attach_y = if sign < 0.0 { axis_rect.y } else { axis_rect.y + axis_rect.height };
            let start = Point::new(c.x, attach_y);
            let end = Point::new(sub_rect.x, sub.y);
            let p1 = Point::new(start.x, end.y);

            let edge = Path::new(|b| {
                b.move_to(start);
                b.line_to(p1);
                b.line_to(end);
            });

            frame.stroke(
                &edge,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
            );
            frame.fill(&sub_path, self.color.scale_alpha(0.16));
            frame.stroke(
                &sub_path,
                Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.7)),
            );
        }

        vec![frame.into_geometry()]
    }
}
