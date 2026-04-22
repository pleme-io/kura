use crate::{
    AgentAutonomy, GuardrailLevel, KuraClassification, Lattice, ProviderTrust, ToolSafety,
};

mod provider_trust {
    use crate::{Lattice, ProviderTrust};
    use proptest::prelude::*;

    fn arb() -> impl Strategy<Value = ProviderTrust> {
        prop_oneof![
            Just(ProviderTrust::Custom),
            Just(ProviderTrust::Ollama),
            Just(ProviderTrust::OpenAi),
            Just(ProviderTrust::Anthropic),
            Just(ProviderTrust::Zen),
        ]
    }

    proptest! {
        #[test]
        fn idempotent(a in arb()) { assert_eq!(a.meet(&a), a); assert_eq!(a.join(&a), a); }
        #[test]
        fn commutative(a in arb(), b in arb()) { assert_eq!(a.meet(&b), b.meet(&a)); assert_eq!(a.join(&b), b.join(&a)); }
        #[test]
        fn associative(a in arb(), b in arb(), c in arb()) { assert_eq!(a.meet(&b.meet(&c)), a.meet(&b).meet(&c)); assert_eq!(a.join(&b.join(&c)), a.join(&b).join(&c)); }
        #[test]
        fn absorption(a in arb(), b in arb()) { assert_eq!(a.meet(&a.join(&b)), a); assert_eq!(a.join(&a.meet(&b)), a); }
        #[test]
        fn leq_meet(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.meet(&b) == a); }
        #[test]
        fn leq_join(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.join(&b) == b); }
        #[test]
        fn bottom_min(a in arb()) { assert!(ProviderTrust::bottom().leq(&a)); }
        #[test]
        fn top_max(a in arb()) { assert!(a.leq(&ProviderTrust::top())); }
    }
}

mod guardrail_level {
    use crate::{GuardrailLevel, Lattice};
    use proptest::prelude::*;

    fn arb() -> impl Strategy<Value = GuardrailLevel> {
        prop_oneof![
            Just(GuardrailLevel::None),
            Just(GuardrailLevel::Permissive),
            Just(GuardrailLevel::Strict)
        ]
    }

    proptest! {
        #[test]
        fn idempotent(a in arb()) { assert_eq!(a.meet(&a), a); assert_eq!(a.join(&a), a); }
        #[test]
        fn commutative(a in arb(), b in arb()) { assert_eq!(a.meet(&b), b.meet(&a)); assert_eq!(a.join(&b), b.join(&a)); }
        #[test]
        fn associative(a in arb(), b in arb(), c in arb()) { assert_eq!(a.meet(&b.meet(&c)), a.meet(&b).meet(&c)); assert_eq!(a.join(&b.join(&c)), a.join(&b).join(&c)); }
        #[test]
        fn absorption(a in arb(), b in arb()) { assert_eq!(a.meet(&a.join(&b)), a); assert_eq!(a.join(&a.meet(&b)), a); }
        #[test]
        fn leq_meet(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.meet(&b) == a); }
        #[test]
        fn leq_join(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.join(&b) == b); }
        #[test]
        fn bottom_min(a in arb()) { assert!(GuardrailLevel::bottom().leq(&a)); }
        #[test]
        fn top_max(a in arb()) { assert!(a.leq(&GuardrailLevel::top())); }
    }
}

mod agent_autonomy {
    use crate::{AgentAutonomy, Lattice};
    use proptest::prelude::*;

    fn arb() -> impl Strategy<Value = AgentAutonomy> {
        prop_oneof![
            Just(AgentAutonomy::AutoApprove),
            Just(AgentAutonomy::ManualApproval)
        ]
    }

    proptest! {
        #[test]
        fn idempotent(a in arb()) { assert_eq!(a.meet(&a), a); assert_eq!(a.join(&a), a); }
        #[test]
        fn commutative(a in arb(), b in arb()) { assert_eq!(a.meet(&b), b.meet(&a)); assert_eq!(a.join(&b), b.join(&a)); }
        #[test]
        fn associative(a in arb(), b in arb(), c in arb()) { assert_eq!(a.meet(&b.meet(&c)), a.meet(&b).meet(&c)); assert_eq!(a.join(&b.join(&c)), a.join(&b).join(&c)); }
        #[test]
        fn absorption(a in arb(), b in arb()) { assert_eq!(a.meet(&a.join(&b)), a); assert_eq!(a.join(&a.meet(&b)), a); }
        #[test]
        fn leq_meet(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.meet(&b) == a); }
        #[test]
        fn leq_join(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.join(&b) == b); }
        #[test]
        fn bottom_min(a in arb()) { assert!(AgentAutonomy::bottom().leq(&a)); }
        #[test]
        fn top_max(a in arb()) { assert!(a.leq(&AgentAutonomy::top())); }
    }
}

mod tool_safety {
    use crate::{Lattice, ToolSafety};
    use proptest::prelude::*;

    fn arb() -> impl Strategy<Value = ToolSafety> {
        prop_oneof![Just(ToolSafety::Unguarded), Just(ToolSafety::Guardrailed)]
    }

    proptest! {
        #[test]
        fn idempotent(a in arb()) { assert_eq!(a.meet(&a), a); assert_eq!(a.join(&a), a); }
        #[test]
        fn commutative(a in arb(), b in arb()) { assert_eq!(a.meet(&b), b.meet(&a)); assert_eq!(a.join(&b), b.join(&a)); }
        #[test]
        fn associative(a in arb(), b in arb(), c in arb()) { assert_eq!(a.meet(&b.meet(&c)), a.meet(&b).meet(&c)); assert_eq!(a.join(&b.join(&c)), a.join(&b).join(&c)); }
        #[test]
        fn absorption(a in arb(), b in arb()) { assert_eq!(a.meet(&a.join(&b)), a); assert_eq!(a.join(&a.meet(&b)), a); }
        #[test]
        fn leq_meet(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.meet(&b) == a); }
        #[test]
        fn leq_join(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.join(&b) == b); }
        #[test]
        fn bottom_min(a in arb()) { assert!(ToolSafety::bottom().leq(&a)); }
        #[test]
        fn top_max(a in arb()) { assert!(a.leq(&ToolSafety::top())); }
    }
}

mod classification {
    #[allow(unused_imports)]
    use crate::{
        AgentAutonomy, GuardrailLevel, KuraClassification, Lattice, ProviderTrust, ToolSafety,
    };
    use proptest::prelude::*;

    fn arb() -> impl Strategy<Value = KuraClassification> {
        (0..5usize, 0..3usize, 0..2usize, 0..2usize).prop_map(|(p, g, a, s)| KuraClassification {
            provider_trust: match p {
                0 => ProviderTrust::Custom,
                1 => ProviderTrust::Ollama,
                2 => ProviderTrust::OpenAi,
                3 => ProviderTrust::Anthropic,
                _ => ProviderTrust::Zen,
            },
            guardrail: match g {
                0 => GuardrailLevel::None,
                1 => GuardrailLevel::Permissive,
                _ => GuardrailLevel::Strict,
            },
            autonomy: match a {
                0 => AgentAutonomy::AutoApprove,
                _ => AgentAutonomy::ManualApproval,
            },
            tool_safety: match s {
                0 => ToolSafety::Unguarded,
                _ => ToolSafety::Guardrailed,
            },
        })
    }

    proptest! {
        #[test]
        fn idempotent(a in arb()) { assert_eq!(a.meet(&a), a); assert_eq!(a.join(&a), a); }
        #[test]
        fn commutative(a in arb(), b in arb()) { assert_eq!(a.meet(&b), b.meet(&a)); assert_eq!(a.join(&b), b.join(&a)); }
        #[test]
        fn associative(a in arb(), b in arb(), c in arb()) { assert_eq!(a.meet(&b.meet(&c)), a.meet(&b).meet(&c)); assert_eq!(a.join(&b.join(&c)), a.join(&b).join(&c)); }
        #[test]
        fn absorption(a in arb(), b in arb()) { assert_eq!(a.meet(&a.join(&b)), a); assert_eq!(a.join(&a.meet(&b)), a); }
        #[test]
        fn leq_meet(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.meet(&b) == a); }
        #[test]
        fn leq_join(a in arb(), b in arb()) { assert_eq!(a.leq(&b), a.join(&b) == b); }
        #[test]
        fn bottom_min(a in arb()) { assert!(KuraClassification::bottom().leq(&a)); }
        #[test]
        fn top_max(a in arb()) { assert!(a.leq(&KuraClassification::top())); }
        #[test]
        fn satisfies_equiv_leq(a in arb(), b in arb()) { assert_eq!(a.satisfies(&b), b.leq(&a)); }
    }

    #[test]
    fn bottom_values() {
        let bot = KuraClassification::bottom();
        assert_eq!(bot.provider_trust, ProviderTrust::Custom);
        assert_eq!(bot.guardrail, GuardrailLevel::None);
        assert_eq!(bot.autonomy, AgentAutonomy::AutoApprove);
        assert_eq!(bot.tool_safety, ToolSafety::Unguarded);
    }

    #[test]
    fn top_values() {
        let top = KuraClassification::top();
        assert_eq!(top.provider_trust, ProviderTrust::Zen);
        assert_eq!(top.guardrail, GuardrailLevel::Strict);
        assert_eq!(top.autonomy, AgentAutonomy::ManualApproval);
        assert_eq!(top.tool_safety, ToolSafety::Guardrailed);
    }

    #[test]
    fn satisfies_strict() {
        let session = KuraClassification {
            provider_trust: ProviderTrust::Zen,
            guardrail: GuardrailLevel::Strict,
            autonomy: AgentAutonomy::ManualApproval,
            tool_safety: ToolSafety::Guardrailed,
        };
        assert!(session.satisfies(&KuraClassification::top()));
    }

    #[test]
    fn fails_unsafe() {
        let session = KuraClassification {
            provider_trust: ProviderTrust::Custom,
            guardrail: GuardrailLevel::None,
            autonomy: AgentAutonomy::AutoApprove,
            tool_safety: ToolSafety::Unguarded,
        };
        assert!(!session.satisfies(&KuraClassification::top()));
    }

    #[test]
    fn zen_satisfies_custom() {
        let session = KuraClassification {
            provider_trust: ProviderTrust::Zen,
            guardrail: GuardrailLevel::Strict,
            autonomy: AgentAutonomy::ManualApproval,
            tool_safety: ToolSafety::Guardrailed,
        };
        let requirement = KuraClassification {
            provider_trust: ProviderTrust::Custom,
            guardrail: GuardrailLevel::None,
            autonomy: AgentAutonomy::AutoApprove,
            tool_safety: ToolSafety::Unguarded,
        };
        assert!(session.satisfies(&requirement));
    }
}
