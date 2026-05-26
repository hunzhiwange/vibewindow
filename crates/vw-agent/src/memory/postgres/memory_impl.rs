//! PostgreSQL 记忆后端的 `Memory` trait 实现。
//!
//! 本文件只负责把通用记忆操作映射到 PostgreSQL SQL 语句。阻塞式数据库客户端统一放入
//! `spawn_blocking`，避免占用异步运行时工作线程。

use super::super::traits::{Memory, MemoryCategory, MemoryEntry};
use super::PostgresMemory;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Memory for PostgresMemory {
    /// 返回该记忆后端的稳定名称。
    fn name(&self) -> &str {
        "postgres"
    }

    /// 写入或更新一条记忆。
    ///
    /// # 参数
    ///
    /// - `key`: 记忆的唯一键。
    /// - `content`: 需要保存的正文。
    /// - `category`: 记忆分类。
    /// - `session_id`: 可选会话范围，存在时用于按会话过滤。
    ///
    /// # 错误
    ///
    /// 当阻塞任务调度失败或 PostgreSQL 写入失败时返回错误。
    async fn store(
        &self,
        key: &str,
        content: &str,
        category: MemoryCategory,
        session_id: Option<&str>,
    ) -> Result<()> {
        let client = self.client.clone();
        let qualified_table = self.qualified_table.clone();
        let key = key.to_string();
        let content = content.to_string();
        let category = Self::category_to_str(&category);
        let sid = session_id.map(str::to_string);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let now = Utc::now();
            let mut client = client.lock();
            // 表名由构造阶段完成限定与校验，参数值仍全部使用绑定参数，避免把用户输入拼进 SQL。
            let stmt = format!(
                "
                INSERT INTO {qualified_table}
                    (id, key, content, category, created_at, updated_at, session_id)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (key) DO UPDATE SET
                    content = EXCLUDED.content,
                    category = EXCLUDED.category,
                    updated_at = EXCLUDED.updated_at,
                    session_id = EXCLUDED.session_id
                "
            );

            let id = Uuid::new_v4().to_string();
            client.execute(&stmt, &[&id, &key, &content, &category, &now, &now, &sid])?;
            Ok(())
        })
        .await?
    }

    /// 按查询文本召回记忆。
    ///
    /// # 参数
    ///
    /// - `query`: 用于匹配 key 和 content 的关键词；空字符串表示只按时间取最近记录。
    /// - `limit`: 最大返回条数。
    /// - `session_id`: 可选会话过滤条件。
    ///
    /// # 错误
    ///
    /// 当查询执行失败、行转换失败或阻塞任务失败时返回错误。
    async fn recall(
        &self,
        query: &str,
        limit: usize,
        session_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>> {
        let client = self.client.clone();
        let qualified_table = self.qualified_table.clone();
        let query = query.trim().to_string();
        let sid = session_id.map(str::to_string);

        tokio::task::spawn_blocking(move || -> Result<Vec<MemoryEntry>> {
            let mut client = client.lock();
            let stmt = format!(
                "
                SELECT id, key, content, category, created_at, session_id,
                       (
                         CASE WHEN key ILIKE '%' || $1 || '%' THEN 2.0 ELSE 0.0 END +
                         CASE WHEN content ILIKE '%' || $1 || '%' THEN 1.0 ELSE 0.0 END
                       ) AS score
                FROM {qualified_table}
                WHERE ($2::TEXT IS NULL OR session_id = $2)
                  AND ($1 = '' OR key ILIKE '%' || $1 || '%' OR content ILIKE '%' || $1 || '%')
                ORDER BY score DESC, updated_at DESC
                LIMIT $3
                "
            );

            #[allow(clippy::cast_possible_wrap)]
            let limit_i64 = limit as i64;
            let rows = client.query(&stmt, &[&query, &sid, &limit_i64])?;
            rows.iter().map(Self::row_to_entry).collect::<Result<Vec<MemoryEntry>>>()
        })
        .await?
    }

    /// 按唯一键读取单条记忆。
    ///
    /// # 错误
    ///
    /// 当数据库查询或结果转换失败时返回错误。
    async fn get(&self, key: &str) -> Result<Option<MemoryEntry>> {
        let client = self.client.clone();
        let qualified_table = self.qualified_table.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || -> Result<Option<MemoryEntry>> {
            let mut client = client.lock();
            let stmt = format!(
                "
                SELECT id, key, content, category, created_at, session_id
                FROM {qualified_table}
                WHERE key = $1
                LIMIT 1
                "
            );

            let row = client.query_opt(&stmt, &[&key])?;
            row.as_ref().map(Self::row_to_entry).transpose()
        })
        .await?
    }

    /// 列出指定分类和会话范围内的记忆。
    ///
    /// # 参数
    ///
    /// - `category`: 可选分类过滤。
    /// - `session_id`: 可选会话过滤。
    ///
    /// # 错误
    ///
    /// 当数据库查询或结果转换失败时返回错误。
    async fn list(
        &self,
        category: Option<&MemoryCategory>,
        session_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>> {
        let client = self.client.clone();
        let qualified_table = self.qualified_table.clone();
        let category = category.map(Self::category_to_str);
        let sid = session_id.map(str::to_string);

        tokio::task::spawn_blocking(move || -> Result<Vec<MemoryEntry>> {
            let mut client = client.lock();
            let stmt = format!(
                "
                SELECT id, key, content, category, created_at, session_id
                FROM {qualified_table}
                WHERE ($1::TEXT IS NULL OR category = $1)
                  AND ($2::TEXT IS NULL OR session_id = $2)
                ORDER BY updated_at DESC
                "
            );

            let category_ref = category.as_deref();
            let session_ref = sid.as_deref();
            let rows = client.query(&stmt, &[&category_ref, &session_ref])?;
            rows.iter().map(Self::row_to_entry).collect::<Result<Vec<MemoryEntry>>>()
        })
        .await?
    }

    /// 删除指定键对应的记忆，并返回是否实际删除了记录。
    ///
    /// # 错误
    ///
    /// 当删除语句执行失败时返回错误。
    async fn forget(&self, key: &str) -> Result<bool> {
        let client = self.client.clone();
        let qualified_table = self.qualified_table.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || -> Result<bool> {
            let mut client = client.lock();
            let stmt = format!("DELETE FROM {qualified_table} WHERE key = $1");
            let deleted = client.execute(&stmt, &[&key])?;
            Ok(deleted > 0)
        })
        .await?
    }

    /// 统计当前 PostgreSQL 记忆表中的记录数。
    ///
    /// # 错误
    ///
    /// 当查询失败或数据库返回无法转换为 `usize` 的计数时返回错误。
    async fn count(&self) -> Result<usize> {
        let client = self.client.clone();
        let qualified_table = self.qualified_table.clone();

        tokio::task::spawn_blocking(move || -> Result<usize> {
            let mut client = client.lock();
            let stmt = format!("SELECT COUNT(*) FROM {qualified_table}");
            let count: i64 = client.query_one(&stmt, &[])?.get(0);
            let count = usize::try_from(count)
                .context("PostgreSQL returned a negative memory count")?;
            Ok(count)
        })
        .await?
    }

    /// 执行轻量健康检查。
    ///
    /// 返回 `false` 表示连接不可用、阻塞任务失败或数据库未响应。
    async fn health_check(&self) -> bool {
        let client = self.client.clone();
        tokio::task::spawn_blocking(move || client.lock().simple_query("SELECT 1").is_ok())
            .await
            .unwrap_or(false)
    }
}

#[cfg(test)]
#[path = "memory_impl_tests.rs"]
mod memory_impl_tests;
