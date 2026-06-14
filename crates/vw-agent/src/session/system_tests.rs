use super::*;
use serde_json::json;

fn test_model(api_id: &str, provider_id: &str) -> provider::Model {
    serde_json::from_value(json!({
        "id": api_id,
        "providerID": provider_id,
        "api": {
            "id": api_id,
            "url": "http://localhost",
            "adapter": "openai-compatible"
        },
        "name": api_id,
        "family": null,
        "capabilities": {
            "temperature": true,
            "reasoning": true,
            "attachment": false,
            "toolcall": true,
            "input": {
                "text": true,
                "audio": false,
                "image": false,
                "video": false,
                "pdf": false
            },
            "output": {
                "text": true,
                "audio": false,
                "image": false,
                "video": false,
                "pdf": false
            },
            "interleaved": false
        },
        "cost": {
            "input": 0.0,
            "output": 0.0,
            "cache": {
                "read": 0.0,
                "write": 0.0
            },
            "experimental_over_200k": null
        },
        "limit": {
            "context": 8192,
            "input": null,
            "output": 4096
        },
        "status": "active",
        "options": {},
        "headers": {},
        "release_date": "2026-01-01",
        "variants": {}
    }))
    .expect("test model should deserialize")
}

#[test]
fn yes_no_cn_is_stable() {
    assert_eq!(yes_no_cn(true), "是");
    assert_eq!(yes_no_cn(false), "否");
}

#[test]
fn environment_text_contains_core_fields() {
    let text = environment_text("gpt-5", "openai", "/tmp/work", true, "no extra dirs");
    assert!(text.contains("gpt-5"));
    assert!(text.contains("openai/gpt-5"));
    assert!(text.contains("/tmp/work"));
    assert!(text.contains("是"));
    assert!(text.contains(platform()));
}

#[test]
fn provider_selects_prompt_family_from_model_id() {
    assert_eq!(provider(&test_model("gpt-5-mini", "openai")), vec![PROMPT_CODEX]);
    assert_eq!(provider(&test_model("gpt-4.1", "openai")), vec![PROMPT_BEAST]);
    assert_eq!(provider(&test_model("o3-mini", "openai")), vec![PROMPT_BEAST]);
    assert_eq!(provider(&test_model("gemini-2.5-pro", "google")), vec![PROMPT_GEMINI]);
    assert_eq!(provider(&test_model("claude-sonnet-4", "anthropic")), vec![PROMPT_ANTHROPIC]);
    assert_eq!(provider(&test_model("Trinity-v1", "local")), vec![PROMPT_TRINITY]);
    assert_eq!(provider(&test_model("qwen-plus", "qwen")), vec![PROMPT_ANTHROPIC_WITHOUT_TODO]);
}

#[test]
fn instructions_are_trimmed_codex_prompt() {
    let instructions = instructions();

    assert_eq!(instructions, PROMPT_CODEX.trim());
    assert!(!instructions.starts_with('\n'));
    assert!(!instructions.ends_with('\n'));
}

#[test]
fn resolve_cwd_prefers_non_empty_root_and_trims_it() {
    assert_eq!(resolve_cwd(Some("  /tmp/vw-work  ")), "/tmp/vw-work");
}

#[test]
fn detect_git_from_dir_walks_up_to_parent_repo() {
    let dir = tempfile::tempdir().expect("temp dir");
    let nested = dir.path().join("a/b/c");
    std::fs::create_dir_all(&nested).unwrap();

    assert!(!detect_git_from_dir(nested.to_str().unwrap()));
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    assert!(detect_git_from_dir(nested.to_str().unwrap()));
    assert!(!detect_git_from_dir("   "));
}

#[tokio::test]
async fn environment_from_ref_handles_missing_and_malformed_model_refs() {
    let root = "/tmp/vw-agent-env-test";

    let unknown = environment_from_ref(None, Some(root)).await;
    assert!(unknown.contains("unknown/unknown"));
    assert!(unknown.contains(root));

    let malformed = environment_from_ref(Some("not-a-model-ref"), Some(root)).await;
    assert!(malformed.contains("unknown/not-a-model-ref"));
}
