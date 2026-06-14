use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::client::test_support;

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    theme: String,
}

#[tokio::test]
async fn desktop_settings_api_routes_skills_external_apps_and_preferences() {
    let skill = json!({
        "id": "rust",
        "title": "Rust",
        "description": "systems",
        "kind": "built_in",
        "resource_count": 1,
        "installed": false,
        "enabled": true,
        "source": "built_in",
        "source_path": null
    });
    let detail = json!({
        "id": "rust",
        "title": "Rust",
        "description": "systems",
        "kind": "built_in",
        "installed": true,
        "enabled": true,
        "source": "built_in",
        "source_path": null,
        "document_name": "SKILL.md",
        "document_content": "content",
        "can_install": true,
        "can_toggle": true,
        "can_delete": false
    });
    let server = test_support::server(vec![
        (200, json!([skill])),
        (200, detail),
        (200, json!({"path": "/repo/.codex/skills/new"})),
        (200, json!({"path": "/repo/.codex/skills/rust"})),
        (200, json!({"path": "/repo/.codex/skills/rust"})),
        (200, json!({"path": "/repo/.codex/skills/rust"})),
        (200, json!({"platform": "macos", "apps": [{"id": "finder", "available": true}]})),
        (200, json!({})),
        (200, json!({})),
        (200, json!({"command": "status", "output": "Service: running"})),
        (200, json!({"app_ui": {"system_settings": {"theme": "dark"}}})),
        (200, json!({})),
        (200, json!({"compact": true})),
        (200, json!({"compact": false})),
        (200, json!({"content": "select 1"})),
        (200, json!({})),
        (200, json!(null)),
        (200, json!({})),
        (200, json!({"model": "gpt-5", "auto_model": true, "acp_agent": "codex"})),
        (200, json!({})),
    ]);

    assert_eq!(server.client().skills_get(Some("/repo")).await.expect("skills").len(), 1);
    assert_eq!(
        server.client().skill_detail_get(Some("/repo"), "rust").await.expect("detail").id,
        "rust"
    );
    assert!(server.client().skill_create("/repo").await.expect("create").ends_with("/new"));
    assert!(
        server
            .client()
            .skill_install_builtin("/repo", "rust")
            .await
            .expect("install")
            .ends_with("rust")
    );
    assert!(
        server
            .client()
            .skill_set_enabled(Some("/repo"), "rust", false)
            .await
            .expect("enable")
            .ends_with("rust")
    );
    assert!(
        server
            .client()
            .skill_delete(Some("/repo"), "rust")
            .await
            .expect("delete")
            .ends_with("rust")
    );
    let apps = server.client().desktop_external_apps_get().await.expect("apps");
    assert_eq!(apps.apps, vec![("finder".to_string(), true)]);
    server.client().desktop_external_app_open("/repo", "finder").await.expect("open");
    server.client().desktop_external_path_reveal("/repo").await.expect("reveal");
    assert_eq!(
        server.client().desktop_service_command("status").await.expect("service").output,
        "Service: running"
    );
    let settings: Option<Settings> =
        server.client().desktop_system_settings_get().await.expect("settings");
    assert_eq!(settings.expect("some").theme, "dark");
    server
        .client()
        .desktop_system_settings_patch(&Settings { theme: "light".to_string() })
        .await
        .expect("settings patch");
    assert!(
        server.client().desktop_preferences_get().await.expect("prefs")["compact"]
            .as_bool()
            .unwrap()
    );
    assert!(
        !server
            .client()
            .desktop_preferences_patch(&json!({"compact": false}))
            .await
            .expect("prefs patch")["compact"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(server.client().desktop_tool_content_get("sql").await.expect("tool"), "select 1");
    server.client().desktop_tool_content_put("sql", "select 2").await.expect("tool put");
    assert!(server.client().desktop_mindmap_tabs_get().await.expect("tabs").is_none());
    server.client().desktop_mindmap_tabs_put(&json!({"tabs": []})).await.expect("tabs put");
    assert_eq!(
        server.client().desktop_project_preferences_get("/repo").await.expect("project prefs"),
        Some(("gpt-5".to_string(), true, Some("codex".to_string())))
    );
    server
        .client()
        .desktop_project_preferences_put("/repo", "gpt-5", true, Some("codex"))
        .await
        .expect("project prefs put");

    assert!(server.take_request().path.contains("/v1/skills?project_path=%2Frepo"));
    assert!(
        server.take_request().path.contains("/v1/skills/detail?skill_id=rust&project_path=%2Frepo")
    );
    assert_eq!(server.take_request().path, "/v1/skills/create");
    assert_eq!(server.take_request().path, "/v1/skills/install-built-in");
    assert_eq!(server.take_request().path, "/v1/skills/set-enabled");
    assert_eq!(server.take_request().path, "/v1/skills/delete");
    assert_eq!(server.take_request().path, "/v1/desktop/external-apps");
    assert_eq!(server.take_request().path, "/v1/desktop/external-apps/open");
    assert_eq!(server.take_request().path, "/v1/desktop/external-path/reveal");
    assert_eq!(server.take_request().path, "/v1/desktop/service/status");
    assert_eq!(server.take_request().path, "/v1/global/config");
    let request = server.take_request();
    assert_eq!(request.path, "/v1/global/config");
    assert_eq!(request.body, json!({"app_ui": {"system_settings": {"theme": "light"}}}));
    assert_eq!(server.take_request().path, "/v1/desktop/preferences");
    assert_eq!(server.take_request().path, "/v1/desktop/preferences");
    assert_eq!(server.take_request().path, "/v1/desktop/tool-content/sql");
    assert_eq!(server.take_request().path, "/v1/desktop/tool-content/sql");
    assert_eq!(server.take_request().path, "/v1/desktop/mindmap-tabs");
    assert_eq!(server.take_request().path, "/v1/desktop/mindmap-tabs");
    assert!(
        server.take_request().path.contains("/v1/desktop/project-preferences?project_path=%2Frepo")
    );
    assert!(
        server.take_request().path.contains("/v1/desktop/project-preferences?project_path=%2Frepo")
    );
    server.join();
}

#[tokio::test]
async fn desktop_skill_compatibility_wrappers_delegate_to_skill_routes() {
    let skill = json!({
        "id": "rust",
        "title": "Rust",
        "description": "systems",
        "kind": "built_in",
        "resource_count": 1,
        "installed": false,
        "enabled": true,
        "source": "built_in",
        "source_path": null
    });
    let detail = json!({
        "id": "rust",
        "title": "Rust",
        "description": "systems",
        "kind": "built_in",
        "installed": true,
        "enabled": true,
        "source": "built_in",
        "source_path": null,
        "document_name": "SKILL.md",
        "document_content": "content",
        "can_install": true,
        "can_toggle": true,
        "can_delete": false
    });
    let server = test_support::server(vec![
        (200, json!([skill])),
        (200, detail),
        (200, json!({"path": "/repo/.codex/skills/new"})),
        (200, json!({"path": "/repo/.codex/skills/rust"})),
        (200, json!({"path": "/repo/.codex/skills/rust"})),
        (200, json!({"path": "/repo/.codex/skills/rust"})),
        (200, json!({"app_ui": {}})),
    ]);

    assert_eq!(server.client().desktop_skills_get(None).await.expect("skills").len(), 1);
    assert_eq!(
        server.client().desktop_skill_detail_get(None, "rust").await.expect("detail").id,
        "rust"
    );
    assert!(server.client().desktop_skill_create("/repo").await.expect("create").ends_with("new"));
    assert!(
        server
            .client()
            .desktop_skill_install_builtin("/repo", "rust")
            .await
            .expect("install")
            .ends_with("rust")
    );
    assert!(
        server
            .client()
            .desktop_skill_set_enabled(None, "rust", true)
            .await
            .expect("enable")
            .ends_with("rust")
    );
    assert!(
        server.client().desktop_skill_delete(None, "rust").await.expect("delete").ends_with("rust")
    );
    let settings: Option<Settings> =
        server.client().desktop_system_settings_get().await.expect("no settings");
    assert!(settings.is_none());

    assert_eq!(server.take_request().path, "/v1/skills");
    assert_eq!(server.take_request().path, "/v1/skills/detail?skill_id=rust");
    assert_eq!(server.take_request().path, "/v1/skills/create");
    assert_eq!(server.take_request().path, "/v1/skills/install-built-in");
    assert_eq!(server.take_request().path, "/v1/skills/set-enabled");
    assert_eq!(server.take_request().path, "/v1/skills/delete");
    assert_eq!(server.take_request().path, "/v1/global/config");
    server.join();
}
