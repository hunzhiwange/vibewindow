//! 本地代码节点执行。

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;
use tempfile::Builder;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const CODE_TIMEOUT_SECS: u64 = 60;
const MAX_ERROR_CHARS: usize = 4_000;
const SENSITIVE_KEY_MARKERS: [&str; 7] =
    ["token", "secret", "password", "api_key", "authorization", "auth", "skey"];

#[derive(Debug, Deserialize)]
struct CodeEnvelope {
    ok: bool,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<String>,
}

pub(crate) async fn run_code_node(
    language: &str,
    code: &str,
    inputs: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, String> {
    let language = language.trim().to_ascii_lowercase();
    if language.contains("python") {
        run_script("python3", ".py", &python_script(code), inputs).await
    } else if language.contains("javascript") || language.contains("node") || language == "js" {
        run_script("node", ".js", &javascript_script(code), inputs).await
    } else {
        Err(format!("不支持的 code_language: {language}"))
    }
}

async fn run_script(
    program: &str,
    suffix: &str,
    script: &str,
    inputs: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, String> {
    let mut file = Builder::new()
        .prefix("vw-workflow-code-")
        .suffix(suffix)
        .tempfile()
        .map_err(|error| error.to_string())?;
    file.write_all(script.as_bytes()).map_err(|error| error.to_string())?;
    file.flush().map_err(|error| error.to_string())?;
    let path = file.path().to_path_buf();
    let payload = serde_json::json!({ "inputs": inputs });
    let payload = serde_json::to_vec(&payload).map_err(|error| error.to_string())?;

    let mut child = Command::new(program)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("启动 {program} 失败: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&payload).await.map_err(|error| error.to_string())?;
    }

    let output = timeout(Duration::from_secs(CODE_TIMEOUT_SECS), child.wait_with_output())
        .await
        .map_err(|_| format!("code 节点执行超时: {CODE_TIMEOUT_SECS}s"))?
        .map_err(|error| error.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let envelope = parse_last_json_line(&stdout).ok_or_else(|| {
        format!(
            "code 节点没有返回合法 JSON: {}",
            sanitize_error_text(format!("stdout={stdout}; stderr={stderr}"), inputs)
        )
    })?;

    if !envelope.ok || !output.status.success() {
        let message = envelope.error.unwrap_or_else(|| "code 节点执行失败".to_string());
        return Err(sanitize_error_text(format!("{message}; stderr={stderr}"), inputs));
    }

    Ok(value_to_output_map(envelope.result.unwrap_or(Value::Null)))
}

fn parse_last_json_line(stdout: &str) -> Option<CodeEnvelope> {
    stdout
        .lines()
        .rev()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .find_map(|line| serde_json::from_str::<CodeEnvelope>(line).ok())
}

fn value_to_output_map(value: Value) -> BTreeMap<String, Value> {
    match value {
        Value::Object(object) => object.into_iter().collect(),
        other => BTreeMap::from([("result".to_string(), other)]),
    }
}

fn truncate_error(value: String) -> String {
    value.chars().take(MAX_ERROR_CHARS).collect()
}

fn sanitize_error_text(value: String, inputs: &BTreeMap<String, Value>) -> String {
    let mut redacted = value;
    for (key, value) in inputs {
        if !is_sensitive_key(key) {
            continue;
        }
        redact_value_text(value, &mut redacted);
    }
    truncate_error(redacted)
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SENSITIVE_KEY_MARKERS.iter().any(|marker| lower.contains(marker))
}

fn redact_value_text(value: &Value, text: &mut String) {
    match value {
        Value::String(secret) if !secret.is_empty() => {
            *text = text.replace(secret, "[REDACTED]");
        }
        Value::Array(items) => {
            for item in items {
                redact_value_text(item, text);
            }
        }
        Value::Object(object) => {
            for value in object.values() {
                redact_value_text(value, text);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn python_script(code: &str) -> String {
    format!(
        r#"{code}

if __name__ == "__main__":
    import json
    import sys
    import traceback

    try:
        payload = json.load(sys.stdin)
        inputs = payload.get("inputs") or {{}}
        result = main(**inputs)
        print(json.dumps({{"ok": True, "result": result}}, ensure_ascii=False, default=str))
    except Exception as exc:
        traceback.print_exc(file=sys.stderr)
        print(json.dumps({{"ok": False, "error": str(exc)}}, ensure_ascii=False))
        sys.exit(1)
"#
    )
}

fn javascript_script(code: &str) -> String {
    format!(
        r#"{code}

const __vwReadStdin = () => new Promise((resolve) => {{
  let data = "";
  process.stdin.setEncoding("utf8");
  process.stdin.on("data", chunk => data += chunk);
  process.stdin.on("end", () => resolve(data));
}});

(async () => {{
  try {{
    const payloadText = await __vwReadStdin();
    const payload = payloadText.trim() ? JSON.parse(payloadText) : {{}};
    const fn = typeof main === "function" ? main : module.exports && module.exports.main;
    if (typeof fn !== "function") {{
      throw new Error("main function is required");
    }}
    const result = await fn(payload.inputs || {{}});
    console.log(JSON.stringify({{ ok: true, result }}));
  }} catch (error) {{
    console.error(error && error.stack ? error.stack : String(error));
    console.log(JSON.stringify({{ ok: false, error: error && error.message ? error.message : String(error) }}));
    process.exit(1);
  }}
}})();
"#
    )
}
