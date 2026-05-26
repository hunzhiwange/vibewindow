//! 文本编辑器组件的上下文菜单或滚动面板控件。
//!
//! 本模块主要负责把应用状态转换为桌面端可渲染的 Iced 控件，并把用户操作映射回上层消息。
//! 注释聚焦调用边界、返回值和错误传播方式，便于后续维护设置页与工具栏行为时快速定位职责。

use iced::advanced::layout;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::mouse;
use iced::widget::slider::Rail;
use iced::widget::{container, row, vertical_slider};
use iced::{Border, Color, Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};

#[derive(Debug, Clone, Copy)]
/// `TextEditorScrollPanelMetrics` 结构体，用于表达本模块对该领域对象的建模。
///
/// 该定义保持在当前模块职责内，调用方应通过显式字段、变体或别名理解其语义。
pub struct TextEditorScrollPanelMetrics {
    pub viewport_padding: f32,
    pub line_height: f32,
    pub line_count: usize,
    pub scroll_top_line: f32,
}

/// 构建或处理 `text_editor_scroll_panel` 对应的界面片段与交互数据。
///
/// # 参数
///
/// 参数来自调用方持有的应用状态、配置快照或控件输入，用于保持渲染结果与当前状态同步。
///
/// # 返回值
///
/// 返回本函数生成的状态、样式或辅助值，供同一流程继续组合使用。
///
/// # 错误处理
///
/// 本函数不直接返回错误；无法交互或缺省状态会在控件状态中显式表达。
pub fn text_editor_scroll_panel<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    size: Size,
    metrics: TextEditorScrollPanelMetrics,
    on_wheel: impl Fn(mouse::ScrollDelta, f32) -> Message + 'a,
    on_scrollbar_changed: impl Fn(f32, f32) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let viewport_height = (size.height - metrics.viewport_padding).max(1.0);
    let line_height = metrics.line_height.max(1.0);
    let total_lines = metrics.line_count.max(1) as f32;
    let visible_lines = (viewport_height / line_height).floor().max(1.0);
    let max_scroll = (total_lines - visible_lines).max(0.0);
    let scroll_top_line = metrics.scroll_top_line.clamp(0.0, max_scroll);

    let content = wheel_interceptor(content, move |delta| on_wheel(delta, viewport_height));
    let mut body = row![container(content).width(Length::Fill).height(Length::Fill)];

    if max_scroll > 0.0 {
        let slider =
            vertical_slider(0.0..=max_scroll, max_scroll - scroll_top_line, move |value| {
                on_scrollbar_changed(max_scroll - value, viewport_height)
            })
            .step(1.0)
            .width(10)
            .height(Length::Fill)
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                let thumb = match status {
                    iced::widget::vertical_slider::Status::Active => {
                        palette.background.strong.color.scale_alpha(0.85)
                    }
                    iced::widget::vertical_slider::Status::Hovered => {
                        theme.palette().primary.scale_alpha(0.75)
                    }
                    iced::widget::vertical_slider::Status::Dragged => theme.palette().primary,
                };

                iced::widget::vertical_slider::Style {
                    rail: Rail {
                        backgrounds: (
                            palette.background.weak.color.scale_alpha(0.30).into(),
                            palette.background.weak.color.scale_alpha(0.30).into(),
                        ),
                        width: 4.0,
                        border: Border {
                            radius: 999.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                    },
                    handle: iced::widget::vertical_slider::Handle {
                        shape: iced::widget::vertical_slider::HandleShape::Rectangle {
                            width: 8,
                            border_radius: 999.0.into(),
                        },
                        background: thumb.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            });

        body = body.push(container(slider).width(Length::Fixed(10.0)).height(Length::Fill));
    }

    container(body.spacing(8).height(Length::Fill))
        .padding(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.base.color.into()),
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn wheel_interceptor<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    on_scroll: impl Fn(mouse::ScrollDelta) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a,
{
    Element::new(WheelInterceptor { content: content.into(), on_scroll: Box::new(on_scroll) })
}

struct WheelInterceptor<'a, Message> {
    content: Element<'a, Message>,
    on_scroll: Box<dyn Fn(mouse::ScrollDelta) -> Message + 'a>,
}

impl<Message> Widget<Message, Theme, Renderer> for WheelInterceptor<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if cursor.is_over(layout.bounds())
            && let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                shell.publish((self.on_scroll)(*delta));
                shell.capture_event();
                return;
            }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
