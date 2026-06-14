use super::TelegramChannel;
use super::attachments::TelegramAttachment;
use axum::{
    Router,
    body::Body,
    extract::{OriginalUri, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::Response,
    routing::get,
};
use base64::Engine as _;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use std::{collections::VecDeque, io::Cursor, sync::Arc};
use tokio::sync::{Mutex, oneshot};

#[derive(Clone, Debug)]
struct ResponseSpec {
    status: StatusCode,
    body: Vec<u8>,
    content_type: &'static str,
}

struct TestServerState {
    paths: Mutex<Vec<String>>,
    responses: Mutex<VecDeque<ResponseSpec>>,
}

struct TestServer {
    base_url: String,
    state: Arc<TestServerState>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl TestServer {
    async fn spawn(responses: Vec<ResponseSpec>) -> Self {
        let state = Arc::new(TestServerState {
            paths: Mutex::new(Vec::new()),
            responses: Mutex::new(VecDeque::from(responses)),
        });
        let app = Router::new().route("/{*path}", get(record_request)).with_state(state.clone());
        let listener =
            tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.expect("bind test server");
        let addr = listener.local_addr().expect("server addr");
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("serve test server");
        });

        Self { base_url: format!("http://{addr}"), state, shutdown: Some(shutdown_tx) }
    }

    async fn paths(&self) -> Vec<String> {
        self.state.paths.lock().await.clone()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

async fn record_request(
    State(state): State<Arc<TestServerState>>,
    uri: OriginalUri,
) -> Response<Body> {
    state
        .paths
        .lock()
        .await
        .push(uri.path_and_query().map(ToString::to_string).unwrap_or_default());

    let response = state.responses.lock().await.pop_front().expect("test should provide response");

    Response::builder()
        .status(response.status)
        .header(CONTENT_TYPE, response.content_type)
        .body(Body::from(response.body))
        .expect("response should build")
}

fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_fn(width, height, |x, y| {
        if (x + y) % 2 == 0 { Rgb([255, 0, 0]) } else { Rgb([0, 0, 255]) }
    });
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(image)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .expect("encode png");
    bytes
}

#[test]
fn telegram_attachment_preserves_target_verbatim() {
    let attachment = TelegramAttachment {
        kind: super::attachments::TelegramAttachmentKind::Image,
        target: "https://example.test/a b.png".to_string(),
    };

    assert_eq!(attachment.target, "https://example.test/a b.png");
}

#[tokio::test]
async fn resolve_photo_data_uri_downloads_and_encodes_image() {
    let image = png_bytes(8, 6);
    let server = TestServer::spawn(vec![
        ResponseSpec {
            status: StatusCode::OK,
            body: br#"{"ok":true,"result":{"file_path":"photos/file.png"}}"#.to_vec(),
            content_type: "application/json",
        },
        ResponseSpec { status: StatusCode::OK, body: image, content_type: "image/png" },
    ])
    .await;
    let channel = TelegramChannel::new("123:ABC".to_string(), vec![], false)
        .with_api_base(server.base_url.clone());

    let data_uri = channel.resolve_photo_data_uri("photo-id").await.expect("data uri");

    assert!(data_uri.starts_with("data:image/jpeg;base64,"));
    let encoded = data_uri.trim_start_matches("data:image/jpeg;base64,");
    let decoded = base64::engine::general_purpose::STANDARD.decode(encoded).expect("base64 jpeg");
    assert!(decoded.starts_with(&[0xFF, 0xD8]));

    let paths = server.paths().await;
    assert_eq!(
        paths,
        vec!["/bot123:ABC/getFile?file_id=photo-id", "/file/bot123:ABC/photos/file.png"]
    );
}

#[tokio::test]
async fn resolve_photo_data_uri_errors_when_file_path_missing() {
    let server = TestServer::spawn(vec![ResponseSpec {
        status: StatusCode::OK,
        body: br#"{"ok":true,"result":{}}"#.to_vec(),
        content_type: "application/json",
    }])
    .await;
    let channel = TelegramChannel::new("123:ABC".to_string(), vec![], false)
        .with_api_base(server.base_url.clone());

    let error = channel
        .resolve_photo_data_uri("photo-id")
        .await
        .expect_err("missing file path should fail");

    assert!(error.to_string().contains("no file_path"));
    assert_eq!(server.paths().await, vec!["/bot123:ABC/getFile?file_id=photo-id"]);
}
