pub mod agent;
pub mod keymap;
pub mod plugin;
pub mod provider;
pub mod session;
pub mod tool;

pub use agent::{AgentKind, AgentSpec};
pub use keymap::{Action, KeyBinding, KeymapSpec};
pub use plugin::{HookPhase, PluginKind, PluginSpec};
pub use provider::{OpenAiCompatSpec, ProviderKind, ProviderSpec, ZenProviderSpec};
pub use session::{ContentBlock, MessageRole, SessionSpec};
pub use tool::{ToolDefinition, ToolKind, ToolSpec};

use tatara_lisp::domain::register;

pub fn register_all() {
    register::<ProviderSpec>();
    register::<SessionSpec>();
    register::<ToolSpec>();
    register::<PluginSpec>();
    register::<KeymapSpec>();
    register::<AgentSpec>();
}

pub trait KuraDomain: tatara_lisp::domain::TataraDomain + serde::Serialize {
    fn content_id(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(blake3::hash(&bytes).as_bytes())
    }
}

impl KuraDomain for ProviderSpec {}
impl KuraDomain for SessionSpec {}
impl KuraDomain for ToolSpec {}
impl KuraDomain for PluginSpec {}
impl KuraDomain for KeymapSpec {}
impl KuraDomain for AgentSpec {}

#[cfg(test)]
mod tests;
