use tatara_lisp_derive::TataraDomain as DeriveTataraDomain;

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "deftool")]
pub struct ToolSpec {
    pub name: String,
    pub kind: ToolKind,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub guardrail: bool,
    #[serde(default)]
    pub mcp_server: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolKind {
    Shell,
    FileRead,
    FileWrite,
    FileEdit,
    Glob,
    Grep,
    Git,
    WebFetch,
    WebSearch,
    CodeSearch,
    Mcp,
    Custom,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
