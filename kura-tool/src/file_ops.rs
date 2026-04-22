use crate::executor::ToolExecutor;

impl ToolExecutor {
    pub async fn execute_file_read(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'path' field"))?;

        let offset: usize = input.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let limit: usize = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(2000) as usize;

        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let selected: Vec<String> = lines
            .iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, line)| format!("{:>6}\t{}", offset + i + 1, line))
            .collect();

        Ok(selected.join("\n"))
    }

    pub async fn execute_file_write(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'path' field"))?;
        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'content' field"))?;

        tokio::fs::write(path, content).await?;
        Ok(format!("wrote {} bytes to {}", content.len(), path))
    }

    pub async fn execute_file_edit(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'path' field"))?;
        let old_string = input
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'old_string' field"))?;
        let new_string = input
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'new_string' field"))?;

        let content = tokio::fs::read_to_string(path).await?;

        let count = content.matches(old_string).count();
        if count == 0 {
            anyhow::bail!("old_string not found in {}", path);
        }
        if count > 1 {
            anyhow::bail!(
                "old_string found {} times in {} (provide more context)",
                count,
                path
            );
        }

        let new_content = content.replacen(old_string, new_string, 1);
        tokio::fs::write(path, &new_content).await?;
        Ok(format!("replaced in {}", path))
    }
}
