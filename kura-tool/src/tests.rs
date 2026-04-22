use crate::*;

mod tool_executor {
    use super::*;

    #[test]
    fn new_has_builtin_tools() {
        let tools = ToolExecutor::new();
        let defs = tools.definitions();
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"file_read"));
        assert!(names.contains(&"file_write"));
        assert!(names.contains(&"file_edit"));
        assert!(names.contains(&"glob"));
        assert!(names.contains(&"grep"));
        assert!(names.contains(&"git"));
        assert!(names.contains(&"web_fetch"));
        assert!(names.contains(&"web_search"));
        assert!(names.contains(&"code_search"));
    }

    #[test]
    fn definitions_have_schemas() {
        let tools = ToolExecutor::new();
        for def in tools.definitions() {
            assert!(!def.name.is_empty());
            assert!(!def.description.is_empty());
            assert!(def.input_schema.is_object());
        }
    }

    #[test]
    fn unknown_tool_returns_error() {
        let tools = ToolExecutor::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tools.execute("nonexistent", serde_json::json!({})));
        assert!(result.is_err());
    }

    #[test]
    fn register_custom_tool() {
        let mut tools = ToolExecutor::new();
        tools.register_tool(
            "my_tool".into(),
            kura_core::ToolKind::Custom,
            "A custom tool".into(),
            serde_json::json!({"type": "object", "properties": {"input": {"type": "string"}}}),
            false,
            Some("echo".into()),
            vec![],
            None,
        );
        let defs = tools.definitions();
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"my_tool"));
    }

    #[tokio::test]
    async fn file_read_missing_path() {
        let tools = ToolExecutor::new();
        let result = tools.execute("file_read", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn file_write_then_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let path_str = path.to_str().unwrap().to_string();

        let tools = ToolExecutor::new();

        let write_result = tools
            .execute(
                "file_write",
                serde_json::json!({
                    "path": path_str,
                    "content": "hello world\nline two\n"
                }),
            )
            .await
            .unwrap();
        assert!(write_result.contains("wrote"));

        let read_result = tools
            .execute(
                "file_read",
                serde_json::json!({
                    "path": path_str
                }),
            )
            .await
            .unwrap();
        assert!(read_result.contains("hello world"));
    }

    #[tokio::test]
    async fn file_edit_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("edit.txt");
        let path_str = path.to_str().unwrap().to_string();

        let tools = ToolExecutor::new();

        tools
            .execute(
                "file_write",
                serde_json::json!({
                    "path": &path_str,
                    "content": "foo bar baz"
                }),
            )
            .await
            .unwrap();

        let edit_result = tools
            .execute(
                "file_edit",
                serde_json::json!({
                    "path": &path_str,
                    "old_string": "bar",
                    "new_string": "qux"
                }),
            )
            .await
            .unwrap();
        assert!(edit_result.contains("replaced"));

        let read_result = tools
            .execute(
                "file_read",
                serde_json::json!({
                    "path": &path_str
                }),
            )
            .await
            .unwrap();
        assert!(read_result.contains("qux"));
        assert!(!read_result.contains("bar"));
    }

    #[tokio::test]
    async fn file_edit_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nofile.txt");
        let path_str = path.to_str().unwrap().to_string();

        let tools = ToolExecutor::new();
        tools
            .execute(
                "file_write",
                serde_json::json!({
                    "path": &path_str,
                    "content": "hello"
                }),
            )
            .await
            .unwrap();

        let result = tools
            .execute(
                "file_edit",
                serde_json::json!({
                    "path": &path_str,
                    "old_string": "nonexistent",
                    "new_string": "replacement"
                }),
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn shell_echo() {
        let tools = ToolExecutor::new();
        let result = tools
            .execute(
                "bash",
                serde_json::json!({
                    "command": "echo hello"
                }),
            )
            .await
            .unwrap();
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn guardrail_blocks_dangerous() {
        let tools = ToolExecutor::new();
        let result = tools
            .execute(
                "bash",
                serde_json::json!({
                    "command": "rm -rf /"
                }),
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("guardrail"));
    }
}
