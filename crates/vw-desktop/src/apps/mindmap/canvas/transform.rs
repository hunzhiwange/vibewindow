//! 画布坐标转换模块
//!
//! 本模块提供世界坐标与屏幕坐标之间的双向转换功能，用于支持思维导图的平移和缩放操作。
//!
//! # 坐标系统
//!
//! - **世界坐标**：思维导图节点的逻辑坐标，代表节点在虚拟画布上的绝对位置
//! - **屏幕坐标**：实际显示在屏幕上的像素坐标，考虑了平移和缩放变换
//!
//! # 变换公式
//!
//! - 屏幕坐标 = 世界坐标 × 缩放比例 + 平移向量
//! - 世界坐标 = (屏幕坐标 - 平移向量) ÷ 缩放比例

use iced::{Point, Vector};

/// 将世界坐标转换为屏幕坐标
///
/// 应用平移和缩放变换，将思维导图中的逻辑坐标转换为屏幕显示坐标。
///
/// # 参数
///
/// - `world`: 世界坐标系中的点
/// - `pan`: 平移向量，表示视图在屏幕上的偏移量
/// - `zoom`: 缩放比例，1.0 表示原始大小，>1.0 表示放大，<1.0 表示缩小
///
/// # 返回值
///
/// 返回屏幕坐标系中的对应点
///
/// # 示例
///
/// ```ignore
/// use iced::{Point, Vector};
/// let world_point = Point::new(100.0, 100.0);
/// let pan = Vector::new(50.0, 50.0);
/// let zoom = 2.0;
/// let screen_point = screen_from_world(world_point, pan, zoom);
/// assert_eq!(screen_point, Point::new(250.0, 250.0));
/// ```
pub(super) fn screen_from_world(world: Point, pan: Vector, zoom: f32) -> Point {
    // 先对世界坐标应用缩放，再应用平移偏移
    Point::new(world.x * zoom + pan.x, world.y * zoom + pan.y)
}

/// 将屏幕坐标转换为世界坐标
///
/// 执行逆变换，将屏幕显示坐标还原为思维导图中的逻辑坐标，
/// 常用于处理鼠标点击等屏幕事件。
///
/// # 参数
///
/// - `screen`: 屏幕坐标系中的点
/// - `pan`: 当前视图的平移向量
/// - `zoom`: 当前视图的缩放比例（必须非零）
///
/// # 返回值
///
/// 返回世界坐标系中的对应点
///
/// # 示例
///
/// ```ignore
/// use iced::{Point, Vector};
/// let screen_point = Point::new(250.0, 250.0);
/// let pan = Vector::new(50.0, 50.0);
/// let zoom = 2.0;
/// let world_point = world_from_screen(screen_point, pan, zoom);
/// assert_eq!(world_point, Point::new(100.0, 100.0));
/// ```
pub(super) fn world_from_screen(screen: Point, pan: Vector, zoom: f32) -> Point {
    // 先去除平移偏移，再应用缩放的逆变换
    Point::new((screen.x - pan.x) / zoom, (screen.y - pan.y) / zoom)
}
