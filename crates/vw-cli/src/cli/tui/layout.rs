//! TUI 布局管理模块
//!
//! 本模块提供了终端用户界面（TUI）的布局计算和管理功能。使用 ratatui 库的布局系统，
//! 实现了主界面、首页界面以及居中覆盖层的布局划分。
//!
//! # 主要功能
//!
//! - **主界面布局**：将终端窗口划分为页眉、副页眉、主体和页脚四个区域
//! - **首页布局**：提供带有 logo 和输入框的首页专用布局
//! - **居中覆盖层**：计算在给定区域内居中显示的矩形区域
//!
//! # 布局示意
//!
//! 主界面布局结构（垂直方向）：
//! ```text
//! ┌─────────────────────────────┐
//! │         Header (1行)        │
//! ├─────────────────────────────┤
//! │       Subheader (1行)       │
//! ├─────────────────────────────┤
//! │                             │
//! │          Body (最小5行)      │
//! │                             │
//! ├─────────────────────────────┤
//! │         Footer (1行)        │
//! └─────────────────────────────┘
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 主界面布局结构体
///
/// 管理终端用户界面的四个主要区域：
/// - header：顶部标题栏区域，通常用于显示应用名称或状态
/// - subheader：副标题栏区域，可用于显示次要信息或操作提示
/// - body：主体内容区域，是最大的可变区域，用于显示主要内容
/// - footer：底部状态栏区域，用于显示快捷键提示或状态信息
///
/// # 示例
///
/// ```ignore
/// use ratatui::layout::Rect;
/// use crate::app::agent::agent::loop_::cli::tui::layout::{MainLayout, main_layout};
///
/// let area = Rect::new(0, 0, 80, 24);
/// let layout = main_layout(area);
/// let header = layout.header_area();
/// let body = layout.body_area();
/// ```
pub(crate) struct MainLayout {
    /// 页眉区域（顶部第1行）
    header: Rect,
    /// 副页眉区域（顶部第2行）
    subheader: Rect,
    /// 主体内容区域（中间可变高度，最小5行）
    body: Rect,
    /// 页脚区域（底部第1行）
    footer: Rect,
}

impl MainLayout {
    /// 获取页眉区域
    ///
    /// 返回用于显示顶部标题栏的矩形区域。
    ///
    /// # 返回值
    ///
    /// 返回页眉的 `Rect` 区域，高度固定为 1 行
    pub(crate) fn header_area(&self) -> Rect {
        self.header
    }

    /// 获取副页眉区域
    ///
    /// 返回用于显示次要信息的矩形区域，位于页眉正下方。
    ///
    /// # 返回值
    ///
    /// 返回副页眉的 `Rect` 区域，高度固定为 1 行
    pub(crate) fn subheader_area(&self) -> Rect {
        self.subheader
    }

    /// 获取主体内容区域
    ///
    /// 返回用于显示主要内容的矩形区域，占据界面中心的大部分空间。
    ///
    /// # 返回值
    ///
    /// 返回主体的 `Rect` 区域，高度为最小 5 行的弹性区域
    pub(crate) fn body_area(&self) -> Rect {
        self.body
    }

    /// 获取页脚区域
    ///
    /// 返回用于显示底部状态栏的矩形区域。
    ///
    /// # 返回值
    ///
    /// 返回页脚的 `Rect` 区域，高度固定为 1 行
    pub(crate) fn footer_area(&self) -> Rect {
        self.footer
    }

    /// 获取输入区域
    ///
    /// 在主体区域内计算并返回用于用户输入的矩形区域。
    /// 布局策略：
    /// 1. 将主体区域水平分割为左侧（68%）和右侧（32%）
    /// 2. 将左侧区域垂直分割为顶部（最小5行）和底部（剩余空间）
    /// 3. 返回底部区域作为输入区
    ///
    /// # 布局示意
    ///
    /// ```text
    /// ┌──────────────────┬────────┐
    /// │                  │        │
    /// │   顶部区域       │ 右侧   │
    /// │   (Min(5))       │ (32%)  │
    /// ├──────────────────┤        │
    /// │   输入区域       │        │
    /// │   (返回此区域)   │        │
    /// └──────────────────┴────────┘
    /// ```
    ///
    /// # 返回值
    ///
    /// 返回输入框的 `Rect` 区域，位于主体区域的左下部分
    pub(crate) fn input_area(&self) -> Rect {
        // 将主体区域水平分割为左侧（68%）和右侧（32%）
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(self.body);

        // 将左侧区域垂直分割，为输入框预留空间（主体高度-6）
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(self.body.height.saturating_sub(6)),
            ])
            .split(body_chunks[0]);

        // 返回左下区域作为输入区
        left_chunks[1]
    }
}

/// 创建主界面布局
///
/// 根据给定的终端区域创建并返回一个 `MainLayout` 实例，
/// 将区域划分为页眉、副页眉、主体和页脚四个部分。
///
/// # 参数
///
/// - `area`：终端窗口的可用矩形区域
///
/// # 返回值
///
/// 返回初始化后的 `MainLayout` 实例，可通过其方法访问各个子区域
///
/// # 示例
///
/// ```ignore
/// use ratatui::layout::Rect;
///
/// let full_area = Rect::new(0, 0, 80, 24);
/// let layout = main_layout(full_area);
/// ```
pub(crate) fn main_layout(area: Rect) -> MainLayout {
    MainLayout::new(area)
}

impl MainLayout {
    /// 创建新的主界面布局实例
    ///
    /// 根据给定的终端区域，使用垂直布局将其划分为四个部分：
    /// - chunks[0]：页眉，固定高度 1 行
    /// - chunks[1]：副页眉，固定高度 1 行
    /// - chunks[2]：主体，最小高度 5 行（弹性扩展）
    /// - chunks[3]：页脚，固定高度 1 行
    ///
    /// # 参数
    ///
    /// - `area`：终端窗口的完整可用区域
    ///
    /// # 返回值
    ///
    /// 返回包含四个子区域坐标的 `MainLayout` 实例
    ///
    /// # 布局约束说明
    ///
    /// 使用 `Constraint::Length(1)` 固定页眉、副页眉和页脚的高度为 1 行，
    /// 使用 `Constraint::Min(5)` 确保主体区域至少有 5 行高度，
    /// 剩余空间会自动分配给主体区域。
    pub(crate) fn new(area: Rect) -> Self {
        // 使用垂直布局将区域分割为四个部分
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // 页眉：固定 4 行
                Constraint::Length(0), // 副页眉：固定 0 行
                Constraint::Min(5),    // 主体：最小 5 行
                Constraint::Length(1), // 页脚：固定 1 行
            ])
            .split(area);

        Self { header: chunks[0], subheader: chunks[1], body: chunks[2], footer: chunks[3] }
    }
}

/// 创建首页布局
///
/// 为首页界面计算布局区域，包含 logo 显示区和输入区。
/// 布局策略是将终端区域垂直分割，在中间区域放置 logo 和输入框。
///
/// # 参数
///
/// - `area`：终端窗口的完整可用区域
/// - `logo_height`：logo 区域的高度（行数），为 0 时不显示 logo
/// - `input_height`：输入框区域的高度（行数）
///
/// # 返回值
///
/// 返回一个元组 `(Vec<Rect>, Vec<Rect>)`：
/// - 第一个 `Vec<Rect>`：外层四个垂直分割的区域
///   - [0]：顶部空白区（弹性）
///   - [1]：中心内容区（包含 logo 和输入框）
///   - [2]：中间分隔区（弹性）
///   - [3]：底部状态栏（1行）
/// - 第二个 `Vec<Rect>`：中心内容区的进一步细分
///   - [0]：logo 区域
///   - [1]：logo 与输入框之间的间隔
///   - [2]：输入框区域
///   - [3-5]：其他装饰区域
///   - [6]：底部弹性区域
///
/// # 布局示意
///
/// ```text
/// ┌─────────────────────────────┐
/// │        顶部弹性区            │
/// ├─────────────────────────────┤
/// │         Logo 区域            │
/// │         (间隔)               │
/// │        输入框区域            │
/// │         (装饰)               │
/// ├─────────────────────────────┤
/// │        中间弹性区            │
/// ├─────────────────────────────┤
/// │        底部状态栏            │
/// └─────────────────────────────┘
/// ```
///
/// # 示例
///
/// ```ignore
/// use ratatui::layout::Rect;
///
/// let area = Rect::new(0, 0, 80, 24);
/// let (outer_chunks, center_chunks) = home_layout(area, 5, 3);
/// let logo_area = center_chunks[0];
/// let input_area = center_chunks[2];
/// ```
pub(crate) fn home_layout(
    area: Rect,
    logo_height: u16,
    input_height: u16,
) -> (Vec<Rect>, Vec<Rect>) {
    let logo_gap: u16 = if logo_height > 0 { 1 } else { 0 };
    let center_height =
        logo_height.saturating_add(input_height).saturating_add(logo_gap).saturating_add(2);
    let remaining_height = area.height.saturating_sub(center_height.saturating_add(1));
    let top_space = remaining_height / 2;
    let bottom_space = remaining_height.saturating_sub(top_space);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_space),
            Constraint::Length(center_height),
            Constraint::Length(bottom_space),
            Constraint::Length(1),
        ])
        .split(area);

    let center_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_height),
            Constraint::Length(logo_gap),
            Constraint::Length(input_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(chunks[1]);

    (chunks.to_vec(), center_chunks.to_vec())
}

/// 计算居中覆盖层的矩形区域
///
/// 根据给定的容器区域和尺寸比例，计算一个居中显示的矩形区域。
/// 常用于创建模态对话框、弹出窗口等需要居中显示的界面元素。
///
/// # 参数
///
/// - `area`：父容器的矩形区域
/// - `width_div`：宽度除数，用于计算目标宽度。实际宽度 ≈ area.width × width_div / (width_div + 3)
/// - `height_div`：高度除数，用于计算目标高度。实际高度 ≈ area.height × height_div / (height_div + 3)
/// - `max_height`：最大高度限制，计算出的高度不会超过此值
///
/// # 返回值
///
/// 返回居中定位的 `Rect` 区域，该区域在父容器内水平和垂直居中
///
/// # 计算公式
///
/// ```text
/// width = area.width × width_div / (width_div + 3)
/// height = min(max_height, area.height × height_div / (height_div + 3))
/// x = area.x + (area.width - width) / 2
/// y = area.y + (area.height - height) / 2
/// ```
///
/// # 示例
///
/// ```ignore
/// use ratatui::layout::Rect;
///
/// let screen = Rect::new(0, 0, 80, 24);
/// // 创建一个约为屏幕 1/2 宽度、1/2 高度的居中区域
/// let overlay = centered_overlay_rect(screen, 2, 2, 20);
/// ```
pub(crate) fn centered_overlay_rect(
    area: Rect,
    width_div: u16,
    height_div: u16,
    max_height: u16,
) -> Rect {
    // 计算宽度：area.width × width_div / (width_div + 3)
    let w = area.width.saturating_mul(width_div) / (width_div + 3);

    // 计算高度：取计算值和最大高度限制中的较小值
    let h = max_height.min(area.height.saturating_mul(height_div) / (height_div + 3));

    // 计算 x 坐标：水平居中
    let x = area.x.saturating_add(area.width.saturating_sub(w) / 2);

    // 计算 y 坐标：垂直居中
    let y = area.y.saturating_add(area.height.saturating_sub(h) / 2);

    Rect { x, y, width: w, height: h }
}
#[cfg(test)]
#[path = "layout_tests.rs"]
mod layout_tests;
