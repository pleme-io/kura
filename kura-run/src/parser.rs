//! Parser for prompt DAG definitions
//! 
//! Supports YAML and Lisp formats

use crate::dag::{Node, NodeKind, NodeId, PromptDag, ProviderConfig, RetryConfig, VerificationConfig, VerificationKind, StateMachineConfig, State, StateId, Transition, Event, Action, InputBinding, OutputBinding, OutputTarget, BackoffStrategy};
use anyhow::{Context, Result};
use std::path::Path;
use std::collections::HashMap;

/// Parse a DAG from a file (auto-detect format)
pub fn parse_file(path: impl AsRef<Path>) -> Result<PromptDag> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    
    match path.extension().and_then(|e| e.to_str()) {
        Some("yaml") | Some("yml") => parse_yaml(&content),
        Some("lisp") | Some("cl") => parse_lisp(&content),
        Some("json") => parse_json(&content),
        _ => parse_yaml(&content), // Default to YAML
    }
}

/// Parse from YAML content
pub fn parse_yaml(content: &str) -> Result<PromptDag> {
    serde_yaml_ng::from_str(content)
        .context("parsing YAML")
}

/// Parse from JSON content
pub fn parse_json(content: &str) -> Result<PromptDag> {
    serde_json::from_str(content)
        .context("parsing JSON")
}

/// Parse from Lisp content
pub fn parse_lisp(content: &str) -> Result<PromptDag> {
    // Simple Lisp parser for prompt DAGs
    let mut dag = PromptDag {
        name: String::new(),
        description: String::new(),
        nodes: Vec::new(),
        context: HashMap::new(),
        providers: Vec::new(),
    };
    
    let mut current_node: Option<Node> = None;
    let mut in_defprompt = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        
        // Top-level definitions
        if trimmed.starts_with("(defdag") {
            if let Some(name) = extract_lisp_symbol(trimmed) {
                dag.name = name;
            }
            in_defprompt = false;
            continue;
        }
        
        if trimmed.starts_with("(defdag") {
            in_defprompt = true;
            if let Some(name) = extract_lisp_symbol(trimmed) {
                dag.name = name;
            }
            continue;
        }
        
        if trimmed.starts_with("(desc") {
            if let Some(desc) = extract_lisp_string(trimmed) {
                dag.description = desc;
            }
            continue;
        }
        
        if trimmed.starts_with("(defprovider") {
            if let Some(provider) = parse_lisp_provider(trimmed) {
                dag.providers.push(provider);
            }
            continue;
        }
        
        if trimmed.starts_with("(defctx") {
            if let Some((k, v)) = parse_lisp_context(trimmed) {
                dag.context.insert(k, v);
            }
            continue;
        }
        
        // Node definitions
        if trimmed.starts_with("(defprompt") || trimmed.starts_with("(node") {
            if let Some(node) = parse_lisp_node(trimmed) {
                current_node = Some(node);
            }
            continue;
        }
        
        if trimmed.starts_with("(depends-on") {
            if let Some(current) = current_node.as_mut() {
                if let Some(deps) = extract_lisp_list(trimmed) {
                    for dep in deps {
                        current.depends_on.push(NodeId::new(dep));
                    }
                }
            }
            continue;
        }
        
        if trimmed.starts_with("(retry") {
            if let Some(current) = current_node.as_mut() {
                current.retries = parse_lisp_retry(trimmed);
            }
            continue;
        }
        
        if trimmed.starts_with("(verify") {
            if let Some(current) = current_node.as_mut() {
                current.verification = Some(parse_lisp_verification(trimmed));
            }
            continue;
        }
        
        if trimmed.starts_with("(state-machine") {
            if let Some(current) = current_node.as_mut() {
                current.state_machine = parse_lisp_state_machine(trimmed);
            }
            continue;
        }
        
        // End of node definition
        if trimmed == ")" && current_node.is_some() {
            dag.nodes.push(current_node.take().unwrap());
            continue;
        }
    }
    
    Ok(dag)
}

fn extract_lisp_symbol(s: &str) -> Option<String> {
    let s = s.trim_start_matches('(');
    let s = s.split_whitespace().nth(1)?;
    Some(s.trim_start_matches(':').to_string())
}

fn extract_lisp_string(s: &str) -> Option<String> {
    let parts: Vec<&str> = s.splitn(3, '"').collect();
    if parts.len() >= 3 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

fn extract_lisp_list(s: &str) -> Option<Vec<String>> {
    let s = s.trim_start_matches('(').trim_end_matches(')');
    let items: Vec<String> = s.split_whitespace()
        .skip(1)
        .map(|s| s.trim_start_matches(':').to_string())
        .collect();
    Some(items)
}

fn parse_lisp_provider(s: &str) -> Option<ProviderConfig> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 2 {
        Some(ProviderConfig {
            name: parts[1].trim_start_matches(':').to_string(),
            model: "opencode/claude-sonnet-4".to_string(),
            priority: 10,
        })
    } else {
        None
    }
}

fn parse_lisp_context(s: &str) -> Option<(String, String)> {
    let s = s.trim_start_matches("(defctx").trim_end_matches(')');
    let parts: Vec<&str> = s.splitn(3, '"').collect();
    if parts.len() >= 3 {
        Some((parts[1].to_string(), parts[2].trim_end_matches(')').to_string()))
    } else {
        None
    }
}

fn parse_lisp_node(s: &str) -> Option<Node> {
    let name = extract_lisp_symbol(s)?;
    
    Some(Node {
        id: NodeId::new(&name),
        kind: NodeKind::Prompt,
        prompt: String::new(),
        depends_on: Vec::new(),
        retries: RetryConfig::default(),
        verification: None,
        state_machine: None,
        inputs: HashMap::new(),
        outputs: Vec::new(),
    })
}

fn parse_lisp_retry(s: &str) -> RetryConfig {
    let mut config = RetryConfig::default();
    
    if s.contains("max-retries") {
        if let Some(val) = extract_lisp_kwarg(s, "max-retries") {
            config.max_retries = val.parse().unwrap_or(3);
        }
    }
    if s.contains("delay") {
        if let Some(val) = extract_lisp_kwarg(s, "delay") {
            config.delay_ms = val.parse().unwrap_or(1000);
        }
    }
    if s.contains("exponential") {
        config.backoff = BackoffStrategy::Exponential;
    }
    
    config
}

fn parse_lisp_verification(s: &str) -> VerificationConfig {
    let kind = if s.contains(":json-valid") {
        VerificationKind::JsonValid
    } else if s.contains(":non-empty") {
        VerificationKind::NonEmpty
    } else if let Some(pattern) = extract_lisp_kwarg_str(s, "match") {
        VerificationKind::Match { pattern }
    } else {
        VerificationKind::NonEmpty
    };
    
    VerificationConfig {
        kind,
        assert: None,
        timeout_ms: 30000,
        required: true,
    }
}

fn parse_lisp_state_machine(s: &str) -> Option<StateMachineConfig> {
    let initial = extract_lisp_kwarg(s, "initial")
        .map(StateId::new)
        .unwrap_or_else(|| StateId::new("pending"));
    
    Some(StateMachineConfig {
        initial,
        states: HashMap::new(),
        on_error: StateId::new("error"),
    })
}

fn extract_lisp_kwarg(s: &str, key: &str) -> Option<String> {
    let pattern = format!(":{}", key);
    if let Some(pos) = s.find(&pattern) {
        let after = &s[pos..];
        let parts: Vec<&str> = after.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some(parts[1].to_string());
        }
    }
    None
}

fn extract_lisp_kwarg_str(s: &str, key: &str) -> Option<String> {
    let pattern = format!(":{} \"", key);
    if let Some(pos) = s.find(&pattern) {
        let after = &s[pos + pattern.len()..];
        if let Some(end) = after.find('"') {
            return Some(after[..end].to_string());
        }
    }
    None
}

/// Parse a directory of prompt files into a combined DAG
pub fn parse_directory(dir: impl AsRef<Path>) -> Result<PromptDag> {
    let dir = dir.as_ref();
    let mut combined = PromptDag {
        name: dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("combined")
            .to_string(),
        description: String::new(),
        nodes: Vec::new(),
        context: HashMap::new(),
        providers: Vec::new(),
    };
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "yaml" | "yml" | "json" | "lisp" | "cl") {
                    if let Ok(dag) = parse_file(&path) {
                        combined.nodes.extend(dag.nodes);
                        for (k, v) in dag.context {
                            combined.context.entry(k).or_insert(v);
                        }
                        combined.providers.extend(dag.providers);
                    }
                }
            }
        }
    }
    
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
name: test-dag
description: Test DAG
nodes:
  - id: node1
    type: Prompt
    prompt: Hello world
  - id: node2
    type: Prompt
    prompt: Follow up
    depends_on:
      - node1
"#;
        let dag = parse_yaml(yaml).unwrap();
        assert_eq!(dag.name, "test-dag");
        assert_eq!(dag.nodes.len(), 2);
        assert_eq!(dag.nodes[1].depends_on.len(), 1);
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{
            "name": "json-dag",
            "nodes": [
                {
                    "id": "n1",
                    "type": "Prompt",
                    "prompt": "test"
                }
            ]
        }"#;
        let dag = parse_json(json).unwrap();
        assert_eq!(dag.name, "json-dag");
    }

    #[test]
    fn test_parse_directory() {
        let dir = tempdir().unwrap();
        
        // Create first YAML file
        let mut file1 = std::fs::File::create(dir.path().join("task1.yaml")).unwrap();
        file1.write_all(r#"
name: task1
nodes:
  - id: task1-node
    type: Prompt
    prompt: First task
"#.as_bytes()).unwrap();
        
        // Create second YAML file
        let mut file2 = std::fs::File::create(dir.path().join("task2.yaml")).unwrap();
        file2.write_all(r#"
name: task2
nodes:
  - id: task2-node
    type: Prompt
    prompt: Second task
"#.as_bytes()).unwrap();
        
        let dag = parse_directory(dir.path()).unwrap();
        assert_eq!(dag.nodes.len(), 2);
    }

    #[test]
    fn test_retry_config_defaults() {
        let yaml = r#"
name: retry-test
nodes:
  - id: retry-node
    type: Prompt
    prompt: Test with retry
    retries:
      max_retries: 5
      delay_ms: 2000
"#;
        let dag = parse_yaml(yaml).unwrap();
        assert_eq!(dag.nodes[0].retries.max_retries, 5);
        assert_eq!(dag.nodes[0].retries.delay_ms, 2000);
    }

    #[test]
    fn test_verification_config() {
        let yaml = r#"
name: verify-test
nodes:
  - id: verify-node
    type: Prompt
    prompt: Verify this
    verification:
      type: Match
      pattern: "success"
      required: true
"#;
        let dag = parse_yaml(yaml).unwrap();
        let verify = dag.nodes[0].verification.as_ref().unwrap();
        assert!(matches!(verify.kind, VerificationKind::Match { .. }));
    }
}