//! below_tests.rs 测试模块。
//!
//! 这些测试固定相邻解析器、视图辅助函数或状态计算的行为，防止后续 UI 重排时破坏边界契约。

use super::{
    available_space_below, compute_overlay_position, layout_with_backdrop,
    should_place_overlay_above,
};
/// 重新导出 use iced::advanced::layout，让上层模块通过稳定路径访问。
use iced::advanced::layout;
/// 重新导出 use iced::{Point, Rectangle, Size}，让上层模块通过稳定路径访问。
use iced::{Point, Rectangle, Size};

/// 构建或定位 compute overlay position snaps within viewport，用于把浮层稳定附着到目标控件。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn compute_overlay_position_snaps_within_viewport() {
    let target_bounds = Rectangle { x: 280.0, y: 24.0, width: 32.0, height: 24.0 };
    let viewport = Rectangle::with_size(Size::new(300.0, 200.0));
    let overlay_size = Size::new(120.0, 160.0);

    let position =
        compute_overlay_position(target_bounds, viewport, overlay_size, 4.0, true, false);

    assert_eq!(position, Point::new(180.0, 40.0));
}

/// 构建或定位 below overlay can shrink to stay attached below target，用于把浮层稳定附着到目标控件。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn below_overlay_can_shrink_to_stay_attached_below_target() {
    let target_bounds = Rectangle { x: 280.0, y: 24.0, width: 32.0, height: 24.0 };
    let viewport = Rectangle::with_size(Size::new(300.0, 200.0));

    assert!(!should_place_overlay_above(target_bounds, viewport, 160.0, 4.0));

    let shrunk_overlay = Size::new(120.0, available_space_below(target_bounds, viewport, 4.0));
    let position =
        compute_overlay_position(target_bounds, viewport, shrunk_overlay, 4.0, true, false);

    assert_eq!(position, Point::new(180.0, 52.0));
}

/// 验证 layout with backdrop expands to full viewport 这一行为，确保对应解析或视图契约稳定。
///
/// # 参数
///
/// 参数沿用调用点中的应用状态、工具输入或渲染上下文，不在这里扩大权限或补造状态。
///
/// # 返回值
///
/// 无返回值时，函数通过发布消息或更新局部状态完成交互。
///
/// # 错误处理
///
/// 本函数不吞掉底层错误；没有显式错误通道时，会用空集合、`None` 或现有 UI 状态表达不可用结果。
#[test]
fn layout_with_backdrop_expands_to_full_viewport() {
    let viewport = Rectangle { x: 10.0, y: 20.0, width: 320.0, height: 240.0 };
    let overlay_node = layout::Node::new(Size::new(100.0, 80.0)).move_to(Point::new(40.0, 56.0));

    let root = layout_with_backdrop(viewport, overlay_node);

    assert_eq!(root.bounds(), viewport);
    assert_eq!(root.children().len(), 1);
    assert_eq!(
        root.children()[0].bounds(),
        Rectangle { x: 40.0, y: 56.0, width: 100.0, height: 80.0 }
    );
}
