    use super::*;
    use flate2::Compression;
    use flate2::write::DeflateEncoder;
    use std::io::Write;

    #[test]
    fn test_is_already_compressed_png() {
        // PNG魔法：[137,80,78,71,...]
        let png_data = vec![137, 80, 78, 71, 13, 10, 26, 10];
        assert!(is_already_compressed(&png_data));
    }

    #[test]
    fn test_is_already_compressed_jpeg() {
        // JPEG 魔法：[255, 216, ...]
        let jpeg_data = vec![255, 216, 255, 224, 0, 16];
        assert!(is_already_compressed(&jpeg_data));
    }

    #[test]
    fn test_is_not_compressed() {
        // 随机数据
        let data = vec![120, 156, 1, 2, 3, 4, 5];
        assert!(!is_already_compressed(&data));
    }

    #[test]
    fn test_is_not_compressed_too_small() {
        let data = vec![137];
        assert!(!is_already_compressed(&data));
    }

    #[test]
    fn test_decompress_deflate() {
        // 创建测试数据
        let original = b"Hello, Figma! This is a test string for compression.";

        // 使用原始 DEFLATE 压缩(无 zlib 包装器)
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // 解压
        let decompressed = decompress_chunk(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_zstd() {
        // 创建测试数据
        let original = b"Hello, Figma! This is a test string for Zstandard compression.";

        // 使用 Zstandard 压缩
        let compressed = zstd::encode_all(&original[..], 3).unwrap();

        // 解压
        let decompressed = decompress_chunk(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_already_compressed_png() {
        // PNG 数据应按原样返回
        let png_data = vec![137, 80, 78, 71, 13, 10, 26, 10, 1, 2, 3, 4];
        let result = decompress_chunk(&png_data).unwrap();
        assert_eq!(result, png_data);
    }

    #[test]
    fn test_decompress_already_compressed_jpeg() {
        // JPEG 数据应按原样返回
        let jpeg_data = vec![255, 216, 255, 224, 0, 16, 1, 2, 3];
        let result = decompress_chunk(&jpeg_data).unwrap();
        assert_eq!(result, jpeg_data);
    }

    #[test]
    fn test_decompress_invalid_data() {
        // 无效的压缩数据应该失败
        let invalid_data = vec![1, 2, 3, 4, 5];
        let result = decompress_chunk(&invalid_data);
        assert!(result.is_err());
    }
