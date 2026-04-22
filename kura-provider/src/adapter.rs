use async_trait::async_trait;
use kura_core::{ContentBlock, ToolDefinition};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<RequestMessage>,
    pub max_tokens: Option<i64>,
    pub temperature: Option<f64>,
    pub stream: bool,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    TextDelta(String),
    ToolUseStart { id: String, name: String },
    ToolUseInputDelta { id: String, delta: String },
    ToolUseInputEnd { id: String },
    ThinkingDelta(String),
    Done(CompletionResponse),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse>;
    async fn stream(
        &self,
        request: CompletionRequest,
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()>;
    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}
