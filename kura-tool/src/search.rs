use crate::executor::ToolExecutor;

impl ToolExecutor {
    pub async fn execute_glob(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'pattern' field"))?;
        let base_path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");

        let mut cmd = tokio::process::Command::new("find");
        cmd.arg(base_path)
            .arg("-name")
            .arg(pattern)
            .arg("-type")
            .arg("f");
        let output = cmd.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn execute_grep(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'pattern' field"))?;
        let path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let include = input.get("include").and_then(|v| v.as_str());

        let mut cmd = tokio::process::Command::new("rg");
        cmd.arg("--line-number").arg("-e").arg(pattern).arg(path);
        if let Some(inc) = include {
            cmd.arg("-g").arg(inc);
        }

        let output = cmd.output().await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(String::new())
        }
    }

    pub async fn execute_web_fetch(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let url = input
            .get("url")
            .or_else(|| input.get("query"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'url' field"))?;

        let client = reqwest::Client::new();
        let resp = client.get(url).send().await?;
        let text = resp.text().await?;
        Ok(text.chars().take(50000).collect())
    }

    pub async fn execute_web_search(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let query = input
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'query' field"))?;

        Ok(format!(
            "web search not yet implemented for query: {}",
            query
        ))
    }

    pub async fn execute_code_search(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let query = input
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'query' field"))?;

        Ok(format!(
            "code search not yet implemented for query: {}",
            query
        ))
    }

    pub async fn execute_mcp(
        &self,
        input: &serde_json::Value,
        mcp_server: &Option<String>,
    ) -> anyhow::Result<String> {
        let method = input
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'method' field"))?;

        let server = mcp_server.as_deref().unwrap_or("default");
        Ok(format!(
            "MCP call to server '{}' method '{}' not yet implemented",
            server, method
        ))
    }

    pub async fn execute_custom(
        &self,
        input: &serde_json::Value,
        command: Option<&str>,
        args: &[String],
    ) -> anyhow::Result<String> {
        let cmd_str = command.ok_or_else(|| anyhow::anyhow!("no command for custom tool"))?;
        let mut cmd = tokio::process::Command::new(cmd_str);
        for arg in args {
            cmd.arg(arg);
        }
        let _ = input.as_str();
        cmd.arg(serde_json::to_string(input)?);
        let output = cmd.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
