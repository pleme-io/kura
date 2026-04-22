use tatara_lisp_derive::TataraDomain as DeriveTataraDomain;

#[derive(DeriveTataraDomain, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[tatara(keyword = "defkeymap")]
pub struct KeymapSpec {
    pub name: String,
    #[serde(default)]
    pub bindings: Vec<KeyBinding>,
    #[serde(default)]
    pub extends: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBinding {
    pub key: String,
    pub action: Action,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    SubmitInput,
    CancelInput,
    NewSession,
    SwitchSession,
    ToggleFocus,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    ToggleToolApproval,
    ToggleGhosttyGraphics,
    CycleProvider,
    ToggleThinking,
    OpenCommandPalette,
    Quit,
    Custom(String),
}
