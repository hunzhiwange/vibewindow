use super::*;
use crate::app::App;
use crate::app::state::{SkillsDirectoryScope, SkillsSettingsTab};
use vw_config_types::skills::{SkillsDirectoryProvider, SkillsPromptInjectionMode};
use vw_gateway_client::{DesktopSkillCatalogEntryDto, DesktopSkillDetailDto};

fn app() -> App {
    App::new().0
}

fn catalog_item(id: &str, kind: &str) -> DesktopSkillCatalogEntryDto {
    DesktopSkillCatalogEntryDto {
        id: id.to_string(),
        title: format!("Title {id}"),
        description: "desc".to_string(),
        kind: kind.to_string(),
        resource_count: 2,
        installed: true,
        enabled: true,
        source: "builtin".to_string(),
        source_path: Some("/tmp/skill".to_string()),
    }
}

fn detail(id: &str, kind: &str) -> DesktopSkillDetailDto {
    DesktopSkillDetailDto {
        id: id.to_string(),
        title: format!("Title {id}"),
        description: "desc".to_string(),
        kind: kind.to_string(),
        installed: true,
        enabled: false,
        source: "builtin".to_string(),
        source_path: None,
        document_name: "SKILL.md".to_string(),
        document_content: "# Skill".to_string(),
        can_install: true,
        can_toggle: true,
        can_delete: false,
    }
}

#[test]
fn skill_mapping_loading_detail_and_config_paths() {
    assert_eq!(map_skill_kind("recommended"), SkillsCatalogKind::Recommended);
    assert_eq!(map_skill_kind("personal"), SkillsCatalogKind::Personal);
    assert_eq!(map_skill_kind("anything"), SkillsCatalogKind::System);
    let items =
        map_catalog_items(vec![catalog_item("a", "recommended"), catalog_item("b", "personal")]);
    assert_eq!(items[0].kind, SkillsCatalogKind::Recommended);
    assert_eq!(items[1].kind, SkillsCatalogKind::Personal);
    let mapped = map_skill_detail(detail("a", "recommended"));
    assert_eq!(mapped.document_name, "SKILL.md");

    let mut app = app();
    app.skills_settings.selected_skill_id = Some("missing".to_string());
    let _ = update(
        &mut app,
        SettingsMessage::SkillsLoaded(Ok(vec![catalog_item("present", "recommended")])),
    );
    assert_eq!(app.skills_settings.catalog.len(), 1);
    assert!(app.skills_settings.selected_skill_id.is_none());
    let _ = update(&mut app, SettingsMessage::SkillsLoaded(Err("boom".to_string())));
    assert!(app.skills_settings.status_is_error);

    let _ = update(&mut app, SettingsMessage::SkillsDetailRequested("skill-a".to_string()));
    assert_eq!(app.skills_settings.selected_skill_id.as_deref(), Some("skill-a"));
    let _ = update(
        &mut app,
        SettingsMessage::SkillsDetailLoaded {
            skill_id: "other".to_string(),
            result: Ok(detail("other", "personal")),
        },
    );
    assert!(app.skills_settings.detail_loading);
    let _ = update(
        &mut app,
        SettingsMessage::SkillsDetailLoaded {
            skill_id: "skill-a".to_string(),
            result: Ok(detail("skill-a", "personal")),
        },
    );
    assert_eq!(
        app.skills_settings.selected_skill_detail.as_ref().unwrap().kind,
        SkillsCatalogKind::Personal
    );
    let _ = update(
        &mut app,
        SettingsMessage::SkillsDetailLoaded {
            skill_id: "skill-a".to_string(),
            result: Err("detail failed".to_string()),
        },
    );
    assert_eq!(app.skills_settings.detail_error.as_deref(), Some("detail failed"));
    let _ = update(&mut app, SettingsMessage::SkillsDetailClosed);
    assert!(app.skills_settings.selected_skill_id.is_none());

    app.project_path = None;
    let _ = update(&mut app, SettingsMessage::SkillsTabChanged(SkillsSettingsTab::Plugins));
    let _ = update(&mut app, SettingsMessage::SkillsQueryChanged("rust".to_string()));
    let _ = update(
        &mut app,
        SettingsMessage::SkillsDirectoryScopeChanged(SkillsDirectoryScope::Global),
    );
    assert_eq!(app.skills_settings.active_tab, SkillsSettingsTab::Plugins);
    assert_eq!(app.skills_settings.query, "rust");
    assert_eq!(app.skills_settings.directory_scope, SkillsDirectoryScope::Global);
    let _ = update(&mut app, SettingsMessage::SkillsCreateNewRequested);
    assert!(
        app.skills_settings.status_message.as_deref().unwrap_or("").contains("请先打开一个项目")
    );
    let _ = update(&mut app, SettingsMessage::SkillsInstallBuiltInRequested("builtin".to_string()));
    assert!(
        app.skills_settings.status_message.as_deref().unwrap_or("").contains("请先打开一个项目")
    );

    app.project_path = Some("/tmp/project".to_string());
    app.skills_settings.selected_skill_id = Some("skill-a".to_string());
    let _ = update(
        &mut app,
        SettingsMessage::SkillsCreateNewCompleted(Ok("/tmp/project/.skills/new".to_string())),
    );
    assert!(app.skills_settings.loading);
    let _ = update(
        &mut app,
        SettingsMessage::SkillsCreateNewCompleted(Err("create failed".to_string())),
    );
    assert_eq!(app.skills_settings.status_message.as_deref(), Some("create failed"));
    let _ = update(
        &mut app,
        SettingsMessage::SkillsInstallBuiltInCompleted(Ok("/tmp/project/skill".to_string())),
    );
    assert!(app.skills_settings.detail_loading);
    let _ = update(
        &mut app,
        SettingsMessage::SkillsSetEnabledCompleted {
            skill_id: "skill-a".to_string(),
            enabled: false,
            result: Err("toggle failed".to_string()),
        },
    );
    assert_eq!(app.skills_settings.status_message.as_deref(), Some("toggle failed"));
    let _ = update(
        &mut app,
        SettingsMessage::SkillsDeleteCompleted {
            skill_id: "skill-a".to_string(),
            result: Ok("/tmp/project/skill".to_string()),
        },
    );
    assert!(app.skills_settings.selected_skill_id.is_none());

    app.skills_settings.save_error = Some("old".to_string());
    let _ = update(
        &mut app,
        SettingsMessage::SkillsDirectoryProviderChanged(SkillsDirectoryProvider::Codex),
    );
    assert_eq!(app.skills_settings.directory_provider, SkillsDirectoryProvider::Codex);
    assert!(app.skills_settings.save_error.is_none());
    let _ = update(&mut app, SettingsMessage::SkillsOpenEnabledToggled(true));
    let _ = update(&mut app, SettingsMessage::SkillsOpenDirChanged(" /tmp/skills ".to_string()));
    let _ = update(
        &mut app,
        SettingsMessage::SkillsPromptInjectionModeChanged(SkillsPromptInjectionMode::Full),
    );
    assert_eq!(app.skills_settings.prompt_injection_mode, SkillsPromptInjectionMode::Full);
    let _ = update(&mut app, SettingsMessage::SkillsSave);
    let _ = update(&mut app, SettingsMessage::SkillsHelpOpen);
    assert!(app.skills_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::SkillsHelpClose);
    assert!(!app.skills_settings.show_help_modal);
}
