use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn parse_image_markers_extracts_multiple_markers() {
        let input = "Check this [IMAGE:/tmp/a.png] and this [IMAGE:https://example.com/b.jpg]";
        let (cleaned, refs) = parse_image_markers(input);

        assert_eq!(cleaned, "Check this  and this");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], "/tmp/a.png");
        assert_eq!(refs[1], "https://example.com/b.jpg");
    }

    #[test]
    fn parse_image_markers_keeps_invalid_empty_marker() {
        let input = "hello [IMAGE:] world";
        let (cleaned, refs) = parse_image_markers(input);

        assert_eq!(cleaned, "hello [IMAGE:] world");
        assert!(refs.is_empty());
    }

    #[tokio::test]
    async fn prepare_messages_normalizes_local_image_to_data_uri() {
        let temp = tempfile::tempdir().unwrap();
        let image_path = temp.path().join("sample.png");

        // Minimal PNG signature bytes are enough for MIME detection.
        std::fs::write(&image_path, [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']).unwrap();

        let messages = vec![ChatMessage::user(format!(
            "Please inspect this screenshot [IMAGE:{}]",
            image_path.display()
        ))];

        let prepared =
            prepare_messages_for_provider(&messages, &MultimodalConfig::default()).await.unwrap();

        assert!(prepared.contains_images);
        assert_eq!(prepared.messages.len(), 1);

        let (cleaned, refs) = parse_image_markers(&prepared.messages[0].content);
        assert_eq!(cleaned, "Please inspect this screenshot");
        assert_eq!(refs.len(), 1);
        assert!(refs[0].starts_with("data:image/png;base64,"));
    }

    #[tokio::test]
    async fn prepare_messages_rejects_too_many_images() {
        let messages =
            vec![ChatMessage::user("[IMAGE:/tmp/1.png]\n[IMAGE:/tmp/2.png]".to_string())];

        let config =
            MultimodalConfig { max_images: 1, max_image_size_mb: 5, allow_remote_fetch: false };

        let error = prepare_messages_for_provider(&messages, &config)
            .await
            .expect_err("should reject image count overflow");

        assert!(error.to_string().contains("multimodal image limit exceeded"));
    }

    #[tokio::test]
    async fn prepare_messages_rejects_remote_url_when_disabled() {
        let messages =
            vec![ChatMessage::user("Look [IMAGE:https://example.com/img.png]".to_string())];

        let error = prepare_messages_for_provider(&messages, &MultimodalConfig::default())
            .await
            .expect_err("should reject remote image URL when fetch is disabled");

        assert!(error.to_string().contains("multimodal remote image fetch is disabled"));
    }

    #[tokio::test]
    async fn prepare_messages_rejects_oversized_local_image() {
        let temp = tempfile::tempdir().unwrap();
        let image_path = temp.path().join("big.png");

        let bytes = vec![0u8; 1024 * 1024 + 1];
        std::fs::write(&image_path, bytes).unwrap();

        let messages = vec![ChatMessage::user(format!("[IMAGE:{}]", image_path.display()))];
        let config =
            MultimodalConfig { max_images: 4, max_image_size_mb: 1, allow_remote_fetch: false };

        let error = prepare_messages_for_provider(&messages, &config)
            .await
            .expect_err("should reject oversized local image");

        assert!(error.to_string().contains("multimodal image size limit exceeded"));
    }

    #[test]
    fn extract_ollama_image_payload_supports_data_uris() {
        let payload = extract_ollama_image_payload("data:image/png;base64,abcd==")
            .expect("payload should be extracted");
        assert_eq!(payload, "abcd==");
    }
}
