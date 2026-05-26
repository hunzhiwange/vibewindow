//! 渐变色标停止条模块
//!
//! 本模块实现了渐变色标编辑器的可视化交互组件，提供以下功能：
//! - 渐变色带的实时预览渲染
//! - 色标停止点的可视化显示
//! - 色标停止点的拖拽交互
//! - 新色标停止点的添加
//!
//! 主要组件：
//! - `GradientStopsBar`: 渐变色标条的主体结构，存储色标数据和回调函数
//! - `GradientStopsBarState`: 管理拖拽状态的状态机

use iced::mouse;
use iced::widget::canvas::{self};
use iced::widget::canvas::{Action, Event, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Size, Theme};

use crate::app::Message;
use crate::app::views::design::properties::fill::types::GradientStop;

/// 渐变色标停止条
///
/// 该结构体实现了基于 Canvas 的渐变色标编辑器，提供可视化的色标管理功能。
/// 用户可以通过点击添加新的色标，通过拖拽调整色标的位置。
///
/// # 字段说明
///
/// * `stops` - 色标停止点列表，每个停止点包含颜色和位置信息
/// * `on_change` - 色标变化时的回调函数，用于通知上层组件更新状态
pub(super) struct GradientStopsBar {
    /// 色标停止点集合，按位置顺序排列
    pub(super) stops: Vec<GradientStop>,
    /// 色标变化回调，当用户修改色标时触发
    pub(super) on_change: Box<dyn Fn(Vec<GradientStop>) -> Message>,
}

/// 渐变色标条的状态
///
/// 该结构体用于追踪用户交互状态，特别是拖拽操作的当前状态。
/// 它作为 Canvas 程序的状态存储，在交互过程中持久化。
///
/// # 字段说明
///
/// * `dragging` - 当前正在拖拽的色标索引，None 表示没有拖拽操作
#[derive(Default)]
pub(super) struct GradientStopsBarState {
    /// 当前拖拽中的色标索引
    dragging: Option<usize>,
}

/// 为渐变色标条实现 Canvas 程序接口
///
/// 该实现遵循 iced 的 canvas::Program trait，提供自定义绘制和交互逻辑。
impl canvas::Program<Message> for GradientStopsBar {
    /// 关联的状态类型，用于追踪拖拽等交互状态
    type State = GradientStopsBarState;

    /// 绘制渐变色标条
    ///
    /// 该方法负责渲染渐变色标条的视觉效果，包括：
    /// 1. 渐变色带的绘制（通过逐像素计算渐变颜色）
    /// 2. 色标停止点的圆形手柄绘制
    /// 3. 边框轮廓的绘制
    ///
    /// # 参数说明
    ///
    /// * `_state` - 当前状态（绘制时不使用）
    /// * `renderer` - iced 渲染器，用于创建画布帧
    /// * `_theme` - 当前主题（绘制时不使用）
    /// * `bounds` - 绘制区域的边界矩形
    /// * `_cursor` - 鼠标光标位置（绘制时不使用）
    ///
    /// # 返回值
    ///
    /// 返回包含所有绘制内容的几何图形向量
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // 创建新的画布帧
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // 计算色带的布局参数
        let padding = 4.0; // 上下内边距
        let bar_height = (bounds.height - padding * 2.0).max(8.0); // 色带高度，最小8像素
        let bar_y = padding; // 色带起始Y坐标

        // 绘制渐变色带：通过逐像素填充实现平滑渐变效果
        if bounds.width > 0.0 {
            // 计算需要绘制的像素宽度
            let width = bounds.width.ceil().max(1.0) as i32;
            // 计算归一化分母，用于将像素位置转换为0-1的归一化位置
            let denom = (bounds.width - 1.0).max(1.0);

            // 逐像素绘制渐变颜色
            for x in 0..width {
                // 将像素位置转换为归一化的渐变位置（0.0 到 1.0）
                let t = x as f32 / denom;
                // 根据当前归一化位置计算渐变颜色
                let color = super::utils::gradient_color_at(&self.stops, t);
                // 绘制单像素宽度的垂直条
                frame.fill_rectangle(
                    Point::new(x as f32, bar_y),
                    Size::new(1.0, bar_height),
                    color,
                );
            }
        }

        // 绘制色带边框
        let bar_rect = Path::rectangle(Point::new(0.0, bar_y), Size::new(bounds.width, bar_height));
        frame.stroke(&bar_rect, Stroke::default().with_color(Color::from_rgb(0.3, 0.3, 0.3)));

        // 绘制所有色标停止点的手柄
        // 手柄居中于色带的垂直中心位置
        let center_y = bar_y + bar_height / 2.0;

        for stop in &self.stops {
            // 根据色标位置计算水平坐标
            let x = stop.position as f32 * bounds.width;
            let center = Point::new(x, center_y);

            // 绘制圆形手柄：白色填充 + 深色边框
            let handle = Path::circle(center, 6.0);
            frame.fill(&handle, Color::WHITE);
            frame.stroke(
                &handle,
                Stroke::default().with_color(Color::from_rgb(0.1, 0.1, 0.1)).with_width(1.0),
            );
        }

        // 返回渲染完成的几何图形
        vec![frame.into_geometry()]
    }

    /// 处理用户交互事件
    ///
    /// 该方法实现了渐变色标条的交互逻辑，支持以下操作：
    /// 1. 点击已有色标：开始拖拽操作
    /// 2. 点击空白区域：添加新的色标并开始拖拽
    /// 3. 拖拽移动：实时更新色标位置
    /// 4. 释放鼠标：结束拖拽操作
    ///
    /// # 参数说明
    ///
    /// * `state` - 可变的状态引用，用于追踪拖拽索引
    /// * `event` - 鼠标事件（按下、释放、移动等）
    /// * `bounds` - 组件的边界矩形
    /// * `cursor` - 鼠标光标位置信息
    ///
    /// # 返回值
    ///
    /// 返回可选的动作，可能是：
    /// - `Action::request_redraw()`: 请求重绘界面
    /// - `Action::publish(Message)`: 发布消息通知状态更新
    /// - `None`: 无需执行动作
    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        // 获取光标在组件内的相对位置，如果光标不在组件内则返回 None
        let cursor_pos = cursor.position_in(bounds)?;

        match event {
            // 鼠标左键按下：开始拖拽或添加新色标
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // 检查是否点击到了现有色标
                if let Some(hit) = super::utils::hit_test_stop(&self.stops, bounds, cursor_pos) {
                    // 点击到色标：记录拖拽索引并请求重绘
                    state.dragging = Some(hit);
                    return Some(Action::request_redraw());
                }

                // 未点击到色标：在点击位置添加新色标
                // 计算点击位置的归一化坐标（0.0 到 1.0）
                let t = (cursor_pos.x / bounds.width).clamp(0.0, 1.0);

                // 克隆当前色标列表
                let mut new_stops = self.stops.clone();

                // 根据点击位置计算该位置的渐变颜色
                let color = super::utils::gradient_color_at(&new_stops, t);

                // 将颜色转换为十六进制格式
                let hex =
                    super::super::solid::format_rgba_to_hex(color.r, color.g, color.b, color.a);

                // 添加新的色标停止点
                new_stops.push(GradientStop { color: hex, position: t as f64 });

                // 设置拖拽索引为新添加的色标
                state.dragging = Some(new_stops.len() - 1);

                // 发布变更消息并通知上层组件
                return Some(Action::publish((self.on_change)(new_stops)));
            }

            // 鼠标左键释放：结束拖拽操作
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.dragging = None;
                return Some(Action::request_redraw());
            }

            // 鼠标移动：如果正在拖拽则更新色标位置
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(idx) = state.dragging {
                    // 计算鼠标位置的归一化坐标
                    let t = (cursor_pos.x / bounds.width).clamp(0.0, 1.0);

                    // 克隆色标列表并更新拖拽中的色标位置
                    let mut new_stops = self.stops.clone();
                    if let Some(stop) = new_stops.get_mut(idx) {
                        stop.position = t as f64;
                    }

                    // 发布更新消息
                    return Some(Action::publish((self.on_change)(new_stops)));
                }
            }

            // 其他事件：忽略
            _ => {}
        }

        None
    }
}

#[cfg(test)]
#[path = "stops_bar_tests.rs"]
mod stops_bar_tests;
