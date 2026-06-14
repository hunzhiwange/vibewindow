//! 内存 CLI 模块的单元测试
//!
//! 本模块包含对 `parse_category` 和 `truncate_content` 等辅助函数的测试用例。
//! 这些测试验证了内存分类解析和内容截断的正确性。

use super::*;
use crate::app::agent::config::{Config, StorageProviderConfig};
use crate::app::agent::memory::MemoryEntry;
use std::sync::Mutex;

/// 测试 `parse_category` 函数对已知分类变体的解析
///
/// # 验证点
/// - 能够正确解析标准分类：core、daily、conversation
/// - 解析不区分大小写（CORE 应解析为 Core）
/// - 解析能够忽略首尾空白字符（"  Daily  " 应解析为 Daily）
#[test]
fn parse_category_known_variants() {
    assert_eq!(parse_category("core"), MemoryCategory::Core);
    assert_eq!(parse_category("daily"), MemoryCategory::Daily);
    assert_eq!(parse_category("conversation"), MemoryCategory::Conversation);
    assert_eq!(parse_category("CORE"), MemoryCategory::Core);
    assert_eq!(parse_category("  Daily  "), MemoryCategory::Daily);
}

/// 测试 `parse_category` 函数对自定义分类的回退处理
///
/// # 验证点
/// - 无法识别的分类字符串应作为自定义分类（Custom）返回
/// - 自定义分类保留原始字符串值
#[test]
fn parse_category_custom_fallback() {
    assert_eq!(parse_category("project_notes"), MemoryCategory::Custom("project_notes".into()));
    assert_eq!(parse_category("  Mixed Case  "), MemoryCategory::Custom("mixed case".into()));
}

/// 测试 `truncate_content` 函数对短文本的处理
///
/// # 验证点
/// - 当文本长度小于最大长度时，应保持原样返回
#[test]
fn truncate_content_short_text_unchanged() {
    assert_eq!(truncate_content("hello", 10), "hello");
}

/// 测试 `truncate_content` 函数对长文本的截断处理
///
/// # 验证点
/// - 当文本长度超过最大长度时，应进行截断
/// - 截断后的文本应以 "..." 结尾
/// - 截断后的总字符数不应超过指定的最大长度
#[test]
fn truncate_content_long_text_truncated() {
    let result = truncate_content("this is a very long string", 10);
    assert!(result.ends_with("..."));
    assert!(result.chars().count() <= 10);
}

/// 测试 `truncate_content` 函数对多行文本的处理
///
/// # 验证点
/// - 多行文本应只使用第一行内容
/// - 第一行提取后再应用截断规则
#[test]
fn truncate_content_multiline_uses_first_line() {
    assert_eq!(truncate_content("first\nsecond", 20), "first");
}

/// 测试 `truncate_content` 函数对空字符串的处理
///
/// # 验证点
/// - 空字符串应原样返回，不发生任何变化
#[test]
fn truncate_content_empty_string() {
    assert_eq!(truncate_content("", 10), "");
}

#[test]
fn truncate_content_handles_tiny_limits() {
    assert_eq!(truncate_content("abcdef", 0), "...");
    assert_eq!(truncate_content("abcdef", 2), "...");
    assert_eq!(truncate_content("abcdef", 3), "...");
}

#[test]
fn memory_commands_round_trip_through_json() {
    let commands = [
        MemoryCommands::List {
            category: Some("core".into()),
            session: Some("session-1".into()),
            limit: 25,
            offset: 5,
        },
        MemoryCommands::Get { key: "alpha".into() },
        MemoryCommands::Stats,
        MemoryCommands::Clear { key: Some("alpha".into()), category: None, yes: true },
    ];

    for command in commands {
        let json = serde_json::to_string(&command).expect("command serializes");
        let decoded: MemoryCommands = serde_json::from_str(&json).expect("command deserializes");
        assert_eq!(decoded, command);
    }
}

#[test]
fn parse_category_blank_values_become_empty_custom_category() {
    assert_eq!(parse_category("   "), MemoryCategory::Custom(String::new()));
}

#[test]
fn truncate_content_exact_boundary_is_unchanged() {
    assert_eq!(truncate_content("abcdef", 6), "abcdef");
    assert_eq!(truncate_content("abcdef", 5), "ab...");
}

#[test]
fn create_cli_memory_rejects_disabled_backend() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut config = Config::default();
    config.memory.backend = "none".into();
    config.workspace_dir = tmp.path().to_path_buf();

    let error = match create_cli_memory(&config) {
        Ok(_) => panic!("disabled memory should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("disabled"));
}

#[test]
fn create_cli_memory_prefers_storage_provider_override() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut config = Config::default();
    config.workspace_dir = tmp.path().to_path_buf();
    config.memory.backend = "none".into();
    config.storage.provider.config = StorageProviderConfig {
        provider: "postgres".into(),
        db_url: None,
        ..StorageProviderConfig::default()
    };

    let error = match create_cli_memory(&config) {
        Ok(_) => panic!("postgres override without db_url should fail"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(message.contains("postgres"));
    assert!(!message.contains("disabled"));
}

#[test]
fn create_cli_memory_rejects_mariadb_without_connection_settings() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut config = Config::default();
    config.workspace_dir = tmp.path().to_path_buf();
    config.memory.backend = "mariadb".into();

    let error = match create_cli_memory(&config) {
        Ok(_) => panic!("mariadb without db_url should fail"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(message.contains("mariadb") || message.contains("memory-mariadb"));
}

#[derive(Default)]
struct StubMemory {
    entries: Mutex<Vec<MemoryEntry>>,
    forgotten: Mutex<Vec<String>>,
}

impl StubMemory {
    fn with_entries(entries: Vec<MemoryEntry>) -> Self {
        Self { entries: Mutex::new(entries), forgotten: Mutex::new(Vec::new()) }
    }

    fn forgotten_keys(&self) -> Vec<String> {
        self.forgotten.lock().unwrap().clone()
    }
}

fn entry(key: &str, category: MemoryCategory, session_id: Option<&str>) -> MemoryEntry {
    MemoryEntry {
        id: format!("id-{key}"),
        key: key.to_string(),
        content: format!("content for {key}"),
        category,
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        session_id: session_id.map(str::to_string),
        score: None,
    }
}

#[async_trait::async_trait]
impl Memory for StubMemory {
    fn name(&self) -> &str {
        "stub"
    }

    async fn store(
        &self,
        _key: &str,
        _content: &str,
        _category: MemoryCategory,
        _session_id: Option<&str>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn recall(
        &self,
        _query: &str,
        _limit: usize,
        _session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(Vec::new())
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        Ok(self.entries.lock().unwrap().iter().find(|entry| entry.key == key).cloned())
    }

    async fn list(
        &self,
        category: Option<&MemoryCategory>,
        session_id: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryEntry>> {
        let entries = self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| match category {
                Some(category) => &entry.category == category,
                None => true,
            })
            .filter(|entry| match session_id {
                Some(session_id) => entry.session_id.as_deref() == Some(session_id),
                None => true,
            })
            .cloned()
            .collect();
        Ok(entries)
    }

    async fn forget(&self, key: &str) -> anyhow::Result<bool> {
        self.forgotten.lock().unwrap().push(key.to_string());
        Ok(self.entries.lock().unwrap().iter().any(|entry| entry.key == key))
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.entries.lock().unwrap().len())
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn clear_key_deletes_exact_match_without_prompt_when_confirmed() {
    let memory = StubMemory::with_entries(vec![
        entry("alpha", MemoryCategory::Core, None),
        entry("alphabet", MemoryCategory::Daily, None),
    ]);

    handle_clear_key(&memory, "alpha", true).await.unwrap();

    assert_eq!(memory.forgotten_keys(), vec!["alpha"]);
}

#[tokio::test]
async fn clear_key_deletes_unique_prefix_match() {
    let memory = StubMemory::with_entries(vec![
        entry("alpha", MemoryCategory::Core, None),
        entry("beta", MemoryCategory::Daily, None),
    ]);

    handle_clear_key(&memory, "alp", true).await.unwrap();

    assert_eq!(memory.forgotten_keys(), vec!["alpha"]);
}

#[tokio::test]
async fn clear_key_leaves_missing_prefix_untouched() {
    let memory = StubMemory::with_entries(vec![entry("alpha", MemoryCategory::Core, None)]);

    handle_clear_key(&memory, "missing", true).await.unwrap();

    assert!(memory.forgotten_keys().is_empty());
}

#[tokio::test]
async fn clear_key_leaves_ambiguous_prefix_untouched() {
    let memory = StubMemory::with_entries(vec![
        entry("alpha", MemoryCategory::Core, None),
        entry("alphabet", MemoryCategory::Daily, None),
    ]);

    handle_clear_key(&memory, "alph", true).await.unwrap();

    assert!(memory.forgotten_keys().is_empty());
}
