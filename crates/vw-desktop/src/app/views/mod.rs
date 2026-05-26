//! # 视图模块 (Views Module)
//!
//! 本模块提供了 VibeWindow 应用的所有视图组件和界面相关的功能。
//!
//! ## 模块职责
//!
//! - 组织和管理应用的用户界面视图
//! - 提供各类工具视图的统一入口
//! - 实现视图的模块化组织结构
//!
//! ## 子模块说明
//!
//! - [`apps`] - 应用管理视图
//! - [`base_tool`] - 工具视图基础组件
//! - [`color_tool`] - 颜色处理工具视图
//! - [`cleaner_tool`] - 电脑垃圾清理工具视图
//! - [`design`] - 设计相关视图
//! - [`home`] - 首页视图
//! - [`html_tool`] - HTML 处理工具视图
//! - [`json_diff_tool`] - JSON 差异对比工具视图
//! - [`json_tool`] - JSON 处理工具视图
//! - [`json_yaml_tool`] - JSON/YAML 格式转换工具视图
//! - [`large_file_tool`] - 大文件查找工具视图
//! - [`markdown_tool`] - Markdown 处理工具视图
//! - [`password_tool`] - 密码生成与管理工具视图
//! - [`project`] - 项目管理视图
//! - [`qr_tool`] - 二维码生成与识别工具视图
//! - [`redis_tool`] - Redis 客户端连接管理视图
//! - [`sql_tool`] - SQL 处理工具视图
//! - [`task_board`] - 任务看板视图
//! - [`timestamp_tool`] - 时间戳转换工具视图
//! - [`usage`] - 应用使用情况和统计视图
//!
//! ## 设计理念
//!
//! 本模块采用模块化设计，每个工具或功能都有独立的视图子模块，
//! 便于维护和扩展。新增工具视图时，只需创建新的子模块并在本文件中导出即可。

/// 应用管理视图模块
///
/// 提供应用程序的管理、配置和监控相关的视图组件。
pub mod apps;

/// 基础工具视图模块
///
/// 定义工具视图的通用接口和基础组件，为其他具体工具视图提供基础支持。
pub mod base_tool;

/// 颜色工具视图模块
///
/// 提供颜色选择、格式转换、调色板生成等颜色相关功能的视图。
pub mod color_tool;

/// 电脑垃圾清理工具视图模块
///
/// 提供 macOS 和 Windows 的垃圾清理脚本生成功能视图。
pub mod cleaner_tool;

/// 大文件查找工具视图模块
///
/// 提供本地大文件扫描与容量分类展示功能。
pub mod large_file_tool;

/// 设计视图模块
///
/// 提供设计相关的视图组件和界面元素。
pub mod design;

/// 首页视图模块
///
/// 应用的主页面视图，包含导航、快捷入口等核心界面元素。
pub mod home;

/// HTML 工具视图模块
///
/// 提供 HTML 格式化、压缩、转换等处理功能的视图。
pub mod html_tool;

/// JSON 差异对比工具视图模块
///
/// 提供两个 JSON 数据的差异对比和可视化展示功能。
pub mod json_diff_tool;

/// JSON 工具视图模块
///
/// 提供 JSON 格式化、压缩、验证、路径查询等处理功能的视图。
pub mod json_tool;

/// JSON/YAML 转换工具视图模块
///
/// 提供 JSON 和 YAML 格式之间的相互转换功能视图。
pub mod json_yaml_tool;

/// Markdown 工具视图模块
///
/// 提供 Markdown 文本的编辑、预览、转换等处理功能的视图。
pub mod markdown_tool;

/// 密码工具视图模块
///
/// 提供密码生成、强度检测、安全评估等密码管理相关功能的视图。
pub mod password_tool;

/// 项目视图模块
///
/// 提供项目信息的展示和管理相关视图。
pub mod project;

/// 二维码工具视图模块
///
/// 提供二维码的生成和识别功能的视图。
pub mod qr_tool;

/// Redis 客户端工具视图模块
///
/// 提供 Redis 连接配置、历史记录与导入导出功能的视图。
pub mod redis_tool;

/// SQL 工具视图模块
///
/// 提供 SQL 语句格式化、转换、验证等处理功能的视图。
pub mod sql_tool;

/// 任务看板视图模块
///
/// 提供任务的可视化看板界面，支持任务的状态管理和跟踪。
pub mod task_board;

/// 时间戳工具视图模块
///
/// 提供时间戳与日期时间格式相互转换功能的视图。
pub mod timestamp_tool;

/// 使用情况视图模块
///
/// 提供应用使用统计、资源消耗监控等信息的展示视图。
pub mod usage;
