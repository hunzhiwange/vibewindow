    use super::*;

    #[test]
    fn test_detect_figma_header() {
        let bytes = b"fig-kiwi\x00\x00\x00\x00";
        let result = detect_file_type(bytes).unwrap();
        assert_eq!(result, FileType::Figma);
    }

    #[test]
    fn test_detect_figjam_header() {
        let bytes = b"fig-jam.\x00\x00\x00\x00";
        let result = detect_file_type(bytes).unwrap();
        assert_eq!(result, FileType::FigJam);
    }

    #[test]
    fn test_invalid_header() {
        let bytes = b"invalid!";
        let result = detect_file_type(bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_too_small() {
        let bytes = b"fig";
        let result = detect_file_type(bytes);
        assert!(result.is_err());
        match result {
            Err(FigError::FileTooSmall { expected, actual }) => {
                assert_eq!(expected, 8);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected FileTooSmall error"),
        }
    }

    #[test]
    fn test_is_zip_container() {
        // 有效的 ZIP 签名
        let zip_bytes = b"PK\x03\x04";
        assert!(is_zip_container(zip_bytes));

        // 不是邮政编码
        let fig_bytes = b"fig-kiwi";
        assert!(!is_zip_container(fig_bytes));

        // 太小
        let small_bytes = b"P";
        assert!(!is_zip_container(small_bytes));
    }
