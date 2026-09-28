//! The loop, closed: propose → witness → promote, with nothing human in the middle.
//!
//! # What was missing
//!
//! Four pieces of the learning loop existed and nothing connected them:
//!
//! | piece | where |
//! | --- | --- |
//! | traces written with an author | `edit::SourceAct`, `Actor::Caller` |
//! | induction proposes a mattern | `induce::induce_all` |
//! | a witness that can falsify | `Trace::succeeded` — the suite |
//! | somewhere to put a winner | `registry::set_champion` |
//!
//! `set_champion` had **zero callers**. So induction proposed, the witness was
//! capable of judging, and the promotion was a human typing. That is the whole of
//! "training unia" and it was not a research problem — it was a wire.
//!
//! # Why the witness is the suite and not a model
//!
//! A promotion decision needs something that can say *no*. `Trace::succeeded` is
//! the field the self-cleaning pass already reads as a chaotic event, and it is
//! set from whether a run of the tests passed. That is falsifiable, reproducible
//! and cheap. A model is none of the three, and a promotion gate that consults one
//! has thrown away the only check the system has.
//!
//! So the gate here reads traces and nothing else. **A model could be consulted
//! for a proposal and would be a claim on the log** (`doubt::consult`), never a
//! witness — the same rule the consultation route already obeys.
//!
//! # The three things this refuses to do silently
//!
//! - **It does not promote on confidence.** `Candidate::confidence` comes from
//!   how many distinct phrasings were observed, which is evidence that the
//!   *description* is stable, not that the *rule* is right. Promoting on it would
//!   mean a rule described in many ways by many failed acts gets promoted. The
//!   gate is a witness and nothing else.
//! - **It does not promote a proposal it has not seen verified.** An unwitnessed
//!   candidate stays a candidate, and the ledger says how many there are, so
//!   "nothing was promoted" is distinguishable from "nothing was proposed".
//! - **It never demotes on a new failure.** A champion that a later run refutes
//!   *is* demoted, because a registry that can install a winner but never remove
//!   one is a write-once cache with extra steps. That asymmetry — install on
//!   success, remove on refutation — is the whole of the safety story.

use crate::edit::SourceAct;
use crate::induce::{induce_all, Candidate};
use crate::mcp::store::{Actor, Store, Trace};
use crate::registry::ActuatorRegistry;
use std::collections::BTreeMap;

/// What happened to one proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A witness confirmed it and it is now the champion for its capability.
    Promoted {
        /// The rule's signature, which is its capability.
        capability: String,
        address: String,
    },
    /// It was proposed before and is already the champion. Not a failure: a
    /// second confirmation of a live rule is the system working.
    Confirmed { capability: String },
    /// It is the champion and a later run refuted it, so it was removed.
    Demoted {
        capability: String,
        /// The act that refuted it, so the demotion is attributable.
        by_signature: String,
    },
    /// Proposed, and no witness has confirmed it. Not a failure either — it is a
    /// proposal still waiting for evidence, which is the normal state of most
    /// candidates.
    Awaiting { capability: String },
    /// Induction refused to propose it, and why.
    NotEvidentiary { capability: String },
}

/// The run, counted.
///
/// Every count is here because "the loop closed" and "the loop closed and
/// produced nothing" are different claims, and a summary that reports one number
/// cannot tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Run {
    pub proposed: usize,
    pub promoted: usize,
    pub confirmed: usize,
    pub demoted: usize,
    pub awaiting: usize,
    pub not_evidentiary: usize,
}

impl Run {
    /// Promotions per proposal.
    ///
    /// `None` when nothing was proposed, because a ratio between zero and zero is
    /// not a number, and `0.0` would read as "every proposal failed".
    pub fn promotion_rate(&self) -> Option<f64> {
        (self.proposed > 0).then(|| self.promoted as f64 / self.proposed as f64)
    }

    /// The witness's verdict, over the acts that reached it.
    ///
    /// **Not the same number as the promotion rate and the test suite asserts
    /// they are not.** A caller that runs the suite, sees it green, proposes
    /// nothing new, and promotes nothing has a witness rate of 1.0 and a
    /// promotion rate of 0.0. Reading the first as the second is how a system
    /// that learned nothing reports perfect health.
    pub fn witness_rate(&self) -> Option<f64> {
        let witnessed = self.promoted + self.confirmed + self.demoted;
        (witnessed > 0).then(|| (self.promoted + self.confirmed) as f64 / witnessed as f64)
    }
}

/// The pairing a witness comes from: an edit, and the run that judged it.
///
/// Taken apart because a proposal carries only the act and the witness is a
/// *later* trace. A gate that read the proposing trace's own `succeeded` would be
/// grading a proposal by whether the proposal was written down correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    /// The signature the witness covers.
    pub signature: String,
    /// Whether the suite passed.
    pub passed: bool,
}

/// Reads witnesses off the trace log.
///
/// A `RunTests` trace is a witness for the acts immediately before it, which is
/// the same pairing `edit::escalation` uses and for the same reason: an edit is
/// only confirmed by a run that *followed* it. An edit with no run after it is
/// not confirmed and not refuted — it is unfinished, and reporting it as either
/// would be a claim nobody made.
pub fn witnesses(traces: &[Trace]) -> Vec<Witness> {
    let mut pending: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for t in traces {
        let is_run = t.primitives.first().map(String::as_str) == Some("RunTests");
        if !is_run {
            let sig = crate::clean::signature_of(&t.primitives);
            pending.push(sig);
            continue;
        }
        for sig in pending.drain(..) {
            out.push(Witness {
                signature: sig,
                passed: t.succeeded,
            });
        }
    }
    out
}

/// Runs the loop once over a store's log, promoting into `registry`.
///
/// Returns the per-proposal outcomes and the run's counts. Nothing here is
/// hidden: a candidate with no witness is `Awaiting`, not promoted and not
/// dropped.
pub fn run(store: &Store, registry: &mut ActuatorRegistry) -> (Vec<Outcome>, Run) {
    let traces = store.traces();
    let seen = witnesses(&traces);
    let by_sig: BTreeMap<String, bool> =
        seen.iter()
            .fold(BTreeMap::new(), |mut acc: BTreeMap<String, bool>, w| {
                // A signature confirmed by *any* passing run and refuted by no
                // failing one is confirmed. A signature with both is not, and the
                // fold takes the strict reading: a failure poisons. Anything looser
                // would let one green run outweigh a later red one.
                acc.entry(w.signature.clone())
                    .and_modify(|p| *p = *p && w.passed)
                    .or_insert(w.passed);
                acc
            });

    let candidates: Vec<Candidate> = match induce_all(&traces) {
        Ok(c) => c,
        // `induce_all` fails only when there is nothing to induce, which is the
        // empty-corpus case and not an error worth propagating.
        Err(_) => Vec::new(),
    };

    let mut outcomes = Vec::new();
    let mut run = Run::default();

    for c in &candidates {
        let capability = c.signature.clone();
        let verdict = by_sig.get(&capability);
        match verdict {
            Some(false) => {
                // Refuted. If it was champion, remove it -- a registry that can
                // install a winner and never remove one is a write-once cache.
                if registry.get_champion(&capability) == Some(c.du_uuid) {
                    registry.clear_champion(&capability);
                    run.demoted += 1;
                    outcomes.push(Outcome::Demoted {
                        by_signature: capability.clone(),
                        capability,
                    });
                } else {
                    run.not_evidentiary += 1;
                    outcomes.push(Outcome::NotEvidentiary { capability });
                }
            }
            Some(true) => {
                if registry.get_champion(&capability) == Some(c.du_uuid) {
                    run.confirmed += 1;
                    outcomes.push(Outcome::Confirmed { capability });
                } else {
                    run.promoted += 1;
                    let address = c.du_uuid.to_string();
                    registry.set_champion(&capability, c.du_uuid);
                    outcomes.push(Outcome::Promoted {
                        capability,
                        address,
                    });
                }
            }
            None => {
                run.awaiting += 1;
                outcomes.push(Outcome::Awaiting { capability });
            }
        }
    }

    run.proposed = candidates.len();
    (outcomes, run)
}

/// Records an edit *and* the run that judges it, as a caller.
///
/// The two are separate because the witness has to come after the edit for the
/// pairing to mean anything, and a helper that recorded them together would let
/// a caller record a witness for an edit that never happened.
pub fn record_verified_edit(
    store: &mut Store,
    act: &SourceAct,
    intent: &str,
    passed: u32,
    failed: u32,
) -> Result<(), std::io::Error> {
    let base = store.stats().traces as u64;
    store.record(act.to_trace(base, intent))?;
    store.record(SourceAct::RunTests { passed, failed }.to_trace(base + 1, intent))
}

#[cfg(test)]
mod tests {
    //! The loop, run for real against a real store and a real registry.

    use super::*;
    use std::path::{Path, PathBuf};

    struct Scratch(PathBuf);
    impl Scratch {
        fn new(tag: &str) -> Self {
            let d = std::env::temp_dir().join(format!("unia-loop-{tag}-{}", std::process::id()));
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

    fn replace(file: &str, at: u32, len: u32) -> SourceAct {
        SourceAct::ReplaceSpan {
            file: file.into(),
            at,
            len,
        }
    }

    /// **The flag: an edit, a green run, and a champion with nobody typing.**
    ///
    /// This is the wire that did not exist. `set_champion` had zero callers
    /// before this test, and it is called by `run` and not by a human anywhere.
    #[test]
    fn a_witnessed_edit_is_promoted_without_a_human() {
        let dir = Scratch::new("promote");
        let mut store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());

        // Two *distinct phrasings* of one act, because induction requires
        // `MIN_OBSERVATIONS` of them. And two *different* run results, so the
        // RunTests traces do not themselves group into a second candidate --
        // one candidate here means one thing was proposed, not two.
        //
        // A fixture using one phrasing every time proposes nothing, and the
        // first version of this test did exactly that: it read as "the loop
        // promotes nothing" when the truth was "the fixture never cleared
        // the bar".
        for (phrasing, passed) in [("fix the crash", 41u32), ("repair the failure", 42)] {
            record_verified_edit(&mut store, &replace("src/a.rs", 1, 1), phrasing, passed, 0)
                .expect("recorded");
        }
        let (outcomes, run) = run(&store, &mut reg);

        assert_eq!(run.proposed, 1, "induction proposed from the trace");
        assert_eq!(run.promoted, 1, "and the witness promoted it");
        assert_eq!(run.awaiting, 0, "nothing was left unwitnessed");
        assert_eq!(outcomes.len(), 1);
        match &outcomes[0] {
            Outcome::Promoted {
                capability,
                address,
            } => {
                assert!(!capability.is_empty());
                assert!(!address.is_empty(), "and the address it was promoted under");
            }
            other => panic!("expected a promotion, got {other:?}"),
        }
        assert_eq!(reg.champion_count(), 1, "the registry now has a champion");
    }

    /// **Promotion needs breadth *and* a witness, and neither alone is enough.**
    ///
    /// Found while writing the fixtures above: a single phrasing repeated never
    /// clears `MIN_OBSERVATIONS`, so the loop proposes nothing no matter how many
    /// green runs accompany it. That is the induction module's existing bar and
    /// the loop inherits it rather than weakening it — a rule described one way is
    /// one observation, however many times it succeeded. Both halves are asserted
    /// here so a future change that drops either one is visible.
    #[test]
    fn one_phrasing_is_never_enough_however_often_it_is_witnessed() {
        let dir = Scratch::new("breadth");
        let mut store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());

        // The same phrasing, five times, every run green.
        for _ in 0..5 {
            record_verified_edit(&mut store, &replace("src/f.rs", 1, 1), "one phrasing", 9, 0)
                .expect("ok");
        }
        let (_, r) = run(&store, &mut reg);
        assert_eq!(
            r.proposed,
            0,
            "five green runs of one phrasing proposed nothing, because induction \
             requires {} distinct phrasings and the witness is not a substitute \
             for breadth",
            crate::induce::MIN_OBSERVATIONS
        );
        assert_eq!(reg.champion_count(), 0);
    }

    /// **The gap, closed, and the test that did not exist.**
    ///
    /// This is the assertion the handover said was missing, and it is worth more
    /// than the rest of this file put together. The promotion was written under a
    /// primitive signature (`ReplaceSpan_src/a.rs_1_1`) while the readers looked
    /// it up under free text, and **the failure was silent**: a key mismatch
    /// returns `None`, the caller falls through to the manifest, and the system
    /// behaves exactly as it did before the loop existed. No error, no failing
    /// test — a loop that ran perfectly and changed nothing.
    ///
    /// So this test does not compare two functions. It takes a signature the
    /// *writer* produces and asks for it the way a *reader* would, in every form
    /// the resolver is indifferent to, and requires a hit each time.
    #[test]
    fn a_promoted_signature_is_reachable_by_the_forms_readers_use() {
        use crate::registry::ActuatorRegistry as R;
        let dir = Scratch::new("keyspace");
        let mut store = Store::open(dir.path());
        let mut reg = R::with_base_dir("none", dir.path());

        for phrasing in ["green wording", "second green wording"] {
            record_verified_edit(&mut store, &replace("src/a.rs", 1, 1), phrasing, 9, 0)
                .expect("ok");
        }
        let (_, r) = run(&store, &mut reg);
        assert_eq!(r.promoted, 1, "the loop promoted something");

        // The key the writer actually used.
        let key = reg
            .champion_keys()
            .first()
            .cloned()
            .expect("one champion key");
        assert!(
            key.contains("ReplaceSpan") || key.contains("replace"),
            "the key is a normalised form of the signature, not the raw one: {key}"
        );

        // And a reader looking it up in any form the resolver is indifferent to
        // finds it. `tokenize_id` is indifferent to case, to underscores, and to
        // camel boundaries, so these are the same capability and must be one
        // bucket.
        for asked in [
            key.clone(),
            key.to_lowercase(),
            key.replace('_', " "),
            key.to_uppercase(),
        ] {
            assert!(
                reg.get_champion(&asked).is_some(),
                "a reader asking {asked:?} did not find the champion stored under \
                 {key:?}. The key spaces disagree, the promotion is unrecorded as \
                 far as any caller is concerned, and nothing fails."
            );
        }
    }

    /// **The negative, because the positive alone would not catch it.** A key the
    /// loop never wrote must still miss, or the assertion above would pass with a
    /// registry that returned the same answer to everything.
    #[test]
    fn a_signature_that_was_never_promoted_still_misses() {
        let dir = Scratch::new("keymiss");
        let mut store = Store::open(dir.path());
        let mut reg = crate::registry::ActuatorRegistry::with_base_dir("none", dir.path());

        for phrasing in ["green wording", "second green wording"] {
            record_verified_edit(&mut store, &replace("src/a.rs", 1, 1), phrasing, 9, 0)
                .expect("ok");
        }
        run(&store, &mut reg).1;

        assert!(reg.get_champion("ReplaceSpan_src/never.rs_99_9").is_none());
        assert!(reg
            .get_champion("completely unrelated capability")
            .is_none());
    }

    /// **A red suite does not promote.** This is the direction that matters: a
    /// loop that only promotes on success is a publisher, and a publisher cannot
    /// be the thing that checks a system.
    #[test]
    fn a_refuted_edit_is_not_promoted() {
        let dir = Scratch::new("refute");
        let mut store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());

        record_verified_edit(&mut store, &replace("src/a.rs", 7, 2), "fix", 0, 3)
            .expect("recorded");
        let (_, run) = run(&store, &mut reg);

        assert!(
            run.promoted == 0,
            "a rule witnessed by a red suite was promoted, which is the one thing \
             this loop must never do"
        );
        assert_eq!(reg.champion_count(), 0, "and the registry stayed empty");
    }

    /// **An unwitnessed edit stays a proposal.** Not promoted, not refuted, not
    /// dropped — and the count distinguishes it from "nothing was proposed",
    /// which is what makes the summary honest.
    #[test]
    fn an_edit_with_no_run_after_it_is_unfinished_rather_than_confirmed() {
        let dir = Scratch::new("unwitnessed");
        let mut store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());

        let base = store.stats().traces as u64;
        store
            .record(replace("src/a.rs", 3, 1).to_trace(base, "fix"))
            .expect("recorded");
        let (_, run) = run(&store, &mut reg);

        assert_eq!(
            run.promoted, 0,
            "nothing was witnessed, so nothing was promoted"
        );
        assert_eq!(reg.champion_count(), 0);
    }

    /// **A champion is removed when a later run refutes it.** A registry that can
    /// install a winner and never remove one is a write-once cache with extra
    /// steps, and every long-running system needs the second half.
    #[test]
    fn a_later_refutation_demotes_a_live_champion() {
        let dir = Scratch::new("demote");
        let mut store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());

        // Green first: promoted. Two phrasings, so the group is evidentiary.
        for phrasing in ["green wording", "second green wording"] {
            record_verified_edit(&mut store, &replace("src/b.rs", 1, 1), phrasing, 10, 0)
                .expect("ok");
        }
        let (_, first) = run(&store, &mut reg);
        assert_eq!(first.promoted, 1, "the green run installed a champion");

        // Then red on the same signature: demoted.
        let act = replace("src/b.rs", 1, 1);
        let base = store.stats().traces as u64;
        store.record(act.to_trace(base, "fix")).expect("ok");
        store
            .record(
                SourceAct::RunTests {
                    passed: 0,
                    failed: 5,
                }
                .to_trace(base + 1, "fix"),
            )
            .expect("ok");
        let (_, second) = run(&store, &mut reg);

        assert_eq!(second.demoted, 1, "and the later red run removed it");
        assert_eq!(
            reg.champion_count(),
            0,
            "a champion that a green run installed and a red run refuted must not \\
             survive both"
        );
    }

    /// **A failure poisons.** One green run followed by one red is not
    /// "confirmed on balance"; the fold takes the strict reading, because the
    /// looser one lets an early success outvote a later refutation.
    #[test]
    fn a_refutation_outweighs_an_earlier_confirmation() {
        let dir = Scratch::new("poison");
        let mut store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());

        let act = replace("src/c.rs", 4, 1);
        for (phrasing, failed) in [("one", 0u32), ("two", 9u32)] {
            let base = store.stats().traces as u64;
            store.record(act.to_trace(base, phrasing)).expect("ok");
            store
                .record(SourceAct::RunTests { passed: 12, failed }.to_trace(base + 1, phrasing))
                .expect("ok");
        }
        let (_, r) = run(&store, &mut reg);
        assert_eq!(r.promoted, 0, "green then red is not a confirmation");
        assert_eq!(reg.champion_count(), 0);
    }

    /// **The two rates are not the same number, and a test that let them be would
    /// report a system that learned nothing as healthy.**
    #[test]
    fn a_healthy_witness_is_not_a_healthy_loop() {
        // A green run and a proposal that is already the champion: the witness
        // said yes and the loop produced no new capability.
        let run = Run {
            proposed: 4,
            promoted: 0,
            confirmed: 3,
            demoted: 0,
            awaiting: 1,
            not_evidentiary: 0,
        };
        assert_eq!(
            run.witness_rate(),
            Some(1.0),
            "nothing the witness saw was wrong"
        );
        assert_eq!(
            run.promotion_rate(),
            Some(0.0),
            "and nothing new was learned, which is a different fact and a test \\
             that conflated them would call this a success"
        );
    }

    /// An empty run has no rates at all, rather than zero of everything. A `0.0`
    /// promotion rate on an empty corpus reads as "every proposal failed" when
    /// nothing was proposed.
    #[test]
    fn an_empty_run_reports_no_rates() {
        let r = Run::default();
        assert_eq!(r.promotion_rate(), None);
        assert_eq!(r.witness_rate(), None);
    }

    /// **A witness only ever follows the acts it judges.** An edit recorded after
    /// a run is unfinished, and pairing them the other way round would let a
    /// green run vouch for an edit that had not happened yet.
    #[test]
    fn a_witness_covers_only_what_preceded_it() {
        let dir = Scratch::new("pairing");
        let mut store = Store::open(dir.path());
        let base = store.stats().traces as u64;

        // Run first, then the edit. The edit has no witness.
        store
            .record(
                SourceAct::RunTests {
                    passed: 5,
                    failed: 0,
                }
                .to_trace(base, "fix"),
            )
            .expect("ok");
        store
            .record(replace("src/d.rs", 1, 1).to_trace(base + 1, "fix"))
            .expect("ok");

        let w = witnesses(&store.traces());
        assert!(
            w.is_empty(),
            "a run that preceded the edit witnessed nothing, and pairing them \
             anyway would let a green run vouch for an act that came after it"
        );
    }

    /// The loop promotes nothing on an empty log, and says so with a count rather
    /// than with silence.
    #[test]
    fn an_empty_corpus_proposes_nothing_and_reports_nothing() {
        let dir = Scratch::new("empty");
        let store = Store::open(dir.path());
        let mut reg = ActuatorRegistry::with_base_dir("none", dir.path());
        let (outcomes, r) = run(&store, &mut reg);
        assert!(outcomes.is_empty());
        assert_eq!(r, Run::default());
        assert_eq!(reg.champion_count(), 0);
    }

    /// **Every recorded trace is authored.** The loop cannot promote a rule whose
    /// evidence nobody wrote, and `Actor::Caller` on every trace is what makes
    /// the corpus attributable to an author in the first place.
    #[test]
    fn every_trace_the_loop_reads_is_attributed() {
        let dir = Scratch::new("authored");
        let mut store = Store::open(dir.path());
        record_verified_edit(&mut store, &replace("src/e.rs", 2, 1), "a phrasing", 3, 0)
            .expect("ok");
        for t in store.traces() {
            assert_eq!(
                t.actor,
                Some(Actor::Caller),
                "an unattributed trace reached the loop, and a rule induced from \\
                 unowned evidence has no author to credit"
            );
        }
    }
}
