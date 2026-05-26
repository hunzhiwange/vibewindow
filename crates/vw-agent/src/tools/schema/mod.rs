//! JSON Schema 清洗与验证工具
//!
//! 用于 LLM 工具调用的 JSON Schema 兼容性处理。不同 LLM 提供商对 JSON Schema 的支持程度不同，
//! 本模块提供 Schema 标准化功能，以改善跨提供商兼容性。
//!
//! ## 模块功能
//!
//! 1. 根据提供商策略移除不支持的关键字
//! 2. 从 `$defs` 和 `definitions` 解析本地 `$ref` 引用
//! 3. 将字面量 `anyOf` / `oneOf` 联合类型展平为 `enum`
//! 4. 从联合类型和 `type` 数组中剥离可空变体
//! 5. 将 `const` 转换为单值 `enum`
//! 6. 检测循环引用并安全停止递归
//!
//! # 示例
//!
//! ```rust
//! use serde_json::json;
//! use vibewindow::crate::app::agent::tools::schema::SchemaCleanr;
//!
//! let dirty_schema = json!({
//!     "type": "object",
//!     "properties": {
//!         "name": {
//!             "type": "string",
//!             "minLength": 1,  // Gemini 拒绝此关键字
//!             "pattern": "^[a-z]+$"  // Gemini 拒绝此关键字
//!         },
//!         "age": {
//!             "$ref": "#/$defs/Age"  // 需要解析
//!         }
//!     },
//!     "$defs": {
//!         "Age": {
//!             "type": "integer",
//!             "minimum": 0  // Gemini 拒绝此关键字
//!         }
//!     }
//! });
//!
//! let cleaned = SchemaCleanr::clean_for_gemini(dirty_schema);
//!
//! // 结果：
//! // {
//! //   "type": "object",
//! //   "properties": {
//! //     "name": { "type": "string" },
//! //     "age": { "type": "integer" }
//! //   }
//! // }
//! ```
//!
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};

/// Gemini 在工具 Schema 中拒绝的关键字列表。
///
/// 这些关键字在 Gemini API 中不被支持，需要在清洗过程中移除。
pub const GEMINI_UNSUPPORTED_KEYWORDS: &[&str] = &[
    // Schema 组合关键字
    "$ref",
    "$schema",
    "$id",
    "$defs",
    "definitions",
    // 属性约束关键字
    "additionalProperties",
    "patternProperties",
    // 字符串约束关键字
    "minLength",
    "maxLength",
    "pattern",
    "format",
    // 数值约束关键字
    "minimum",
    "maximum",
    "multipleOf",
    // 数组约束关键字
    "minItems",
    "maxItems",
    "uniqueItems",
    // 对象约束关键字
    "minProperties",
    "maxProperties",
    // 非标准关键字
    "examples", // OpenAPI 关键字，非 JSON Schema 标准
];

/// 清洗过程中应保留的元数据关键字。
const SCHEMA_META_KEYS: &[&str] = &["description", "title", "default"];

/// 不同 LLM 提供商的 Schema 清洗策略。
///
/// 不同提供商对 JSON Schema 的支持程度不同，需要使用相应的策略进行清洗。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleaningStrategy {
    /// Gemini (Google AI / Vertex AI) - 最严格的策略
    ///
    /// 移除大部分约束关键字，仅保留核心类型信息。
    Gemini,
    /// Anthropic Claude - 中等宽松策略
    ///
    /// 移除引用相关关键字，保留大部分约束。
    Anthropic,
    /// OpenAI GPT - 最宽松策略
    ///
    /// 几乎保留所有 JSON Schema 特性。
    OpenAI,
    /// 保守策略：仅移除通用不支持的关键字
    ///
    /// 适用于未知或通用提供商。
    Conservative,
}

impl CleaningStrategy {
    /// 获取当前策略不支持的关键字列表。
    ///
    /// # 返回值
    ///
    /// 返回一个字符串切片数组，包含该策略下需要移除的所有关键字。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use vibewindow::app::agent::tools::schema::CleaningStrategy;
    ///
    /// let keywords = CleaningStrategy::Gemini.unsupported_keywords();
    /// assert!(keywords.contains(&"$ref"));
    /// ```
    pub fn unsupported_keywords(self) -> &'static [&'static str] {
        match self {
            Self::Gemini => GEMINI_UNSUPPORTED_KEYWORDS,
            Self::Anthropic => &["$ref", "$defs", "definitions"], // Anthropic 不解析引用
            Self::OpenAI => &[],                                  // OpenAI 最宽松
            Self::Conservative => &["$ref", "$defs", "definitions", "additionalProperties"],
        }
    }
}

/// 针对 LLM 工具调用优化的 JSON Schema 清洗器。
///
/// 提供多种清洗策略，确保 Schema 与不同 LLM 提供商兼容。
/// 支持引用解析、联合类型简化、约束移除等操作。
pub struct SchemaCleanr;

impl SchemaCleanr {
    /// 为 Gemini 兼容性清洗 Schema（最严格）。
    ///
    /// 这是最激进的清洗策略，移除所有 Gemini API 拒绝的关键字。
    ///
    /// # 参数
    ///
    /// * `schema` - 原始 JSON Schema 值
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 JSON Schema，兼容 Gemini API。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use serde_json::json;
    /// use vibewindow::app::agent::tools::schema::SchemaCleanr;
    ///
    /// let schema = json!({"type": "string", "minLength": 1});
    /// let cleaned = SchemaCleanr::clean_for_gemini(schema);
    /// assert_eq!(cleaned, json!({"type": "string"}));
    /// ```
    pub fn clean_for_gemini(schema: Value) -> Value {
        Self::clean(schema, CleaningStrategy::Gemini)
    }

    /// 为 Anthropic 兼容性清洗 Schema。
    ///
    /// # 参数
    ///
    /// * `schema` - 原始 JSON Schema 值
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 JSON Schema，兼容 Anthropic API。
    pub fn clean_for_anthropic(schema: Value) -> Value {
        Self::clean(schema, CleaningStrategy::Anthropic)
    }

    /// 为 OpenAI 兼容性清洗 Schema（最宽松）。
    ///
    /// # 参数
    ///
    /// * `schema` - 原始 JSON Schema 值
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 JSON Schema，兼容 OpenAI API。
    pub fn clean_for_openai(schema: Value) -> Value {
        Self::clean(schema, CleaningStrategy::OpenAI)
    }

    /// 使用指定策略清洗 Schema。
    ///
    /// # 参数
    ///
    /// * `schema` - 原始 JSON Schema 值
    /// * `strategy` - 清洗策略，决定移除哪些关键字
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 JSON Schema。
    pub fn clean(schema: Value, strategy: CleaningStrategy) -> Value {
        // 提取 $defs 用于引用解析
        let defs = if let Some(obj) = schema.as_object() {
            Self::extract_defs(obj)
        } else {
            HashMap::new()
        };

        Self::clean_with_defs(schema, &defs, strategy, &mut HashSet::new())
    }

    /// 验证 Schema 是否适合 LLM 工具调用。
    ///
    /// 检查 Schema 是否包含必要的字段，并发出潜在问题的警告。
    ///
    /// # 参数
    ///
    /// * `schema` - 要验证的 JSON Schema 引用
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - Schema 有效
    /// * `Err(...)` - Schema 无效或缺少必要字段
    ///
    /// # 错误
    ///
    /// 如果 Schema 不是对象或缺少必需的 `type` 字段，返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use serde_json::json;
    /// use vibewindow::app::agent::tools::schema::SchemaCleanr;
    ///
    /// let valid_schema = json!({"type": "object", "properties": {}});
    /// assert!(SchemaCleanr::validate(&valid_schema).is_ok());
    ///
    /// let invalid_schema = json!({"properties": {}});
    /// assert!(SchemaCleanr::validate(&invalid_schema).is_err());
    /// ```
    pub fn validate(schema: &Value) -> anyhow::Result<()> {
        let obj = schema.as_object().ok_or_else(|| anyhow::anyhow!("Schema 必须是一个对象"))?;

        // 必须包含 'type' 字段
        if !obj.contains_key("type") {
            anyhow::bail!("Schema 缺少必需的 'type' 字段");
        }

        // 如果类型是 'object'，应该有 'properties' 字段
        if let Some(Value::String(t)) = obj.get("type") {
            if t == "object" && !obj.contains_key("properties") {
                tracing::warn!("对象 Schema 没有 'properties' 字段可能导致问题");
            }
        }

        Ok(())
    }

    // --------------------------------------------------------------------
    // 内部实现
    // --------------------------------------------------------------------

    /// 提取 $defs 和 definitions 到扁平映射中，用于引用解析。
    ///
    /// 支持两种定义格式：
    /// - `$defs`：JSON Schema 2019-09+ 标准
    /// - `definitions`：JSON Schema draft-07 标准
    ///
    /// # 参数
    ///
    /// * `obj` - Schema 对象
    ///
    /// # 返回值
    ///
    /// 返回定义名称到定义值的映射。
    fn extract_defs(obj: &Map<String, Value>) -> HashMap<String, Value> {
        let mut defs = HashMap::new();

        // 从 $defs 提取（JSON Schema 2019-09+）
        if let Some(Value::Object(defs_obj)) = obj.get("$defs") {
            for (key, value) in defs_obj {
                defs.insert(key.clone(), value.clone());
            }
        }

        // 从 definitions 提取（JSON Schema draft-07）
        if let Some(Value::Object(defs_obj)) = obj.get("definitions") {
            for (key, value) in defs_obj {
                defs.insert(key.clone(), value.clone());
            }
        }

        defs
    }

    /// 递归清洗 Schema 值。
    ///
    /// 根据值类型分派到相应的清洗逻辑：
    /// - 对象：调用 `clean_object` 处理
    /// - 数组：递归清洗每个元素
    /// - 其他类型：原样返回
    ///
    /// # 参数
    ///
    /// * `schema` - 要清洗的 Schema 值
    /// * `defs` - 定义映射，用于引用解析
    /// * `strategy` - 清洗策略
    /// * `ref_stack` - 引用栈，用于检测循环引用
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 Schema 值。
    fn clean_with_defs(
        schema: Value,
        defs: &HashMap<String, Value>,
        strategy: CleaningStrategy,
        ref_stack: &mut HashSet<String>,
    ) -> Value {
        match schema {
            Value::Object(obj) => Self::clean_object(obj, defs, strategy, ref_stack),
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|v| Self::clean_with_defs(v, defs, strategy, ref_stack))
                    .collect(),
            ),
            other => other,
        }
    }

    /// 清洗对象 Schema。
    ///
    /// 执行以下操作：
    /// 1. 解析 `$ref` 引用
    /// 2. 简化 `anyOf`/`oneOf` 联合类型
    /// 3. 移除不支持的关键字
    /// 4. 转换 `const` 为 `enum`
    /// 5. 清理 `type` 数组中的 null
    /// 6. 递归清洗嵌套 Schema
    ///
    /// # 参数
    ///
    /// * `obj` - 要清洗的对象 Schema
    /// * `defs` - 定义映射
    /// * `strategy` - 清洗策略
    /// * `ref_stack` - 引用栈
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 Schema 值。
    fn clean_object(
        obj: Map<String, Value>,
        defs: &HashMap<String, Value>,
        strategy: CleaningStrategy,
        ref_stack: &mut HashSet<String>,
    ) -> Value {
        // 处理 $ref 解析
        if let Some(Value::String(ref_value)) = obj.get("$ref") {
            return Self::resolve_ref(ref_value, &obj, defs, strategy, ref_stack);
        }

        // 处理 anyOf/oneOf 简化
        if obj.contains_key("anyOf") || obj.contains_key("oneOf") {
            if let Some(simplified) = Self::try_simplify_union(&obj, defs, strategy, ref_stack) {
                return simplified;
            }
        }

        // 构建清洗后的对象
        let mut cleaned = Map::new();
        let unsupported: HashSet<&str> = strategy.unsupported_keywords().iter().copied().collect();
        let has_union = obj.contains_key("anyOf") || obj.contains_key("oneOf");

        for (key, value) in obj {
            // 跳过不支持的关键字
            if unsupported.contains(key.as_str()) {
                continue;
            }

            // 对特定关键字进行特殊处理
            match key.as_str() {
                // 将 const 转换为 enum
                "const" => {
                    cleaned.insert("enum".to_string(), json!([value]));
                }
                // 如果存在 anyOf/oneOf，跳过 type（它们已定义类型）
                "type" if has_union => {
                    // 跳过
                }
                // 处理 type 数组（移除 null）
                "type" if matches!(value, Value::Array(_)) => {
                    let cleaned_value = Self::clean_type_array(value);
                    cleaned.insert(key, cleaned_value);
                }
                // 递归清洗嵌套 Schema
                "properties" => {
                    let cleaned_value = Self::clean_properties(value, defs, strategy, ref_stack);
                    cleaned.insert(key, cleaned_value);
                }
                "items" => {
                    let cleaned_value = Self::clean_with_defs(value, defs, strategy, ref_stack);
                    cleaned.insert(key, cleaned_value);
                }
                "anyOf" | "oneOf" | "allOf" => {
                    let cleaned_value = Self::clean_union(value, defs, strategy, ref_stack);
                    cleaned.insert(key, cleaned_value);
                }
                // 保留所有其他关键字，递归清洗嵌套对象/数组
                _ => {
                    let cleaned_value = match value {
                        Value::Object(_) | Value::Array(_) => {
                            Self::clean_with_defs(value, defs, strategy, ref_stack)
                        }
                        other => other,
                    };
                    cleaned.insert(key, cleaned_value);
                }
            }
        }

        Value::Object(cleaned)
    }

    /// 解析 $ref 到其定义。
    ///
    /// 支持本地引用格式：`#/$defs/Name` 或 `#/definitions/Name`。
    /// 如果检测到循环引用或无法解析，返回空对象并保留元数据。
    ///
    /// # 参数
    ///
    /// * `ref_value` - 引用字符串（如 `#/$defs/Age`）
    /// * `obj` - 包含引用的原始对象
    /// * `defs` - 定义映射
    /// * `strategy` - 清洗策略
    /// * `ref_stack` - 引用栈，用于检测循环
    ///
    /// # 返回值
    ///
    /// 返回解析并清洗后的 Schema 值。
    fn resolve_ref(
        ref_value: &str,
        obj: &Map<String, Value>,
        defs: &HashMap<String, Value>,
        strategy: CleaningStrategy,
        ref_stack: &mut HashSet<String>,
    ) -> Value {
        // 防止循环引用
        if ref_stack.contains(ref_value) {
            tracing::warn!("检测到循环 $ref: {}", ref_value);
            return Self::preserve_meta(obj, Value::Object(Map::new()));
        }

        // 尝试解析本地引用（#/$defs/Name 或 #/definitions/Name）
        if let Some(def_name) = Self::parse_local_ref(ref_value) {
            if let Some(definition) = defs.get(def_name.as_str()) {
                ref_stack.insert(ref_value.to_string());
                let cleaned = Self::clean_with_defs(definition.clone(), defs, strategy, ref_stack);
                ref_stack.remove(ref_value);
                return Self::preserve_meta(obj, cleaned);
            }
        }

        // 无法解析：返回空对象并保留元数据
        tracing::warn!("无法解析 $ref: {}", ref_value);
        Self::preserve_meta(obj, Value::Object(Map::new()))
    }

    /// 解析本地 JSON Pointer 引用（#/$defs/Name）。
    ///
    /// 支持两种前缀格式：
    /// - `#/$defs/` - JSON Schema 2019-09+
    /// - `#/definitions/` - JSON Schema draft-07
    ///
    /// # 参数
    ///
    /// * `ref_value` - 引用字符串
    ///
    /// # 返回值
    ///
    /// 如果是有效的本地引用，返回定义名称；否则返回 None。
    fn parse_local_ref(ref_value: &str) -> Option<String> {
        ref_value
            .strip_prefix("#/$defs/")
            .or_else(|| ref_value.strip_prefix("#/definitions/"))
            .map(Self::decode_json_pointer)
    }

    /// 解码 JSON Pointer 转义字符。
    ///
    /// JSON Pointer 使用以下转义规则：
    /// - `~0` 表示 `~`
    /// - `~1` 表示 `/`
    ///
    /// # 参数
    ///
    /// * `segment` - 要解码的字符串片段
    ///
    /// # 返回值
    ///
    /// 返回解码后的字符串。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use vibewindow::app::agent::tools::schema::SchemaCleanr;
    ///
    /// assert_eq!(SchemaCleanr::decode_json_pointer("foo~0bar"), "foo~bar");
    /// assert_eq!(SchemaCleanr::decode_json_pointer("foo~1bar"), "foo/bar");
    /// ```
    fn decode_json_pointer(segment: &str) -> String {
        // 快速路径：如果没有波浪号，无需解码
        if !segment.contains('~') {
            return segment.to_string();
        }

        let mut decoded = String::with_capacity(segment.len());
        let mut chars = segment.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '~' {
                match chars.peek().copied() {
                    Some('0') => {
                        chars.next();
                        decoded.push('~');
                    }
                    Some('1') => {
                        chars.next();
                        decoded.push('/');
                    }
                    _ => decoded.push('~'),
                }
            } else {
                decoded.push(ch);
            }
        }

        decoded
    }

    /// 尝试将 anyOf/oneOf 简化为更简单的形式。
    ///
    /// 执行以下简化：
    /// 1. 清洗所有变体
    /// 2. 过滤掉 null 变体
    /// 3. 如果只剩一个变体，直接返回该变体
    /// 4. 尝试将字面量联合展平为 enum
    ///
    /// # 参数
    ///
    /// * `obj` - 包含联合类型的对象 Schema
    /// * `defs` - 定义映射
    /// * `strategy` - 清洗策略
    /// * `ref_stack` - 引用栈
    ///
    /// # 返回值
    ///
    /// 如果简化成功，返回简化后的 Schema；否则返回 None。
    fn try_simplify_union(
        obj: &Map<String, Value>,
        defs: &HashMap<String, Value>,
        strategy: CleaningStrategy,
        ref_stack: &mut HashSet<String>,
    ) -> Option<Value> {
        // 确定联合类型关键字
        let union_key = if obj.contains_key("anyOf") {
            "anyOf"
        } else if obj.contains_key("oneOf") {
            "oneOf"
        } else {
            return None;
        };

        let variants = obj.get(union_key)?.as_array()?;

        // 首先清洗所有变体
        let cleaned_variants: Vec<Value> = variants
            .iter()
            .map(|v| Self::clean_with_defs(v.clone(), defs, strategy, ref_stack))
            .collect();

        // 过滤掉 null 变体
        let non_null: Vec<Value> =
            cleaned_variants.into_iter().filter(|v| !Self::is_null_schema(v)).collect();

        // 如果过滤后只剩一个变体，直接返回
        if non_null.len() == 1 {
            return Some(Self::preserve_meta(obj, non_null[0].clone()));
        }

        // 尝试展平为 enum（如果所有变体都是字面量）
        if let Some(enum_value) = Self::try_flatten_literal_union(&non_null) {
            return Some(Self::preserve_meta(obj, enum_value));
        }

        None
    }

    /// 检查 Schema 是否表示 null 类型。
    ///
    /// 检测以下形式的 null Schema：
    /// - `{ "const": null }`
    /// - `{ "enum": [null] }`
    /// - `{ "type": "null" }`
    ///
    /// # 参数
    ///
    /// * `value` - 要检查的 Schema 值
    ///
    /// # 返回值
    ///
    /// 如果是 null Schema 返回 true，否则返回 false。
    fn is_null_schema(value: &Value) -> bool {
        if let Some(obj) = value.as_object() {
            // { const: null }
            if let Some(Value::Null) = obj.get("const") {
                return true;
            }
            // { enum: [null] }
            if let Some(Value::Array(arr)) = obj.get("enum") {
                if arr.len() == 1 && matches!(arr[0], Value::Null) {
                    return true;
                }
            }
            // { type: "null" }
            if let Some(Value::String(t)) = obj.get("type") {
                if t == "null" {
                    return true;
                }
            }
        }
        false
    }

    /// 尝试将仅包含字面量值的 anyOf/oneOf 展平为 enum。
    ///
    /// # 转换示例
    ///
    /// ```json
    /// // 输入
    /// { "anyOf": [{ "const": "a" }, { "const": "b" }] }
    ///
    /// // 输出
    /// { "type": "string", "enum": ["a", "b"] }
    /// ```
    ///
    /// # 参数
    ///
    /// * `variants` - 联合类型的变体数组
    ///
    /// # 返回值
    ///
    /// 如果所有变体都是相同类型的字面量，返回展平后的 Schema；否则返回 None。
    fn try_flatten_literal_union(variants: &[Value]) -> Option<Value> {
        if variants.is_empty() {
            return None;
        }

        let mut all_values = Vec::new();
        let mut common_type: Option<String> = None;

        for variant in variants {
            let obj = variant.as_object()?;

            // 从 const 或单元素 enum 中提取字面量值
            let literal_value = if let Some(const_val) = obj.get("const") {
                const_val.clone()
            } else if let Some(Value::Array(arr)) = obj.get("enum") {
                if arr.len() == 1 {
                    arr[0].clone()
                } else {
                    return None;
                }
            } else {
                return None;
            };

            // 检查类型一致性
            let variant_type = obj.get("type")?.as_str()?;
            match &common_type {
                None => common_type = Some(variant_type.to_string()),
                Some(t) if t != variant_type => return None,
                _ => {}
            }

            all_values.push(literal_value);
        }

        // 构建展平后的 enum Schema
        common_type.map(|t| {
            json!({
                "type": t,
                "enum": all_values
            })
        })
    }

    /// 清洗 type 数组，移除 null。
    ///
    /// # 处理逻辑
    ///
    /// - 如果数组为空或只包含 null，返回 `"null"`
    /// - 如果数组只包含一个非 null 类型，返回该类型字符串
    /// - 如果数组包含多个非 null 类型，返回过滤后的数组
    ///
    /// # 参数
    ///
    /// * `value` - type 字段的值
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 type 值。
    fn clean_type_array(value: Value) -> Value {
        if let Value::Array(types) = value {
            // 过滤掉 null 类型
            let non_null: Vec<Value> =
                types.into_iter().filter(|v| v.as_str() != Some("null")).collect();

            match non_null.len() {
                0 => Value::String("null".to_string()),
                1 => non_null.into_iter().next().unwrap_or(Value::String("null".to_string())),
                _ => Value::Array(non_null),
            }
        } else {
            value
        }
    }

    /// 清洗 properties 对象。
    ///
    /// 递归清洗每个属性的 Schema。
    ///
    /// # 参数
    ///
    /// * `value` - properties 对象值
    /// * `defs` - 定义映射
    /// * `strategy` - 清洗策略
    /// * `ref_stack` - 引用栈
    ///
    /// # 返回值
    ///
    /// 返回清洗后的 properties 对象。
    fn clean_properties(
        value: Value,
        defs: &HashMap<String, Value>,
        strategy: CleaningStrategy,
        ref_stack: &mut HashSet<String>,
    ) -> Value {
        if let Value::Object(props) = value {
            let cleaned: Map<String, Value> = props
                .into_iter()
                .map(|(k, v)| (k, Self::clean_with_defs(v, defs, strategy, ref_stack)))
                .collect();
            Value::Object(cleaned)
        } else {
            value
        }
    }

    /// 清洗联合类型（anyOf/oneOf/allOf）。
    ///
    /// 递归清洗联合类型中的每个变体。
    ///
    /// # 参数
    ///
    /// * `value` - 联合类型数组值
    /// * `defs` - 定义映射
    /// * `strategy` - 清洗策略
    /// * `ref_stack` - 引用栈
    ///
    /// # 返回值
    ///
    /// 返回清洗后的联合类型数组。
    fn clean_union(
        value: Value,
        defs: &HashMap<String, Value>,
        strategy: CleaningStrategy,
        ref_stack: &mut HashSet<String>,
    ) -> Value {
        if let Value::Array(variants) = value {
            let cleaned: Vec<Value> = variants
                .into_iter()
                .map(|v| Self::clean_with_defs(v, defs, strategy, ref_stack))
                .collect();
            Value::Array(cleaned)
        } else {
            value
        }
    }

    /// 从源对象保留元数据到目标对象。
    ///
    /// 保留以下元数据字段（如果存在）：
    /// - `description` - 描述文本
    /// - `title` - 标题
    /// - `default` - 默认值
    ///
    /// # 参数
    ///
    /// * `source` - 源对象，从中提取元数据
    /// * `target` - 目标值，将元数据添加到其中
    ///
    /// # 返回值
    ///
    /// 返回包含元数据的目标值。
    fn preserve_meta(source: &Map<String, Value>, mut target: Value) -> Value {
        if let Value::Object(target_obj) = &mut target {
            for &key in SCHEMA_META_KEYS {
                if let Some(value) = source.get(key) {
                    target_obj.insert(key.to_string(), value.clone());
                }
            }
        }
        target
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
