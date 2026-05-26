//! 右侧内联覆盖层组件
//!
//! 本模块实现了一个可在主内容右侧显示的覆盖层组件。该组件基于 Iced GUI 框架，
//! 提供了一种便捷的方式来在主内容旁边显示附加信息或交互界面。
//!
//! # 主要功能
//!
//! - **右侧定位**: 覆盖层始终显示在主内容区域的右侧
//! - **可配置间距**: 支持设置覆盖层与主内容之间的间距
//! - **视口边界控制**: 可选择是否将覆盖层限制在视口范围内
//! - **点击外部关闭**: 支持点击覆盖层外部区域时触发关闭消息
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use crate::components::overlays::inline_right::InlineRightOverlay;
//!
//! let content = text("主内容");
//! let overlay = text("覆盖层内容");
//!
//! InlineRightOverlay::new(content, overlay)
//!     .show(true)
//!     .gap(10.0)
//!     .on_close(Message::CloseOverlay)
//! ```

use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::{Element, Length};
use iced::{Event, Point, Rectangle, Size, Theme, Vector};

/// 右侧内联覆盖层组件
///
/// 该组件在主内容区域右侧显示一个覆盖层，适用于显示详细信息面板、
/// 上下文菜单、设置面板等需要与主内容并列显示的界面元素。
///
/// # 类型参数
///
/// - `'a`: 元素的生命周期参数
/// - `Message`: 组件产生的消息类型
/// - `ThemeT`: 主题类型，默认为 Iced 的 `Theme`
/// - `RendererT`: 渲染器类型，默认为 `iced::Renderer`
///
/// # 字段说明
///
/// - `content`: 主内容区域，即覆盖层左侧的基础内容
/// - `overlay`: 覆盖层内容，将在主内容右侧显示
/// - `show`: 控制覆盖层是否显示的标志
/// - `gap`: 覆盖层与主内容之间的间距（像素）
/// - `snap_within_viewport`: 是否将覆盖层限制在视口边界内
/// - `on_close`: 点击覆盖层外部时触发的关闭消息
pub struct InlineRightOverlay<'a, Message, ThemeT = Theme, RendererT = iced::Renderer> {
    /// 主内容区域的界面元素
    content: Element<'a, Message, ThemeT, RendererT>,
    /// 右侧覆盖层的界面元素
    overlay: Element<'a, Message, ThemeT, RendererT>,
    /// 控制覆盖层显示状态的标志
    show: bool,
    /// 覆盖层与主内容之间的间距（像素）
    gap: f32,
    /// 是否将覆盖层限制在视口边界内
    snap_within_viewport: bool,
    /// 点击覆盖层外部区域时触发的消息
    on_close: Option<Message>,
}

impl<'a, Message, ThemeT, RendererT> InlineRightOverlay<'a, Message, ThemeT, RendererT> {
    /// 创建新的右侧内联覆盖层实例
    ///
    /// # 参数
    ///
    /// - `content`: 主内容区域的界面元素，将被转换为 `Element`
    /// - `overlay`: 覆盖层的界面元素，将显示在主内容右侧
    ///
    /// # 返回值
    ///
    /// 返回新创建的 `InlineRightOverlay` 实例，默认配置为：
    /// - 覆盖层隐藏（`show` 为 `false`）
    /// - 间距为 0
    /// - 启用视口边界限制
    /// - 无关闭消息
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let overlay = InlineRightOverlay::new(
    ///     text("主内容"),
    ///     text("覆盖层")
    /// );
    /// ```
    pub fn new(
        content: impl Into<Element<'a, Message, ThemeT, RendererT>>,
        overlay: impl Into<Element<'a, Message, ThemeT, RendererT>>,
    ) -> Self {
        Self {
            content: content.into(),
            overlay: overlay.into(),
            show: false,
            gap: 0.0,
            snap_within_viewport: true,
            on_close: None,
        }
    }

    /// 设置覆盖层的显示状态
    ///
    /// # 参数
    ///
    /// - `show`: `true` 显示覆盖层，`false` 隐藏覆盖层
    ///
    /// # 返回值
    ///
    /// 返回修改后的 `InlineRightOverlay` 实例（构建器模式）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let overlay = InlineRightOverlay::new(content, overlay_content)
    ///     .show(true);  // 显示覆盖层
    /// ```
    pub fn show(mut self, show: bool) -> Self {
        self.show = show;
        self
    }

    /// 设置覆盖层与主内容之间的间距
    ///
    /// # 参数
    ///
    /// - `gap`: 间距值（单位：像素）
    ///
    /// # 返回值
    ///
    /// 返回修改后的 `InlineRightOverlay` 实例（构建器模式）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let overlay = InlineRightOverlay::new(content, overlay_content)
    ///     .gap(10.0);  // 设置 10 像素间距
    /// ```
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// 设置点击覆盖层外部时触发的关闭消息
    ///
    /// 当用户点击覆盖层和主内容区域之外的任何位置时，
    /// 将发送指定的消息。这通常用于关闭覆盖层。
    ///
    /// # 参数
    ///
    /// - `msg`: 点击外部时触发的消息
    ///
    /// # 返回值
    ///
    /// 返回修改后的 `InlineRightOverlay` 实例（构建器模式）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let overlay = InlineRightOverlay::new(content, overlay_content)
    ///     .on_close(Message::ClosePanel);
    /// ```
    pub fn on_close(mut self, msg: Message) -> Self {
        self.on_close = Some(msg);
        self
    }

    /// 设置是否将覆盖层限制在视口边界内
    ///
    /// 启用时，覆盖层将自动调整位置以确保完全可见；
    /// 禁用时，覆盖层可能超出视口边界。
    ///
    /// # 参数
    ///
    /// - `snap`: `true` 启用视口边界限制，`false` 禁用
    ///
    /// # 返回值
    ///
    /// 返回修改后的 `InlineRightOverlay` 实例（构建器模式）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 允许覆盖层超出视口边界
    /// let overlay = InlineRightOverlay::new(content, overlay_content)
    ///     .snap_within_viewport(false);
    /// ```
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }
}

/// 为 `InlineRightOverlay` 实现 `Widget` trait
///
/// 该实现将组件集成到 Iced 的组件系统中，处理布局、绘制、
/// 事件处理等核心功能。
impl<Message, ThemeT, RendererT> Widget<Message, ThemeT, RendererT>
    for InlineRightOverlay<'_, Message, ThemeT, RendererT>
where
    RendererT: iced::advanced::Renderer,
    Message: Clone,
{
    /// 获取组件的子组件树
    ///
    /// 返回主内容和覆盖层两个子组件的组件树。
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content), widget::Tree::new(&self.overlay)]
    }

    /// 比较并更新组件树
    ///
    /// 当组件状态变化时，更新子组件树以保持同步。
    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.content.as_widget(), self.overlay.as_widget()]);
    }

    /// 获取组件的尺寸约束
    ///
    /// 返回主内容区域的尺寸约束。
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    /// 获取组件的尺寸提示
    ///
    /// 返回主内容区域的尺寸提示信息。
    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    /// 计算组件的布局
    ///
    /// 基于给定的限制条件，计算主内容区域的布局节点。
    /// 覆盖层的布局在 `overlay` 方法中单独处理。
    ///
    /// # 参数
    ///
    /// - `tree`: 组件树，用于存储布局状态
    /// - `renderer`: 渲染器引用
    /// - `limits`: 布局限制条件
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &RendererT,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    /// 绘制组件
    ///
    /// 绘制主内容区域。覆盖层的绘制在 `overlay` 方法中处理。
    ///
    /// # 参数
    ///
    /// - `tree`: 组件树
    /// - `renderer`: 渲染器实例
    /// - `theme`: 当前主题
    /// - `style`: 渲染样式
    /// - `layout`: 布局信息
    /// - `cursor`: 鼠标光标状态
    /// - `viewport`: 视口矩形区域
    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut RendererT,
        theme: &ThemeT,
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

    /// 处理组件的事件
    ///
    /// 将事件转发给主内容区域进行处理。
    ///
    /// # 参数
    ///
    /// - `tree`: 组件树
    /// - `event`: 待处理的事件
    /// - `layout`: 布局信息
    /// - `cursor`: 鼠标光标状态
    /// - `renderer`: 渲染器引用
    /// - `clipboard`: 剪贴板接口
    /// - `shell`: 消息 shell，用于发布消息
    /// - `viewport`: 视口矩形区域
    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &RendererT,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
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

    /// 获取鼠标交互状态
    ///
    /// 返回主内容区域的鼠标交互类型（如指针、手型等）。
    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &RendererT,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    /// 创建覆盖层元素
    ///
    /// 这是该组件的核心方法，负责创建右侧覆盖层的显示逻辑。
    /// 当 `show` 为 `true` 时，创建并返回覆盖层元素。
    ///
    /// # 工作流程
    ///
    /// 1. 获取主内容的覆盖层元素（如果有）
    /// 2. 如果 `show` 为 `true`，创建 `InlineRightOverlayElement`
    /// 3. 将所有覆盖层元素组合成一个 `Group` 返回
    ///
    /// # 参数
    ///
    /// - `tree`: 组件树
    /// - `layout`: 主内容的布局信息
    /// - `renderer`: 渲染器引用
    /// - `viewport`: 视口矩形区域
    /// - `translation`: 位置偏移向量
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &RendererT,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, ThemeT, RendererT>> {
        // 获取子组件树的迭代器
        let mut children = tree.children.iter_mut();

        // 获取主内容区域自己的覆盖层元素（如果有）
        let content = self.content.as_widget_mut().overlay(
            children.next().unwrap(),
            layout,
            renderer,
            viewport,
            translation,
        );

        // 如果覆盖层需要显示，创建 InlineRightOverlayElement
        let inside_right = if self.show {
            Some(overlay::Element::new(Box::new(InlineRightOverlayElement {
                // 计算覆盖层的基准位置（主内容位置 + 偏移）
                position: layout.position() + translation,
                // 主内容的边界矩形
                target_bounds: layout.bounds(),
                // 当前视口
                viewport: *viewport,
                // 覆盖层与主内容的间距
                gap: self.gap,
                // 是否限制在视口内
                snap_within_viewport: self.snap_within_viewport,
                // 覆盖层内容
                overlay: &mut self.overlay,
                // 覆盖层的组件树
                tree: children.next().unwrap(),
                // 关闭消息
                on_close: self.on_close.clone(),
            })))
        } else {
            None
        };

        // 如果有任何覆盖层元素，将它们组合成 Group 返回
        if content.is_some() || inside_right.is_some() {
            Some(
                overlay::Group::with_children(content.into_iter().chain(inside_right).collect())
                    .overlay(),
            )
        } else {
            None
        }
    }
}

/// 实现从 `InlineRightOverlay` 到 `Element` 的转换
///
/// 该实现允许将 `InlineRightOverlay` 直接用作 `Element`，
/// 使其能够在任何接受 `Element` 的地方使用。
impl<'a, Message, ThemeT, RendererT> From<InlineRightOverlay<'a, Message, ThemeT, RendererT>>
    for Element<'a, Message, ThemeT, RendererT>
where
    Message: 'a + Clone,
    RendererT: iced::advanced::Renderer + 'a,
    ThemeT: 'a,
{
    fn from(widget: InlineRightOverlay<'a, Message, ThemeT, RendererT>) -> Self {
        Element::new(widget)
    }
}

/// 右侧内联覆盖层元素
///
/// 这是实际的覆盖层渲染元素，由 `InlineRightOverlay` 的 `overlay` 方法创建。
/// 该结构体实现了 Iced 的 `Overlay` trait，负责覆盖层的布局、绘制和交互。
///
/// # 生命周期参数
///
/// - `'a`: 覆盖层元素内容的生命周期
/// - `'b`: 对外部数据的可变借用生命周期
///
/// # 字段说明
///
/// - `position`: 覆盖层的基准位置（相对于窗口）
/// - `target_bounds`: 主内容区域的边界矩形
/// - `viewport`: 当前视口区域
/// - `gap`: 覆盖层与主内容之间的间距
/// - `snap_within_viewport`: 是否将覆盖层限制在视口内
/// - `overlay`: 覆盖层的内容元素
/// - `tree`: 覆盖层的组件树
/// - `on_close`: 点击外部时触发的关闭消息
struct InlineRightOverlayElement<'a, 'b, Message, ThemeT, RendererT> {
    /// 覆盖层的基准位置坐标
    position: Point,
    /// 主内容区域的边界矩形
    target_bounds: Rectangle,
    /// 视口矩形区域
    viewport: Rectangle,
    /// 覆盖层与主内容的间距（像素）
    gap: f32,
    /// 是否将覆盖层限制在视口边界内
    snap_within_viewport: bool,
    /// 覆盖层的界面元素
    overlay: &'b mut Element<'a, Message, ThemeT, RendererT>,
    /// 覆盖层的组件树
    tree: &'b mut widget::Tree,
    /// 点击外部区域时触发的关闭消息
    on_close: Option<Message>,
}

/// 为 `InlineRightOverlayElement` 实现 `Overlay` trait
///
/// 该实现处理覆盖层的核心行为，包括布局计算、事件处理和渲染。
impl<Message, ThemeT, RendererT> overlay::Overlay<Message, ThemeT, RendererT>
    for InlineRightOverlayElement<'_, '_, Message, ThemeT, RendererT>
where
    RendererT: iced::advanced::Renderer,
    Message: Clone,
{
    /// 计算覆盖层的布局
    ///
    /// 该方法计算覆盖层的位置和尺寸，将其放置在主内容区域的右侧。
    ///
    /// # 布局算法
    ///
    /// 1. 计算覆盖层的内部布局，限制其高度与主内容相同
    /// 2. 计算覆盖层的 X 坐标：主内容右边界 - 覆盖层宽度 - 间距
    /// 3. 计算覆盖层的 Y 坐标：与主内容顶部对齐
    /// 4. 如果启用了视口限制，将位置限制在视口范围内
    ///
    /// # 参数
    ///
    /// - `renderer`: 渲染器引用
    /// - `bounds`: 可用的尺寸边界
    ///
    /// # 返回值
    ///
    /// 返回描述覆盖层位置和尺寸的布局节点
    fn layout(&mut self, renderer: &RendererT, bounds: Size) -> layout::Node {
        // 创建基于边界的视口矩形
        let viewport = Rectangle::with_size(bounds);

        // 计算覆盖层的内部布局
        // 高度限制为主内容的高度，宽度限制为视口宽度
        let node = self.overlay.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(
                Size::new(0.0, self.target_bounds.height),
                Size::new(viewport.width, self.target_bounds.height),
            ),
        );

        // 获取计算出的覆盖层尺寸
        let size = node.size();

        // 计算覆盖层的初始位置
        // X: 主内容右边界 - 覆盖层宽度 - 间距
        // Y: 与主内容顶部对齐
        let mut x = self.position.x + self.target_bounds.width + self.gap;
        let mut y = self.position.y;

        // 如果启用了视口限制，将覆盖层位置限制在视口范围内
        if self.snap_within_viewport {
            x = x.clamp(0.0, (self.viewport.width - size.width).max(0.0));
            y = y.clamp(0.0, (self.viewport.height - size.height).max(0.0));
        }

        // 将布局节点移动到计算出的位置
        node.move_to(Point::new(x, y))
    }

    /// 处理覆盖层的事件
    ///
    /// 该方法处理用户交互事件，包括点击外部关闭覆盖层的逻辑。
    ///
    /// # 事件处理逻辑
    ///
    /// 1. 检测鼠标左键点击事件
    /// 2. 如果点击发生在覆盖层和主内容之外，触发关闭消息
    /// 3. 将事件转发给覆盖层内容处理
    /// 4. 捕获所有鼠标事件，防止穿透到下层组件
    ///
    /// # 参数
    ///
    /// - `event`: 待处理的事件
    /// - `layout`: 覆盖层的布局信息
    /// - `cursor`: 鼠标光标状态
    /// - `renderer`: 渲染器引用
    /// - `clipboard`: 剪贴板接口
    /// - `shell`: 消息 shell，用于发布消息
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &RendererT,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        // 获取覆盖层的边界矩形
        let bounds = layout.bounds();

        // 处理点击外部关闭的逻辑
        // 条件：1) 鼠标左键按下 2) 有关闭消息 3) 光标位置存在
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(on_close) = &self.on_close
            && let Some(cursor_position) = cursor.position()
        {
            // 如果点击不在覆盖层内，也不在主内容区域内，触发关闭
            if !bounds.contains(cursor_position) && !self.target_bounds.contains(cursor_position) {
                shell.publish(on_close.clone());
                // 捕获事件，防止进一步传播
                shell.capture_event();
            }
        }

        // 将事件转发给覆盖层内容处理
        self.overlay
            .as_widget_mut()
            .update(self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds);

        // 捕获所有鼠标事件，防止穿透到下层组件
        if matches!(event, Event::Mouse(_)) {
            shell.capture_event();
        }
    }

    /// 获取覆盖层的鼠标交互状态
    ///
    /// 返回覆盖层内容区域的鼠标交互类型。
    ///
    /// # 参数
    ///
    /// - `layout`: 覆盖层的布局信息
    /// - `cursor`: 鼠标光标状态
    /// - `renderer`: 渲染器引用
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &RendererT,
    ) -> mouse::Interaction {
        self.overlay.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &self.viewport,
            renderer,
        )
    }

    /// 绘制覆盖层
    ///
    /// 渲染覆盖层的内容。
    ///
    /// # 参数
    ///
    /// - `renderer`: 渲染器实例
    /// - `theme`: 当前主题
    /// - `defaults`: 默认渲染样式
    /// - `layout`: 覆盖层的布局信息
    /// - `cursor`: 鼠标光标状态
    fn draw(
        &self,
        renderer: &mut RendererT,
        theme: &ThemeT,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        // 获取覆盖层的边界矩形
        let bounds = layout.bounds();

        // 绘制覆盖层内容
        self.overlay
            .as_widget()
            .draw(self.tree, renderer, theme, defaults, layout, cursor, &bounds);
    }
}
