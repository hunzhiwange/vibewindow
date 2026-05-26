//! 思维导图标签页管理模块
//!
//! 本模块负责管理思维导图应用的标签页生命周期，包括创建、激活、关闭等操作。
//! 提供标签页ID生成、顶部导航栏同步以及持久化触发等功能。
//!
//! # 主要功能
//!
//! - **标签页ID管理**: 生成唯一的思维导图标签页标识符
//! - **标签页生命周期**: 创建新标签页、关闭现有标签页
//! - **UI同步**: 确保顶部导航栏与当前激活的标签页保持一致
//! - **状态持久化**: 在标签页变更后自动触发状态保存

use crate::app::{App, AppTab, Screen};
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::MindMapTab;
use iced::Task;

use super::persist::persist;

/// 将思维导图标签页ID转换为应用级别的标签页ID
///
/// 思维导图标签页在顶部导航栏中显示时需要使用统一格式的ID，
/// 此函数负责添加 `mindmap:` 前缀以确保ID在应用范围内的唯一性。
///
/// # 参数
///
/// - `tab_id`: 思维导图内部的标签页ID
///
/// # 返回值
///
/// 返回格式为 `"mindmap:{tab_id}"` 的应用级标签页ID
///
/// # 示例
///
/// ```ignore
/// let app_tab_id = mindmap_app_tab_id("mindmap-1");
/// assert_eq!(app_tab_id, "mindmap:mindmap-1");
/// ```
pub(super) fn mindmap_app_tab_id(tab_id: &str) -> String {
    format!("mindmap:{tab_id}")
}

/// 生成下一个可用的思维导图标签页ID
///
/// 通过递增数字后缀的方式生成唯一的标签页ID。如果生成的ID已存在，
/// 则继续递增直到找到未被使用的ID。
///
/// # 参数
///
/// - `app`: 应用状态引用，用于检查现有标签页ID
///
/// # 返回值
///
/// 返回一个未被使用的标签页ID，格式为 `"mindmap-{n}"`
///
/// # 算法说明
///
/// 1. 初始编号为当前标签页数量加1
/// 2. 循环检查编号是否已被使用
/// 3. 如已使用则递增编号继续尝试
/// 4. 找到可用ID后立即返回
fn next_tab_id(app: &App) -> String {
    let base = "mindmap".to_string();
    let mut n = app.mindmap_tabs.len() + 1;
    loop {
        let id = format!("{base}-{n}");
        // 检查该ID是否已被现有标签页使用
        if !app.mindmap_tabs.iter().any(|t| t.id == id) {
            return id;
        }
        n += 1;
    }
}

/// 确保顶部导航栏显示并激活指定的思维导图标签页
///
/// 此函数负责在顶部导航栏中管理思维导图标签页的显示状态。
/// 如果标签页已存在则更新其属性，否则创建新标签页，最后将其设为激活状态。
///
/// # 参数
///
/// - `app`: 可变的应用状态引用
/// - `id`: 思维导图标签页ID（不含前缀）
/// - `title`: 标签页显示标题
///
/// # 行为说明
///
/// 1. 移除可能存在的通用"apps"标签页（避免重复）
/// 2. 查找是否已有对应的应用级标签页
///    - 如存在：更新标题、屏幕类型和项目路径
///    - 如不存在：创建新的应用级标签页
/// 3. 将该标签页设为当前激活标签页
/// 4. 切换应用屏幕到思维导图工具界面
pub(super) fn ensure_top_tab(app: &mut App, id: &str, title: &str) {
    // 移除通用的"apps"标签页，为思维导图专用标签页腾出位置
    if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
        app.open_tabs.remove(pos);
    }

    // 生成应用级标签页ID
    let app_id = mindmap_app_tab_id(id);

    // 查找并更新现有标签页，或创建新标签页
    if let Some(t) = app.open_tabs.iter_mut().find(|t| t.id == app_id) {
        // 标签页已存在，更新其属性
        t.title = title.to_string();
        t.screen = Screen::MindMapTool;
        t.project_path = None;
    } else {
        // 标签页不存在，创建新的应用级标签页
        app.open_tabs.push(AppTab {
            id: mindmap_app_tab_id(id),
            title: title.to_string(),
            screen: Screen::MindMapTool,
            project_path: None,
        });
    }

    // 激活该标签页并切换屏幕
    app.active_tab_id = Some(mindmap_app_tab_id(id));
    app.screen = Screen::MindMapTool;
}

/// 创建新的空白思维导图标签页
///
/// 生成一个新的思维导图标签页，包含默认的空文档内容。
/// 创建后会自动在顶部导航栏显示并激活该标签页，同时触发状态持久化。
///
/// # 参数
///
/// - `app`: 可变的应用状态引用
///
/// # 行为说明
///
/// 1. 生成唯一的标签页ID
/// 2. 构建标签页标题（格式：`思维导图 {n}`）
/// 3. 创建包含默认文档的新标签页
/// 4. 将标签页添加到思维导图标签页列表
/// 5. 设置为当前激活的标签页
/// 6. 触发状态持久化
/// 7. 在顶部导航栏中显示并激活该标签页
pub(super) fn sync_top_tabs(app: &mut App) {
    let tabs: Vec<(String, String)> =
        app.mindmap_tabs.iter().map(|t: &MindMapTab| (t.id.clone(), t.title.clone())).collect();

    for (id, title) in tabs {
        ensure_top_tab(app, &id, &title);
    }

    if let Some(active_id) = app
        .mindmap_active_tab_id
        .as_ref()
        .cloned()
        .or_else(|| app.mindmap_tabs.first().map(|t| t.id.clone()))
    {
        app.active_tab_id = Some(mindmap_app_tab_id(&active_id));
        app.screen = Screen::MindMapTool;
    }
}

pub fn new_blank_tab(app: &mut App) -> Task<crate::app::Message> {
    // 生成唯一的标签页ID
    let id = next_tab_id(app);

    // 构建标签页标题，序号基于当前标签页数量
    let title = format!("思维导图 {}", app.mindmap_tabs.len() + 1);

    // 创建新的思维导图标签页，使用默认空文档
    let tab = MindMapTab::new(id.clone(), title, None, model::default_doc());

    // 添加到标签页列表并设为激活状态
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some(id);

    // 触发状态持久化
    let persist_task = persist(app);

    // 获取当前激活标签页的信息用于顶部导航栏同步
    let top: Option<(String, String)> = app
        .mindmap_active_tab_id
        .as_ref()
        .cloned()
        .and_then(|active| app.mindmap_tabs.iter().find(|t: &&MindMapTab| t.id == active))
        .map(|t: &MindMapTab| (t.id.clone(), t.title.clone()));

    // 在顶部导航栏中确保该标签页可见并激活
    if let Some((id, title)) = top {
        ensure_top_tab(app, &id, &title);
    }

    persist_task
}

/// 关闭指定的思维导图标签页
///
/// 从思维导图标签页列表和顶部导航栏中移除指定标签页，
/// 并智能处理激活标签页的切换逻辑。
///
/// # 参数
///
/// - `app`: 可变的应用状态引用
/// - `id`: 要关闭的标签页ID
///
/// # 行为说明
///
/// 1. 从思维导图标签页列表中移除指定标签页
/// 2. 从顶部导航栏中移除对应的应用级标签页
/// 3. 处理激活标签页状态：
///    - 如果没有剩余标签页：清空激活标签页ID
///    - 如果关闭的是当前激活标签页：激活最后一个标签页
///    - 否则：保持当前激活状态不变
/// 4. 触发状态持久化
pub fn close_tab(app: &mut App, id: &str) -> Task<crate::app::Message> {
    // 从思维导图标签页列表中移除
    if let Some(pos) = app.mindmap_tabs.iter().position(|t| t.id == id) {
        app.mindmap_tabs.remove(pos);
    }

    // 从顶部导航栏中移除对应的应用级标签页
    let app_id = mindmap_app_tab_id(id);
    if let Some(pos) = app.open_tabs.iter().position(|t| t.id == app_id) {
        app.open_tabs.remove(pos);
    }

    // 智能处理激活标签页状态
    if app.mindmap_tabs.is_empty() {
        // 没有剩余标签页，清空激活状态
        app.mindmap_active_tab_id = None;
    } else if app.mindmap_active_tab_id.as_deref() == Some(id) {
        // 关闭的是当前激活标签页，切换到最后一个标签页
        app.mindmap_active_tab_id = app.mindmap_tabs.last().map(|t| t.id.clone());
    }

    // 触发状态持久化
    persist(app)
}
