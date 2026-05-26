use super::{App, Message, RedisToolMessage, Task};
use super::{draft_inputs, navigation, operations};

/// 更新 Redis 工具状态。
pub fn update(app: &mut App, message: RedisToolMessage) -> Task<Message> {
    match message {
        RedisToolMessage::OpenSettingsModal => navigation::open_settings_modal(app),
        RedisToolMessage::CloseSettingsModal => navigation::close_settings_modal(app),
        RedisToolMessage::OpenHistoryModal => navigation::open_history_modal(app),
        RedisToolMessage::CloseHistoryModal => navigation::close_history_modal(app),
        RedisToolMessage::OpenCreateKeyModal => navigation::open_create_key_modal(app),
        RedisToolMessage::CloseCreateKeyModal => navigation::close_create_key_modal(app),
        RedisToolMessage::NewConnection => navigation::new_connection(app),
        RedisToolMessage::SearchConnectionsChanged(value) => {
            navigation::search_connections_changed(app, value)
        }
        RedisToolMessage::SelectConnection(connection_id) => {
            navigation::select_connection(app, connection_id)
        }
        RedisToolMessage::SelectConnectionCompleted(result) => {
            navigation::select_connection_completed(app, result)
        }
        RedisToolMessage::DetailTabChanged(value) => navigation::detail_tab_changed(app, value),
        RedisToolMessage::RefreshSelectedRuntime => navigation::refresh_selected_runtime(app),
        RedisToolMessage::ReloadSelectedKeys => navigation::reload_selected_keys(app),
        RedisToolMessage::LoadMoreKeys => navigation::load_more_keys(app),
        RedisToolMessage::SelectKey(key) => navigation::select_key(app, key),
        RedisToolMessage::RefreshSelectedKeyAnalysis => {
            navigation::refresh_selected_key_analysis(app)
        }
        RedisToolMessage::KeyAnalysisLoaded {
            connection_id,
            key,
            result,
        } => navigation::key_analysis_loaded(app, connection_id, key, result),
        RedisToolMessage::RuntimeLoaded {
            connection_id,
            success_message,
            result,
        } => navigation::runtime_loaded(app, connection_id, success_message, result),
        RedisToolMessage::KeyPageLoaded {
            connection_id,
            append,
            result,
        } => navigation::key_page_loaded(app, connection_id, append, result),
        RedisToolMessage::DraftNameChanged(value) => draft_inputs::draft_name_changed(app, value),
        RedisToolMessage::DraftHostChanged(value) => draft_inputs::draft_host_changed(app, value),
        RedisToolMessage::DraftPortChanged(value) => draft_inputs::draft_port_changed(app, value),
        RedisToolMessage::DraftDbChanged(value) => draft_inputs::draft_db_changed(app, value),
        RedisToolMessage::DraftUsernameChanged(value) => {
            draft_inputs::draft_username_changed(app, value)
        }
        RedisToolMessage::DraftPasswordChanged(value) => {
            draft_inputs::draft_password_changed(app, value)
        }
        RedisToolMessage::DraftTabChanged(value) => draft_inputs::draft_tab_changed(app, value),
        RedisToolMessage::DraftTlsToggled(value) => draft_inputs::draft_tls_toggled(app, value),
        RedisToolMessage::DraftTlsPrivateKeyPathChanged(value) => {
            draft_inputs::draft_tls_private_key_path_changed(app, value)
        }
        RedisToolMessage::DraftTlsPublicCertPathChanged(value) => {
            draft_inputs::draft_tls_public_cert_path_changed(app, value)
        }
        RedisToolMessage::DraftTlsCaCertPathChanged(value) => {
            draft_inputs::draft_tls_ca_cert_path_changed(app, value)
        }
        RedisToolMessage::PickTlsPrivateKeyFile => draft_inputs::pick_tls_private_key_file(),
        RedisToolMessage::TlsPrivateKeyFilePicked(opt) => {
            draft_inputs::tls_private_key_file_picked(app, opt)
        }
        RedisToolMessage::PickTlsPublicCertFile => draft_inputs::pick_tls_public_cert_file(),
        RedisToolMessage::TlsPublicCertFilePicked(opt) => {
            draft_inputs::tls_public_cert_file_picked(app, opt)
        }
        RedisToolMessage::PickTlsCaCertFile => draft_inputs::pick_tls_ca_cert_file(),
        RedisToolMessage::TlsCaCertFilePicked(opt) => {
            draft_inputs::tls_ca_cert_file_picked(app, opt)
        }
        RedisToolMessage::DraftSshEnabledToggled(value) => {
            draft_inputs::draft_ssh_enabled_toggled(app, value)
        }
        RedisToolMessage::DraftSshHostChanged(value) => {
            draft_inputs::draft_ssh_host_changed(app, value)
        }
        RedisToolMessage::DraftSshPortChanged(value) => {
            draft_inputs::draft_ssh_port_changed(app, value)
        }
        RedisToolMessage::DraftSshUsernameChanged(value) => {
            draft_inputs::draft_ssh_username_changed(app, value)
        }
        RedisToolMessage::DraftSshPasswordChanged(value) => {
            draft_inputs::draft_ssh_password_changed(app, value)
        }
        RedisToolMessage::DraftSshPrivateKeyPathChanged(value) => {
            draft_inputs::draft_ssh_private_key_path_changed(app, value)
        }
        RedisToolMessage::PickSshPrivateKeyFile => draft_inputs::pick_ssh_private_key_file(),
        RedisToolMessage::SshPrivateKeyFilePicked(opt) => {
            draft_inputs::ssh_private_key_file_picked(app, opt)
        }
        RedisToolMessage::DraftSshPassphraseChanged(value) => {
            draft_inputs::draft_ssh_passphrase_changed(app, value)
        }
        RedisToolMessage::DraftSshTimeoutSecsChanged(value) => {
            draft_inputs::draft_ssh_timeout_secs_changed(app, value)
        }
        RedisToolMessage::DraftSentinelEnabledToggled(value) => {
            draft_inputs::draft_sentinel_enabled_toggled(app, value)
        }
        RedisToolMessage::DraftSentinelMasterNameChanged(value) => {
            draft_inputs::draft_sentinel_master_name_changed(app, value)
        }
        RedisToolMessage::DraftSentinelNodePasswordChanged(value) => {
            draft_inputs::draft_sentinel_node_password_changed(app, value)
        }
        RedisToolMessage::DraftClusterToggled(value) => {
            draft_inputs::draft_cluster_toggled(app, value)
        }
        RedisToolMessage::DraftReadOnlyToggled(value) => {
            draft_inputs::draft_read_only_toggled(app, value)
        }
        RedisToolMessage::DraftKeyPatternChanged(value) => {
            draft_inputs::draft_key_pattern_changed(app, value)
        }
        RedisToolMessage::SaveDraft => operations::save_draft(app),
        RedisToolMessage::DeleteSelected => operations::delete_selected(app),
        RedisToolMessage::TestSelected => operations::test_selected(app),
        RedisToolMessage::TestSelectedCompleted(result) => {
            operations::test_selected_completed(app, result)
        }
        RedisToolMessage::CopySelectedUri => operations::copy_selected_uri(app),
        RedisToolMessage::ExportConfigs => operations::export_configs(app),
        RedisToolMessage::ExportCompleted(result) => operations::export_completed(app, result),
        RedisToolMessage::ImportConfigs => operations::import_configs(app),
        RedisToolMessage::ImportCompleted(result) => operations::import_completed(app, result),
        RedisToolMessage::DefaultLoadCountChanged(value) => {
            operations::default_load_count_changed(app, value)
        }
        RedisToolMessage::SaveDefaultLoadCount => operations::save_default_load_count(app),
        RedisToolMessage::IncreaseDefaultLoadCount => operations::increase_default_load_count(app),
        RedisToolMessage::DecreaseDefaultLoadCount => operations::decrease_default_load_count(app),
        RedisToolMessage::CreateKeyNameChanged(value) => {
            operations::create_key_name_changed(app, value)
        }
        RedisToolMessage::CreateKeyTypeChanged(value) => {
            operations::create_key_type_changed(app, value)
        }
        RedisToolMessage::ConfirmCreateKey => operations::confirm_create_key(app),
        RedisToolMessage::CreateKeyCompleted {
            connection_id,
            key,
            result,
        } => operations::create_key_completed(app, connection_id, key, result),
        RedisToolMessage::CommandInputChanged(value) => {
            operations::command_input_changed(app, value)
        }
        RedisToolMessage::RunCommand => operations::run_command(app),
        RedisToolMessage::CommandCompleted {
            connection_id,
            result,
        } => operations::command_completed(app, connection_id, result),
        RedisToolMessage::KeyBrowserPatternChanged(value) => {
            navigation::key_browser_pattern_changed(app, value)
        }
        RedisToolMessage::ToggleKeyTreePath(path) => navigation::toggle_key_tree_path(app, path),
        RedisToolMessage::InfoFilterChanged(value) => navigation::info_filter_changed(app, value),
        RedisToolMessage::HistoryPreviousPage => navigation::history_previous_page(app),
        RedisToolMessage::HistoryNextPage => navigation::history_next_page(app),
        RedisToolMessage::HistoryFilterChanged(value) => {
            navigation::history_filter_changed(app, value)
        }
        RedisToolMessage::HistoryOnlyWriteToggled(value) => {
            navigation::history_only_write_toggled(app, value)
        }
        RedisToolMessage::SnapshotLoaded {
            success_message,
            result,
        } => navigation::snapshot_loaded(app, success_message, result),
        RedisToolMessage::ClearNotification => navigation::clear_notification(app),
        RedisToolMessage::ClearGatewayError => navigation::clear_gateway_error(app),
    }
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod update_tests;
