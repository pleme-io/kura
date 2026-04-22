pub mod adapter;
pub mod openai_compat;
pub mod router;
pub mod zen;

pub use adapter::{
    CompletionRequest, CompletionResponse, ProviderAdapter, RequestMessage, StreamEvent,
};
pub use kura_core::ToolDefinition;
pub use openai_compat::OpenAiCompatAdapter;
pub use router::ProviderRouter;
pub use zen::ZenAdapter;

use kura_core::ProviderSpec;

pub fn provider_from_spec(spec: &ProviderSpec) -> Box<dyn ProviderAdapter> {
    match spec.kind {
        kura_core::ProviderKind::Zen => Box::new(ZenAdapter::from_spec(spec)),
        _ => Box::new(OpenAiCompatAdapter::from_spec(spec)),
    }
}

#[cfg(test)]
mod tests;
