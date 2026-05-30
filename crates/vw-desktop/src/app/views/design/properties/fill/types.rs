//! # 填充类型模块
//!
//! 本模块定义了设计视图中使用的填充类型系统，支持多种填充样式：
//! - **纯色填充**：单色填充
//! - **渐变填充**：线性、径向、角度渐变
//! - **网格渐变填充**：复杂的网格渐变效果
//! - **图像填充**：使用图片作为填充
//!
//! ## 核心类型
//!
//! - [`FillItem`]：填充项，表示一个填充配置
//! - [`FillObject`]：填充对象，具体的填充类型枚举
//! - [`GradientFill`]：渐变填充配置
//! - [`MeshFill`]：网格渐变填充配置
//! - [`ImageFill`]：图像填充配置
//!
//! ## 序列化说明
//!
//! 所有类型均实现了 `Serialize` 和 `Deserialize` trait，支持 JSON 序列化。
//! 使用 `#[serde]` 属性进行字段重命名和默认值配置，以兼容外部 API 格式。

use serde::{Deserialize, Serialize};
use web_time::{SystemTime, UNIX_EPOCH};

/// 填充项枚举
///
/// 表示一个填充配置，可以是简单的颜色字符串或复杂的填充对象。
/// 使用 `#[serde(untagged)]` 实现无标签序列化，允许在反序列化时
/// 根据内容自动推断类型。
///
/// # 序列化示例
///
/// ```json
/// // 简单颜色字符串
/// "#ff0000"
///
/// // 填充对象
/// {
///   "type": "solid",
///   "color": "#ff0000",
///   "enabled": true
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FillItem {
    /// 简单颜色字符串（如 "#ff0000"）
    Color(String),
    /// 复杂填充对象
    Object(FillObject),
}

/// 填充对象枚举
///
/// 定义具体的填充类型，使用 `#[serde(tag = "type")]` 实现
/// 内部标签序列化，通过 "type" 字段区分不同变体。
///
/// # 变体说明
///
/// - `Solid` / `Color`：纯色填充
/// - `Gradient`：渐变填充（线性、径向、角度）
/// - `Mesh`：网格渐变填充
/// - `Image`：图像填充
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum FillObject {
    /// 纯色填充（solid 类型）
    #[serde(rename = "solid")]
    Solid {
        /// 颜色值（十六进制格式，如 "#ff0000"）
        color: String,
        /// 是否启用此填充
        #[serde(default = "default_true")]
        enabled: bool,
    },
    /// 纯色填充（color 类型）
    #[serde(rename = "color")]
    Color {
        /// 颜色值（十六进制格式）
        color: String,
        /// 是否启用此填充
        #[serde(default = "default_true")]
        enabled: bool,
    },
    /// 渐变填充
    #[serde(rename = "gradient")]
    Gradient(GradientFill),
    /// 网格渐变填充
    #[serde(rename = "mesh_gradient")]
    Mesh(MeshFill),
    /// 图像填充
    #[serde(rename = "image")]
    Image(ImageFill),
}

/// 返回 `true` 的默认函数
///
/// 用于 serde 的 `default` 属性，为布尔字段提供默认值 `true`。
fn default_true() -> bool {
    true
}

/// 渐变填充配置
///
/// 支持线性、径向和角度渐变，可配置颜色停靠点、旋转角度、
/// 中心点和尺寸等参数。
///
/// # 字段说明
///
/// - `gradient_type`：渐变类型（"linear"、"radial"、"angular"）
/// - `rotation`：旋转角度（度数）
/// - `colors`：颜色停靠点列表
/// - `center`：渐变中心点（用于径向渐变）
/// - `size`：渐变尺寸（用于径向渐变）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradientFill {
    /// 渐变类型："linear"（线性）、"radial"（径向）、"angular"（角度）
    #[serde(default, rename = "gradientType")]
    pub gradient_type: String,
    /// 是否启用此渐变
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 旋转角度（度数）
    #[serde(default)]
    pub rotation: f64,
    /// 颜色停靠点列表
    #[serde(default)]
    pub colors: Vec<GradientStop>,
    /// 渐变中心点（径向渐变使用）
    #[serde(default)]
    pub center: Option<GradientCenter>,
    /// 渐变尺寸（径向渐变使用）
    #[serde(default)]
    pub size: Option<GradientSize>,
    /// 高度尺寸（用于特定场景）
    #[serde(default, rename = "sizeH")]
    pub size_h: Option<f64>,
}

/// 渐变中心点
///
/// 表示渐变的中心位置，坐标范围为 [0.0, 1.0]。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradientCenter {
    /// X 坐标（0.0 到 1.0）
    #[serde(default)]
    pub x: f64,
    /// Y 坐标（0.0 到 1.0）
    #[serde(default)]
    pub y: f64,
}

/// 渐变尺寸
///
/// 表示渐变的宽度和高度，用于径向渐变。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradientSize {
    /// 宽度（可选）
    #[serde(default)]
    pub width: Option<f64>,
    /// 高度（可选）
    #[serde(default)]
    pub height: Option<f64>,
}

/// 渐变颜色停靠点
///
/// 定义渐变中某一位置的颜色值。
///
/// # 示例
///
/// ```json
/// {
///   "color": "#ff0000",
///   "position": 0.5
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradientStop {
    /// 颜色值（十六进制格式）
    pub color: String,
    /// 位置（0.0 到 1.0）
    pub position: f64,
}

/// 网格渐变填充
///
/// 实现复杂的网格渐变效果，由行列网格点组成，每个网格点
/// 有独立的颜色和控制手柄。支持镜像模式和轮廓显示。
///
/// # 核心概念
///
/// - **网格点**：由行列定义的二维网格，每个点有颜色和位置
/// - **控制手柄**：每个点有 4 对控制手柄（8 个值），用于调整渐变曲线
/// - **镜像模式**：支持 X/Y 轴镜像
///
/// # 网格点索引
///
/// 网格点按行优先顺序索引：
/// ```text
/// | 0 | 1 | 2 |
/// | 3 | 4 | 5 |
/// | 6 | 7 | 8 |
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshFill {
    /// 是否启用此网格渐变
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 网格列数
    pub columns: usize,
    /// 网格行数
    pub rows: usize,
    /// 每个网格点的颜色（十六进制格式）
    pub colors: Vec<String>,
    /// 每个网格点的位置 [x, y]
    pub points: Vec<Vec<f64>>,
    /// 每个网格点的控制手柄 [h0x, h0y, h1x, h1y, h2x, h2y, h3x, h3y]
    #[serde(default)]
    pub handles: Vec<Vec<f64>>,
    /// 镜像模式："x" 或 "y"
    #[serde(default)]
    pub mirroring: Option<String>,
    /// 是否显示轮廓
    #[serde(default = "default_true")]
    pub outline: bool,
    /// 当前选中的网格点索引
    #[serde(default)]
    pub selected_point_index: Option<usize>,
}

impl MeshFill {
    /// 创建随机颜色的网格渐变
    ///
    /// 生成指定行列数的网格渐变，颜色随机生成，位置和手柄使用默认值。
    ///
    /// # 参数
    ///
    /// - `columns`：网格列数
    /// - `rows`：网格行数
    ///
    /// # 返回
    ///
    /// 返回配置好随机颜色的新 `MeshFill` 实例。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mesh = MeshFill::new_random(3, 3);
    /// assert_eq!(mesh.columns, 3);
    /// assert_eq!(mesh.rows, 3);
    /// assert_eq!(mesh.colors.len(), 9);
    /// ```
    pub fn new_random(columns: usize, rows: usize) -> Self {
        let count = columns.saturating_mul(rows);
        let (points, handles) = Self::default_points_and_handles(columns, rows);
        Self {
            enabled: true,
            columns,
            rows,
            colors: Self::random_colors(count),
            points,
            handles,
            mirroring: Some("x".to_string()),
            outline: true,
            selected_point_index: None,
        }
    }

    /// 规范化网格参数
    ///
    /// 确保网格参数在有效范围内：
    /// - 行列数限制在 [2, 6] 范围内
    /// - 颜色数量与网格点数量匹配
    /// - 位置和手柄数组长度正确
    /// - 选中索引有效
    /// - 镜像模式有效
    pub fn normalize(&mut self) {
        // 限制行列数在 [2, 6] 范围内
        self.columns = self.columns.clamp(2, 6);
        self.rows = self.rows.clamp(2, 6);
        let expected = self.columns.saturating_mul(self.rows);

        // 如果乘积为 0，重置为 2x2
        if expected == 0 {
            self.columns = 2;
            self.rows = 2;
        }

        let expected = self.columns.saturating_mul(self.rows);

        // 调整颜色数组长度
        if self.colors.is_empty() {
            // 无颜色时填充白色
            self.colors = vec!["#ffffff".to_string(); expected];
        } else if self.colors.len() < expected {
            // 不足时使用最后一个颜色填充
            let last = self.colors.last().cloned().unwrap_or_else(|| "#ffffff".to_string());
            self.colors.resize(expected, last);
        } else if self.colors.len() > expected {
            // 超出时截断
            self.colors.truncate(expected);
        }

        // 重新生成默认位置和手柄，然后复制现有值
        let (mut new_points, mut new_handles) =
            Self::default_points_and_handles(self.columns, self.rows);
        let copy = self.points.len().min(expected);
        for i in 0..copy {
            let p = &self.points[i];
            let x = p.first().copied().unwrap_or(new_points[i][0]);
            let y = p.get(1).copied().unwrap_or(new_points[i][1]);
            new_points[i] = vec![x, y];
        }

        // 复制手柄数据
        let copy_h = self.handles.len().min(expected);
        for i in 0..copy_h {
            let h = &self.handles[i];
            if h.len() >= 8 {
                new_handles[i] = vec![h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]];
            }
        }

        self.points = new_points;
        self.handles = new_handles;

        // 验证选中索引
        if self.selected_point_index.is_some_and(|idx| idx >= expected) {
            self.selected_point_index = None;
        }

        // 规范化镜像模式（仅支持 "x" 或 "y"）
        let s = self.mirroring.as_deref().unwrap_or("").trim().to_ascii_lowercase();
        let next = if s.contains('y') && !s.contains('x') { "y" } else { "x" };
        self.mirroring = Some(next.to_string());
    }

    /// 获取指定点的有效控制手柄
    ///
    /// 计算指定网格点的 4 对控制手柄（共 8 个值）。如果手柄未设置，
    /// 将根据点的位置和网格边界自动生成合理的默认值。
    ///
    /// # 参数
    ///
    /// - `point_index`：网格点索引（行优先顺序）
    ///
    /// # 返回
    ///
    /// 返回 8 个值的数组：`[h0x, h0y, h1x, h1y, h2x, h2y, h3x, h3y]`
    /// - h0: 左手柄
    /// - h1: 上手柄
    /// - h2: 右手柄
    /// - h3: 下手柄
    ///
    /// # 手柄生成逻辑
    ///
    /// 1. 如果手柄数据存在且有效，直接使用
    /// 2. 否则，手柄默认为点位置
    /// 3. 对于边缘点，自动向外扩展手柄以产生更好的渐变效果
    pub fn effective_handles(&self, point_index: usize) -> [f64; 8] {
        let columns = self.columns.max(2);
        let rows = self.rows.max(2);
        let expected = columns.saturating_mul(rows);

        // 边界检查：无效索引返回零数组
        if expected == 0 || point_index >= expected {
            return [0.0; 8];
        }

        // 计算归一化坐标的分母
        let denom_x = (columns - 1).max(1) as f64;
        let denom_y = (rows - 1).max(1) as f64;

        // 计算默认的归一化坐标
        let default_x = (point_index % columns) as f64 / denom_x;
        let default_y = (point_index / columns) as f64 / denom_y;

        // 获取点的实际位置，限制在 [0.0, 1.0] 范围内
        let (px, py) = self
            .points
            .get(point_index)
            .map(|p| {
                (
                    p.first().copied().unwrap_or(default_x).clamp(0.0, 1.0),
                    p.get(1).copied().unwrap_or(default_y).clamp(0.0, 1.0),
                )
            })
            .unwrap_or((default_x, default_y));

        // 获取手柄数据或使用点位置作为默认值
        let raw = self.handles.get(point_index);
        let mut out = if let Some(h) = raw
            && h.len() >= 8
        {
            [h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]]
        } else {
            [px, py, px, py, px, py, px, py]
        };

        // 计算手柄偏移量（基于网格间距）
        let dx = 1.0 / denom_x;
        let dy = 1.0 / denom_y;

        // 手柄扩展量，限制在合理范围内
        let off_x = (dx * 0.35).clamp(0.03, 0.12);
        let off_y = (dy * 0.35).clamp(0.03, 0.12);

        // 计算当前点的行列位置
        let r = point_index / columns.max(1);
        let c = point_index % columns.max(1);

        // 判断是否在边缘
        let has_left = c > 0;
        let has_top = r > 0;
        let has_right = c + 1 < columns;
        let has_bottom = r + 1 < rows;

        // 浮点数比较精度
        let eps = 1e-9;
        // 手柄值范围限制
        let handle_min = -0.5;
        let handle_max = 1.5;

        // 内部闭包：如果手柄为默认值（等于点位置），则自动扩展
        let mut maybe_expand = |hi: usize, enabled: bool, nx: f64, ny: f64| {
            if !enabled {
                return;
            }
            let base = hi * 2;
            let hx = out[base];
            let hy = out[base + 1];

            // 仅当手柄为默认值时才扩展
            if (hx - px).abs() <= eps && (hy - py).abs() <= eps {
                out[base] = nx.clamp(handle_min, handle_max);
                out[base + 1] = ny.clamp(handle_min, handle_max);
            }
        };

        // 对边缘点的手柄进行自动扩展
        maybe_expand(0, has_left, px - off_x, py); // 左手柄向左扩展
        maybe_expand(1, has_top, px, py - off_y); // 上手柄向上扩展
        maybe_expand(2, has_right, px + off_x, py); // 右手柄向右扩展
        maybe_expand(3, has_bottom, px, py + off_y); // 下手柄向下扩展

        out
    }

    /// 将计算的有效手柄值写入存储
    ///
    /// 计算指定点的有效手柄，如果与当前值不同则更新。
    ///
    /// # 参数
    ///
    /// - `point_index`：网格点索引
    ///
    /// # 返回
    ///
    /// - `true`：手柄值已更新
    /// - `false`：无需更新或索引无效
    pub fn materialize_effective_handles(&mut self, point_index: usize) -> bool {
        let columns = self.columns.max(2);
        let rows = self.rows.max(2);
        let expected = columns.saturating_mul(rows);

        // 边界检查
        if expected == 0 || point_index >= expected {
            return false;
        }

        // 确保数据已规范化
        if self.points.len() < expected || self.handles.len() < expected {
            self.normalize();
        }
        if point_index >= self.handles.len() {
            return false;
        }

        // 获取有效手柄和当前手柄
        let eff = self.effective_handles(point_index);
        let cur = self.handles.get(point_index);
        let cur_arr = if let Some(h) = cur
            && h.len() >= 8
        {
            [h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]]
        } else {
            // 当前值无效时使用点位置
            let (px, py) = self
                .points
                .get(point_index)
                .map(|p| (p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0)))
                .unwrap_or((0.0, 0.0));
            [px, py, px, py, px, py, px, py]
        };

        // 检查是否有差异
        let differs = (0..8).any(|i| (cur_arr[i] - eff[i]).abs() > 1e-12);
        if differs && let Some(dst) = self.handles.get_mut(point_index) {
            *dst = eff.to_vec();
        }

        differs
    }

    /// 生成默认的网格点位置和手柄
    ///
    /// 创建均匀分布的网格点，每个点的手柄初始为点位置。
    ///
    /// # 参数
    ///
    /// - `columns`：网格列数
    /// - `rows`：网格行数
    ///
    /// # 返回
    ///
    /// 返回元组 `(points, handles)`：
    /// - `points`：每个点的 `[x, y]` 坐标
    /// - `handles`：每个点的 8 个手柄值
    pub fn default_points_and_handles(
        columns: usize,
        rows: usize,
    ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let columns = columns.max(2);
        let rows = rows.max(2);
        let count = columns.saturating_mul(rows);

        let mut points = Vec::with_capacity(count);
        let mut handles = Vec::with_capacity(count);

        // 计算归一化坐标的分母
        let denom_x = (columns - 1).max(1) as f64;
        let denom_y = (rows - 1).max(1) as f64;

        // 按行列顺序生成网格点
        for r in 0..rows {
            for c in 0..columns {
                let x = c as f64 / denom_x;
                let y = r as f64 / denom_y;
                points.push(vec![x, y]);
                // 初始手柄为点位置（4 对相同的值）
                handles.push(vec![x, y, x, y, x, y, x, y]);
            }
        }

        (points, handles)
    }

    /// 生成随机颜色数组
    ///
    /// 使用 HSL 颜色空间生成视觉上协调的随机颜色。
    ///
    /// # 参数
    ///
    /// - `count`：颜色数量
    ///
    /// # 返回
    ///
    /// 返回十六进制颜色字符串数组（如 "#ff0000"）
    ///
    /// # 颜色生成策略
    ///
    /// - 色相：随机 + 索引偏移，产生色彩变化
    /// - 饱和度：0.60-0.85，避免过于鲜艳或灰暗
    /// - 亮度：0.55-0.70，保持可读性
    pub fn random_colors(count: usize) -> Vec<String> {
        // 使用时间戳和计数器创建种子
        let seed =
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0);
        let mut rng = XorShift64::new(seed ^ (count as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));

        (0..count)
            .map(|i| {
                // 色相：随机值 + 索引偏移，确保颜色多样性
                let hue =
                    ((rng.next_u32() as f64 / u32::MAX as f64) * 360.0 + (i as f64 * 37.0)) % 360.0;
                // 饱和度：0.60-0.85 范围
                let sat = 0.60 + (rng.next_u32() as f64 / u32::MAX as f64) * 0.25;
                // 亮度：0.55-0.70 范围
                let light = 0.55 + (rng.next_u32() as f64 / u32::MAX as f64) * 0.15;
                let (r, g, b) = hsl_to_rgb_u8(hue, sat, light);
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            })
            .collect()
    }
}

/// XorShift64 随机数生成器
///
/// 简单快速的伪随机数生成器，基于 XorShift 算法。
/// 不适用于加密场景，仅用于生成随机颜色等非安全需求。
struct XorShift64 {
    /// 内部状态
    state: u64,
}

impl XorShift64 {
    /// 创建新的随机数生成器
    ///
    /// # 参数
    ///
    /// - `seed`：种子值（为零时使用固定非零值）
    fn new(seed: u64) -> Self {
        // 避免零种子（会导致全零输出）
        let seed = if seed == 0 { 0xD1B5_4A32_D192_ED03 } else { seed };
        Self { state: seed }
    }

    /// 生成下一个 32 位随机数
    ///
    /// # 返回
    ///
    /// 返回 32 位无符号整数
    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        // XorShift 变换：三次位移和异或
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        // 返回高 32 位
        (x >> 32) as u32
    }
}

/// HSL 转 RGB（u8 输出）
///
/// 将 HSL 颜色值转换为 RGB 颜色值。
///
/// # 参数
///
/// - `h`：色相（0-360 度）
/// - `s`：饱和度（0.0-1.0）
/// - `l`：亮度（0.0-1.0）
///
/// # 返回
///
/// 返回 `(r, g, b)` 元组，每个分量范围 0-255
fn hsl_to_rgb_u8(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    // 归一化色相到 [0, 1)
    let h = (h % 360.0) / 360.0;
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);

    // 饱和度为零时为灰度
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    // HSL 到 RGB 转换的中间变量
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;

    // 分别计算 R、G、B 分量
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

/// 色相到 RGB 分量转换
///
/// HSL 转 RGB 的辅助函数，计算单个颜色分量。
///
/// # 参数
///
/// - `p`：中间变量
/// - `q`：中间变量
/// - `t`：归一化色相值
///
/// # 返回
///
/// 返回 RGB 分量值（0.0-1.0）
fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    // 将 t 归一化到 [0, 1) 范围
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    // 根据色相区间计算颜色值
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// 图像填充配置
///
/// 使用图片作为填充内容，支持不同的缩放模式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageFill {
    /// 是否启用此图像填充
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 图片 URL
    pub url: String,
    /// 缩放模式：
    /// - "fit"：适应（保持比例，可能留白）
    /// - "fill"：填充（保持比例，可能裁剪）
    /// - "stretch"：拉伸（不保持比例）
    #[serde(default)]
    pub mode: String,
}

impl FillItem {
    /// 检查填充是否启用
    ///
    /// # 返回
    ///
    /// - `true`：填充已启用
    /// - `false`：填充已禁用
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let fill = FillItem::Color("#ff0000".to_string());
    /// assert!(fill.is_enabled());
    /// ```
    pub fn is_enabled(&self) -> bool {
        match self {
            // 简单颜色始终视为启用
            FillItem::Color(_) => true,
            FillItem::Object(obj) => match obj {
                FillObject::Solid { enabled, .. } => *enabled,
                FillObject::Color { enabled, .. } => *enabled,
                FillObject::Gradient(g) => g.enabled,
                FillObject::Mesh(m) => m.enabled,
                FillObject::Image(i) => i.enabled,
            },
        }
    }

    /// 设置填充的启用状态
    ///
    /// # 参数
    ///
    /// - `enabled`：是否启用
    ///
    /// # 行为说明
    ///
    /// 对于简单的颜色字符串，调用此方法会将其转换为
    /// `FillObject::Solid` 结构，以支持启用/禁用状态。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut fill = FillItem::Color("#ff0000".to_string());
    /// fill.set_enabled(false);
    /// assert!(!fill.is_enabled());
    /// ```
    pub fn set_enabled(&mut self, enabled: bool) {
        match self {
            // 简单颜色转换为 Solid 对象
            FillItem::Color(c) => {
                *self = FillItem::Object(FillObject::Solid { color: c.clone(), enabled });
            }
            FillItem::Object(obj) => match obj {
                FillObject::Solid { enabled: e, color: _ } => *e = enabled,
                FillObject::Color { enabled: e, .. } => *e = enabled,
                FillObject::Gradient(g) => g.enabled = enabled,
                FillObject::Mesh(m) => m.enabled = enabled,
                FillObject::Image(i) => i.enabled = enabled,
            },
        }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
