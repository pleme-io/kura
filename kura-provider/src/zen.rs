use async_trait::async_trait;
use futures_util::StreamExt;
use kura_core::{ContentBlock, ProviderSpec};
use tokio::sync::mpsc;

use crate::adapter::*;

pub struct ZenAdapter {
    pub(crate) base_url: String,
    pub(crate) api_key_env: String,
    pub(crate) default_model: String,
    pub(crate) max_tokens: Option<i64>,
    pub(crate) temperature: Option<f64>,
    pub(crate) priority: i64,
}

impl ZenAdapter {
    pub fn from_spec(spec: &ProviderSpec) -> Self {
        Self {
            base_url: spec
                .base_url
                .clone()
                .unwrap_or_else(|| "https://opencode.ai/zen/v1".to_string()),
            api_key_env: spec
                .api_key_env
                .clone()
                .unwrap_or_else(|| "OPENCODE_API_KEY".to_string()),
            default_model: spec
                .model
                .clone()
                .unwrap_or_else(|| "opencode/claude-sonnet-4-20250514".to_string()),
            max_tokens: spec.max_tokens,
            temperature: spec.temperature,
            priority: spec.priority,
        }
    }

    fn api_key(&self) -> Option<String> {
        std::env::var(&self.api_key_env).ok()
    }

    pub(crate) fn resolve_model(&self, request: &CompletionRequest) -> String {
        if request.model.is_empty() || request.model == "default" {
            self.default_model.clone()
        } else if request.model.contains('/') {
            request.model.clone()
        } else {
            format!("opencode/{}", request.model)
        }
    }
}

#[async_trait]
impl ProviderAdapter for ZenAdapter {
    fn name(&self) -> &str {
        "zen"
    }

    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        let api_key = self
            .api_key()
            .ok_or_else(|| anyhow::anyhow!("{} not set", self.api_key_env))?;
        let model = self.resolve_model(&request);
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": model,
            "messages": request.messages,
            "stream": false,
        });
        if let Some(mt) = self.max_tokens.or(request.max_tokens) {
            body["max_tokens"] = serde_json::json!(mt);
        }
        if let Some(t) = self.temperature.or(request.temperature) {
            body["temperature"] = serde_json::json!(t);
        }
        if !request.tools.is_empty() {
            body["tools"] = serde_json::json!(request.tools);
        }

        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Zen API error {}: {}", status, text);
        }

        let zen_resp: serde_json::Value = resp.json().await?;
        CompletionResponse::from_openai_response(zen_resp, &model)
    }

    async fn stream(
        &self,
        request: CompletionRequest,
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()> {
        let api_key = self
            .api_key()
            .ok_or_else(|| anyhow::anyhow!("{} not set", self.api_key_env))?;
        let model = self.resolve_model(&request);
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": model,
            "messages": request.messages,
            "stream": true,
        });
        if let Some(mt) = self.max_tokens.or(request.max_tokens) {
            body["max_tokens"] = serde_json::json!(mt);
        }
        if let Some(t) = self.temperature.or(request.temperature) {
            body["temperature"] = serde_json::json!(t);
        }
        if !request.tools.is_empty() {
            body["tools"] = serde_json::json!(request.tools);
        }

        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            anyhow::bail!("Zen API error {}: {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut current_tool: Option<(String, String, String)> = None;

        loop {
            let chunk_opt = stream.next().await;
            let chunk = match chunk_opt {
                Some(Ok(b)) => b,
                Some(Err(e)) => {
                    let _ = tx.send(StreamEvent::Error(e.to_string()));
                    break;
                }
                None => break,
            };
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        let _ = tx.send(StreamEvent::Done(CompletionResponse {
                            id: String::new(),
                            model: model.clone(),
                            content: vec![],
                            usage: Usage {
                                input_tokens: 0,
                                output_tokens: 0,
                            },
                        }));
                        return Ok(());
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            for choice in choices {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) =
                                        delta.get("content").and_then(|c| c.as_str())
                                    {
                                        let _ =
                                            tx.send(StreamEvent::TextDelta(content.to_string()));
                                    }
                                    if let Some(tool_calls) =
                                        delta.get("tool_calls").and_then(|c| c.as_array())
                                    {
                                        for tc in tool_calls {
                                            let id = tc
                                                .get("id")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            let name = tc
                                                .get("function")
                                                .and_then(|f| f.get("name"))
                                                .and_then(|n| n.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            let args = tc
                                                .get("function")
                                                .and_then(|f| f.get("arguments"))
                                                .and_then(|a| a.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            if !id.is_empty() {
                                                if let Some((prev_id, _, _)) = current_tool.take() {
                                                    let _ = tx.send(StreamEvent::ToolUseInputEnd {
                                                        id: prev_id,
                                                    });
                                                }
                                                current_tool =
                                                    Some((id.clone(), name.clone(), String::new()));
                                                let _ =
                                                    tx.send(StreamEvent::ToolUseStart { id, name });
                                            }
                                            if !args.is_empty() {
                                                let _ = tx.send(StreamEvent::ToolUseInputDelta {
                                                    id: current_tool
                                                        .as_ref()
                                                        .map(|(id, _, _)| id.clone())
                                                        .unwrap_or_default(),
                                                    delta: args,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((id, _, _)) = current_tool.take() {
            let _ = tx.send(StreamEvent::ToolUseInputEnd { id });
        }

        Ok(())
    }

    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let api_key = self
            .api_key()
            .ok_or_else(|| anyhow::anyhow!("{} not set", self.api_key_env))?;
        let client = reqwest::Client::new();

        let resp = client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("Zen models API error: {}", resp.status());
        }

        let json: serde_json::Value = resp.json().await?;
        let models = json
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        Some(ModelInfo {
                            id: m.get("id")?.as_str()?.to_string(),
                            name: m.get("id")?.as_str()?.to_string(),
                            provider: "zen".to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }
}

impl CompletionResponse {
    pub fn from_openai_response(json: serde_json::Value, model: &str) -> anyhow::Result<Self> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let usage = json
            .get("usage")
            .map(|u| Usage {
                input_tokens: u.get("prompt_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
                output_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            })
            .unwrap_or(Usage {
                input_tokens: 0,
                output_tokens: 0,
            });

        let content = json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .map(|msg| {
                let mut blocks = vec![];
                if let Some(text) = msg.get("content").and_then(|c| c.as_str()) {
                    blocks.push(ContentBlock::Text {
                        text: text.to_string(),
                    });
                }
                if let Some(tool_calls) = msg.get("tool_calls").and_then(|c| c.as_array()) {
                    for tc in tool_calls {
                        if let Some(func) = tc.get("function") {
                            let id = tc
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let name = func
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let input_str = func
                                .get("arguments")
                                .and_then(|v| v.as_str())
                                .unwrap_or("{}");
                            let input: serde_json::Value =
                                serde_json::from_str(input_str).unwrap_or(serde_json::json!({}));
                            blocks.push(ContentBlock::ToolUse { id, name, input });
                        }
                    }
                }
                blocks
            })
            .unwrap_or_default();

        Ok(Self {
            id,
            model: model.to_string(),
            content,
            usage,
        })
    }
}
