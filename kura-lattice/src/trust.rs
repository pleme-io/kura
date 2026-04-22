use serde::{Deserialize, Serialize};

use crate::Lattice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderTrust {
    Custom = 0,
    Ollama = 1,
    OpenAi = 2,
    Anthropic = 3,
    Zen = 4,
}

impl Lattice for ProviderTrust {
    fn meet(&self, other: &Self) -> Self {
        std::cmp::min(*self, *other)
    }

    fn join(&self, other: &Self) -> Self {
        std::cmp::max(*self, *other)
    }

    fn leq(&self, other: &Self) -> bool {
        self <= other
    }

    fn bottom() -> Self {
        Self::Custom
    }

    fn top() -> Self {
        Self::Zen
    }
}
