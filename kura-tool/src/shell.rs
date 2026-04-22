use crate::executor::ToolExecutor;

impl ToolExecutor {
    pub async fn execute_shell(
        &self,
        input: &serde_json::Value,
        guardrail: bool,
    ) -> anyhow::Result<String> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'command' field"))?;

        if guardrail {
            Self::guardrail_check(command)?;
        }

        let workdir = input.get("workdir").and_then(|v| v.as_str());

        let mut cmd = tokio::process::Command::new("bash");
        cmd.arg("-c").arg(command);
        if let Some(dir) = workdir {
            cmd.current_dir(dir);
        }
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            anyhow::bail!("exit {}: {}", output.status, stderr);
        }

        if stderr.is_empty() {
            Ok(stdout)
        } else {
            Ok(format!("{}\n{}", stdout, stderr))
        }
    }

    fn guardrail_check(command: &str) -> anyhow::Result<()> {
        let dangerous = [
            "rm -rf /",
            "mkfs",
            "dd if=",
            ":(){ :|:&",
            "> /dev/sd",
            "curl | bash",
            "wget | bash",
            "chmod -R 777 /",
        ];
        for pattern in &dangerous {
            if command.contains(pattern) {
                anyhow::bail!("guardrail: blocked dangerous pattern '{}'", pattern);
            }
        }
        Ok(())
    }

    pub async fn execute_git(&self, input: &serde_json::Value) -> anyhow::Result<String> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'command' field"))?;

        Self::guardrail_check(command)?;

        let workdir = input.get("workdir").and_then(|v| v.as_str());
        let mut cmd = tokio::process::Command::new("git");
        cmd.arg("-c").arg(command);
        if let Some(dir) = workdir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            anyhow::bail!("git exit {}: {}", output.status, stderr);
        }
        Ok(stdout)
    }
}
