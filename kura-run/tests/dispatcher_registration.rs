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
