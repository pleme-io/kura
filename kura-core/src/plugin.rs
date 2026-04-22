use tatara_lisp_derive::TataraDomain as DeriveTataraDomain;

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "defplugin")]
pub struct PluginSpec {
    pub name: String,
    pub kind: PluginKind,
    #[serde(default)]
    pub phase: HookPhase,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub lisp_transform: Option<String>,
    #[serde(default)]
    pub mcp_server: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginKind {
    Hook,
    Transformer,
    McpBridge,
    Skill,
    Custom,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum HookPhase {
    #[default]
    PreToolUse,
    PostToolUse,
    PreSession,
    PostSession,
    OnMessage,
    OnError,
    OnReload,
}
