use crate::app::Message;
use crate::apps::mindmap::state::OrgChartLayoutFormat;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 组织结构图布局格式预览器
///
/// 在画布上渲染组织结构图布局的缩略预览，展示层级结构的排列方式。
#[derive(Debug, Clone, Copy)]
pub(crate) struct OrgChartLayoutFormatPreview {
    /// 要预览的组织结构图布局格式
    pub(crate) format: OrgChartLayoutFormat,
    /// 预览颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for OrgChartLayoutFormatPreview {
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
        let node_w = (w * 0.30).clamp(18.0, 30.0);
        let radius = (node_h / 2.0).max(0.0);
        let stroke_w = (h * 0.06).clamp(1.0, 1.6);
        let root = Point::new(w * 0.50, h * 0.32);
        let child_gap = h * 0.26;
        let sibling_gap = w * 0.20;

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

        match self.format {
            OrgChartLayoutFormat::TopDown => {
                let child_y = root.y + child_gap;
                let children = [
                    Point::new(root.x - sibling_gap, child_y),
                    Point::new(root.x + sibling_gap, child_y),
                ];

                for c in children {
                    let child_rect = to_rect(c, node_w * 0.92, node_h * 0.90);
                    let child_path = Path::rounded_rectangle(
                        child_rect.position(),
                        child_rect.size(),
                        radius.into(),
                    );
                    let start = Point::new(root.x, root_rect.y + root_rect.height);
                    let end = Point::new(c.x, child_rect.y);
                    let dy = (end.y - start.y).abs();
                    let control = (dy * 0.5).clamp(10.0, 60.0);
                    let dir = if end.y >= start.y { 1.0 } else { -1.0 };
                    let c1 = Point::new(start.x, start.y + control * dir);
                    let c2 = Point::new(end.x, end.y - control * dir);

                    let edge = Path::new(|b| {
                        b.move_to(start);
                        b.bezier_curve_to(c1, c2, end);
                    });

                    frame.stroke(
                        &edge,
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.55)),
                    );
                    frame.fill(&child_path, self.color.scale_alpha(0.16));
                    frame.stroke(
                        &child_path,
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.7)),
                    );
                }
            }
            OrgChartLayoutFormat::LeftRight => {
                let child_y = root.y + child_gap;
                let children = [
                    Point::new(root.x - sibling_gap, child_y),
                    Point::new(root.x + sibling_gap, child_y),
                ];

                for c in children {
                    let child_rect = to_rect(c, node_w * 0.92, node_h * 0.90);
                    let child_path = Path::rounded_rectangle(
                        child_rect.position(),
                        child_rect.size(),
                        radius.into(),
                    );
                    let start = Point::new(root.x, root_rect.y + root_rect.height);
                    let end = Point::new(c.x, child_rect.y);
                    let mid_y = (start.y + end.y) / 2.0;
                    let p1 = Point::new(start.x, mid_y);
                    let p2 = Point::new(end.x, mid_y);

                    let edge = Path::new(|b| {
                        b.move_to(start);
                        b.line_to(p1);
                        b.line_to(p2);
                        b.line_to(end);
                    });

                    frame.stroke(
                        &edge,
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.55)),
                    );
                    frame.fill(&child_path, self.color.scale_alpha(0.16));
                    frame.stroke(
                        &child_path,
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.7)),
                    );
                }
            }
        }

        vec![frame.into_geometry()]
    }
}
