    use super::*;

    #[test]
    fn test_extract_chunks_minimal() {
        // 创建最小有效的 .fig 文件结构
        let mut bytes = Vec::new();

        // 魔法头
        bytes.extend_from_slice(b"fig-kiwi");

        // 版本(48，小端)
        bytes.extend_from_slice(&48u32.to_le_bytes());

        // 块 0：长度 5
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.extend_from_slice(b"chunk");

        // 块 1：长度 4
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(b"data");

        let result = extract_chunks(&bytes).unwrap();
        assert_eq!(result.version, 48);
        assert_eq!(result.chunks.len(), 2);
        assert_eq!(result.chunks[0], b"chunk");
        assert_eq!(result.chunks[1], b"data");
    }

    #[test]
    fn test_extract_chunks_multiple() {
        // 使用多个块进行测试(模式+数据+图像)
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&101u32.to_le_bytes());

        // 三块
        for i in 0..3 {
            let chunk_data = format!("chunk{}", i);
            bytes.extend_from_slice(&(chunk_data.len() as u32).to_le_bytes());
            bytes.extend_from_slice(chunk_data.as_bytes());
        }

        let result = extract_chunks(&bytes).unwrap();
        assert_eq!(result.version, 101);
        assert_eq!(result.chunks.len(), 3);
    }

    #[test]
    fn test_extract_chunks_file_too_small() {
        let bytes = b"fig-kiwi\x00";
        let result = extract_chunks(bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::FileTooSmall { .. }) => (),
            _ => panic!("Expected FileTooSmall error"),
        }
    }

    #[test]
    fn test_extract_chunks_incomplete_chunk() {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&48u32.to_le_bytes());

        // 长度为 100 但只有 5 个字节的数据
        bytes.extend_from_slice(&100u32.to_le_bytes());
        bytes.extend_from_slice(b"short");

        let result = extract_chunks(&bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::IncompleteChunk { .. }) => (),
            _ => panic!("Expected IncompleteChunk error"),
        }
    }

    #[test]
    fn test_extract_chunks_not_enough_chunks() {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"fig-kiwi");
        bytes.extend_from_slice(&48u32.to_le_bytes());

        // 只有一个块(至少需要 2 个)
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.extend_from_slice(b"chunk");

        let result = extract_chunks(&bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::NotEnoughChunks { expected, actual }) => {
                assert_eq!(expected, 2);
                assert_eq!(actual, 1);
            }
            _ => panic!("Expected NotEnoughChunks error"),
        }
    }
