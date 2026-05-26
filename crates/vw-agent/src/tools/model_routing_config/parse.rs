use super::ModelRoutingConfigTool;
use crate::app::agent::util::MaybeSet;
use serde_json::Value;

impl ModelRoutingConfigTool {
    /// 解析字符串列表参数。
    pub(super) fn parse_string_list(raw: &Value, field: &str) -> anyhow::Result<Vec<String>> {
        if let Some(raw_string) = raw.as_str() {
            return Ok(raw_string
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToOwned::to_owned)
                .collect());
        }

        if let Some(array) = raw.as_array() {
            let mut out = Vec::new();
            for item in array {
                let value = item
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("'{field}' array must only contain strings"))?;
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            }
            return Ok(out);
        }

        anyhow::bail!("'{field}' must be a string or string[]")
    }

    /// 解析必需的非空字符串参数。
    pub(super) fn parse_non_empty_string(args: &Value, field: &str) -> anyhow::Result<String> {
        let value = args
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing '{field}'"))?
            .trim();

        if value.is_empty() {
            anyhow::bail!("'{field}' must not be empty");
        }

        Ok(value.to_string())
    }

    /// 解析可选的字符串更新操作。
    pub(super) fn parse_optional_string_update(
        args: &Value,
        field: &str,
    ) -> anyhow::Result<MaybeSet<String>> {
        let Some(raw) = args.get(field) else {
            return Ok(MaybeSet::Unset);
        };

        if raw.is_null() {
            return Ok(MaybeSet::Null);
        }

        let value = raw
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'{field}' must be a string or null"))?
            .trim()
            .to_string();

        let output = if value.is_empty() { MaybeSet::Null } else { MaybeSet::Set(value) };
        Ok(output)
    }

    /// 解析可选的 f64 更新操作。
    pub(super) fn parse_optional_f64_update(
        args: &Value,
        field: &str,
    ) -> anyhow::Result<MaybeSet<f64>> {
        let Some(raw) = args.get(field) else {
            return Ok(MaybeSet::Unset);
        };

        if raw.is_null() {
            return Ok(MaybeSet::Null);
        }

        let value =
            raw.as_f64().ok_or_else(|| anyhow::anyhow!("'{field}' must be a number or null"))?;
        Ok(MaybeSet::Set(value))
    }

    /// 解析可选的 usize 更新操作。
    pub(super) fn parse_optional_usize_update(
        args: &Value,
        field: &str,
    ) -> anyhow::Result<MaybeSet<usize>> {
        let Some(raw) = args.get(field) else {
            return Ok(MaybeSet::Unset);
        };

        if raw.is_null() {
            return Ok(MaybeSet::Null);
        }

        let raw_value = raw
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("'{field}' must be a non-negative integer or null"))?;
        let value = usize::try_from(raw_value)
            .map_err(|_| anyhow::anyhow!("'{field}' is too large for this platform"))?;
        Ok(MaybeSet::Set(value))
    }

    /// 解析可选的 u32 更新操作。
    pub(super) fn parse_optional_u32_update(
        args: &Value,
        field: &str,
    ) -> anyhow::Result<MaybeSet<u32>> {
        let Some(raw) = args.get(field) else {
            return Ok(MaybeSet::Unset);
        };

        if raw.is_null() {
            return Ok(MaybeSet::Null);
        }

        let raw_value = raw
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("'{field}' must be a non-negative integer or null"))?;
        let value =
            u32::try_from(raw_value).map_err(|_| anyhow::anyhow!("'{field}' must fit in u32"))?;
        Ok(MaybeSet::Set(value))
    }

    /// 解析可选的 i32 更新操作。
    pub(super) fn parse_optional_i32_update(
        args: &Value,
        field: &str,
    ) -> anyhow::Result<MaybeSet<i32>> {
        let Some(raw) = args.get(field) else {
            return Ok(MaybeSet::Unset);
        };

        if raw.is_null() {
            return Ok(MaybeSet::Null);
        }

        let raw_value =
            raw.as_i64().ok_or_else(|| anyhow::anyhow!("'{field}' must be an integer or null"))?;
        let value =
            i32::try_from(raw_value).map_err(|_| anyhow::anyhow!("'{field}' must fit in i32"))?;
        Ok(MaybeSet::Set(value))
    }

    /// 解析可选的布尔值。
    pub(super) fn parse_optional_bool(args: &Value, field: &str) -> anyhow::Result<Option<bool>> {
        let Some(raw) = args.get(field) else {
            return Ok(None);
        };

        let value = raw.as_bool().ok_or_else(|| anyhow::anyhow!("'{field}' must be a boolean"))?;
        Ok(Some(value))
    }
}
#[cfg(test)]
#[path = "parse_tests.rs"]
mod parse_tests;
