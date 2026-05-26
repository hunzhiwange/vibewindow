//! 脑图数据模型模块
//!
//! 本模块提供脑图应用的数据模型定义和默认值构造器。
//! 主要用于初始化和构造脑图节点结构。

use crate::app::components::mind_map::MindNode;

/// 创建一个默认的脑图根节点
///
/// 返回一个包含默认文本且无子节点的脑图节点实例。
/// 该节点通常用作脑图的中心主题或初始状态。
///
/// # 返回值
///
/// 返回一个 `MindNode` 实例，其中：
/// - `text` 字段被设置为 "中心主题"
/// - `children` 字段为空向量（无子节点）
///
/// # 示例
///
/// ```ignore
/// use crate::apps::mindmap::model::default_doc;
///
/// let root = default_doc();
/// assert_eq!(root.text, "中心主题");
/// assert!(root.children.is_empty());
/// ```
pub fn default_doc() -> MindNode {
    MindNode { text: "中心主题".to_string(), children: Vec::new() }
}
