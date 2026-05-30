//! 设计生成提示词与执行路由辅助。
//!
//! 本模块集中维护：
//! - 主题资料与 prompt 片段
//! - 端类型推断与宽度策略
//! - 执行器路由与流式生成入口

use crate::app::task::{
    TaskExecutorBackend, TaskLogStream, build_executor_command,
    execute_gateway_prompt_with_streaming, execute_task_command_with_streaming,
    legacy_executor_to_task_acp_agent, normalize_task_acp_agent_input,
};
use crate::app::views::design::models::{DesignDoc, VariableDef};
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationPage, DesignGenerationTheme, DesignStyle,
};
use std::collections::HashMap;
use std::sync::mpsc;

const DESIGN_PROMPT_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/design.md"));
const DESIGN_THEME_HALO_PROMPT_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/halo/THEME.md"));
const DESIGN_THEME_HALO_PROMPT_STRUCTURE_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/halo/DESIGN.md"));
const DESIGN_THEME_HALO_PROMPT_THEME_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/halo/theme.json"));
const DESIGN_THEME_HALO_PROMPT_COMPONENTS_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/halo/components.json"));
const DESIGN_THEME_LUNARIS_PROMPT_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/lunaris/THEME.md"));
const DESIGN_THEME_LUNARIS_PROMPT_STRUCTURE_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/lunaris/DESIGN.md"));
const DESIGN_THEME_LUNARIS_PROMPT_THEME_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/lunaris/theme.json"));
const DESIGN_THEME_LUNARIS_PROMPT_COMPONENTS_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/design/lunaris/components.json"
));
const DESIGN_THEME_SHADCN_PROMPT_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/shadcn/THEME.md"));
const DESIGN_THEME_SHADCN_PROMPT_STRUCTURE_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/shadcn/DESIGN.md"));
const DESIGN_THEME_SHADCN_PROMPT_THEME_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/shadcn/theme.json"));
const DESIGN_THEME_SHADCN_PROMPT_COMPONENTS_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/design/shadcn/components.json"
));
const DESIGN_THEME_NITRO_PROMPT_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/nitro/THEME.md"));
const DESIGN_THEME_NITRO_PROMPT_STRUCTURE_DOC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/nitro/DESIGN.md"));
const DESIGN_THEME_NITRO_PROMPT_THEME_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/nitro/theme.json"));
const DESIGN_THEME_NITRO_PROMPT_COMPONENTS_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/design/nitro/components.json"));

pub(super) fn design_model_input_placeholder() -> &'static str {
    "auto / provider/model / ACP 模型 ID"
}

fn design_theme_id(theme: DesignGenerationTheme) -> &'static str {
    match theme {
        DesignGenerationTheme::Shadcn => "shadcn",
        DesignGenerationTheme::Nitro => "nitro",
        DesignGenerationTheme::Halo => "halo",
        DesignGenerationTheme::Lunaris => "lunaris",
    }
}

fn design_theme_json(theme: DesignGenerationTheme) -> &'static str {
    match theme {
        DesignGenerationTheme::Shadcn => DESIGN_THEME_SHADCN_PROMPT_THEME_JSON,
        DesignGenerationTheme::Nitro => DESIGN_THEME_NITRO_PROMPT_THEME_JSON,
        DesignGenerationTheme::Halo => DESIGN_THEME_HALO_PROMPT_THEME_JSON,
        DesignGenerationTheme::Lunaris => DESIGN_THEME_LUNARIS_PROMPT_THEME_JSON,
    }
}

fn design_prompt_contract_doc(theme: DesignGenerationTheme) -> &'static str {
    match theme {
        DesignGenerationTheme::Shadcn => DESIGN_THEME_SHADCN_PROMPT_STRUCTURE_DOC,
        DesignGenerationTheme::Nitro => DESIGN_THEME_NITRO_PROMPT_STRUCTURE_DOC,
        DesignGenerationTheme::Halo => DESIGN_THEME_HALO_PROMPT_STRUCTURE_DOC,
        DesignGenerationTheme::Lunaris => DESIGN_THEME_LUNARIS_PROMPT_STRUCTURE_DOC,
    }
}

fn design_prompt_theme_doc(theme: DesignGenerationTheme) -> &'static str {
    match theme {
        DesignGenerationTheme::Shadcn => DESIGN_THEME_SHADCN_PROMPT_DOC,
        DesignGenerationTheme::Nitro => DESIGN_THEME_NITRO_PROMPT_DOC,
        DesignGenerationTheme::Halo => DESIGN_THEME_HALO_PROMPT_DOC,
        DesignGenerationTheme::Lunaris => DESIGN_THEME_LUNARIS_PROMPT_DOC,
    }
}

fn design_prompt_component_doc(theme: DesignGenerationTheme) -> &'static str {
    match theme {
        DesignGenerationTheme::Shadcn => DESIGN_THEME_SHADCN_PROMPT_COMPONENTS_JSON,
        DesignGenerationTheme::Nitro => DESIGN_THEME_NITRO_PROMPT_COMPONENTS_JSON,
        DesignGenerationTheme::Halo => DESIGN_THEME_HALO_PROMPT_COMPONENTS_JSON,
        DesignGenerationTheme::Lunaris => DESIGN_THEME_LUNARIS_PROMPT_COMPONENTS_JSON,
    }
}

fn design_prompt_reference_summary(theme: DesignGenerationTheme) -> String {
    match theme {
        DesignGenerationTheme::Shadcn => format!(
            "风格速记: {} 结构速记: {} 组件规则索引: {} 风格变量: {}",
            compact_multiline(DESIGN_THEME_SHADCN_PROMPT_DOC),
            compact_multiline(DESIGN_THEME_SHADCN_PROMPT_STRUCTURE_DOC),
            compact_multiline(DESIGN_THEME_SHADCN_PROMPT_COMPONENTS_JSON),
            compact_multiline(DESIGN_THEME_SHADCN_PROMPT_THEME_JSON)
        ),
        DesignGenerationTheme::Nitro => format!(
            "风格速记: {} 结构速记: {} 组件规则索引: {} 风格变量: {}",
            compact_multiline(DESIGN_THEME_NITRO_PROMPT_DOC),
            compact_multiline(DESIGN_THEME_NITRO_PROMPT_STRUCTURE_DOC),
            compact_multiline(DESIGN_THEME_NITRO_PROMPT_COMPONENTS_JSON),
            compact_multiline(DESIGN_THEME_NITRO_PROMPT_THEME_JSON)
        ),
        DesignGenerationTheme::Halo => format!(
            "风格速记: {} 结构速记: {} 组件规则索引: {} 风格变量: {}",
            compact_multiline(DESIGN_THEME_HALO_PROMPT_DOC),
            compact_multiline(DESIGN_THEME_HALO_PROMPT_STRUCTURE_DOC),
            compact_multiline(DESIGN_THEME_HALO_PROMPT_COMPONENTS_JSON),
            compact_multiline(DESIGN_THEME_HALO_PROMPT_THEME_JSON)
        ),
        DesignGenerationTheme::Lunaris => format!(
            "风格速记: {} 结构速记: {} 组件规则索引: {} 风格变量: {}",
            compact_multiline(DESIGN_THEME_LUNARIS_PROMPT_DOC),
            compact_multiline(DESIGN_THEME_LUNARIS_PROMPT_STRUCTURE_DOC),
            compact_multiline(DESIGN_THEME_LUNARIS_PROMPT_COMPONENTS_JSON),
            compact_multiline(DESIGN_THEME_LUNARIS_PROMPT_THEME_JSON)
        ),
    }
}

pub(super) fn design_executor_uses_gateway(executor: TaskExecutorBackend) -> bool {
    matches!(executor, TaskExecutorBackend::Internal | TaskExecutorBackend::OpenCode)
}

pub(super) fn resolve_design_acp_agent(
    executor: TaskExecutorBackend,
    selected_acp_agent: Option<&str>,
) -> Option<String> {
    match executor {
        TaskExecutorBackend::Internal => {
            selected_acp_agent.and_then(normalize_task_acp_agent_input)
        }
        TaskExecutorBackend::OpenCode => legacy_executor_to_task_acp_agent(executor),
        TaskExecutorBackend::Claude | TaskExecutorBackend::Codex => None,
    }
}

fn build_design_generation_session_id(scope: &str) -> String {
    let normalized = scope
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' })
        .collect::<String>();
    let normalized = normalized.trim_matches('-');
    let scope = if normalized.is_empty() { "design" } else { normalized };
    format!("design-{}-{}-{}", scope, crate::app::time::now_ms(), rand::random::<u32>())
}

pub(super) fn execute_design_generation_with_streaming(
    executor: TaskExecutorBackend,
    project_path: &str,
    model: &str,
    prompt: &str,
    acp_agent: Option<String>,
    sender: mpsc::Sender<TaskLogStream>,
    session_scope: &str,
) -> Result<String, String> {
    if design_executor_uses_gateway(executor) {
        return execute_gateway_prompt_with_streaming(
            &build_design_generation_session_id(session_scope),
            project_path,
            model,
            prompt,
            acp_agent,
            Some(&sender),
        );
    }

    let command = build_executor_command(executor, project_path, model, prompt);
    execute_task_command_with_streaming(&command, sender)
}

fn design_style_reference(theme: DesignGenerationTheme) -> String {
    format!(
        "系统已注入 {} 主题设计系统资料（{}）。保持 version=2.6，一个项目只生成一个 .json 文件，文件内用多个页面 frame 组织页面；优先使用 frame + auto layout、design token、slot/reusable 组件与清晰的容器层级，并尽量复用 $--background/$--foreground/$--card/$--border/$--primary/$--sidebar 以及字体变量等命名。",
        design_theme_id(theme),
        theme.description()
    )
}

fn design_style_prompt_hint(style: DesignStyle) -> &'static str {
    match style {
        DesignStyle::Default => "保持中性平衡，避免过度装饰，重点保证层级清晰与可读性。",
        DesignStyle::Minimalist => "减少装饰元素，使用克制留白与简洁容器，强调信息密度控制。",
        DesignStyle::Modern => "强调现代感与结构秩序，使用清晰分区、适度圆角与统一节奏。",
        DesignStyle::Business => "强调专业可信、稳重克制，突出信息可靠与转化动作。",
        DesignStyle::Creative => "允许更具表现力的版式与视觉节奏，但保持信息结构可读。",
        DesignStyle::Retro => "加入复古语义元素与配色倾向，但避免影响交互与可读性。",
        DesignStyle::Tech => "突出科技感、组件秩序与数据感表达，保持清晰层级与节奏。",
        DesignStyle::Elegant => "使用细腻间距、精致排版和柔和层次，避免视觉噪声。",
        DesignStyle::Vibrant => "提高视觉活力与色彩对比，但要控制噪声并保持一致语义。",
        DesignStyle::Dark => "优先暗色层级和对比可读性，控制高亮面积与信息聚焦。",
    }
}

fn design_layout_constraints_prompt() -> &'static str {
    "布局硬约束：\
1) 模块左右间距要更宽，默认桌面端模块容器左右至少 24px，移动端至少 16px；\
2) 模块过高时必须拉开上下间距并增加页面总高度，避免模块互相挤压；\
3) 模块与其子元素不能超出页面可视宽度，若桌面内容密度高可适度加宽页面 frame；\
4) 移动端与 APP 页面宽度要克制，避免过宽导致单手阅读困难；\
5) 画布高度需随内容自动增大，禁止固定高度导致裁切、溢出或重叠；\
6) 细节要统一：圆角、描边、阴影、字号、间距节奏和组件命名必须跨模块一致。"
}

fn infer_design_generation_device(user_prompt: &str) -> DesignGenerationDevice {
    let normalized = user_prompt.to_ascii_lowercase();
    if user_prompt.contains("移动")
        || user_prompt.contains("手机")
        || user_prompt.contains("小程序")
        || normalized.contains("app")
        || normalized.contains("iphone")
        || normalized.contains("android")
        || normalized.contains("mobile")
    {
        DesignGenerationDevice::MobileApp
    } else if user_prompt.contains("平板")
        || normalized.contains("tablet")
        || normalized.contains("ipad")
    {
        DesignGenerationDevice::Tablet
    } else if user_prompt.contains("桌面")
        || user_prompt.contains("PC")
        || user_prompt.contains("pc")
        || normalized.contains("desktop")
        || normalized.contains("web")
    {
        DesignGenerationDevice::DesktopWeb
    } else {
        DesignGenerationDevice::DesktopWeb
    }
}

pub(super) fn resolve_design_generation_device(
    selected_device: DesignGenerationDevice,
    user_prompt: &str,
) -> DesignGenerationDevice {
    if selected_device == DesignGenerationDevice::Auto {
        infer_design_generation_device(user_prompt)
    } else {
        selected_device
    }
}

fn design_device_width_hint(device: DesignGenerationDevice) -> &'static str {
    match device {
        DesignGenerationDevice::Auto => {
            "目标端类型：自动识别。默认按桌面端并保留响应式：桌面 1200-1440px，平板 768-1024px，移动端 360-430px。"
        }
        DesignGenerationDevice::DesktopWeb => {
            "目标端类型：PC 桌面。页面 frame 宽度建议 1200-1440px，容器内容宽度建议 1120-1320px。"
        }
        DesignGenerationDevice::MobileApp => {
            "目标端类型：移动端 / APP。页面 frame 宽度建议 360-430px，容器内容宽度建议 328-398px。"
        }
        DesignGenerationDevice::Tablet => {
            "目标端类型：平板。页面 frame 宽度建议 768-1024px，容器内容宽度建议 704-960px。"
        }
    }
}

pub(super) fn design_plan_page_width(device: DesignGenerationDevice) -> f32 {
    match device {
        DesignGenerationDevice::Auto => 420.0,
        DesignGenerationDevice::DesktopWeb => 1280.0,
        DesignGenerationDevice::MobileApp => 390.0,
        DesignGenerationDevice::Tablet => 900.0,
    }
}

pub(super) fn compact_multiline(text: &str) -> String {
    text.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>().join(" ")
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let truncated = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        format!("{}...", truncated.trim_end())
    } else {
        truncated
    }
}

pub(super) fn format_plan_parse_error(error: &str, raw: &str) -> String {
    let compact = compact_multiline(raw);
    let snippet =
        if compact.is_empty() { "<空输出>".to_string() } else { truncate_chars(&compact, 320) };
    format!("页面计划解析失败：{}；原始输出片段（截断）：{}", error, snippet)
}

fn load_design_reference_doc(theme: DesignGenerationTheme) -> Option<DesignDoc> {
    serde_json::from_str::<DesignDoc>(design_theme_json(theme)).ok()
}

pub(super) fn extract_theme_reference_summary(theme: DesignGenerationTheme) -> String {
    let theme_summary = compact_multiline(design_prompt_theme_doc(theme));
    let component_summary = compact_multiline(design_prompt_component_doc(theme));
    let reference_summary = design_prompt_reference_summary(theme);
    format!(
        "{} 主题说明: {} 组件索引: {} 主题参考: {}",
        design_style_reference(theme),
        theme_summary,
        component_summary,
        reference_summary
    )
}

pub(super) fn design_reference_tokens_and_theme(
    theme: DesignGenerationTheme,
) -> (HashMap<String, VariableDef>, Option<serde_json::Value>) {
    let Some(doc) = load_design_reference_doc(theme) else {
        let fallback = match theme {
            DesignGenerationTheme::Lunaris => serde_json::json!({ "Mode": "Dark" }),
            _ => serde_json::json!({ "Mode": "Light" }),
        };
        return (HashMap::new(), Some(fallback));
    };

    let fallback = match theme {
        DesignGenerationTheme::Lunaris => serde_json::json!({ "Mode": "Dark" }),
        _ => serde_json::json!({ "Mode": "Light" }),
    };
    (
        doc.variables,
        doc.theme.map(|theme| serde_json::json!({ "Mode": theme.mode })).or(Some(fallback)),
    )
}

fn extract_design_contract_summary(theme: DesignGenerationTheme) -> String {
    let guide = compact_multiline(design_prompt_contract_doc(theme));
    let prompt = compact_multiline(DESIGN_PROMPT_DOC);
    format!("设计结构约束: {}\n提示词工作流: {}", guide, prompt)
}

pub(super) fn build_design_generation_prompt(
    user_prompt: &str,
    executor: TaskExecutorBackend,
    theme: DesignGenerationTheme,
    style: DesignStyle,
    selected_device: DesignGenerationDevice,
) -> String {
    let resolved_device = resolve_design_generation_device(selected_device, user_prompt);
    let device_width_hint = design_device_width_hint(resolved_device);
    if executor == TaskExecutorBackend::OpenCode || executor == TaskExecutorBackend::Internal {
        return format!(
            "你是 .json 项目设计规划助手。按“页面任务并行”流程工作：先输出页面结构计划，随后系统会按页面并行生成整页内容。\n\n当前选择：\n- 主题: {}\n- 视觉风格: {}\n- 端类型宽度策略: {}\n\n风格参考：\n{}\n\n风格细化要求：\n{}\n\n布局安全约束：\n{}\n\n设计资产摘要：\n{}\n\n主题文档摘要：\n{}\n\n主题组件索引摘要：\n{}\n\n主题参考摘要（系统注入，可直接参考 token/reusable 命名）：\n{}\n\n要求：\n1. 输出只允许 JSON，禁止 Markdown 代码块和解释文字\n2. 至少输出 3 个页面，每个页面至少 3 个模块\n3. 页面命名必须是真实导航页名\n4. 模块描述必须说明内容目的、信息类型、交互重点\n5. 严格兼容 version=2.6 与字段约束\n6. 优先沿用注入资料中的 token 和 reusable 组件命名风格\n7. 不要虚构品牌名、价格、案例或指标\n8. 不要输出 module_doc，页面计划只需要页面与模块描述\n9. 页面 status 建议设置为 queued，模块 status 建议设置为 queued\n10. 对高模块必须增加页面高度与模块间距，避免上下挤压与重叠\n11. 严格保证模块不越界，桌面可适度扩页面宽度，移动端与 APP 需控制宽度\n12. 只输出 JSON，结构如下：\n{{\n  \"summary\": \"一句话概括整体站点结构\",\n  \"pages\": [\n    {{\n      \"title\": \"页面标题\",\n      \"objective\": \"页面目标\",\n      \"status\": \"queued\",\n      \"modules\": [\n        {{\n          \"title\": \"模块标题\",\n          \"description\": \"模块描述\",\n          \"status\": \"queued\"\n        }}\n      ]\n    }}\n  ]\n}}\n\n用户需求：\n{}",
            theme.label(),
            style.label(),
            device_width_hint,
            extract_theme_reference_summary(theme),
            design_style_prompt_hint(style),
            design_layout_constraints_prompt(),
            extract_design_contract_summary(theme),
            compact_multiline(design_prompt_theme_doc(theme)),
            compact_multiline(design_prompt_component_doc(theme)),
            design_prompt_reference_summary(theme),
            user_prompt.trim()
        );
    }

    format!(
        "你是 .json 项目设计规划助手。按“页面任务并行”流程工作：先输出页面结构计划，随后系统会按页面并行生成整页内容。\n\n当前选择：\n- 主题: {}\n- 视觉风格: {}\n- 端类型宽度策略: {}\n\n风格参考：\n{}\n\n风格细化要求：\n{}\n\n布局安全约束：\n{}\n\n设计资产摘要：\n{}\n\n主题文档摘要：\n{}\n\n主题组件索引摘要：\n{}\n\n主题参考摘要（系统注入，可直接参考 token/reusable 命名）：\n{}\n\n要求：\n1. 严格依据用户需求组织页面和模块，不要擅自虚构具体公司名称、价格、案例或数据\n2. 输出结果必须符合“一个项目一个 .pen，.json 内多个页面”的结构思路\n3. 优先按真实网站信息架构组织页面，不要输出任务拆解式页面\n4. 至少输出 3 个页面；每个页面至少 3 个模块\n5. 页面命名要是用户真实会访问的页面，例如首页、定价页、案例页、帮助中心、文章详情页、商品详情页等\n6. 不要输出 module_doc，只输出标题、描述与状态\n7. 页面 status 与模块 status 默认使用 queued\n8. 输出结果要兼容 version=2.6 结构与 token 约束\n9. 优先复用注入资料中的 token 与 reusable 组件命名，不要发明全新的 token 体系\n10. 对模块间距、页面宽高、响应式宽度做显式约束，避免挤压、越界和裁切\n11. 细节风格必须跨页面/模块一致（圆角、字重、间距、描边、阴影、组件命名）\n12. 不要输出 Markdown 代码块，不要解释\n13. 只输出 JSON，对象结构如下：\n{{\n  \"summary\": \"一句话概括整体站点结构\",\n  \"pages\": [\n    {{\n      \"title\": \"页面标题\",\n      \"objective\": \"页面目标\",\n      \"status\": \"queued\",\n      \"modules\": [\n        {{\n          \"title\": \"模块标题\",\n          \"description\": \"模块描述\",\n          \"status\": \"queued\"\n        }}\n      ]\n    }}\n  ]\n}}\n\n用户需求：\n{}",
        theme.label(),
        style.label(),
        device_width_hint,
        extract_theme_reference_summary(theme),
        design_style_prompt_hint(style),
        design_layout_constraints_prompt(),
        extract_design_contract_summary(theme),
        compact_multiline(design_prompt_theme_doc(theme)),
        compact_multiline(design_prompt_component_doc(theme)),
        design_prompt_reference_summary(theme),
        user_prompt.trim()
    )
}

pub(super) fn build_page_generation_prompt(
    user_brief: &str,
    executor: TaskExecutorBackend,
    theme: DesignGenerationTheme,
    style: DesignStyle,
    selected_device: DesignGenerationDevice,
    page: &DesignGenerationPage,
    generated_pages_summary: &str,
) -> String {
    let brief = if user_brief.trim().is_empty() { "当前设计需求" } else { user_brief.trim() };
    let resolved_device = resolve_design_generation_device(selected_device, brief);
    let device_width_hint = design_device_width_hint(resolved_device);
    let page_modules_layout_summary = page
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| {
            format!("{}. {} | {}", index + 1, module.title.trim(), module.description.trim())
        })
        .collect::<Vec<_>>()
        .join("\n");
    if executor == TaskExecutorBackend::OpenCode || executor == TaskExecutorBackend::Internal {
        return format!(
            "你是 .json 设计文档生成助手。当前按页面回调生成：每次只生成一个完整页面，页面之间可并行。\n\n项目需求：{}\n\n当前选择：\n- 主题: {}\n- 视觉风格: {}\n- 端类型宽度策略: {}\n\n风格参考：\n{}\n\n风格细化要求：\n{}\n\n布局安全约束：\n{}\n\n设计资产摘要：\n{}\n\n主题文档摘要：\n{}\n\n主题组件索引摘要：\n{}\n\n主题参考摘要（系统注入，可直接参考 token/reusable 命名）：\n{}\n\n当前页面模块清单（需在该页面内完整体现）：\n{}\n\n已完成页面摘要（用于跨页面一致性）：\n{}\n\n要求：\n1. 输出 version=2.6\n2. 根 children 必须且只能有 1 个 frame，代表当前页面\n3. 该页面 frame 内应完整组织当前页面所有模块内容，不要拆成多个顶层页面\n4. 输出必须匹配当前页面标题与页面目标，不要偏离页面语义\n5. 优先复用 token/reusable 组件命名\n6. 维持 frame + auto layout 的层级\n7. 参考已完成页面的层级、变量和命名风格，保证跨页面一致\n8. 页面内模块左右间距要更宽并保持统一，避免视觉拥挤\n9. 对高内容自动增高页面画布，禁止内容裁切、堆叠或重叠\n10. 禁止横向越界；桌面可适度扩页面宽度，移动端与 APP 不可过宽\n11. 注意细节一致性：圆角、阴影、描边、字号、字重、间距、组件命名语义\n12. 不要输出 Markdown，不要解释\n13. 不要虚构品牌、价格、客户或统计数据\n\n页面标题: {}\n页面目标: {}",
            brief,
            theme.label(),
            style.label(),
            device_width_hint,
            extract_theme_reference_summary(theme),
            design_style_prompt_hint(style),
            design_layout_constraints_prompt(),
            extract_design_contract_summary(theme),
            compact_multiline(design_prompt_theme_doc(theme)),
            compact_multiline(design_prompt_component_doc(theme)),
            design_prompt_reference_summary(theme),
            page_modules_layout_summary,
            generated_pages_summary,
            page.title,
            page.objective
        );
    }

    format!(
        "你是 .json 设计文档生成助手。当前项目只有一个 .json 文件，所有页面都组织在同一个项目文档里。请根据页面和模块描述，只输出一个合法 JSON 设计文档，结构兼容 .json / DesignDoc，作为这个项目 .json 中某个页面的完整内容。\n\n项目需求：{}\n\n当前选择：\n- 主题: {}\n- 视觉风格: {}\n- 端类型宽度策略: {}\n\n风格参考：\n{}\n\n风格细化要求：\n{}\n\n布局安全约束：\n{}\n\n设计资产摘要：\n{}\n\n主题文档摘要：\n{}\n\n主题组件索引摘要：\n{}\n\n主题参考摘要（系统注入，可直接参考 token/reusable 命名）：\n{}\n\n当前页面模块清单（需在该页面内完整体现）：\n{}\n\n已完成页面摘要（用于跨页面一致性）：\n{}\n\n要求：\n1. 输出 version=2.6\n2. 使用所选主题对应的 design system 风格与 token 命名\n3. 整个输出只允许一个顶层页面 frame：根 children 必须且只能有 1 个 frame\n4. 这个唯一的顶层 frame 就代表当前页面本身；当前生成整页，不是单模块，不是整站\n5. 输出必须匹配当前页面标题与目标，不要替换为其他页面内容\n6. 顶层 frame 内部按模块清单组织页面结构\n7. 严格依据用户需求和页面描述生成，不要自行补充固定业务模板、虚构品牌名或写死业务数据\n8. 优先使用注入资料里的变量 token、auto layout、slot/reusable 组件命名和页面容器结构\n9. 页面内容要有明确信息层级、可替换文案占位和可继续扩展容器结构\n10. 如需要按钮、标签、卡片、表单、导航等基础部件，优先沿用注入主题 JSON 中已有 reusable 组件命名风格\n11. 参考已完成页面摘要统一视觉层级和组件语义，但避免简单复制\n12. 必须显式考虑页面高度：frame 高度要与内部内容匹配，新增元素后能自然撑开，不要输出会导致内容溢出或裁切的固定高度结构\n13. 页面左右间距需加大并保持一致，避免贴边与挤压\n14. 严格禁止横向越界；桌面端可扩页面宽度，移动端与 APP 不能过宽\n15. 不要输出 Markdown 代码块，不要解释\n\n页面标题: {}\n页面目标: {}",
        brief,
        theme.label(),
        style.label(),
        device_width_hint,
        extract_theme_reference_summary(theme),
        design_style_prompt_hint(style),
        design_layout_constraints_prompt(),
        extract_design_contract_summary(theme),
        compact_multiline(design_prompt_theme_doc(theme)),
        compact_multiline(design_prompt_component_doc(theme)),
        design_prompt_reference_summary(theme),
        page_modules_layout_summary,
        generated_pages_summary,
        page.title,
        page.objective
    )
}
