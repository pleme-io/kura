//! DAG Executor - runs the prompt graph with retries, verifications, and message passing

use crate::dag::{Node, NodeId, NodeKind, PromptDag, VerificationKind, VerificationConfig, BackoffStrategy, RetryConfig};
use crate::state::{ExecutionContext, ExecutionState};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DagResult {
    pub success: bool,
    pub node_results: HashMap<NodeId, NodeResult>,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub output: Option<String>,
    pub error: Option<String>,
    pub attempts: u32,
    pub state: ExecutionState,
}

impl DagResult {
    pub fn all_success(&self) -> bool {
        self.success && self.node_results.values().all(|r| r.state == ExecutionState::Success)
    }
}

pub struct DagExecutor {
    dag: PromptDag,
    context: Arc<ExecutionContext>,
}

impl DagExecutor {
    pub fn new(dag: PromptDag) -> Self {
        Self {
            dag,
            context: Arc::new(ExecutionContext::new()),
        }
    }

    pub async fn execute(&self) -> Result<DagResult> {
        for node in &self.dag.nodes {
            self.context.init_node(node.id.clone());
        }

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
        
        for node in &self.dag.nodes {
            in_degree.insert(node.id.0.clone(), node.depends_on.len());
            for dep in &node.depends_on {
                dependents.entry(dep.0.clone()).or_default().push(node.id.0.clone());
            }
        }

        let mut queue: Vec<String> = in_degree.iter()
            .filter(|(_id, deg)| **deg == 0)
            .map(|(id, _deg)| id.clone())
            .collect();

        let mut completed = HashSet::new();
        let mut node_results: HashMap<NodeId, NodeResult> = HashMap::new();

        while !queue.is_empty() {
            let ready: Vec<String> = queue.drain(..).collect();
            
            for node_id_str in &ready {
                let node = self.dag.nodes.iter().find(|n| &n.id.0 == node_id_str).unwrap();
                let result = self.execute_node(node).await;
                
                let state = if result.is_ok() { ExecutionState::Success } else { ExecutionState::Failed };

                self.context.set_state(&node.id, state);
                
                let attempts = self.context.get_attempts(&node.id);
                let output = result.as_ref().ok().cloned();
                let error = result.as_ref().err().map(|e| e.to_string());
                node_results.insert(node.id.clone(), NodeResult {
                    output,
                    error,
                    attempts,
                    state,
                });

                if state == ExecutionState::Success {
                    completed.insert(node_id_str.clone());
                    
                    if let Some(deps) = dependents.get(node_id_str) {
                        for dependent_id in deps {
                            if !completed.contains(dependent_id) {
                                *in_degree.get_mut(dependent_id).unwrap() -= 1;
                                if *in_degree.get(dependent_id).unwrap() == 0 {
                                    queue.push(dependent_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        let success = completed.len() == self.dag.nodes.len();
        
        let mut final_context = self.dag.context.clone();
        for (k, v) in self.context.get_all_outputs() {
            final_context.insert(k.0, v);
        }

        Ok(DagResult {
            success,
            node_results,
            context: final_context,
        })
    }

    async fn execute_node(&self, node: &Node) -> Result<String> {
        self.context.set_state(&node.id, ExecutionState::Running);
        let attempts = self.context.increment_attempts(&node.id);
        let max_retries = node.retries.max_retries;
        
        loop {
            let result: Result<String> = match &node.kind {
                NodeKind::Prompt => Ok(format!("[EXEC] {}", self.substitute_inputs(&node.prompt, node)?)),
                NodeKind::PromptWithModel { model } => Ok(format!("[EXEC {}] {}", model, self.substitute_inputs(&node.prompt, node)?)),
                NodeKind::Shell { command } => {
                    let expanded = self.substitute_inputs(command, node)?;
                    let output = tokio::process::Command::new("sh")
                        .arg("-c")
                        .arg(&expanded)
                        .output()
                        .await?;
                    if output.status.success() {
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    } else {
                        anyhow::bail!("shell failed: {}", String::from_utf8_lossy(&output.stderr))
                    }
                }
                NodeKind::FileRead { path } => {
                    let expanded = self.substitute_inputs(path, node)?;
                    tokio::fs::read_to_string(&expanded).await.map_err(|e| anyhow::anyhow!("{}", e))
                }
                NodeKind::Conditional { condition } => Ok(self.substitute_inputs(condition, node)?),
                NodeKind::FanOut { count } => Ok(format!("[FANOUT {}]", count)),
                NodeKind::FanIn => Ok(self.context.get_output(&node.id).unwrap_or_default()),
            };

            match result {
                Ok(output) => {
                    let verified = if let Some(verify) = &node.verification {
                        self.verify_output(&output, verify)
                    } else { true };
                    
                    if verified || !node.verification.as_ref().map(|v| v.required).unwrap_or(false) {
                        self.context.set_output(&node.id, output.clone());
                        return Ok(output);
                    } else {
                        anyhow::bail!("verification failed")
                    }
                }
                Err(e) => {
                    if attempts < max_retries {
                        self.context.set_state(&node.id, ExecutionState::Retrying);
                        let delay = self.calculate_backoff(attempts, &node.retries);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    } else {
                        self.context.set_error(&node.id, e.to_string());
                        return Err(e);
                    }
                }
            }
        }
    }

    fn substitute_inputs(&self, template: &str, node: &Node) -> Result<String> {
        let mut result = template.to_string();
        for (key, binding) in &node.inputs {
            let value = match binding {
                crate::dag::InputBinding::Context { key } => self.context.get_context(key).unwrap_or_default(),
                crate::dag::InputBinding::Node { id, output: _ } => self.context.get_output(id).unwrap_or_default(),
                crate::dag::InputBinding::Default { value } => value.clone(),
                crate::dag::InputBinding::Env { name } => std::env::var(name).unwrap_or_default(),
                crate::dag::InputBinding::File { path } => path.clone(),
            };
            result = result.replace(&format!("${{{}}}", key), &value);
            result = result.replace(&format!("${}", key), &value);
        }
        Ok(result)
    }

    fn verify_output(&self, output: &str, config: &VerificationConfig) -> bool {
        match &config.kind {
            VerificationKind::Match { pattern } => output.contains(pattern),
            VerificationKind::JsonValid => serde_json::from_str::<serde_json::Value>(output).is_ok(),
            VerificationKind::NonEmpty => !output.trim().is_empty(),
            VerificationKind::Command { command } => {
                std::process::Command::new("sh").arg("-c").arg(command).output().map(|o| o.status.success()).unwrap_or(false)
            }
            VerificationKind::LLM { prompt: _ } => true,
        }
    }

    fn calculate_backoff(&self, attempt: u32, config: &RetryConfig) -> u64 {
        match config.backoff {
            BackoffStrategy::Fixed => config.delay_ms,
            BackoffStrategy::Linear => config.delay_ms * attempt as u64,
            BackoffStrategy::Exponential => config.delay_ms * 2u64.pow(attempt.saturating_sub(1)),
            BackoffStrategy::Jitter => config.delay_ms,
        }
    }
}

pub async fn run_file(path: impl AsRef<std::path::Path>) -> Result<DagResult> {
    let dag = crate::parser::parse_file(path)?;
    let executor = DagExecutor::new(dag);
    executor.execute().await
}

pub async fn run_str(content: &str) -> Result<DagResult> {
    let dag = crate::parser::parse_yaml(content)?;
    let executor = DagExecutor::new(dag);
    executor.execute().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_dag() {
        let yaml = r#"
name: simple-test
nodes:
  - id: node1
    type: Prompt
    prompt: Hello World
  - id: node2
    type: Prompt
    prompt: Response
    depends_on:
      - node1
"#;
        let result = run_str(yaml).await.unwrap();
        assert!(result.all_success());
    }

    #[tokio::test]
    async fn test_shell_node() {
        let yaml = r#"
name: shell-test
nodes:
  - id: echo-test
    type: Shell
    shell:
      command: "echo hello"
"#;
        let result = run_str(yaml).await.unwrap();
        assert!(result.all_success());
    }

    #[tokio::test]
    async fn test_input_substitution() {
        let yaml = r#"
name: substitution-test
context:
  greeting: "Hello"
nodes:
  - id: node1
    type: Prompt
    prompt: "$greeting World"
"#;
        let result = run_str(yaml).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_parallel_nodes() {
        let yaml = r#"
name: parallel-test
nodes:
  - id: node1
    type: Shell
    shell:
      command: "echo node1"
  - id: node2
    type: Shell
    shell:
      command: "echo node2"
"#;
        let result = run_str(yaml).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_dependency_chain() {
        let yaml = r#"
name: chain-test
nodes:
  - id: step1
    type: Shell
    shell:
      command: "echo step1"
  - id: step2
    type: Shell
    shell:
      command: "echo step2"
    depends_on:
      - step1
  - id: step3
    type: Shell
    shell:
      command: "echo step3"
    depends_on:
      - step2
"#;
        let result = run_str(yaml).await.unwrap();
        assert!(result.success);
    }
}