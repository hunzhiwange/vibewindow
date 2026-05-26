use crate::app::Message;
use crate::apps::mindmap::state::TreeLayoutFormat;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

/// 树形布局格式预览器
///
/// 在画布上渲染树形布局的缩略预览，展示层级结构的各种排列方式。
#[derive(Debug, Clone, Copy)]
pub(crate) struct TreeLayoutFormatPreview {
    /// 要预览的树形布局格式
    pub(crate) format: TreeLayoutFormat,
    /// 预览颜色
    pub(crate) color: Color,
}

impl canvas::Program<Message> for TreeLayoutFormatPreview {
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

        let to_rect = |center: Point| {
            Rectangle::new(
                Point::new(center.x - node_w / 2.0, center.y - node_h / 2.0),
                Size::new(node_w, node_h),
            )
        };

        let root = Point::new(w * 0.50, h * 0.26);
        let root_rect = to_rect(root);
        let root_path =
            Path::rounded_rectangle(root_rect.position(), root_rect.size(), radius.into());
        frame.fill(&root_path, self.color.scale_alpha(0.22));
        frame.stroke(
            &root_path,
            Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.8)),
        );

        match self.format {
            TreeLayoutFormat::FanDown => {
                let b1 = Point::new(w * 0.25, h * 0.68);
                let b2 = Point::new(w * 0.50, h * 0.76);
                let b3 = Point::new(w * 0.75, h * 0.68);

                for b in [b1, b2, b3] {
                    frame.stroke(
                        &Path::line(
                            Point::new(root.x, root_rect.y + root_rect.height),
                            Point::new(b.x, b.y - node_h / 2.0),
                        ),
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.55)),
                    );

                    let r = to_rect(b);
                    let p = Path::rounded_rectangle(r.position(), r.size(), radius.into());
                    frame.fill(&p, self.color.scale_alpha(0.16));
                    frame.stroke(
                        &p,
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.7)),
                    );
                }
            }
            TreeLayoutFormat::SymmetricSplit => {
                let junction = Point::new(w * 0.50, h * 0.52);
                let left = Point::new(w * 0.28, h * 0.74);
                let right = Point::new(w * 0.72, h * 0.74);
                let top = Point::new(root.x, root_rect.y + root_rect.height);

                frame.stroke(
                    &Path::line(top, junction),
                    Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
                );
                frame.stroke(
                    &Path::line(Point::new(left.x, junction.y), Point::new(right.x, junction.y)),
                    Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
                );

                for b in [left, right] {
                    frame.stroke(
                        &Path::line(
                            Point::new(b.x, junction.y),
                            Point::new(b.x, b.y - node_h / 2.0),
                        ),
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.55)),
                    );

                    let r = to_rect(b);
                    let p = Path::rounded_rectangle(r.position(), r.size(), radius.into());
                    frame.fill(&p, self.color.scale_alpha(0.16));
                    frame.stroke(
                        &p,
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.7)),
                    );
                }
            }
            TreeLayoutFormat::LeftAligned | TreeLayoutFormat::RightAligned => {
                let dir = if self.format == TreeLayoutFormat::LeftAligned { -1.0 } else { 1.0 };
                let spine_x = w * 0.52;
                let child_x = (spine_x + dir * w * 0.26).clamp(node_w / 2.0, w - node_w / 2.0);
                let top = Point::new(spine_x, root_rect.y + root_rect.height);
                let y1 = h * 0.52;
                let y2 = h * 0.80;

                frame.stroke(
                    &Path::line(top, Point::new(spine_x, y2 - node_h / 2.0)),
                    Stroke::default().with_width(stroke_w).with_color(self.color.scale_alpha(0.55)),
                );

                for y in [y1, y2] {
                    frame.stroke(
                        &Path::line(Point::new(spine_x, y), Point::new(child_x, y)),
                        Stroke::default()
                            .with_width(stroke_w)
                            .with_color(self.color.scale_alpha(0.55)),
                    );

                    let child = Point::new(child_x, y);
                    let r = to_rect(child);
                    let p = Path::rounded_rectangle(r.position(), r.size(), radius.into());
                    frame.fill(&p, self.color.scale_alpha(0.16));
                    frame.stroke(
                        &p,
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
