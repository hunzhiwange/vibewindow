    use super::normalize_tool_ids;

    #[test]
    fn normalize_tool_ids_sorts_and_dedups() {
        let tools = normalize_tool_ids(vec![
            "bash".to_string(),
            String::new(),
            "file_read".to_string(),
            "bash".to_string(),
        ]);

        assert_eq!(tools, vec!["bash".to_string(), "file_read".to_string()]);
    }
