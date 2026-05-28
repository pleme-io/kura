//! Verify kura-run registers four DAG-shape typed enums into the
//! gen-platform fleet catalog. kura is the typed agent runner;
//! eighth consumer class adopting the catamorphism.
//!
//! Registered:
//! - kura.node-kind         (7 variants — Prompt / PromptWithModel /
//!   Shell / FileRead / Conditional / FanOut / FanIn)
//! - kura.backoff-strategy  (4 variants — Fixed / Linear /
//!   Exponential / Jitter)
//! - kura.verification-kind (5 variants — Match / JsonValid /
//!   NonEmpty / Command / LLM)
//! - kura.event             (4 variants — Success / Failure /
//!   Timeout / Custom)

use gen_platform::{catalog, TypedDispatcherTrait};
use kura_run::{BackoffStrategy, Event, NodeKind, VerificationKind};

#[test]
fn node_kind_registers() {
    let entry =
        catalog::by_label("kura.node-kind").expect("NodeKind must register");
    assert_eq!((entry.variant_count)(), 7);
}

#[test]
fn backoff_strategy_registers() {
    let entry = catalog::by_label("kura.backoff-strategy")
        .expect("BackoffStrategy must register");
    assert_eq!((entry.variant_count)(), 4);
}

#[test]
fn verification_kind_registers() {
    let entry = catalog::by_label("kura.verification-kind")
        .expect("VerificationKind must register");
    assert_eq!((entry.variant_count)(), 5);
}

#[test]
fn event_registers() {
    let entry = catalog::by_label("kura.event").expect("Event must register");
    assert_eq!((entry.variant_count)(), 4);
}

#[test]
fn node_kind_variant_kinds_kebab() {
    let kinds = NodeKind::variant_kinds();
    assert_eq!(
        kinds,
        vec![
            "prompt",
            "prompt-with-model",
            "shell",
            "file-read",
            "conditional",
            "fan-out",
            "fan-in"
        ]
    );
}

#[test]
fn backoff_strategy_kinds_kebab() {
    let kinds = BackoffStrategy::variant_kinds();
    assert_eq!(kinds, vec!["fixed", "linear", "exponential", "jitter"]);
}

#[test]
fn verification_kind_kinds_kebab() {
    let kinds = VerificationKind::variant_kinds();
    assert_eq!(
        kinds,
        vec!["match", "json-valid", "non-empty", "command", "llm"]
    );
}

#[test]
fn event_kinds_kebab() {
    let kinds = Event::variant_kinds();
    assert_eq!(kinds, vec!["success", "failure", "timeout", "custom"]);
}

#[test]
fn variant_counts_via_trait() {
    assert_eq!(NodeKind::variant_count(), 7);
    assert_eq!(BackoffStrategy::variant_count(), 4);
    assert_eq!(VerificationKind::variant_count(), 5);
    assert_eq!(Event::variant_count(), 4);
}

#[test]
fn backoff_strategy_quintet_round_trip() {
    use std::str::FromStr;
    // BackoffStrategy carries the FULL typed-reflection quintet:
    // TypedDispatcher + Discriminant + IsVariant + FromStrKind +
    // also_display. Proves the typed round-trip:
    //   enum → discriminant() → str → from_str() → enum
    for variant in [
        BackoffStrategy::Fixed,
        BackoffStrategy::Linear,
        BackoffStrategy::Exponential,
        BackoffStrategy::Jitter,
    ] {
        let kind = variant.discriminant();
        let parsed = BackoffStrategy::from_str(kind)
            .unwrap_or_else(|_| panic!("from_str must accept own discriminant {kind}"));
        // Variant identity proved via discriminant equality.
        assert_eq!(parsed.discriminant(), variant.discriminant());
    }
}

#[test]
fn backoff_strategy_display_delegates_to_discriminant() {
    // #[discriminant(also_display)] emits Display → discriminant().
    use std::fmt::Write;
    let mut s = String::new();
    write!(&mut s, "{}", BackoffStrategy::Fixed).unwrap();
    assert_eq!(s, "fixed");
    assert_eq!(BackoffStrategy::Exponential.to_string(), "exponential");
}

#[test]
fn backoff_strategy_predicates() {
    let fixed = BackoffStrategy::Fixed;
    assert!(fixed.is_fixed());
    assert!(!fixed.is_linear());
    assert!(!fixed.is_exponential());
    assert!(!fixed.is_jitter());
}

#[test]
fn backoff_strategy_from_str_rejects_unknown() {
    use std::str::FromStr;
    let r = BackoffStrategy::from_str("does-not-exist");
    assert!(r.is_err(), "unknown kind must error");
}
