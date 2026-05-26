use std::path::PathBuf;

use super::client::send_discord_message_with_files;

#[tokio::test]
async fn send_message_with_files_reports_missing_local_file_before_network() {
    let client = reqwest::Client::new();
    let missing = PathBuf::from("/tmp/vibewindow-plan6-missing-attachment.txt");

    let error = send_discord_message_with_files(&client, "token", "channel", "content", &[missing])
        .await
        .unwrap_err()
        .to_string();

    assert!(error.contains("Discord attachment read failed"));
}
