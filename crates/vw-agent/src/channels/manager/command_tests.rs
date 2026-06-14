use super::*;

#[tokio::test]
async fn handle_command_rejects_runtime_owned_commands() {
    let config = Config::default();

    let start = handle_command(ChannelCommands::Start, &config).await.expect_err("start");
    assert!(start.to_string().contains("main.rs"));

    let doctor = handle_command(ChannelCommands::Doctor, &config).await.expect_err("doctor");
    assert!(doctor.to_string().contains("main.rs"));
}

#[tokio::test]
async fn handle_command_list_accepts_default_config() {
    handle_command(ChannelCommands::List, &Config::default())
        .await
        .expect("list should only print configured channels");
}

#[tokio::test]
async fn handle_command_add_and_remove_report_manual_configuration() {
    let config = Config::default();

    let add = handle_command(
        ChannelCommands::Add { channel_type: "unknown".to_string(), config: "{}".to_string() },
        &config,
    )
    .await
    .expect_err("add");
    assert!(add.to_string().contains("unknown"));

    let remove = handle_command(ChannelCommands::Remove { name: "telegram".to_string() }, &config)
        .await
        .expect_err("remove");
    assert!(remove.to_string().contains("telegram"));
}
