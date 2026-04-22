use crate::executor::ToolExecutor;

impl ToolExecutor {
    pub async fn execute_mcp_bridge(
        &self,
        server: &str,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "MCP bridge to '{}' method '{}' with params: {}",
            server, method, params
        ))
    }
}
