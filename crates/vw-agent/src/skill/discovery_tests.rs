use super::*;

#[cfg(not(target_arch = "wasm32"))]
fn spawn_http_server(routes: Vec<(String, u16, String)>) -> (String, std::thread::JoinHandle<()>) {
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
    let request_count = routes.len();
    let routes = routes
        .into_iter()
        .map(|(path, status, body)| (path.to_string(), (status, body)))
        .collect::<HashMap<_, _>>();

    let handle = std::thread::spawn(move || {
        for _ in 0..request_count {
            let Ok((mut stream, _)) = listener.accept() else {
                continue;
            };
            let mut buf = [0_u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");
            let (status, body) =
                routes.get(path).cloned().unwrap_or((404, "not found".to_string()));
            let reason = if status == 200 { "OK" } else { "Not Found" };
            let response = format!(
                "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                reason,
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });

    (base_url, handle)
}

#[test]
fn index_deserializes_skill_file_lists() {
    let index: Index = serde_json::from_str(
        r#"{"skills":[{"name":"rust","description":"Rust helper","files":["SKILL.md"]}]}"#,
    )
    .unwrap();

    assert_eq!(index.skills.len(), 1);
    assert_eq!(index.skills[0].name, "rust");
    assert_eq!(index.skills[0].files, vec!["SKILL.md"]);
}

#[tokio::test]
async fn pull_empty_url_returns_no_paths() {
    assert!(pull("  ").await.is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn download_if_missing_treats_existing_file_as_success_without_network() {
    let dir = tempfile::tempdir().expect("temp dir");
    let dest = dir.path().join("skill/SKILL.md");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::write(&dest, "already here").unwrap();

    assert!(download_if_missing("http://127.0.0.1:9/nope", &dest).await);
    assert_eq!(std::fs::read_to_string(dest).unwrap(), "already here");
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn download_if_missing_writes_success_and_reports_http_errors() {
    let dir = tempfile::tempdir().expect("temp dir");
    let (base_url, handle) = spawn_http_server(vec![
        ("/ok.md".to_string(), 200, "downloaded".to_string()),
        ("/missing.md".to_string(), 404, "missing".to_string()),
    ]);

    let ok_dest = dir.path().join("ok.md");
    let missing_dest = dir.path().join("missing.md");

    assert!(download_if_missing(&format!("{base_url}/ok.md"), &ok_dest).await);
    assert_eq!(std::fs::read_to_string(&ok_dest).unwrap(), "downloaded");
    assert!(!download_if_missing(&format!("{base_url}/missing.md"), &missing_dest).await);
    assert!(!missing_dest.exists());

    handle.join().expect("server thread");
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn pull_downloads_valid_index_entries_and_skips_invalid_skills() {
    let skill_name = format!("served-skill-{}", std::process::id());
    let skill_root = dir().join(&skill_name);
    let _ = std::fs::remove_dir_all(&skill_root);

    let index = serde_json::json!({
        "skills": [
            {
                "name": skill_name,
                "description": "served over http",
                "files": ["SKILL.md", "nested/info.md", " ", "missing.md"]
            },
            { "name": "   ", "files": ["SKILL.md"] },
            { "name": "empty-files", "files": [] }
        ]
    })
    .to_string();
    let (base_url, handle) = spawn_http_server(vec![
        ("/index.json".to_string(), 200, index),
        (format!("/{skill_name}/SKILL.md"), 200, "# Served\n\nUse it.".to_string()),
        (format!("/{skill_name}/nested/info.md"), 200, "nested".to_string()),
        (format!("/{skill_name}/missing.md"), 404, "missing".to_string()),
    ]);

    let paths = pull(&base_url).await;

    assert_eq!(paths, vec![skill_root.clone()]);
    assert_eq!(
        std::fs::read_to_string(skill_root.join("SKILL.md")).unwrap(),
        "# Served\n\nUse it."
    );
    assert_eq!(std::fs::read_to_string(skill_root.join("nested/info.md")).unwrap(), "nested");
    assert!(!skill_root.join("missing.md").exists());

    let _ = std::fs::remove_dir_all(&skill_root);
    handle.join().expect("server thread");
}
