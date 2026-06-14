use super::*;
use tempfile::TempDir;
use tokio::time::{Duration, sleep};

#[test]
fn size_and_status_debug_are_stable() {
    let size = Size { cols: 80, rows: 24 };
    assert_eq!(size.cols, 80);
    assert!(format!("{:?}", Status::Running).contains("Running"));
}

#[test]
fn buffer_state_tracks_cursors_and_trims_old_bytes() {
    let mut buffer = BufferState::new();
    assert_eq!(buffer.slice_from(-1), (Vec::new(), 0, 0));

    buffer.push(b"hello");
    assert_eq!(buffer.slice_from(0), (b"hello".to_vec(), 0, 5));
    assert_eq!(buffer.slice_from(2), (b"llo".to_vec(), 0, 5));
    assert_eq!(buffer.slice_from(99), (Vec::new(), 0, 5));

    buffer.push(&vec![b'a'; BUFFER_LIMIT + 3]);
    let (bytes, start, end) = buffer.slice_from(0);
    assert_eq!(bytes.len(), BUFFER_LIMIT);
    assert_eq!(start, end - BUFFER_LIMIT);
}

#[test]
fn pty_error_display_and_conversions_are_stable() {
    let invalid = Error::Invalid("bad input".to_string());
    assert_eq!(invalid.to_string(), "bad input");

    let io: Error = std::io::Error::new(std::io::ErrorKind::Other, "io boom").into();
    assert_eq!(io.to_string(), "io boom");

    let json: Error = serde_json::from_str::<serde_json::Value>("{").unwrap_err().into();
    assert!(json.to_string().contains("EOF"));
}

#[cfg(all(not(target_arch = "wasm32"), unix))]
#[tokio::test]
async fn create_update_read_write_and_remove_session() {
    let temp = TempDir::new().expect("tempdir should create");
    let info = create(CreateInput {
        command: Some("/bin/sh".to_string()),
        args: Some(vec!["-c".to_string(), "printf pty-ok; sleep 2".to_string()]),
        cwd: Some(temp.path().to_string_lossy().to_string()),
        title: Some("Test PTY".to_string()),
        env: Some(HashMap::from([("VW_PTY_TEST".to_string(), "1".to_string())])),
    })
    .await
    .expect("pty should create");

    assert_eq!(info.title, "Test PTY");
    assert_eq!(get(&info.id).await.expect("session should exist").id, info.id);
    assert!(list().await.iter().any(|s| s.id == info.id));

    let updated = update(
        &info.id,
        UpdateInput {
            title: Some("Renamed PTY".to_string()),
            size: Some(Size { rows: 30, cols: 100 }),
        },
    )
    .await
    .expect("update should succeed")
    .expect("session should update");
    assert_eq!(updated.title, "Renamed PTY");

    sleep(Duration::from_millis(150)).await;
    let (data, cursor) = read(&info.id, 0).await.expect("session should read");
    assert!(cursor >= data.len());
    assert!(data.contains("pty-ok"));
    let (empty, _) = read(&info.id, -1).await.expect("session should read from end");
    assert!(empty.is_empty());

    write(&info.id, "\n").await;
    resize(&info.id, 120, 40).await;
    assert!(remove(&info.id).await.expect("remove should succeed"));
    assert!(remove("missing-session").await.expect("missing remove should succeed"));
}
