pub mod autonomy;
pub mod classification;
pub mod guardrail;
pub mod safety;
pub mod trust;

pub use autonomy::AgentAutonomy;
pub use classification::KuraClassification;
pub use guardrail::GuardrailLevel;
pub use safety::ToolSafety;
pub use trust::ProviderTrust;

pub trait Lattice: Clone + PartialEq + std::fmt::Debug + Send + Sync {
    fn meet(&self, other: &Self) -> Self;
    fn join(&self, other: &Self) -> Self;
    fn leq(&self, other: &Self) -> bool;
    fn bottom() -> Self;
    fn top() -> Self;
}

#[cfg(test)]
mod tests;
