use tatara_lisp_derive::TataraDomain as DeriveTataraDomain;

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "defprovider")]
pub struct ProviderSpec {
    pub name: String,
    pub kind: ProviderKind,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub max_tokens: Option<i64>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderKind {
    Zen,
    OpenAi,
    Anthropic,
    Ollama,
    Custom,
}

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "defzen")]
pub struct ZenProviderSpec {
    pub model: String,
    #[serde(default = "default_zen_base_url")]
    pub base_url: String,
    #[serde(default = "default_zen_api_key_env")]
    pub api_key_env: String,
    #[serde(default)]
    pub max_tokens: Option<i64>,
    #[serde(default)]
    pub temperature: Option<f64>,
}

fn default_zen_base_url() -> String {
    "https://opencode.ai/zen/v1".to_string()
}

fn default_zen_api_key_env() -> String {
    "OPENCODE_API_KEY".to_string()
}

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "defopenai")]
pub struct OpenAiCompatSpec {
    pub name: String,
    pub base_url: String,
    pub api_key_env: String,
    pub model: String,
    #[serde(default)]
    pub max_tokens: Option<i64>,
    #[serde(default)]
    pub temperature: Option<f64>,
}
