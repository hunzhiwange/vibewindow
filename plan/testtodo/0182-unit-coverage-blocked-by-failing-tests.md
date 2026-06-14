# 覆盖任务：unit-coverage 阻塞修复

- 目标命令：`make unit-coverage`
- 当前结果：失败
- 通过用例：6287
- 失败用例：15
- 忽略用例：3
- 覆盖率报告状态：未生成新的完整报告
- 旧报告位置：`coverage/workspace/html/index.html`
- 旧报告时间：2026-05-25 14:41:50
- 目标结果：命令成功完成并生成最新覆盖率报告

## 失败用例

- `channels::lark::token::token_tests::extract_lark_token_ttl_seconds_accepts_expire_variants_and_defaults`
- `channels::matrix::api::api_tests::resolve_room_alias_reports_sanitized_error_body`
- `channels::whatsapp_storage::app_sync_store_tests::mutation_macs_can_be_inserted_loaded_and_deleted`
- `channels::whatsapp_storage::app_sync_store_tests::sync_key_round_trip_and_missing_key_returns_none`
- `channels::whatsapp_storage::app_sync_store_tests::version_round_trip_preserves_hash_state`
- `channels::whatsapp_storage::protocol_store_tests::base_key_collision_detection_compares_saved_key_and_deletes`
- `channels::whatsapp_storage::protocol_store_tests::device_registry_round_trip_and_missing_user`
- `channels::whatsapp_storage::protocol_store_tests::forget_marks_are_consumed_once`
- `channels::whatsapp_storage::protocol_store_tests::lid_mapping_round_trip_all_and_latest_phone_mapping`
- `channels::whatsapp_storage::protocol_store_tests::skdm_recipients_are_inserted_deduplicated_and_cleared`
- `channels::whatsapp_storage::protocol_store_tests::tc_tokens_round_trip_list_delete_and_expire`
- `channels::whatsapp_storage::signal_store_tests::identity_session_and_sender_key_round_trips`
- `channels::whatsapp_storage::signal_store_tests::prekeys_and_signed_prekeys_round_trip_and_delete`
- `runtime::wasm::tests::tests::execute_module_errors_when_tools_path_is_file`
- `tools::browser::native_backend::session::session_tests::ensure_session_reports_webdriver_connection_context`

## 测试任务

- 先修复上述失败用例，避免覆盖率命令在生成报告前退出。
- 修复后重新执行 `make unit-coverage`。
- 仅基于最新报告生成未达到 100% 的覆盖任务。
- 不使用旧报告覆盖当前任务结果。

## 验收命令

- `make unit-coverage`
