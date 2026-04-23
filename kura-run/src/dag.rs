//! DAG domain types for prompt orchestration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the DAG representing a prompt or task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub prompt: String,
    #[serde(default)]
    pub depends_on: Vec<NodeId>,
    #[serde(default)]
    pub retries: RetryConfig,
    #[serde(default)]
    pub verification: Option<VerificationConfig>,
    #[serde(default)]
    pub state_machine: Option<StateMachineConfig>,
    #[serde(default)]
    pub inputs: HashMap<String, InputBinding>,
    #[serde(default)]
    pub outputs: Vec<OutputBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// What kind of node this is
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeKind {
    /// Simple prompt execution
    Prompt,
    /// Prompt with a model specified
    PromptWithModel { model: String },
    /// Shell command execution
    Shell { command: String },
    /// File read operation
    FileRead { path: String },
    /// Conditional branching
    Conditional { condition: String },
    /// Parallel fan-out
    FanOut { count: usize },
    /// Fan-in (wait for multiple)
    FanIn,
}

/// Retry configuration for failed nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay_ms")]
    pub delay_ms: u64,
    #[serde(default)]
    pub backoff: BackoffStrategy,
    #[serde(default)]
    pub retry_on: Vec<String>,
}

fn default_max_retries() -> u32 { 3 }
fn default_retry_delay_ms() -> u64 { 1000 }

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            delay_ms: 1000,
            backoff: BackoffStrategy::Exponential,
            retry_on: vec!["rate_limit".to_string(), "timeout".to_string()],
        }
    }
}

/// Backoff strategy for retries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackoffStrategy {
    #[default]
    Fixed,
    Linear,
    Exponential,
    Jitter,
}

/// Verification configuration to validate node output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub kind: VerificationKind,
    #[serde(default)]
    pub assert: Option<String>,
    #[serde(default)]
    pub timeout_ms: u64,
    #[serde(default = "default_verification_required")]
    pub required: bool,
}

fn default_verification_required() -> bool { true }

/// What kind of verification to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VerificationKind {
    /// Check output matches a pattern
    Match { pattern: String },
    /// Check output is valid JSON
    JsonValid,
    /// Check output is non-empty
    NonEmpty,
    /// Run a custom verification command
    Command { command: String },
    /// LLM-based verification
    LLM { prompt: String },
}

/// State machine configuration for complex node lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineConfig {
    pub initial: StateId,
    pub states: HashMap<StateId, State>,
    #[serde(default)]
    pub on_error: StateId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StateId(pub String);

impl StateId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Default for StateId {
    fn default() -> Self {
        Self("pending".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub on_entry: Vec<Action>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub event: Event,
    pub to: StateId,
    #[serde(default)]
    pub guard: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Success,
    Failure,
    Timeout,
    Custom { name: String },
}

/// Actions that can be taken on state entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    /// Send a message to another node
    Send { to: NodeId, message: String },
    /// Set a context variable
    Set { key: String, value: String },
    /// Log something
    Log { message: String },
    /// Execute a side effect
    Exec { command: String },
}

/// Input binding - where to get data from
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputBinding {
    /// From a context variable
    Context { key: String },
    /// From another node's output
    Node { id: NodeId, output: String },
    /// From a file
    File { path: String },
    /// Default value
    Default { value: String },
    /// From environment
    Env { name: String },
}

/// Output binding - where to send results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBinding {
    pub key: String,
    pub target: OutputTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputTarget {
    Context,
    File { path: String },
    Node { id: NodeId, input: String },
    Env { name: String },
}

/// The complete DAG definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDag {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub context: HashMap<String, String>,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(default = "default_provider_model")]
    pub model: String,
    #[serde(default = "default_provider_priority")]
    pub priority: u32,
}

fn default_provider_model() -> String { "opencode/claude-sonnet-4".to_string() }
fn default_provider_priority() -> u32 { 10 }