use async_trait::async_trait;
use kura_core::ProviderSpec;
use tokio::sync::mpsc;

use crate::adapter::*;
use crate::zen::ZenAdapter;

pub struct OpenAiCompatAdapter {
    pub(crate) base_url: String,
    pub(crate) api_key_env: String,
    pub(crate) default_model: String,
    pub(crate) max_tokens: Option<i64>,
    pub(crate) temperature: Option<f64>,
}

impl OpenAiCompatAdapter {
    pub fn from_spec(spec: &ProviderSpec) -> Self {
        Self {
            base_url: spec
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            api_key_env: spec
                .api_key_env
                .clone()
                .unwrap_or_else(|| "OPENAI_API_KEY".to_string()),
            default_model: spec.model.clone().unwrap_or_else(|| "gpt-4o".to_string()),
            max_tokens: spec.max_tokens,
            temperature: spec.temperature,
        }
    }

    pub fn from_spec_with_base(spec: &ProviderSpec, default_base: &str) -> Self {
        Self {
            base_url: spec
                .base_url
                .clone()
                .unwrap_or_else(|| default_base.to_string()),
            api_key_env: spec
                .api_key_env
                .clone()
                .unwrap_or_else(|| "OLLAMA_API_KEY".to_string()),
            default_model: spec.model.clone().unwrap_or_else(|| "llama3".to_string()),
            max_tokens: spec.max_tokens,
            temperature: spec.temperature,
        }
    }
}

#[async_trait]
impl ProviderAdapter for OpenAiCompatAdapter {
    fn name(&self) -> &str {
        "openai-compat"
    }

    async fn complete(&self, request: CompletionRequest) -> anyhow::Result<CompletionResponse> {
        let zen = ZenAdapter::from_spec(&ProviderSpec {
            name: "compat".to_string(),
            kind: kura_core::ProviderKind::OpenAi,
            base_url: Some(self.base_url.clone()),
            api_key_env: Some(self.api_key_env.clone()),
            model: Some(self.default_model.clone()),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            priority: 0,
            disabled: false,
        });
        zen.complete(request).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
        tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> anyhow::Result<()> {
        let zen = ZenAdapter::from_spec(&ProviderSpec {
            name: "compat".to_string(),
            kind: kura_core::ProviderKind::OpenAi,
            base_url: Some(self.base_url.clone()),
            api_key_env: Some(self.api_key_env.clone()),
            model: Some(self.default_model.clone()),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            priority: 0,
            disabled: false,
        });
        zen.stream(request, tx).await
    }

    async fn list_models(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let api_key = std::env::var(&self.api_key_env).ok();
        let client = reqwest::Client::new();
        let mut req = client.get(format!("{}/models", self.base_url));
        if let Some(key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req.send().await?;
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
                            provider: "openai-compat".to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(models)
    }
}
