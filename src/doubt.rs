//! The boundary where doubt leaves for something that is not this machine.
//!
//! # The gap this fills
//!
//! `SemanticSLM::map_intent_semantic` has the right *shape* — it takes a
//! `FnMut` fetch so the same code runs natively and in a browser — and **nothing
//! in the tree ever constructs one**. `build_prompt` has zero callers. So the
//! route for unresolvable doubt outward exists on paper and nowhere else, and
//! every demo in the repository runs on `MockSlm`, which as of this commit can
//! no longer claim to have measured anything.
//!
//! This module is that route, with three things the paper-shaped version needs
//! and a bare HTTP call does not have.
//!
//! # 1. Doubt is a value, not a string
//!
//! [`Doubt`] is a question plus the evidence that made it unanswerable here. A
//! string is not enough: a consultation whose answer is recorded without the
//! question it answered is a log entry that cannot be checked, and this project's
//! whole standard is that an unverified claim is worse than none.
//!
//! # 2. The model is a capability, not an oracle
//!
//! A consultation is recorded as a trace with a signature like any other act, so
//! it is subject to the same [`crate::clean`] pass, the same induction, and the
//! same escalation accounting. **The model is not the witness.** The witness for
//! a source edit is a test suite (`RunTests`, `Trace::succeeded`), and a
//! consultation never gets to overwrite that. An engine that says an edit is fine
//! is a claim, exactly like a human saying so, and it is priced as one.
//!
//! This is the load-bearing constraint, and it is the answer to "should unia be
//! an agent with an LLM inference engine": the engine sits *outside* the loop.
//! An agent's witness is a model; this one's is `cargo test`, which is
//! falsifiable and reproducible and a model is not.
//!
//! # 3. A refusal is recorded, not smoothed over
//!
//! [`Resolver`] is a trait so the transport can differ per target and a test can
//! substitute a script. The important part is that **failing to consult is a
//! first-class outcome** with its own reason, and it costs a failed trace. A
//! system that quietly stops asking when the model is down learns to work without
//! ever asking, and that is how a fallback becomes the default.

use crate::mcp::store::{Actor, Store, Trace};
use std::collections::BTreeSet;

/// Why a doubt could not be resolved by the machine that raised it.
///
/// Typed because the three have three different fixes, and a single "unsure"
/// loses that. This is the same reasoning as `link::Unbound`, applied to doubt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unresolvable {
    /// No rule in the corpus reaches this case.
    NoRule,
    /// Two rules reach it and they disagree — a contradiction, not a gap.
    Contradiction { a: String, b: String },
    /// A rule reaches it and the world does not answer as the rule says.
    Refuted { rule: String, observed: String },
    /// The budget for thinking was spent before a verdict formed.
    Exhausted,
}

impl Unresolvable {
    /// A short name, and the first token of the consultation's signature.
    pub fn tag(&self) -> &'static str {
        match self {
            Unresolvable::NoRule => "NoRule",
            Unresolvable::Contradiction { .. } => "Contradiction",
            Unresolvable::Refuted { .. } => "Refuted",
            Unresolvable::Exhausted => "Exhausted",
        }
    }
}

/// A question this machine could not answer, with the reason it could not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doubt {
    /// What is being asked. Carried, never resolved against — it is the payload,
    /// not the identity.
    pub question: String,
    pub why: Unresolvable,
}

impl Doubt {
    pub fn new(question: &str, why: Unresolvable) -> Self {
        Doubt {
            question: question.to_string(),
            why,
        }
    }

    /// The signature of *consulting about this kind of doubt*.
    ///
    /// **The question is deliberately not in it.** Two different questions that
    /// are the same shape of unanswerability are one capability, exactly as two
    /// `ReplaceSpan`s at the same place are one. Including the question would
    /// make every consultation a new capability and the escalation rate would
    /// measure how many times somebody asked, which is the rumination the
    /// economy was rebuilt to price at nothing.
    pub fn signature(&self) -> String {
        format!("Consult_{}", self.why.tag())
    }
}

/// Something outside the machine that can be asked.
///
/// A trait rather than an `impl FnMut` so that a real transport, a test script,
/// and a refusal are all expressible, and so that the failure mode is part of the
/// type rather than an error string.
pub trait Resolver {
    /// What to call it, for the trace. An unnameable resolver is not a peer.
    fn name(&self) -> &str;

    /// Asks. Returning `Err` is a refusal and is recorded as a failed trace — it
    /// is not an absence.
    fn ask(&self, doubt: &Doubt) -> Result<String, String>;
}

/// The outcome of a consultation, including whether it happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consultation {
    /// The doubt as raised, carried so a receipt can be read without the caller.
    pub doubt: Doubt,
    /// The engine's answer, if one came back.
    pub answer: Option<String>,
    /// The reason no answer came back.
    pub refusal: Option<String>,
    /// The engine that was asked.
    pub resolver: String,
}

impl Consultation {
    /// Whether an answer came back at all.
    pub fn answered(&self) -> bool {
        self.answer.is_some()
    }

    /// A trace for this consultation.
    ///
    /// `actor` is [`Actor::Caller`]: the trace was authored from outside, by
    /// something this machine did not verify. It is a `Caller` and not a new
    /// variant because the taxonomy is about *provenance*, and "written by
    /// something whose output we have not checked" is already what a caller is.
    pub fn to_trace(&self, ts: u64) -> Trace {
        Trace {
            ts,
            intent: format!("consult {}", self.doubt.why.tag()),
            resource_id: None,
            outcome: if self.answered() { "hit" } else { "miss" }.to_string(),
            tokens_in: 0,
            tokens_out: 0,
            // The signature carries the *shape* of the doubt and the name of the
            // engine, so the same question asked of a different engine is a
            // different capability, which it is.
            primitives: vec![
                self.doubt.signature(),
                self.resolver.clone(),
                self.answer
                    .as_ref()
                    .map(|_| "answered".to_string())
                    .unwrap_or_else(|| "refused".to_string()),
            ],
            actor: Some(Actor::Caller),
            // A consultation is a *claim*, never a witness. `succeeded` is the
            // flag the self-cleaning pass reads as a chaotic event and the flag
            // the escalation count credits, so an answer is deliberately not
            // credited as production. What confirms an edit is the test suite.
            succeeded: false,
        }
    }
}

/// Asks, and records having asked — including when the answer is no.
pub fn consult(
    store: &mut Store,
    doubt: Doubt,
    resolver: &dyn Resolver,
) -> Result<Consultation, String> {
    let base = store.stats().traces as u64;
    let outcome = match resolver.ask(&doubt) {
        Ok(answer) => Consultation {
            doubt,
            answer: Some(answer),
            refusal: None,
            resolver: resolver.name().to_string(),
        },
        Err(refusal) => Consultation {
            doubt,
            answer: None,
            refusal: Some(refusal),
            resolver: resolver.name().to_string(),
        },
    };
    store
        .record(outcome.to_trace(base))
        .map_err(|e| e.to_string())?;
    Ok(outcome)
}

/// What the consultations on the log add up to.
///
/// Counted per **signature**, for the same reason the escalation rate is: the
/// corpus must not escalate in activity. A machine that asks the same shape of
/// question a thousand times has one capability, and the four shapes of
/// [`Unresolvable`] are the four it can grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DoubtLedger {
    /// Distinct doubt shapes asked about.
    pub shapes: usize,
    /// Consultations that came back with an answer.
    pub answered: usize,
    /// Consultations that did not. Counted, because a resolver that is down is
    /// a fact about the system and not an absence of one.
    pub refused: usize,
}

impl DoubtLedger {
    /// Answers per consultation.
    ///
    /// `None` when nothing was asked, because a ratio between zero and zero is
    /// not a number, and `0.0` would read as "the model always refuses".
    pub fn answer_rate(&self) -> Option<f64> {
        let total = self.answered + self.refused;
        (total > 0).then(|| self.answered as f64 / total as f64)
    }
}

/// Reads the ledger out of the trace log.
pub fn ledger(store: &Store) -> DoubtLedger {
    let mut shapes: BTreeSet<String> = BTreeSet::new();
    let (mut answered, mut refused) = (0usize, 0usize);
    for t in store.traces() {
        let Some(first) = t.primitives.first() else {
            continue;
        };
        if !first.starts_with("Consult_") {
            continue;
        }
        shapes.insert(first.clone());
        match t.primitives.get(2).map(String::as_str) {
            Some("answered") => answered += 1,
            _ => refused += 1,
        }
    }
    DoubtLedger {
        shapes: shapes.len(),
        answered,
        refused,
    }
}

#[cfg(test)]
mod tests {
    //! The route, exercised with a scriptable resolver and nothing else.

    use super::*;
    use std::path::{Path, PathBuf};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new(tag: &str) -> Self {
            let d = std::env::temp_dir().join(format!("unia-doubt-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).expect("scratch");
            Scratch(d)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A resolver that answers, and one that refuses, both named.
    struct Fixed {
        name: &'static str,
        answer: Option<String>,
    }
    impl Resolver for Fixed {
        fn name(&self) -> &str {
            self.name
        }
        fn ask(&self, _d: &Doubt) -> Result<String, String> {
            self.answer
                .clone()
                .ok_or_else(|| "endpoint unreachable".to_string())
        }
    }

    /// **The flag: a doubt leaves, comes back, and the crossing is on the log.**
    #[test]
    fn a_doubt_leaves_and_the_crossing_is_recorded() {
        let dir = Scratch::new("out");
        let mut store = Store::open(dir.path());
        let d = Doubt::new(
            "which rule governs a dormant creature?",
            Unresolvable::NoRule,
        );

        let c = consult(
            &mut store,
            d,
            &Fixed {
                name: "test-engine",
                answer: Some("tend wins".into()),
            },
        )
        .expect("recorded");

        assert!(c.answered());
        assert_eq!(c.answer.as_deref(), Some("tend wins"));
        assert_eq!(ledger(&store).answered, 1, "the crossing is on the log");
    }

    /// **The load-bearing one: the model is a caller, not a witness.**
    ///
    /// `Trace::succeeded` is the flag the self-cleaning pass reads and the one
    /// the escalation count credits. A consultation writes `false` whatever came
    /// back, so an engine that answers "your edit is fine" cannot be mistaken for
    /// a green suite. This is the whole reason the model sits outside the loop.
    #[test]
    fn an_answer_is_a_claim_and_never_a_witness() {
        let dir = Scratch::new("witness");
        let mut store = Store::open(dir.path());
        consult(
            &mut store,
            Doubt::new("is this edit correct?", Unresolvable::NoRule),
            &Fixed {
                name: "very-confident-engine",
                answer: Some("yes, definitely".into()),
            },
        )
        .expect("recorded");

        let t = store.traces().last().expect("a trace was written");
        assert!(
            !t.succeeded,
            "an engine said the edit was fine and that was recorded as a \
             confirmation; only a test suite may do that"
        );
        assert_eq!(
            t.actor,
            Some(Actor::Caller),
            "authored from outside, unverified"
        );
    }

    /// **A refusal is recorded, and it costs a failed trace.** A resolver that is
    /// down must not look like a resolver that was never asked, or the system
    /// learns to work without asking and the fallback becomes the default.
    #[test]
    fn a_refusal_is_recorded_rather_than_skipped() {
        let dir = Scratch::new("refused");
        let mut store = Store::open(dir.path());
        let c = consult(
            &mut store,
            Doubt::new(
                "q",
                Unresolvable::Contradiction {
                    a: "r1".into(),
                    b: "r2".into(),
                },
            ),
            &Fixed {
                name: "down",
                answer: None,
            },
        )
        .expect("a refusal is still a recorded crossing");

        assert!(!c.answered());
        assert_eq!(c.refusal.as_deref(), Some("endpoint unreachable"));
        let l = ledger(&store);
        assert_eq!(l.refused, 1);
        assert_eq!(l.answered, 0);
        assert_eq!(l.answer_rate(), Some(0.0), "asked once, answered never");
    }

    /// **Escalation is per shape of doubt, and the question is not in it.**
    /// Two thousand different questions that are all "no rule reaches this" are
    /// one capability. Including the question would make the rate a count of
    /// how often somebody asked, which is rumination.
    #[test]
    fn a_thousand_different_questions_of_one_shape_are_one_capability() {
        let dir = Scratch::new("shapes");
        let mut store = Store::open(dir.path());
        let engine = Fixed {
            name: "e",
            answer: Some("a".into()),
        };
        for i in 0..1_000 {
            consult(
                &mut store,
                Doubt::new(&format!("question {i}"), Unresolvable::NoRule),
                &engine,
            )
            .expect("recorded");
        }
        let l = ledger(&store);
        assert_eq!(l.answered, 1_000, "a thousand crossings happened");
        assert_eq!(
            l.shapes, 1,
            "and they were all the same capability, because the question is not \
             part of the signature"
        );
    }

    /// Four shapes of unanswerability, four capabilities, and the count is the
    /// growth ceiling this route has.
    #[test]
    fn the_four_reasons_are_four_capabilities() {
        let dir = Scratch::new("four");
        let mut store = Store::open(dir.path());
        let engine = Fixed {
            name: "e",
            answer: Some("a".into()),
        };
        for d in [
            Doubt::new("a", Unresolvable::NoRule),
            Doubt::new("b", Unresolvable::Exhausted),
            Doubt::new(
                "c",
                Unresolvable::Contradiction {
                    a: "x".into(),
                    b: "y".into(),
                },
            ),
            Doubt::new(
                "d",
                Unresolvable::Refuted {
                    rule: "r".into(),
                    observed: "o".into(),
                },
            ),
        ] {
            consult(&mut store, d, &engine).expect("recorded");
        }
        assert_eq!(ledger(&store).shapes, 4, "four reasons, four capabilities");
    }

    /// The engine is part of the signature, so the same question asked of two
    /// engines is two capabilities — which it is, because the answers can differ.
    #[test]
    fn two_engines_are_two_capabilities_for_the_same_doubt() {
        let dir = Scratch::new("engines");
        let mut store = Store::open(dir.path());
        let d = Doubt::new("q", Unresolvable::NoRule);
        consult(
            &mut store,
            d.clone(),
            &Fixed {
                name: "engine-a",
                answer: Some("1".into()),
            },
        )
        .expect("recorded");
        consult(
            &mut store,
            d,
            &Fixed {
                name: "engine-b",
                answer: Some("2".into()),
            },
        )
        .expect("recorded");

        // Same doubt shape, so `shapes` counts the `Consult_NoRule` signature
        // once — and the engine name is in the primitive sequence beside it, so
        // the two crossings are distinguishable on the log even though the
        // signature is shared.
        let l = ledger(&store);
        assert_eq!(l.shapes, 1);
        assert_eq!(l.answered, 2);
        let engines: BTreeSet<String> = store
            .traces()
            .iter()
            .filter_map(|t| t.primitives.get(1).cloned())
            .collect();
        assert_eq!(engines.len(), 2, "and both engines are named on the log");
    }

    /// An empty ledger has no answer rate, and saying `0.0` would read as "the
    /// engine always refuses", which is a claim about an engine that was never
    /// asked.
    #[test]
    fn an_empty_ledger_reports_no_rate_rather_than_zero() {
        let dir = Scratch::new("empty");
        let store = Store::open(dir.path());
        let l = ledger(&store);
        assert_eq!(l, DoubtLedger::default());
        assert_eq!(
            l.answer_rate(),
            None,
            "nothing was asked, so there is no rate"
        );
    }
}
