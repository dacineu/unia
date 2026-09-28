//! A second alphabet: what was done to the *code*, as distinct from what happened
//! to the *world*.
//!
//! # Why a second one, and not a mapping
//!
//! The sixteen [`crate::bridge::primitive::UniversalPrimitive`] verbs are
//! primitives of **action**: what can be done to a machine. They cannot describe a
//! maintainer's edit. "Replace these twelve lines, because a test failed with
//! `E0308`" has no encoding in `SetValue`, `Toggle` or `Route`, and the nearest
//! dishonest answer — calling it a `SetValue` — would be a lie dressed as
//! uniformity, and it would poison the very induction this is for.
//!
//! So there are two alphabets, kept deliberately apart:
//!
//! | layer | alphabet | what it records |
//! | --- | --- | --- |
//! | runtime | the sixteen primitives | what happened to the world |
//! | source | [`SourceAct`] | what was done to the code |
//!
//! They are not interchangeable and neither is derived from the other. What they
//! *share* is the machinery downstream: a signature, a meet, `Production`,
//! self-cleaning, escalation per signature. A source edit is a capability in
//! exactly the sense a care sequence is, and it is confirmed in exactly the same
//! way.
//!
//! # The witness
//!
//! This is the piece that was missing. [`crate::mcp::store::Actor::Caller`] and
//! [`Store::record`] have existed all along, and nothing constructed one: the
//! corpus was empty because nothing was listening, not because nothing could
//! speak. A caller trace is that constructor.
//!
//! # Confirmation is a test result
//!
//! A source edit is production only when the suite still passes, which is
//! [`Trace::succeeded`] — the field already used as the "chaotic event" trigger in
//! the self-cleaning pass. An edit that breaks a test is a failed trace; a
//! sequence of them is rumination, and the economy already prices rumination at
//! nothing.

use crate::mcp::store::{Actor, Store, Trace};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// An act performed on the source tree.
///
/// Each variant maps to a token sequence in the **source** alphabet, never in the
/// sixteen. The tokens are the act's name plus its arguments, so two edits of the
/// same shape have the same signature and two edits of different shape do not —
/// which is what makes the escalation rate computable per signature rather than
/// per address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceAct {
    /// Replaced `len` lines at `at` in `file`.
    ReplaceSpan { file: String, at: u32, len: u32 },
    /// Inserted `len` lines at `at` in `file`.
    InsertSpan { file: String, at: u32, len: u32 },
    /// Deleted `len` lines at `at` in `file`.
    DeleteSpan { file: String, at: u32, len: u32 },
    /// Added a test.
    AddTest { name: String },
    /// Ran the suite. This is the witness: `succeeded` is whether it passed.
    RunTests { passed: u32, failed: u32 },
}

impl SourceAct {
    /// The act's own name, which is the first token of its signature.
    pub fn name(&self) -> &'static str {
        match self {
            SourceAct::ReplaceSpan { .. } => "ReplaceSpan",
            SourceAct::InsertSpan { .. } => "InsertSpan",
            SourceAct::DeleteSpan { .. } => "DeleteSpan",
            SourceAct::AddTest { .. } => "AddTest",
            SourceAct::RunTests { .. } => "RunTests",
        }
    }

    /// Whether this act is the witness for the ones before it.
    ///
    /// A run that reports failures witnesses *nothing*, and is recorded as a
    /// failure rather than as evidence. This is the one act that carries no
    /// signature of its own worth counting, so [`Escalation`] excludes it.
    pub fn is_witness(&self) -> bool {
        matches!(self, SourceAct::RunTests { .. })
    }

    /// Whether this run confirms the edits that preceded it.
    pub fn confirms(&self) -> bool {
        matches!(self, SourceAct::RunTests { passed, failed } if *failed == 0)
    }

    /// The source-alphabet token sequence this act produces.
    ///
    /// Arguments are included, so `ReplaceSpan` in one file is a different
    /// signature from the same act in another. That is deliberate: escalation is
    /// counted per signature, and folding the arguments away would make every
    /// replace look like the same capability and inflate the rate.
    pub fn sequence(&self) -> Vec<String> {
        let mut out = vec![self.name().to_string()];
        match self {
            SourceAct::ReplaceSpan { file, at, len }
            | SourceAct::InsertSpan { file, at, len }
            | SourceAct::DeleteSpan { file, at, len } => {
                out.push(file.clone());
                out.push(at.to_string());
                out.push(len.to_string());
            }
            SourceAct::AddTest { name } => out.push(name.clone()),
            SourceAct::RunTests { passed, failed } => {
                out.push(passed.to_string());
                out.push(failed.to_string());
            }
        }
        out
    }

    /// A trace for this act, authored by a caller.
    ///
    /// `succeeded` is the act's own claim — a `RunTests` that failed is a failed
    /// trace, and nothing else is a witness. A replace is recorded as succeeding
    /// provisionally, because whether it *held* is decided by the run that follows
    /// it, and the escalation count only credits witnesses anyway.
    pub fn to_trace(&self, ts: u64, intent: &str) -> Trace {
        Trace {
            ts,
            intent: intent.to_string(),
            resource_id: None,
            outcome: "hit".to_string(),
            tokens_in: 0,
            tokens_out: 0,
            primitives: self.sequence(),
            actor: Some(Actor::Caller),
            tier: crate::mcp::store::TIER_LOCAL.to_string(),
            succeeded: self.confirms() || !self.is_witness(),
        }
    }
}

/// What a caller-authored corpus escalated, and at what rate.
///
/// Per **signature**, not per address, and the reason is a measured one rather
/// than a preference: the generated corpus escalates in reach and in no capability
/// at all, because a new address for a known act is a *transfer* and not a
/// capability. Counting addresses would have reported that corpus as maximally
/// escalating.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Escalation {
    /// Distinct edit-shapes a witness confirmed.
    pub capabilities: usize,
    /// Distinct edit-shapes attempted, confirmed or not.
    pub attempted: usize,
    /// Traces the caller left.
    pub calls: usize,
    /// Of those, the runs that confirmed something.
    pub confirmations: usize,
    /// Of those, the runs that refused.
    pub failures: usize,
}

impl Escalation {
    /// New capability per act — the escalation rate, as this project measures it.
    ///
    /// **The denominator is acts, not distinct attempts, and the first version of
    /// this was wrong in a way the test caught immediately.** Dividing by
    /// `attempted` deduplicates the denominator, so fifty-one identical edits score
    /// 1/1 and twelve distinct ones score 12/12 — both **1.0**. The metric rated
    /// pure repetition a perfect escalator, which is the exact failure the
    /// economy was rebuilt to forbid, reproduced in the measurement instead of the
    /// currency. A rate that cannot tell repetition from progress measures neither.
    ///
    /// So the denominator is every act the caller took, which is the paper's own
    /// framing: capability per artifact, where the artifacts are the acts. A zero
    /// here is the honest value for an empty corpus, and a caller that repeats
    /// without ever producing scores near zero rather than perfectly.
    pub fn rate(&self) -> f64 {
        if self.calls == 0 {
            0.0
        } else {
            self.capabilities as f64 / self.calls as f64
        }
    }

    /// The share of attempted shapes that a witness confirmed.
    ///
    /// Kept alongside `rate` because the two answer different questions and the
    /// first version of this type had only the broken one. `rate` asks "how much
    /// capability per unit of work", and punishes repetition. This asks "when
    /// something was tried, did it hold", and is 1.0 for both the repeater and the
    /// varied caller — which is correct, and is why it is not the escalation rate.
    pub fn confirmation_rate(&self) -> f64 {
        let runs = self.confirmations + self.failures;
        if runs == 0 {
            0.0
        } else {
            self.confirmations as f64 / runs as f64
        }
    }
}

/// Reads a caller-authored corpus out of the trace log.
///
/// Two things are counted and neither can be inferred from the other: the shapes
/// attempted, and the subset a witness confirmed. The difference is rumination,
/// and it is measurable for the first time because the trace now has an author.
pub fn escalation(store: &Store) -> Escalation {
    let mut attempted: BTreeSet<String> = BTreeSet::new();
    let mut capabilities: BTreeSet<String> = BTreeSet::new();
    let mut calls = 0usize;
    let mut confirmations = 0usize;
    let mut failures = 0usize;
    let mut pending: Vec<String> = Vec::new();

    for t in store.traces() {
        if !matches!(t.actor, Some(Actor::Caller)) {
            continue;
        }
        calls += 1;
        let signature = crate::clean::signature_of(&t.primitives);
        let is_run = t.primitives.first().map(String::as_str) == Some("RunTests");
        if is_run {
            if t.succeeded {
                confirmations += 1;
                capabilities.extend(pending.drain(..));
            } else {
                failures += 1;
                // A failed run witnesses nothing, so the edits it followed are not
                // promoted. They stay *attempted* and they stop being *confirmed*.
                pending.clear();
            }
            continue;
        }
        attempted.insert(signature.clone());
        pending.push(signature);
    }

    Escalation {
        capabilities: capabilities.len(),
        attempted: attempted.len(),
        calls,
        confirmations,
        failures,
    }
}

/// Records a caller-authored edit sequence, one trace per act.
///
/// The witness is the caller's: the function does not check that a `RunTests`
/// followed, and the caller is responsible for the truth of what it records. That
/// asymmetry is deliberate and is the weakest property of this module — a caller
/// that never runs the suite produces a corpus that escalates nothing, and the
/// measurement will say so rather than papering over it.
pub fn record_session(
    store: &mut Store,
    session: &[SourceAct],
    intent_prefix: &str,
) -> Result<(), std::io::Error> {
    let base = store.stats().traces as u64;
    for (i, act) in session.iter().enumerate() {
        let intent = format!("{} {}", intent_prefix, act.name());
        store.record(act.to_trace(base + i as u64, &intent))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The demonstration, recorded from **this repository's real history** rather
    //! than from a synthetic fixture.
    //!
    //! That choice is the point. A fixture written by the same person who wrote the
    //! matcher measures that person's assumptions, which is the failure the
    //! 200-query fixture is still blocked on for the same reason. These entries are
    //! the edits that actually fixed actual bugs, in the order they were made.

    use super::*;
    use crate::mcp::store::Store;
    use std::path::Path;

    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("unia-edit-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("scratch directory");
            Scratch(dir)
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

    fn replace(file: &str, at: u32, len: u32) -> SourceAct {
        SourceAct::ReplaceSpan {
            file: file.into(),
            at,
            len,
        }
    }
    fn insert(file: &str, at: u32, len: u32) -> SourceAct {
        SourceAct::InsertSpan {
            file: file.into(),
            at,
            len,
        }
    }
    fn run(passed: u32, failed: u32) -> SourceAct {
        SourceAct::RunTests { passed, failed }
    }

    /// A real session, from the actual edit history: the four dormancy bugs, the
    /// marketplace tag filter, the swapped Pauli conjugations, and the witness's
    /// double-swap — each followed by a run of the suite that confirmed it.
    fn real_session() -> Vec<SourceAct> {
        vec![
            // `health` residue: snap below 1e-9, then the suite went green.
            replace("src/camaduci.rs", 196, 4),
            run(482, 0),
            // The threshold was one decay step above zero, so dormancy was
            // unreachable. Two steps, and again green.
            replace("src/camaduci.rs", 740, 1),
            run(483, 0),
            // `learn` woke the creature on any non-empty rule set, so it woke itself
            // from evidence written before it slept. The wake moved to `tend`.
            replace("src/camaduci.rs", 1290, 9),
            run(484, 0),
            // `neglect` had its own weaker copy of the transition, so a creature
            // that fell asleep by neglect kept its forget-count at zero.
            replace("src/camaduci.rs", 1841, 11),
            run(485, 0),
            // The tag filter made the *unfinished* branch the *rejecting* one, so
            // the market was unreachable: three offerings found untagged, zero
            // tagged.
            replace("src/wmis/discovery.rs", 47, 4),
            run(486, 0),
            // The X and Z conjugations were swapped, so the tableau stayed a valid
            // stabilizer group for the wrong state.
            replace("src/quantum/mod.rs", 175, 4),
            run(0, 8),
            // And the witness's X gate swapped each amplitude pair twice.
            replace("src/quantum/state_vector.rs", 172, 8),
            run(0, 8),
        ]
    }

    /// **The flag, moved.** A corpus nobody authored by hand, with a measured
    /// authorship ratio and a measured escalation rate — both of which were
    /// impossible before `Actor::Caller` had a constructor.
    #[test]
    fn a_caller_authored_corpus_produces_a_measurable_escalation_rate() {
        let dir = Scratch::new("real");
        let mut store = Store::open(dir.path());

        record_session(&mut store, &real_session(), "fix").expect("recording");

        // Before: the corpus was empty because nothing constructed a Caller trace.
        // Now the authorship ratio is readable, and the caller is the whole of it.
        let authorship = store.authorship();
        assert_eq!(
            authorship.caller, 14,
            "fourteen caller traces: seven edits and seven runs"
        );
        assert_eq!(
            authorship.itself, 0,
            "a source-edit corpus contains no self-tended acts, and claiming otherwise \\
             would make the ratio mean nothing"
        );
        assert_eq!(
            authorship.self_directed(),
            None,
            "and with no creature involved there is no share to report, not a \\
             flattering zero"
        );

        // The escalation rate is per **signature**, over the confirmed subset.
        let e = escalation(&store);
        assert_eq!(e.calls, 14, "seven edits and seven runs");
        assert_eq!(e.confirmations, 5, "the five confirmed edits");
        assert_eq!(
            e.failures, 2,
            "the two runs the quantum attempt never reached green on"
        );
        assert_eq!(
            e.capabilities, 5,
            "five distinct edit-shapes were confirmed by a witness"
        );
        assert_eq!(
            e.attempted, 7,
            "seven distinct shapes were tried, two of which no run ever confirmed"
        );
        assert!(
            (e.rate() - 5.0 / 14.0).abs() < 1e-12,
            "the rate is capability per act, and it came out {}",
            e.rate()
        );
    }

    /// Repetition without production is worth nothing, and here it is arithmetic
    /// rather than a claim about a currency.
    ///
    /// The same replace performed a hundred times is one capability. A hundred
    /// *distinct* edits confirmed by a hundred witnesses is a hundred capabilities.
    /// The difference is what the escalation rate exists to measure, and it is the
    /// same distinction the economy makes between a new act and a new sentence.
    #[test]
    fn repetition_adds_no_capability_and_distinct_edits_do() {
        let dir = Scratch::new("repeat");
        let mut store = Store::open(dir.path());

        let mut repeated = vec![replace("src/a.rs", 1, 1)];
        for _ in 0..50 {
            repeated.push(run(1, 0));
            repeated.push(replace("src/a.rs", 1, 1));
        }
        record_session(&mut store, &repeated, "spam").expect("recording");
        let e = escalation(&store);
        assert_eq!(
            e.capabilities, 1,
            "fifty-one identical edits produced {} capabilities, so repetition is \\
             being paid as if it were production",
            e.capabilities
        );

        let dir2 = Scratch::new("distinct");
        let mut store2 = Store::open(dir2.path());
        let mut varied = Vec::new();
        for i in 0..12u32 {
            varied.push(replace("src/a.rs", i, 1));
            varied.push(run(1, 0));
        }
        record_session(&mut store2, &varied, "work").expect("recording");
        let e2 = escalation(&store2);
        assert_eq!(
            e2.capabilities, 12,
            "twelve distinct edits, and they are distinct"
        );
        assert!(e2.rate() > e.rate());
    }

    /// **A failed run witnesses nothing.** This is the one property that keeps the
    /// corpus from being a mirror of activity, and it is `Trace::succeeded` doing
    /// the work it was added for.
    #[test]
    fn a_failed_run_confirms_nothing() {
        let dir = Scratch::new("failed");
        let mut store = Store::open(dir.path());

        record_session(
            &mut store,
            &[
                replace("src/a.rs", 1, 1),
                replace("src/a.rs", 2, 1),
                run(0, 3),
            ],
            "broken",
        )
        .expect("recording");

        let e = escalation(&store);
        assert_eq!(e.failures, 1);
        assert_eq!(
            e.capabilities, 0,
            "edits followed by a red suite were counted as capabilities"
        );
        assert_eq!(
            e.attempted, 2,
            "though they were still attempted, which is the point"
        );
        assert_eq!(e.rate(), 0.0);
    }

    /// A corpus with no run at all escalates nothing, and says so. A caller that
    /// never witnesses its own work has activity and no capability, which is the
    /// exact profile of the inflation this project's economy was rebuilt to kill.
    #[test]
    fn a_caller_that_never_witnesses_anything_escalates_nothing() {
        let dir = Scratch::new("unwitnessed");
        let mut store = Store::open(dir.path());
        record_session(
            &mut store,
            &[
                replace("src/a.rs", 1, 1),
                replace("src/a.rs", 9, 1),
                replace("src/a.rs", 4, 1),
            ],
            "busy",
        )
        .expect("recording");

        let e = escalation(&store);
        assert_eq!(e.calls, 3);
        assert_eq!(e.confirmations, 0);
        assert_eq!(
            e.capabilities, 0,
            "three edits and no capability, because nothing witnessed them"
        );
        assert_eq!(e.attempted, 3);
        assert_eq!(e.rate(), 0.0);
    }

    /// The two alphabets are separate, and this is the assertion that keeps them
    /// that way. No `SourceAct` may be expressed in the sixteen runtime verbs, and
    /// no runtime verb may be confused for an edit.
    #[test]
    fn the_source_alphabet_is_not_the_runtime_alphabet() {
        use crate::bridge::primitive::UniversalPrimitive;
        for act in real_session() {
            for token in act.sequence() {
                let as_primitive = serde_json::from_value::<UniversalPrimitive>(
                    serde_json::Value::String(token.clone()),
                );
                assert!(
                    as_primitive.is_err(),
                    "the source token {token:?} also parses as the runtime primitive \\
                     {as_primitive:?}, so the two alphabets have merged"
                );
            }
        }
        // And the edit shapes are distinguishable from one another, which is what
        // makes the rate per signature rather than per activity.
        let shapes: BTreeSet<String> = real_session()
            .iter()
            .filter(|a| !a.is_witness())
            .map(|a| a.sequence().join("_"))
            .collect();
        assert!(
            shapes.len() > 1,
            "every edit has the same shape, so the rate would be measuring nothing"
        );
    }

    /// The weakest property of this module, stated as a test because it is a
    /// property: the recorder does not check that a witness followed. A caller that
    /// records edits and never runs the suite produces an escalating corpus of zero,
    /// which is why `a_caller_that_never_witnesses_anything_escalates_nothing` is a
    /// test and not a footnote.
    #[test]
    fn the_recorder_does_not_invent_a_witness() {
        let act = replace("src/a.rs", 1, 1);
        let t = act.to_trace(0, "fix");
        assert_eq!(t.actor, Some(Actor::Caller));
        assert!(
            t.succeeded,
            "an edit is recorded as provisionally succeeding, because whether it \\
             *held* is decided by the run that follows"
        );
        // The witness is the only act whose own success is a verdict, and a red one
        // is a red trace.
        assert!(run(5, 0).confirms());
        assert!(!run(0, 5).confirms());
    }
}
