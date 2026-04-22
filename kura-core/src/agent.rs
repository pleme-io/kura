use tatara_lisp_derive::TataraDomain as DeriveTataraDomain;

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "defagent")]
pub struct AgentSpec {
    pub name: String,
    pub kind: AgentKind,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub plugins: Vec<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_turns: Option<i64>,
    #[serde(default)]
    pub auto_approve: bool,
    #[serde(default)]
    pub thinking_budget: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentKind {
    Coder,
    Reviewer,
    Explorer,
    Planner,
    Custom,
}
