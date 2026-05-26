//! 系统设置 - 定时任务配置界面组件
//!
//! 本模块提供 Cron 定时任务子系统的配置界面，允许用户通过图形界面配置以下内容：
//! - 是否启用 Cron 子系统（`enabled`）
//! - 每个任务保留的执行历史记录数量（`max_run_history`）
//!
//! # 功能特性
//!
//! - 提供直观的开关和滑块控件
//! - 内置详细的帮助文档模态框
//! - 实时预览配置变更
//! - 错误提示显示
//!
//! # 配置文件位置
//!
//! 配置保存于 `~/.vibewindow/vibewindow.json` 的 `cron` 字段中。

use crate::app::components::system_settings_common::{
    SETTINGS_LABEL_WIDTH, settings_checkbox_style, settings_error_banner, settings_help_button,
    settings_muted_text_style, settings_page_intro, settings_panel, settings_section_card,
    settings_value_badge,
};
use crate::app::{App, Message, message};
use iced::widget::{checkbox, column, container, row, slider, text};
use iced::{Alignment, Element, Length};

fn field_row<'a>(
    label: &'static str,
    description: &'static str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            column![
                text(label).size(13),
                text(description).size(11).style(settings_muted_text_style),
            ]
            .spacing(4)
            .width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
            container(control.into()).width(Length::Fill),
        ]
        .spacing(22)
        .align_y(Alignment::Center),
    )
    .padding([14, 0])
    .width(Length::Fill)
    .into()
}

/// 渲染 定时任务配置设置界面
///
/// 创建一个完整的 定时任务配置面板，包括启用开关、历史记录数量配置，
/// 以及可展开的帮助文档模态框。
///
/// # 参数
///
/// - `app`: 应用程序状态的不可变引用，从中读取当前的 定时任务配置设置
///
/// # 返回值
///
/// 返回一个 Iced `Element`，包含完整的配置界面组件树
///
/// # UI 结构
///
/// 1. **标题区域**: 显示"定时任务配置"标题和帮助按钮
/// 2. **副标题**: 说明配置文件位置
/// 3. **启用开关**: 复选框控制是否启用 Cron 子系统
/// 4. **历史保留数量**: 滑块控制每个任务保留的历史记录数
/// 5. **帮助模态框**: 当 `show_help_modal` 为 true 时显示详细帮助文档
/// 6. **错误提示**: 当 `save_error` 存在时显示错误信息
///
/// # 示例
///
/// ```ignore
/// let element = system_settings_cron::view(&app);
/// // 在设置页面中使用此 element
/// ```
pub fn view(app: &App) -> Element<'_, Message> {
    let s = &app.cron_settings;
    let help_btn = settings_help_button(Message::Settings(message::SettingsMessage::CronHelpOpen));

    let enabled_row = field_row(
        "启用",
        "控制是否启用 Cron 子系统。",
        checkbox(s.enabled)
            .label("开启 Cron 子系统")
            .on_toggle(|v| Message::Settings(message::SettingsMessage::CronEnabledToggled(v)))
            .style(settings_checkbox_style),
    );

    let history_slider = slider(1.0..=500.0, s.max_run_history as f32, |v| {
        Message::Settings(message::SettingsMessage::CronMaxRunHistoryChanged(v.round() as u32))
    })
    .width(Length::Fixed(280.0));

    let history_row = field_row(
        "历史保留",
        "每个任务保留的执行历史记录数量。",
        row![history_slider, settings_value_badge(format!("{} 条", s.max_run_history))]
            .spacing(16)
            .align_y(Alignment::Center),
    );

    let hint_row = row![
        container(text("")).width(Length::Fixed(SETTINGS_LABEL_WIDTH)),
        text("建议范围 20-200。调大可追溯更多历史，调小可减少存储占用。")
            .size(12)
            .style(settings_muted_text_style),
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    let mut col = column![
        row![
            container(settings_page_intro("定时任务配置", "配置 Cron 子系统的开关与历史保留策略。"))
                .width(Length::Fill),
            help_btn
        ]
        .align_y(Alignment::Start),
        settings_section_card("基础行为", "控制 Cron 子系统是否运行与历史留存数量。"),
        settings_panel(column![enabled_row, history_row].spacing(0)),
        hint_row,
    ]
    .spacing(16)
    .width(Length::Fill);

    if let Some(err) = &s.save_error {
        col = col.push(settings_error_banner(err));
    }

    col.into()
}

pub fn view_overlays<'a>(app: &'a App, dialog: Element<'a, Message>) -> Element<'a, Message> {
    let s = &app.cron_settings;
    if !s.show_help_modal {
        return dialog;
    }

    let help_text = r#"定时任务配置说明

一、作用
- cron 是定时任务子系统的全局开关与历史保留策略。
- 你创建的定时任务（cron_add / cron_update 等）都依赖这里的全局配置。
- 本节不定义具体任务表达式，只控制"是否运行"与"历史留存"。

二、字段含义
1) enabled
- 类型：布尔（true / false）
- 含义：是否启用 Cron 子系统。
- true：调度器会持续扫描并触发到期任务。
- false：Cron 调度器不启动，已有任务不会按计划执行（但任务定义仍保留在工作区）。

2) max_run_history
- 类型：整数
- 含义：每个任务最多保留多少条执行历史记录。
- 默认：50。
- 当历史条数超过上限时，会按"旧记录优先清理"的方式裁剪，避免记录无限增长。

三、典型示例

示例 A：默认推荐（大多数场景）
{
  "cron": {
    "enabled": true,
    "max_run_history": 50
  }
}

示例 B：高审计需求（保留更多历史）
{
  "cron": {
    "enabled": true,
    "max_run_history": 200
  }
}

示例 C：临时停用调度器（任务定义不删除）
{
  "cron": {
    "enabled": false,
    "max_run_history": 50
  }
}

四、配置建议
1) 开发/测试环境
- max_run_history 可设为 20-50，便于快速迭代并减少噪音。

2) 生产环境
- 建议保持 enabled=true，避免遗漏计划任务。
- max_run_history 可按审计需求设为 50-200。

3) 存储与追溯平衡
- 数值越大，历史追溯越充分，但磁盘占用与查询成本会增加。
- 如果只关心近期失败，可使用较小值（如 30-50）。

五、排查建议（任务没按时跑）
1) 先确认 cron.enabled=true。
2) 检查任务本身是否 enabled，表达式是否有效。
3) 检查主进程是否在运行，避免误以为后台仍在调度。
4) 查看最近 run history，确认是"未触发"还是"触发后执行失败"。
5) 修改 ~/.vibewindow/vibewindow.json 后，重启应用再观察日志。
"#;

    crate::app::components::system_settings_common::with_settings_help_modal(
        app,
        dialog,
        "定时任务配置帮助",
        help_text,
        Message::Settings(message::SettingsMessage::CronHelpClose),
    )
}
