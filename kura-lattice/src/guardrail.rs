use serde::{Deserialize, Serialize};

use crate::Lattice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardrailLevel {
    None = 0,
    Permissive = 1,
    Strict = 2,
}

impl Lattice for GuardrailLevel {
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
        Self::None
    }

    fn top() -> Self {
        Self::Strict
    }
}
