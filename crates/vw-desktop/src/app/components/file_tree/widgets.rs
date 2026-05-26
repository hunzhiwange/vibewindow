//! 文件树自定义组件模块
//!
//! 本模块提供了文件树组件所需的底层交互组件实现，包括：
//! - 右键菜单支持
//! - 左键拖拽识别
//! - 鼠标事件转发

use iced::advanced::{Clipboard, Layout, Shell, Widget, mouse, widget};
use iced::{Element, Event, Length, Point, Rectangle, Theme};

/// 右键点击区域组件
///
/// 该组件包装一个内部元素，为其添加右键菜单和左键拖拽支持。
/// 组件会智能区分点击和拖拽操作，只有当鼠标移动超过阈值时才触发拖拽。
///
/// # 类型参数
///
/// * `'a` - 组件的生命周期，与内部元素的生存期绑定
/// * `Message` - 组件产生的事件消息类型
/// * `ThemeT` - 主题类型，默认为 iced::Theme
/// * `RendererT` - 渲染器类型，默认为 iced::Renderer
///
/// # 示例
///
/// ```ignore
/// let area = RightClickArea::new(
///     content.into(),
///     Box::new(|pos| Message::RightClick(pos)),
///     Some(Message::LeftPress),
///     Some(Message::LeftRelease),
/// );
/// ```
pub struct RightClickArea<'a, Message, ThemeT = Theme, RendererT = iced::Renderer> {
    /// 被包装的内部内容元素
    content: Element<'a, Message, ThemeT, RendererT>,
    /// 右键点击回调，接收相对于组件的本地坐标
    on_right_click: Box<dyn Fn(Point) -> Message + 'a>,
    /// 左键按下时发送的消息（仅在识别为拖拽时触发）
    on_left_press: Option<Message>,
    /// 左键释放时发送的消息（仅在之前触发了拖拽时发送）
    on_left_release: Option<Message>,
}

/// 右键点击区域的状态数据
///
/// 用于跟踪鼠标按下和拖拽的状态，以便区分点击和拖拽操作。
/// 拖拽阈值设置为 3 像素（移动距离的平方 >= 9）。
#[derive(Default)]
struct RightClickAreaState {
    /// 左键是否在组件内部按下
    left_downinside: bool,
    /// 左键按下时的鼠标位置（用于计算拖拽距离）
    press_pos: Option<Point>,
    /// 是否已经开始拖拽（移动距离超过阈值）
    dragging_started: bool,
}

impl<'a, Message, ThemeT, RendererT> RightClickArea<'a, Message, ThemeT, RendererT> {
    /// 创建新的右键点击区域组件
    ///
    /// # 参数
    ///
    /// * `content` - 被包装的内容元素
    /// * `on_right_click` - 右键点击回调，参数为相对于组件边界的本地坐标点
    /// * `on_left_press` - 左键按下消息（可选），仅在识别为拖拽时发送
    /// * `on_left_release` - 左键释放消息（可选），仅在之前触发了拖拽时发送
    ///
    /// # 返回值
    ///
    /// 返回配置好的 RightClickArea 实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let area = RightClickArea::new(
    ///     text("文件名").into(),
    ///     Box::new(|pos| Message::ShowContextMenu(pos)),
    ///     Some(Message::StartDrag),
    ///     Some(Message::EndDrag),
    /// );
    /// ```
    pub fn new(
        content: Element<'a, Message, ThemeT, RendererT>,
        on_right_click: Box<dyn Fn(Point) -> Message + 'a>,
        on_left_press: Option<Message>,
        on_left_release: Option<Message>,
    ) -> Self {
        Self { content, on_right_click, on_left_press, on_left_release }
    }
}

/// 为 RightClickArea 实现 Iced Widget trait
///
/// 该实现委托大部分功能给内部 content 元素，
/// 同时拦截鼠标事件以实现右键菜单和拖拽识别功能。
impl<'a, Message, ThemeT, RendererT> Widget<Message, ThemeT, RendererT>
    for RightClickArea<'a, Message, ThemeT, RendererT>
where
    RendererT: iced::advanced::Renderer,
    Message: Clone,
{
    /// 返回状态类型标签，用于类型识别
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<RightClickAreaState>()
    }

    /// 创建组件状态实例
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(RightClickAreaState::default())
    }

    /// 返回子组件树，包含内部 content 元素
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    /// 对比并更新子组件树的差异
    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.content]);
    }

    /// 返回组件的尺寸需求
    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    /// 执行布局计算
    ///
    /// # 参数
    ///
    /// * `tree` - 组件树，包含状态和子组件
    /// * `renderer` - 渲染器引用
    /// * `limits` - 布局限制条件
    ///
    /// # 返回值
    ///
    /// 返回布局节点，描述组件的位置和大小
    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &RendererT,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }

    /// 绘制组件
    ///
    /// 委托给内部 content 元素进行实际绘制。
    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut RendererT,
        theme: &ThemeT,
        style: &iced::advanced::renderer::Style,
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

    /// 处理事件并更新状态
    ///
    /// 该方法实现以下交互逻辑：
    ///
    /// 1. **左键按下检测**：记录按下位置和状态
    /// 2. **拖拽识别**：当左键按下后移动超过 3 像素时，触发拖拽
    ///    - 移动距离的平方 >= 9（即距离 >= 3 像素）
    ///    - 首次识别为拖拽时，发送 on_left_press 消息
    /// 3. **右键点击**：直接触发 on_right_click 回调，传递相对于组件的本地坐标
    /// 4. **左键释放**：如果之前开始了拖拽，发送 on_left_release 消息
    /// 5. **事件转发**：所有事件都会转发给内部 content 元素
    ///
    /// # 参数
    ///
    /// * `tree` - 组件树，包含状态和子组件
    /// * `event` - 待处理的事件
    /// * `layout` - 组件的布局信息
    /// * `cursor` - 鼠标光标状态
    /// * `renderer` - 渲染器引用
    /// * `clipboard` - 剪贴板接口
    /// * `shell` - 消息外壳，用于发布消息
    /// * `viewport` - 视口矩形
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
        let state = tree.state.downcast_mut::<RightClickAreaState>();

        // 处理左键按下：记录初始位置
        if let Event::Mouse(mouse::Event::ButtonPressed(button)) = event
            && matches!(button, mouse::Button::Left)
            && let Some(pos) = cursor.position()
            && layout.bounds().contains(pos)
        {
            state.left_downinside = true;
            state.press_pos = Some(pos);
            state.dragging_started = false;
        }

        // 处理鼠标移动：检测是否开始拖拽
        // 拖拽阈值：移动距离 >= 3 像素（距离平方 >= 9）
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event
            && state.left_downinside
            && !state.dragging_started
            && let Some(anchor) = state.press_pos
        {
            let dx = position.x - anchor.x;
            let dy = position.y - anchor.y;
            let moved_enough = (dx * dx + dy * dy) >= 9.0;
            if moved_enough {
                state.dragging_started = true;
                // 首次识别为拖拽时，发送左键按下消息
                if let Some(msg) = &self.on_left_press {
                    shell.publish(msg.clone());
                }
            }
        }

        // 处理右键点击：计算相对于组件边界的本地坐标
        if let Event::Mouse(mouse::Event::ButtonPressed(button)) = event
            && matches!(button, mouse::Button::Right)
            && let Some(pos) = cursor.position()
            && layout.bounds().contains(pos)
        {
            let bounds = layout.bounds();
            // 转换为相对于组件左上角的本地坐标
            let local = Point::new(pos.x - bounds.x, pos.y - bounds.y);
            shell.publish((self.on_right_click)(local));
        }

        // 处理左键释放：如果之前开始了拖拽，发送释放消息
        if let Event::Mouse(mouse::Event::ButtonReleased(button)) = event
            && matches!(button, mouse::Button::Left)
        {
            if state.dragging_started
                && let Some(msg) = &self.on_left_release
            {
                shell.publish(msg.clone());
            }
            // 重置所有状态
            state.left_downinside = false;
            state.press_pos = None;
            state.dragging_started = false;
        }

        // 将事件转发给内部 content 元素处理
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

    /// 返回鼠标交互状态
    ///
    /// 委托给内部 content 元素决定鼠标光标样式。
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
}
