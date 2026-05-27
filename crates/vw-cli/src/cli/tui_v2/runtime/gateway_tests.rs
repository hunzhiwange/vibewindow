    use super::*;
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn gateway_health_probe_keeps_write_side_open_until_response_arrives() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind health probe fixture");
        let port = listener
            .local_addr()
            .expect("read fixture addr")
            .port();

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept health probe client");
            stream
                .set_read_timeout(Some(Duration::from_millis(50)))
                .expect("set fixture read timeout");

            let mut request = Vec::new();
            let mut buffer = [0_u8; 512];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut buffer).expect("read health probe request");
                if read == 0 {
                    return;
                }
                request.extend_from_slice(&buffer[..read]);
            }

            let mut extra = [0_u8; 1];
            match stream.read(&mut extra) {
                Ok(0) => return,
                Ok(_) => {}
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(err) => panic!("unexpected fixture read error: {err}"),
            }

            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"ok\"}",
                )
                .expect("write health probe response");
        });

        let endpoint = GatewayEndpoint::new("127.0.0.1", port);

        assert!(gateway_health_ready(&endpoint));

        server.join().expect("join health probe fixture");
    }
