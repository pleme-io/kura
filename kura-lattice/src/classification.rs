use serde::{Deserialize, Serialize};

use crate::{AgentAutonomy, GuardrailLevel, Lattice, ProviderTrust, ToolSafety};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KuraClassification {
    pub provider_trust: ProviderTrust,
    pub guardrail: GuardrailLevel,
    pub autonomy: AgentAutonomy,
    pub tool_safety: ToolSafety,
}

impl Lattice for KuraClassification {
    fn meet(&self, other: &Self) -> Self {
        Self {
            provider_trust: self.provider_trust.meet(&other.provider_trust),
            guardrail: self.guardrail.meet(&other.guardrail),
            autonomy: self.autonomy.meet(&other.autonomy),
            tool_safety: self.tool_safety.meet(&other.tool_safety),
        }
    }

    fn join(&self, other: &Self) -> Self {
        Self {
            provider_trust: self.provider_trust.join(&other.provider_trust),
            guardrail: self.guardrail.join(&other.guardrail),
            autonomy: self.autonomy.join(&other.autonomy),
            tool_safety: self.tool_safety.join(&other.tool_safety),
        }
    }

    fn leq(&self, other: &Self) -> bool {
        self.provider_trust.leq(&other.provider_trust)
            && self.guardrail.leq(&other.guardrail)
            && self.autonomy.leq(&other.autonomy)
            && self.tool_safety.leq(&other.tool_safety)
    }

    fn bottom() -> Self {
        Self {
            provider_trust: ProviderTrust::bottom(),
            guardrail: GuardrailLevel::bottom(),
            autonomy: AgentAutonomy::bottom(),
            tool_safety: ToolSafety::bottom(),
        }
    }

    fn top() -> Self {
        Self {
            provider_trust: ProviderTrust::top(),
            guardrail: GuardrailLevel::top(),
            autonomy: AgentAutonomy::top(),
            tool_safety: ToolSafety::top(),
        }
    }
}

impl KuraClassification {
    pub fn satisfies(&self, requirement: &Self) -> bool {
        requirement.leq(self)
    }
}
