use serde::{Deserialize, Serialize};

use crate::Lattice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentAutonomy {
    AutoApprove = 0,
    ManualApproval = 1,
}

impl Lattice for AgentAutonomy {
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
        Self::AutoApprove
    }

    fn top() -> Self {
        Self::ManualApproval
    }
}
