use std::sync::Arc;

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::types::AcpMessageDirection;

use super::{MessageTap, TappedReader, TappedWriter};

#[tokio::test]
async fn tap_buffers_partial_lines_and_ignores_non_jsonrpc_payloads() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let tap = MessageTap::new(
        AcpMessageDirection::Inbound,
        Some({
            let seen = Arc::clone(&seen);
            Arc::new(move |direction, message| {
                seen.lock().push((direction, message));
            })
        }),
        None,
    );
    let (mut writer, reader) = tokio::io::duplex(128);
    writer
        .write_all(br#"{"jsonrpc":"2.0","method":"session/up"#)
        .await
        .expect("write partial input");
    assert!(seen.lock().is_empty());
    writer.write_all(br#"date"}"#).await.expect("write rest");
    writer
        .write_all(b"\nnot-json\n{\"jsonrpc\":\"1.0\",\"method\":\"bad\"}\n")
        .await
        .expect("write invalid input");
    drop(writer);

    let mut tapped_reader = TappedReader::new(reader, tap);
    let mut output = Vec::new();
    tapped_reader.read_to_end(&mut output).await.expect("read tapped");

    let seen = seen.lock();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0, AcpMessageDirection::Inbound);
}

#[tokio::test]
async fn tapped_reader_and_writer_forward_successful_bytes() {
    let read_seen = Arc::new(Mutex::new(0));
    let reader_tap = MessageTap::new(
        AcpMessageDirection::Inbound,
        Some({
            let read_seen = Arc::clone(&read_seen);
            Arc::new(move |_, _| *read_seen.lock() += 1)
        }),
        None,
    );
    let input = tokio::io::duplex(128);
    let (mut writer, reader) = input;
    writer.write_all(br#"{"jsonrpc":"2.0","method":"session/update"}"#).await.expect("write input");
    writer.write_all(b"\n").await.expect("write newline");
    drop(writer);

    let mut tapped_reader = TappedReader::new(reader, reader_tap);
    let mut sink = Vec::new();
    tapped_reader.read_to_end(&mut sink).await.expect("read tapped");
    assert_eq!(*read_seen.lock(), 1);

    let write_seen = Arc::new(Mutex::new(0));
    let writer_tap = MessageTap::new(
        AcpMessageDirection::Outbound,
        Some({
            let write_seen = Arc::clone(&write_seen);
            Arc::new(move |_, _| *write_seen.lock() += 1)
        }),
        None,
    );
    let (client, mut server) = tokio::io::duplex(128);
    let mut tapped_writer = TappedWriter::new(client, writer_tap);
    tapped_writer
        .write_all(br#"{"jsonrpc":"2.0","id":1,"result":{}}"#)
        .await
        .expect("write tapped");
    tapped_writer.write_all(b"\n").await.expect("write newline");
    tapped_writer.shutdown().await.expect("shutdown");
    let mut output = Vec::new();
    server.read_to_end(&mut output).await.expect("read output");

    assert_eq!(*write_seen.lock(), 1);
}
