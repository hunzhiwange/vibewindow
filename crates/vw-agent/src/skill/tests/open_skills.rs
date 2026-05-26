//! # Open Skills 模块测试
//!
//! 本模块包含 Open Skills（开放技能）功能的单元测试。
//!
//! ## 主要功能
//!
//! - 测试 `open_skills_enabled` 配置解析的优先级逻辑
//! - 测试 `open_skills_dir` 目录解析的优先级逻辑
//! - 测试从本地 Open Skills 目录加载技能的功能
//!
//! ## 测试覆盖
//!
//! 1. **启用状态解析**：验证环境变量、配置文件、默认值的优先级顺序
//! 2. **目录解析**：验证技能目录的解析逻辑（环境变量 > 配置 > 默认位置）
//! 3. **技能加载**：验证从 Open Skills 目录加载技能时不依赖网络请求

use super::super::*;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// 获取 Open Skills 测试环境的全局互斥锁。
///
/// 由于部分测试需要修改环境变量，使用此互斥锁确保这些测试串行执行，
/// 避免因并发访问环境变量导致测试不稳定（test flakiness）。
///
/// # 返回值
///
/// 返回一个静态生命周期 的 `Mutex<()>` 引用，用于同步测试执行。
fn open_skills_env_lock() -> &'static Mutex<()> {
    /// 全局环境变量测试锁，确保涉及环境变量修改的测试串行执行。
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

/// 环境变量守护器，用于在测试中安全地临时修改环境变量。
///
/// 当 `EnvVarGuard` 被创建时，它会移除指定的环境变量（保存原始值）；
/// 当它被销毁（drop）时，会自动恢复原始值。这确保了测试的隔离性，
/// 不会影响其他测试或系统状态。
///
/// # 示例
///
/// ```ignore
/// {
///     let _guard = EnvVarGuard::unset("MY_VAR");
///     // 在此作用域内，MY_VAR 不存在
/// } // 离开作用域后，MY_VAR 自动恢复
/// ```
struct EnvVarGuard {
    /// 环境变量的键名
    key: &'static str,
    /// 环境变量的原始值（如果存在）
    original: Option<String>,
}

impl EnvVarGuard {
    /// 创建一个环境变量守护器，移除并保存指定环境变量的当前值。
    ///
    /// # 参数
    ///
    /// * `key` - 要移除的环境变量名称
    ///
    /// # 返回值
    ///
    /// 返回一个 `EnvVarGuard` 实例，它会在被销毁时自动恢复原始值。
    ///
    /// # 安全性
    ///
    /// 此方法使用 `unsafe` 块调用 `std::env::remove_var`，
    /// 因为在 Rust 中修改环境变量不是线程安全的操作。
    /// 在测试中使用时，应配合 `open_skills_env_lock()` 确保串行执行。
    fn unset(key: &'static str) -> Self {
        let original = std::env::var(key).ok();
        unsafe {
            std::env::remove_var(key);
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    /// 当守护器被销毁时，自动恢复环境变量的原始值。
    ///
    /// 如果原始值存在，则恢复；如果原始值不存在，则确保变量被移除。
    ///
    /// # 安全性
    ///
    /// 使用 `unsafe` 块调用 `std::env::set_var` 和 `remove_var`，
    /// 因为环境变量修改在多线程环境中不是安全的操作。
    fn drop(&mut self) {
        unsafe {
            if let Some(value) = &self.original {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}

/// 测试 `open_skills_enabled` 配置解析的优先级逻辑。
///
/// 验证启用状态的解析顺序：
/// 1. **环境变量**（`VIBEWINDOW_OPEN_SKILLS_ENABLED`）优先级最高
/// 2. **配置文件**次之
/// 3. **默认值**（`false`）优先级最低
///
/// # 测试用例
///
/// - `None, None` → 默认禁用（`false`）
/// - `Some(true), None` → 配置启用
/// - `Some(true), Some("0")` → 环境变量 `"0"` 覆盖配置（禁用）
/// - `Some(false), Some("yes")` → 环境变量 `"yes"` 覆盖配置（启用）
/// - `Some(true), Some("invalid")` → 无效环境变量值时回退到配置（启用）
/// - `Some(false), Some("invalid")` → 无效环境变量值时回退到配置（禁用）
#[test]
fn open_skills_enabled_resolution_prefers_env_then_config_then_default_false() {
    assert!(!open_skills_enabled_from_sources(None, None));
    assert!(open_skills_enabled_from_sources(Some(true), None));
    assert!(!open_skills_enabled_from_sources(Some(true), Some("0")));
    assert!(open_skills_enabled_from_sources(Some(false), Some("yes")));
    assert!(open_skills_enabled_from_sources(Some(true), Some("invalid")));
    assert!(!open_skills_enabled_from_sources(Some(false), Some("invalid")));
}

/// 测试 `resolve_open_skills_dir` 目录解析的优先级逻辑。
///
/// 验证 Open Skills 目录的解析顺序：
/// 1. **环境变量**（`VIBEWINDOW_OPEN_SKILLS_DIR`）优先级最高
/// 2. **配置文件**中的 `open_skills_dir` 次之
/// 3. **默认位置**（`{home}/open-skills`）优先级最低
///
/// # 测试用例
///
/// - 环境变量有效 → 返回环境变量指定的路径
/// - 环境变量为空白 → 回退到配置文件路径
/// - 两者都未设置 → 使用 `{home}/open-skills` 默认路径
/// - 所有来源都为 `None` → 返回 `None`
#[test]
fn resolve_open_skills_dir_resolution_prefers_env_then_config_then_home() {
    let home = Path::new("/tmp/home-dir");
    assert_eq!(
        resolve_open_skills_dir_from_sources(
            Some("/tmp/env-skills"),
            Some("/tmp/config"),
            Some(home)
        ),
        Some(PathBuf::from("/tmp/env-skills"))
    );
    assert_eq!(
        resolve_open_skills_dir_from_sources(Some("   "), Some("/tmp/config-skills"), Some(home)),
        Some(PathBuf::from("/tmp/config-skills"))
    );
    assert_eq!(
        resolve_open_skills_dir_from_sources(None, None, Some(home)),
        Some(PathBuf::from("/tmp/home-dir/open-skills"))
    );
    assert_eq!(resolve_open_skills_dir_from_sources(None, None, None), None);
}

/// 测试从配置加载技能时能够本地读取 Open Skills 目录，无需网络请求。
///
/// 此测试验证 `load_skills_with_config` 函数能够：
/// 1. 正确读取配置中的 `open_skills_enabled` 和 `open_skills_dir` 设置
/// 2. 从本地文件系统加载技能定义，而不是从远程仓库克隆
/// 3. 正确解析 `SKILL.md` 文件并提取技能名称
///
/// # 测试场景
///
/// 1. 清理相关环境变量，确保测试隔离
/// 2. 创建临时目录结构：
///    - workspace 目录
///    - open-skills 目录，包含 `skills/http_request/SKILL.md`
/// 3. 配置 `open_skills_enabled = true` 和 `open_skills_dir`
/// 4. 验证加载的技能数量和名称正确
#[test]
fn load_skills_with_config_reads_open_skills_dir_without_network() {
    // 获取全局锁，确保环境变量修改不会与其他测试冲突
    let _env_guard = open_skills_env_lock().lock().unwrap();
    // 移除可能干扰测试的环境变量
    let _enabled_guard = EnvVarGuard::unset("VIBEWINDOW_OPEN_SKILLS_ENABLED");
    let _dir_guard = EnvVarGuard::unset("VIBEWINDOW_OPEN_SKILLS_DIR");

    // 创建临时测试目录
    let dir = tempfile::tempdir().unwrap();
    let workspace_dir = dir.path().join("workspace");
    fs::create_dir_all(workspace_dir.join("skills")).unwrap();

    // 创建 Open Skills 目录结构
    let open_skills_dir = dir.path().join("open-skills-local");
    fs::create_dir_all(open_skills_dir.join("skills/http_request")).unwrap();

    // 写入 Open Skills 仓库的元数据文件
    fs::write(open_skills_dir.join("README.md"), "# open skills\n").unwrap();
    fs::write(open_skills_dir.join("CONTRIBUTING.md"), "# contribution guide\n").unwrap();

    // 写入技能定义文件
    fs::write(
        open_skills_dir.join("skills/http_request/SKILL.md"),
        "# HTTP request\nFetch API responses.\n",
    )
    .unwrap();

    // 配置技能运行时参数
    let mut config = SkillRuntimeConfig::default();
    config.workspace_dir = workspace_dir.clone();
    config.skills.open_skills_enabled = true;
    config.skills.open_skills_dir = Some(open_skills_dir.to_string_lossy().to_string());

    // 加载技能并验证结果
    let skills = load_skills_with_config(&workspace_dir, &config);
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "http_request");
    assert_ne!(skills[0].name, "CONTRIBUTING");
}
