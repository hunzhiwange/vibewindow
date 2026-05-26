//! # Embedding Provider 测试模块
//!
//! 本模块包含向量嵌入提供者的单元测试和集成测试。
//!
//! ## 测试覆盖范围
//!
//! - **NoopEmbedding 测试**：验证空实现嵌入提供者的行为
//! - **工厂函数测试**：验证 `create_embedding_provider` 对各种配置的正确处理
//! - **OpenAiEmbedding 测试**：验证 OpenAI 兼容提供者的 URL 构建和维度配置
//!
//! ## 测试分类
//!
//! | 类别 | 测试数量 | 说明 |
//! |------|----------|------|
//! | NoopEmbedding | 4 | 空操作嵌入测试 |
//! | 工厂函数 | 7 | 提供者创建测试 |
//! | OpenAI URL | 6 | 端点 URL 构建测试 |

use super::*;

/// 测试 NoopEmbedding 的名称和维度属性
///
/// # 验证点
///
/// - `name()` 返回 `"none"`
/// - `dimensions()` 返回 `0`
#[test]
fn noop_name() {
    let p = NoopEmbedding;
    assert_eq!(p.name(), "none");
    assert_eq!(p.dimensions(), 0);
}

/// 测试 NoopEmbedding 的 embed 方法返回空结果
///
/// # 验证点
///
/// - 对非空输入数组，`embed()` 应返回空向量
/// - 不应返回错误
#[tokio::test]
async fn noop_embed_returns_empty() {
    let p = NoopEmbedding;
    let result = p.embed(&["hello"]).await.unwrap();
    assert!(result.is_empty());
}

/// 测试工厂函数创建 none 提供者
///
/// # 验证点
///
/// - 当 provider_type 为 `"none"` 时，应返回 NoopEmbedding
/// - 名称应为 `"none"`
#[test]
fn factory_none() {
    let p = create_embedding_provider("none", None, "model", 1536);
    assert_eq!(p.name(), "none");
}

/// 测试工厂函数创建 OpenAI 提供者
///
/// # 验证点
///
/// - 当 provider_type 为 `"openai"` 时，应返回 OpenAI 兼容提供者
/// - 应正确配置 API key、模型名称和维度
#[test]
fn factory_openai() {
    let p = create_embedding_provider("openai", Some("key"), "text-embedding-3-small", 1536);
    assert_eq!(p.name(), "openai");
    assert_eq!(p.dimensions(), 1536);
}

/// 测试工厂函数通过 OpenRouter 创建 OpenAI 兼容提供者
///
/// # 验证点
///
/// - 当 provider_type 为 `"openrouter"` 时，底层使用 OpenAI 兼容实现
/// - OpenRouter 是一个 AI API 聚合服务，支持多种模型提供商
/// - 名称应为 `"openai"`（底层实现）
#[test]
fn factory_openrouter() {
    let p = create_embedding_provider(
        "openrouter",
        Some("sk-or-test"),
        "openai/text-embedding-3-small",
        1536,
    );
    assert_eq!(p.name(), "openai");
    assert_eq!(p.dimensions(), 1536);
}

/// 测试工厂函数创建自定义 URL 的提供者
///
/// # 验证点
///
/// - 支持通过 `custom:<url>` 格式指定自定义 API 端点
/// - 适用于本地部署或私有服务的场景
/// - 底层使用 OpenAI 兼容实现
#[test]
fn factory_custom_url() {
    let p = create_embedding_provider("custom:http://localhost:1234", None, "model", 768);
    assert_eq!(p.name(), "openai");
    assert_eq!(p.dimensions(), 768);
}

/// 测试 NoopEmbedding 的 embed_one 方法返回错误
///
/// # 验证点
///
/// - `embed_one()` 应返回错误而非空向量
/// - 区别于 `embed()` 方法的行为
#[tokio::test]
async fn noop_embed_one_returns_error() {
    let p = NoopEmbedding;
    let result = p.embed_one("hello").await;
    assert!(result.is_err());
}

/// 测试 NoopEmbedding 处理空批次
///
/// # 验证点
///
/// - 对空数组调用 `embed()` 应返回空向量
/// - 不应产生错误
#[tokio::test]
async fn noop_embed_empty_batch() {
    let p = NoopEmbedding;
    let result = p.embed(&[]).await.unwrap();
    assert!(result.is_empty());
}

/// 测试 NoopEmbedding 处理多个文本
///
/// # 验证点
///
/// - 无论输入多少个文本，NoopEmbedding 始终返回空结果
/// - 这是空操作提供者的预期行为
#[tokio::test]
async fn noop_embed_multiple_texts() {
    let p = NoopEmbedding;
    let result = p.embed(&["a", "b", "c"]).await.unwrap();
    assert!(result.is_empty());
}

/// 测试工厂函数处理空字符串返回 Noop
///
/// # 验证点
///
/// - 当 provider_type 为空字符串时，应回退到 NoopEmbedding
/// - 这是安全的默认行为
#[test]
fn factory_empty_string_returns_noop() {
    let p = create_embedding_provider("", None, "model", 1536);
    assert_eq!(p.name(), "none");
}

/// 测试工厂函数处理未知提供者返回 Noop
///
/// # 验证点
///
/// - 当 provider_type 为不支持的值时，应回退到 NoopEmbedding
/// - 例如 "cohere" 目前不被支持
#[test]
fn factory_unknown_provider_returns_noop() {
    let p = create_embedding_provider("cohere", None, "model", 1536);
    assert_eq!(p.name(), "none");
}

/// 测试工厂函数处理 custom 前缀但空 URL 的情况
///
/// # 验证点
///
/// - 当 provider_type 为 `"custom:"` 但 URL 为空时
/// - 仍应创建 OpenAI 兼容提供者
#[test]
fn factory_custom_empty_url() {
    let p = create_embedding_provider("custom:", None, "model", 768);
    assert_eq!(p.name(), "openai");
}

/// 测试工厂函数在没有 API key 时创建 OpenAI 提供者
///
/// # 验证点
///
/// - 即使没有提供 API key，也应正确创建提供者实例
/// - API key 可能通过环境变量等方式在运行时注入
#[test]
fn factory_openai_no_api_key() {
    let p = create_embedding_provider("openai", None, "text-embedding-3-small", 1536);
    assert_eq!(p.name(), "openai");
    assert_eq!(p.dimensions(), 1536);
}

/// 测试 OpenAI 提供者自动移除 base URL 尾部斜杠
///
/// # 验证点
///
/// - 当 base_url 以 `/` 结尾时，应自动移除
/// - 避免后续拼接时出现双斜杠问题
#[test]
fn openai_trailing_slash_stripped() {
    let p = OpenAiEmbedding::new("https://api.openai.com/", "key", "model", 1536);
    assert_eq!(p.base_url, "https://api.openai.com");
}

/// 测试 OpenAI 提供者支持自定义维度
///
/// # 验证点
///
/// - 维度参数应正确存储和返回
/// - 支持非标准维度值（如 384）
#[test]
fn openai_dimensions_custom() {
    let p = OpenAiEmbedding::new("http://localhost", "k", "m", 384);
    assert_eq!(p.dimensions(), 384);
}

/// 测试 OpenRouter 端点的 embeddings URL 构建
///
/// # 验证点
///
/// - OpenRouter 的 API 端点已包含 `/api/v1`
/// - 不应重复添加 `/v1` 前缀
/// - 最终 URL 应为 `https://openrouter.ai/api/v1/embeddings`
#[test]
fn embeddings_url_openrouter() {
    let p = OpenAiEmbedding::new(
        "https://openrouter.ai/api/v1",
        "key",
        "openai/text-embedding-3-small",
        1536,
    );
    assert_eq!(p.embeddings_url(), "https://openrouter.ai/api/v1/embeddings");
}

/// 测试标准 OpenAI API 端点的 embeddings URL 构建
///
/// # 验证点
///
/// - 标准 OpenAI API 端点不含版本前缀
/// - 应自动添加 `/v1/embeddings` 路径
#[test]
fn embeddings_url_standard_openai() {
    let p = OpenAiEmbedding::new("https://api.openai.com", "key", "model", 1536);
    assert_eq!(p.embeddings_url(), "https://api.openai.com/v1/embeddings");
}

/// 测试 base URL 已包含 /v1 时不重复添加
///
/// # 验证点
///
/// - 当 base_url 以 `/v1` 结尾时，直接追加 `/embeddings`
/// - 避免生成 `/v1/v1/embeddings` 这样的错误路径
#[test]
fn embeddings_url_base_with_v1_no_duplicate() {
    let p = OpenAiEmbedding::new("https://api.example.com/v1", "key", "model", 1536);
    assert_eq!(p.embeddings_url(), "https://api.example.com/v1/embeddings");
}

/// 测试非标准 API 路径的 embeddings URL 构建
///
/// # 验证点
///
/// - 当 base_url 包含非 `/v1` 的路径时（如 `/api/coding/v3`）
/// - 直接在原路径后追加 `/embeddings`
/// - 不进行任何版本路径转换
#[test]
fn embeddings_url_non_v1_api_path_uses_raw_suffix() {
    let p = OpenAiEmbedding::new("https://api.example.com/api/coding/v3", "key", "model", 1536);
    assert_eq!(p.embeddings_url(), "https://api.example.com/api/coding/v3/embeddings");
}

/// 测试完整端点 URL 直接作为 base_url 的情况
///
/// # 验证点
///
/// - 当 base_url 已经是完整的 embeddings 端点时
/// - `embeddings_url()` 应原样返回，不做任何修改
/// - 支持用户指定完全自定义的端点地址
#[test]
fn embeddings_url_custom_full_endpoint() {
    let p =
        OpenAiEmbedding::new("https://my-api.example.com/api/v2/embeddings", "key", "model", 1536);
    assert_eq!(p.embeddings_url(), "https://my-api.example.com/api/v2/embeddings");
}
