//! MCP 服务器配置字段的解析与校验。

use std::path::PathBuf;

use agent_client_protocol::{
    EnvVariable, HttpHeader, McpServer, McpServerHttp, McpServerSse, McpServerStdio, Meta,
};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct ParseMcpServersError {
    message: String,
}

impl ParseMcpServersError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

fn parse_non_empty_string(
    value: Option<&Value>,
    path: &str,
) -> Result<String, ParseMcpServersError> {
    match value.and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
        _ => Err(ParseMcpServersError::new(format!("Invalid {path}: expected non-empty string"))),
    }
}

fn parse_headers(
    value: Option<&Value>,
    path: &str,
) -> Result<Vec<HttpHeader>, ParseMcpServersError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };

    let headers = value
        .as_array()
        .ok_or_else(|| ParseMcpServersError::new(format!("Invalid {path}: expected array")))?;

    headers
        .iter()
        .enumerate()
        .map(|(index, raw_header)| {
            let header = raw_header.as_object().ok_or_else(|| {
                ParseMcpServersError::new(format!("Invalid {path}[{index}]: expected object"))
            })?;
            let name =
                parse_non_empty_string(header.get("name"), &format!("{path}[{index}].name"))?;
            let value =
                parse_non_empty_string(header.get("value"), &format!("{path}[{index}].value"))?;
            Ok(HttpHeader::new(name, value))
        })
        .collect()
}

fn parse_args(value: Option<&Value>, path: &str) -> Result<Vec<String>, ParseMcpServersError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };

    let args = value
        .as_array()
        .ok_or_else(|| ParseMcpServersError::new(format!("Invalid {path}: expected array")))?;

    args.iter()
        .enumerate()
        .map(|(index, raw_arg)| {
            raw_arg.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                ParseMcpServersError::new(format!("Invalid {path}[{index}]: expected string"))
            })
        })
        .collect()
}

fn parse_env(value: Option<&Value>, path: &str) -> Result<Vec<EnvVariable>, ParseMcpServersError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };

    let env = value
        .as_array()
        .ok_or_else(|| ParseMcpServersError::new(format!("Invalid {path}: expected array")))?;

    env.iter()
        .enumerate()
        .map(|(index, raw_entry)| {
            let entry = raw_entry.as_object().ok_or_else(|| {
                ParseMcpServersError::new(format!("Invalid {path}[{index}]: expected object"))
            })?;
            let name = parse_non_empty_string(entry.get("name"), &format!("{path}[{index}].name"))?;
            let value =
                parse_non_empty_string(entry.get("value"), &format!("{path}[{index}].value"))?;
            Ok(EnvVariable::new(name, value))
        })
        .collect()
}

fn parse_meta(value: Option<&Value>, path: &str) -> Result<Option<Meta>, ParseMcpServersError> {
    match value {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(Value::Object(meta)) => Ok(Some(meta.clone())),
        Some(_) => {
            Err(ParseMcpServersError::new(format!("Invalid {path}: expected object or null")))
        }
    }
}

fn parse_server(raw_server: &Value, path: &str) -> Result<McpServer, ParseMcpServersError> {
    let server = raw_server
        .as_object()
        .ok_or_else(|| ParseMcpServersError::new(format!("Invalid {path}: expected object")))?;

    let name = parse_non_empty_string(server.get("name"), &format!("{path}.name"))?;
    let meta = parse_meta(server.get("_meta"), &format!("{path}._meta"))?;
    let server_type = match server.get("type") {
        None => "stdio".to_string(),
        Some(value) => parse_non_empty_string(Some(value), &format!("{path}.type"))?,
    };

    match server_type.as_str() {
        "http" => {
            let url = parse_non_empty_string(server.get("url"), &format!("{path}.url"))?;
            let headers = parse_headers(server.get("headers"), &format!("{path}.headers"))?;
            let server = McpServerHttp::new(name, url).headers(headers);
            let server = match meta {
                Some(meta) => server.meta(meta),
                None => server,
            };
            Ok(McpServer::Http(server))
        }
        "sse" => {
            let url = parse_non_empty_string(server.get("url"), &format!("{path}.url"))?;
            let headers = parse_headers(server.get("headers"), &format!("{path}.headers"))?;
            let server = McpServerSse::new(name, url).headers(headers);
            let server = match meta {
                Some(meta) => server.meta(meta),
                None => server,
            };
            Ok(McpServer::Sse(server))
        }
        "stdio" => {
            let command =
                parse_non_empty_string(server.get("command"), &format!("{path}.command"))?;
            let args = parse_args(server.get("args"), &format!("{path}.args"))?;
            let env = parse_env(server.get("env"), &format!("{path}.env"))?;
            let server = McpServerStdio::new(name, PathBuf::from(command)).args(args).env(env);
            let server = match meta {
                Some(meta) => server.meta(meta),
                None => server,
            };
            Ok(McpServer::Stdio(server))
        }
        _ => Err(ParseMcpServersError::new(format!(
            "Invalid {path}.type: expected http, sse, or stdio"
        ))),
    }
}

pub fn parse_mcp_servers(
    value: &Value,
    source_path: &str,
) -> Result<Vec<McpServer>, ParseMcpServersError> {
    parse_mcp_servers_with_field_name(value, source_path, "mcpServers")
}

pub fn parse_mcp_servers_with_field_name(
    value: &Value,
    source_path: &str,
    field_name: &str,
) -> Result<Vec<McpServer>, ParseMcpServersError> {
    let field_path = format!("{field_name} in {source_path}");
    let servers = value.as_array().ok_or_else(|| {
        ParseMcpServersError::new(format!("Invalid {field_path}: expected array"))
    })?;

    servers
        .iter()
        .enumerate()
        .map(|(index, raw_server)| {
            parse_server(raw_server, &format!("{field_name}[{index}] in {source_path}"))
        })
        .collect()
}

pub fn parse_optional_mcp_servers(
    value: Option<&Value>,
    source_path: &str,
) -> Result<Option<Vec<McpServer>>, ParseMcpServersError> {
    parse_optional_mcp_servers_with_field_name(value, source_path, "mcpServers")
}

pub fn parse_optional_mcp_servers_with_field_name(
    value: Option<&Value>,
    source_path: &str,
    field_name: &str,
) -> Result<Option<Vec<McpServer>>, ParseMcpServersError> {
    value.map(|value| parse_mcp_servers_with_field_name(value, source_path, field_name)).transpose()
}

#[cfg(test)]
#[path = "mcp_servers_tests.rs"]
mod mcp_servers_tests;
