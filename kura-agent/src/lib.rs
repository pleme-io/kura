pub mod context;
pub mod loop_runner;
pub mod message;
pub mod session_store;

pub use context::{Conversation, Message};
pub use loop_runner::AgentLoop;
pub use message::MessageExt;
pub use session_store::SessionStore;

#[cfg(test)]
mod tests;

use kura_core::AgentSpec;

pub fn agent_from_lisp(src: &str) -> anyhow::Result<Vec<kura_core::AgentSpec>> {
    let defs = tatara_lisp::compile_named::<AgentSpec>(src)?;
    Ok(defs.into_iter().map(|d| d.spec).collect())
}
