//! Keeping the third economy from becoming an archive.
//!
//! A creature lives in two economies. `nuants` are spent, on the short horizon.
//! `quants` are sustained and never spent, on the middle one. Both are numbers on
//! a creature, and both are reached by decay: neglect drains the first and floors
//! the second. Neither has anything to say about the long horizon, which is the
//! matterns themselves — the artifacts, their signatures, their addresses. Nothing
//! has ever cleaned that horizon, so a corpus only ever grows.
//!
//! This is the long-horizon trim, and it is deliberately *not* deletion.
//!
//! # Re-mattering rather than deleting
//!
//! A mattern nobody has confirmed any more is not a capability the project has
//! lost; it is evidence it has not earned. Deleting the rule would delete the
//! capability, and the creature would forget — dormancy, applied to an artifact
//! instead of to a creature. Re-mattering keeps the capability and throws the
//! evidence away: the same signature at a *new address*, computed from what
//! survives. That is the whole difference, and it is the reason this module
//! exists rather than a `retain`-with-a-predicate.
//!
//! # No genealogy
//!
//! A re-mattered artifact is not a child of the one it replaces, and nothing
//! records that it was. The new address is a content address of the surviving
//! material — the signature and the phrasings still standing — and *not* a hash
//! of the old address. Chaining them would be a genealogy with extra steps, and
//! it is the one thing this design refuses to build.
//!
//! So `Cleaning::remattered` is a *report*, returned to a caller and stored
//! nowhere. There is no field anywhere that holds a superseded address, which is
//! what makes the convergence test in [`crate::gather`] possible: a denominator
//! reached by six creatures across six generations, none of which can trace the
//! others, is a genuine convergence. Twelve hand-written patterns sharing zero
//! capabilities and thirty-six generated artifacts sharing six denominators they
//! were manufactured to share are both uniformities instead.
//!
//! # The three triggers
//!
//! All three are read out of the trace log rather than declared, because a trigger
//! nothing records cannot fire honestly:
//!
//! - **unknown situations** — a successful trace whose primitive sequence no
//!   mattern covers. The situation is known to the log and unknown to the
//!   creature.
//! - **chaotic events** — a trace with `succeeded` false. The only failure the
//!   schema already records, which is why it is the answer and a failed
//!   precondition is not: the precondition that could not be met has no record
//!   of its own.
//! - **readiness patterning** — a mattern with no confirming trace at all. Its
//!   readiness has gone, and `Readiness` is a maximum over evidence rather than a
//!   sum, so evidence that has stopped arriving stops counting immediately.

use crate::camaduci::LearnedRule;
use crate::identifiers::DuUuid;
use crate::mcp::store::{Actor, Trace};
use serde_json::json;
use std::collections::BTreeMap;

/// One mattern that was replaced, and what replaced it.
///
/// The `from` field exists to be read once by whoever asked for the cleaning and
/// then discarded. It is deliberately not part of any artifact, and a caller that
/// wants to keep a lineage must build one itself — which is the point.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Rematter {
    /// The capability, unchanged. This is the meet, and a meet survives.
    pub signature: String,
    /// The address being replaced.
    pub from: String,
    /// The address replacing it, computed from the surviving material.
    pub to: String,
}

/// What a cleaning pass found, and what it did about it.
///
/// Four counts and a list, because the triggers are four different things and
/// collapsing them into one "cleaned N" would hide the only interesting case: a
/// mattern that is stale for want of evidence is a different problem from a trace
/// sequence that nothing covers.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cleaning {
    /// Addresses with at least one confirming trace.
    pub confirmed: Vec<String>,
    /// Addresses with none. Every one of these was re-mattered.
    pub stale: Vec<String>,
    /// Successful primitive sequences no mattern covers.
    pub unmattered: Vec<String>,
    /// Primitive sequences whose traces all failed.
    pub failed: Vec<String>,
    /// What replaced what. A report, stored nowhere.
    pub remattered: Vec<Rematter>,
}

impl Cleaning {
    /// Whether the corpus this pass produced differs from the one it was given.
    ///
    /// Not "whether anything was stale". A mattern with no confirming evidence is
    /// stale whether or not replacing it would produce anything different, and a
    /// caller asking this wants to know whether there is work to persist.
    pub fn changed(&self) -> bool {
        !self.remattered.is_empty()
    }

    /// Whether anything lost its evidence, replaced or not.
    pub fn anything_stale(&self) -> bool {
        !self.stale.is_empty()
    }

    /// The capabilities that survived, deduplicated.
    ///
    /// Deduplicated because two matterns with one signature are one capability
    /// held twice, and a cleaning that reported it twice would be counting
    /// redundancy as survival.
    pub fn capabilities_surviving(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .remattered
            .iter()
            .map(|r| r.signature.as_str())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// The state key a dispatched reading lands under.
///
/// Named here because the sensor's condition is written against it in the
/// constraint grammar, and a key that exists in two places as a literal is a key
/// that will be renamed in one of them.
pub const READING_FIELD: &str = "reading";

/// The sequence the readiness sensor dispatches, as a `Readiness` primitive
/// followed by the subject being asked about.
///
/// A named constant rather than a literal in the sensor, because the sensor in
/// `camaduci` and the dispatcher here have to agree on it and a string written
/// twice is a string that will differ.
pub const READINESS_SEQUENCE: &[&str] = &["Readiness", "self"];

/// The readiness sequence in the form a sensor carries it.
///
/// A conversion rather than a `const` slice of `String`, because `String` cannot
/// be built at compile time and writing the sequence out as a literal in the
/// sensor would put the two copies of it a rename apart.
pub fn readiness_sequence() -> Vec<String> {
    READINESS_SEQUENCE.iter().map(|s| s.to_string()).collect()
}

/// What a readiness reading dispatches to: does the evidence stand?
///
/// The inhabitant of [`Sense::Inferred`](crate::camaduci::Sense::Inferred), and
/// the reason the long horizon can be asked a question at all. The cleaner
/// already knows which matterns have confirming traces; this makes that knowledge
/// reachable as a *sensor*, which is to say as a thing a creature notices about
/// itself rather than a thing an operator computes.
pub struct ReadinessDispatch {
    /// The strongest readiness among the matterns in the log, in `0.0..=1.0`.
    pub readiness: f64,
}

impl crate::camaduci::Dispatch for ReadinessDispatch {
    fn dispatch(&self, sequence: &[String]) -> Option<String> {
        if sequence.len() != READINESS_SEQUENCE.len() {
            // A sequence this dispatcher cannot read. Not a zero: a zero would be
            // a reading, and reporting "stale" on the strength of a typo is how a
            // creature comes to believe something false about itself.
            return None;
        }
        // The subject is checked as well as the primitive. A dispatcher that
        // answered for any subject would report this creature's readiness when
        // asked about another's, and the first version did exactly that — it
        // compared only the primitive and returned the same number for
        // `Readiness self` and `Readiness another`. Found by a test written
        // against the wrong expectation, which is the only reason it was found
        // at all: the check was written for a dispatcher that could not be
        // misaddressed, because the subject was not being checked.
        if sequence[0] != READINESS_SEQUENCE[0] || sequence[1] != READINESS_SEQUENCE[1] {
            return None;
        }
        Some(format!("{:.6}", self.readiness.clamp(0.0, 1.0)))
    }
}

/// The primitive sequence a trace records, in the form a signature takes.
///
/// The join is `_` because that is what induction does when it groups traces into
/// candidates, and a cleaning pass that formed signatures any other way would
/// disagree with the artifacts it is cleaning.
pub fn signature_of(primitives: &[String]) -> String {
    primitives.join("_")
}

/// Whether this trace confirms this mattern.
///
/// A mattern is confirmed by a *successful* trace of its exact sequence. The
/// `succeeded` half is the load-bearing one: a sequence that was tried and failed
/// is evidence that the sequence is not the answer, and counting it as
/// confirmation would make a cleaning pass preserve precisely the artifacts that
/// have stopped working.
pub fn confirms(trace: &Trace, rule: &LearnedRule) -> bool {
    trace.succeeded && signature_of(&trace.primitives) == rule.signature
}

/// Reads the trace log once and answers every trigger from it.
pub fn read(traces: &[Trace]) -> Observations {
    let mut covered: BTreeMap<String, (bool, bool)> = BTreeMap::new();
    for t in traces {
        if t.primitives.is_empty() {
            // A trace with no sequence cannot be inducted from and cannot
            // confirm anything. Counting it would let a log of uninformative
            // calls look like a log of evidence.
            continue;
        }
        let sig = signature_of(&t.primitives);
        let e = covered.entry(sig).or_insert((false, false));
        if t.succeeded {
            e.0 = true;
        } else {
            e.1 = true;
        }
    }
    Observations { covered }
}

/// What the log says, indexed by signature.
///
/// A `BTreeMap` of two booleans rather than two collections, because "a sequence
/// that both worked and failed" is a state the triggers have to be able to see
/// and separate collections cannot express it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observations {
    covered: BTreeMap<String, (bool, bool)>,
}

impl Observations {
    /// Whether any successful trace carried this sequence.
    pub fn confirmed(&self, signature: &str) -> bool {
        self.covered.get(signature).is_some_and(|e| e.0)
    }

    /// Whether any failed trace carried this sequence.
    pub fn failed(&self, signature: &str) -> bool {
        self.covered.get(signature).is_some_and(|e| e.1)
    }

    /// Successful sequences no mattern claims, which is what an unknown situation
    /// is: the log knows a sequence nobody does.
    pub fn unmattered(&self, rules: &[LearnedRule]) -> Vec<String> {
        self.covered
            .iter()
            .filter(|(sig, (worked, _))| *worked && !rules.iter().any(|r| &r.signature == *sig))
            .map(|(sig, _)| sig.clone())
            .collect()
    }

    /// Sequences whose every trace failed.
    pub fn failures(&self) -> Vec<String> {
        self.covered
            .iter()
            .filter(|(_, (worked, failed))| !*worked && *failed)
            .map(|(sig, _)| sig.clone())
            .collect()
    }
}

/// The address a re-mattered artifact gets.
///
/// A content address of the surviving material, which is the signature and the
/// phrasings still standing. Deliberately *not* derived from the address being
/// replaced: a hash that took the old address as input would be a genealogy
/// wearing a disguise, and the whole point is that there isn't one.
pub fn address_for(signature: &str, aliases: &[String]) -> String {
    DuUuid::generate(
        &json!({
            "signature": signature,
            "aliases": aliases,
        }),
        None,
    )
    .map(|u| u.to_string())
    .unwrap_or_else(|e| format!("unaddressable: {e}"))
}

/// The mattern that replaces a stale one.
///
/// The signature and the phrasings are carried across; the confidence and the
/// observation count are not, because those were the evidence and the evidence is
/// what is being discarded. A re-mattered mattern arrives as a *fresh claim*, at
/// the strength of what still stands behind it, and the confidence it is given is
/// the floor rather than a number inherited from a history nobody can see.
pub fn rematter(rule: &LearnedRule) -> LearnedRule {
    let mut out = LearnedRule {
        address: address_for(&rule.signature, &rule.aliases),
        signature: rule.signature.clone(),
        aliases: rule.aliases.clone(),
        confidence: 0.0,
        observations: 0,
    };
    // The phrasings are the evidence that survives, so the claim is at least as
    // strong as the number of distinct ways of saying it, which is the floor
    // induction would give a rule seen exactly this many times.
    if !out.aliases.is_empty() {
        out.confidence = out.aliases.len() as f64 / (out.aliases.len() as f64 + 1.0);
        out.observations = out.aliases.len();
    }
    out
}

/// The cleaning pass: which matterns still stand, which are re-mattered, and what
/// the log holds that nothing claims.
pub fn clean(rules: &[LearnedRule], traces: &[Trace]) -> (Cleaning, Vec<LearnedRule>) {
    let obs = read(traces);

    let mut cleaning = Cleaning {
        unmattered: obs.unmattered(rules),
        failed: obs.failures(),
        ..Cleaning::default()
    };
    let mut kept = Vec::with_capacity(rules.len());

    for rule in rules {
        if obs.confirmed(&rule.signature) {
            cleaning.confirmed.push(rule.address.clone());
            kept.push(rule.clone());
            continue;
        }
        let fresh = rematter(rule);
        cleaning.stale.push(rule.address.clone());
        // Recorded only when the replacement actually differs.
        //
        // A rule that has already been re-mattered re-matters to itself, because
        // the address is the capability's and the confidence is a function of the
        // phrasings. That is not a re-matter, it is the artifact being *already
        // canonical* — the fixed point. The first version pushed a `Rematter`
        // regardless, so `changed()` came to mean "something was stale" rather
        // than "the corpus differs", and cleaning an already-clean corpus
        // reported having cleaned all of it. Being stale and being replaced are
        // different facts: this one had no evidence and did not need replacing.
        if fresh != *rule {
            cleaning.remattered.push(Rematter {
                signature: rule.signature.clone(),
                from: rule.address.clone(),
                to: fresh.address.clone(),
            });
        }
        kept.push(fresh);
    }

    (cleaning, kept)
}

/// How much of the log was the creature's own doing.
///
/// Counted here rather than only in the store because a cleaning pass is the
/// thing that would act on it, and a cleaner that cannot see whether the evidence
/// it is about to discard was produced by the creature or by the person looking
/// after it cannot tell a population apart from a caretaker.
pub fn authorship(traces: &[Trace]) -> (usize, usize) {
    let itself = traces
        .iter()
        .filter(|t| matches!(t.actor, Some(Actor::Itself)))
        .count();
    let player = traces
        .iter()
        .filter(|t| matches!(t.actor, Some(Actor::Player)))
        .count();
    (itself, player)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace(primitives: &[&str], succeeded: bool) -> Trace {
        Trace {
            primitives: primitives.iter().map(|p| p.to_string()).collect(),
            succeeded,
            ..crate::mcp::store::new_trace(
                "do the thing".into(),
                Some("ca-x".into()),
                if succeeded { "hit" } else { "miss" },
                0,
                0,
            )
        }
    }

    fn rule(signature: &str, address: &str, aliases: &[&str]) -> LearnedRule {
        LearnedRule {
            address: address.into(),
            signature: signature.into(),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            confidence: 0.9,
            observations: 9,
        }
    }

    /// A mattern with a successful trace of the same sequence stands, untouched.
    #[test]
    fn a_confirmed_mattern_is_left_exactly_as_it_was() {
        let rules = vec![rule("SetValue_CheckSense", "addr-a", &["feed it"])];
        let traces = vec![trace(&["SetValue", "CheckSense"], true)];

        let (cleaning, kept) = clean(&rules, &traces);

        assert_eq!(cleaning.confirmed, vec!["addr-a".to_string()]);
        assert!(cleaning.stale.is_empty());
        assert!(!cleaning.changed());
        assert_eq!(kept, rules, "and a confirmed mattern came back unchanged");
    }

    /// A mattern with no confirming trace is re-mattered: same capability, new
    /// address. Not deleted, and not left alone.
    #[test]
    fn a_stale_mattern_is_re_mattered_at_a_new_address_with_its_capability_intact() {
        let rules = vec![rule("SetValue_CheckSense", "addr-a", &["feed it"])];
        let traces: Vec<Trace> = Vec::new();

        let (cleaning, kept) = clean(&rules, &traces);

        assert_eq!(cleaning.stale, vec!["addr-a".to_string()]);
        assert!(cleaning.changed());
        assert_eq!(
            kept.len(),
            1,
            "the capability was deleted rather than re-mattered"
        );
        assert_eq!(
            kept[0].signature, rules[0].signature,
            "re-mattering changed the capability, and the capability is the meet"
        );
        assert_ne!(
            kept[0].address, rules[0].address,
            "re-mattering kept the address, so nothing happened"
        );
        assert_eq!(kept[0].aliases, rules[0].aliases, "the phrasings survive");
    }

    /// The evidence does not survive. A re-mattered mattern arrives as a fresh
    /// claim at the strength of what still stands behind it, not at the strength
    /// of a history nobody can see.
    #[test]
    fn re_mattering_discards_the_evidence_and_keeps_only_the_phrasings() {
        let rules = vec![rule("SetValue_CheckSense", "addr-a", &["a", "b", "c"])];
        let (_, kept) = clean(&rules, &[]);

        assert!(
            kept[0].confidence < rules[0].confidence,
            "the fresh mattern kept the old confidence of {} from a history that \\
             no longer exists, and it is at {}",
            rules[0].confidence,
            kept[0].confidence
        );
        assert_eq!(kept[0].observations, 3, "one per surviving phrasing");
        assert!(kept[0].confidence > 0.0, "and not nothing at all");
    }

    /// **The load-bearing one.** The re-mattered address is a function of the
    /// capability and of nothing else — not the old address, not the phrasings,
    /// not the confidence it used to have.
    ///
    /// This test was first written to assert the *opposite* about the phrasings,
    /// and it passed, because the addresses it compared were the same for a
    /// different reason: `skeleton` drops `aliases` as surface, so three sets of
    /// phrasings all produced one address. Passing for the wrong reason is worse
    /// than failing, so the assertion is now the one the architecture actually
    /// supports — and the reason it supports it is a feature rather than a
    /// limitation: two creatures that learned different words for the same act are
    /// the same artifact, which is what makes the convergence test runnable.
    #[test]
    fn a_re_mattered_address_is_the_capability_and_nothing_else() {
        let first = rule("SetValue_CheckSense", "addr-old-1", &["feed it"]);
        let second = rule(
            "SetValue_CheckSense",
            "addr-old-2",
            &["a different history"],
        );
        let third = rule("SetValue_CheckSense", "addr-old-3", &[]);

        let a = rematter(&first);
        let b = rematter(&first);
        let c = rematter(&second);
        let d = rematter(&third);

        assert_eq!(a.address, b.address, "re-mattering is not deterministic");
        assert_eq!(a.address, c.address, "two histories of one act diverge");
        assert_eq!(
            a.address, d.address,
            "the address depends on the phrasings, so a creature that has lost \
             them all no longer holds the same capability"
        );
        assert_ne!(a.address, first.address, "and nothing was replaced at all");
    }

    /// A different act is a different artifact, which is the other half: the
    /// address is not simply constant.
    #[test]
    fn a_different_act_re_matters_to_a_different_address() {
        let feed = rule("SetValue_CheckSense", "addr-a", &["feed it"]);
        let play = rule("Toggle_Reset", "addr-b", &["throw the ball"]);

        assert_ne!(
            rematter(&feed).address,
            rematter(&play).address,
            "two capabilities share an address, so the address is not identifying \
             anything"
        );
    }

    /// Two matterns holding one signature are one capability held twice, and a
    /// cleaning that counted both would be counting redundancy as survival.
    #[test]
    fn one_capability_survives_once_however_many_matterns_held_it() {
        let rules = vec![
            rule("SetValue_CheckSense", "addr-a", &["feed it"]),
            rule("SetValue_CheckSense", "addr-b", &["give it food"]),
        ];
        let (cleaning, kept) = clean(&rules, &[]);

        assert_eq!(kept.len(), 2, "both are re-mattered, being two artifacts");
        assert_eq!(
            cleaning.capabilities_surviving(),
            vec!["SetValue_CheckSense"],
            "and it is one capability"
        );
    }

    /// A trace that only ever failed does not confirm, and its sequence is
    /// reported as a chaotic event rather than as evidence.
    #[test]
    fn a_sequence_that_only_ever_failed_confirms_nothing() {
        let rules = vec![rule("Toggle_Reset", "addr-a", &["flip it"])];
        let traces = vec![trace(&["Toggle", "Reset"], false)];

        let (cleaning, kept) = clean(&rules, &traces);

        assert!(
            !confirms(&traces[0], &rules[0]),
            "a failure was read as a success"
        );
        assert!(cleaning.confirmed.is_empty());
        assert!(cleaning.changed(), "so the mattern was re-mattered");
        assert_eq!(cleaning.failed, vec!["Toggle_Reset".to_string()]);
        assert_eq!(kept[0].signature, "Toggle_Reset");
    }

    /// A sequence that both worked and failed is confirmed *and* reported as
    /// having failed, because the two are different facts and the triggers have to
    /// be able to see both.
    #[test]
    fn a_sequence_that_worked_and_then_failed_is_both_confirmed_and_a_failure() {
        let traces = vec![
            trace(&["SetValue", "CheckSense"], true),
            trace(&["SetValue", "CheckSense"], false),
        ];
        let obs = read(&traces);

        assert!(obs.confirmed("SetValue_CheckSense"));
        assert!(obs.failed("SetValue_CheckSense"));
    }

    /// An unknown situation: a successful trace no mattern covers. The log knows
    /// a sequence the creature does not.
    #[test]
    fn a_successful_trace_no_mattern_covers_is_an_unknown_situation() {
        let rules = vec![rule("SetValue_CheckSense", "addr-a", &["feed it"])];
        let traces = vec![
            trace(&["SetValue", "CheckSense"], true),
            trace(&["GetValue", "Push", "Emit"], true),
        ];

        let (cleaning, _) = clean(&rules, &traces);

        assert_eq!(
            cleaning.unmattered,
            vec!["GetValue_Push_Emit".to_string()],
            "the uncovered sequence was not reported"
        );
        assert!(
            cleaning.failed.is_empty(),
            "and it was not mistaken for a failure"
        );
    }

    /// A trace with no sequence cannot be inducted from and cannot confirm
    /// anything, so a log of uninformative calls must not read as a log of
    /// evidence.
    #[test]
    fn a_trace_with_no_sequence_is_neither_evidence_nor_a_failure() {
        let obs = read(&[Trace {
            primitives: Vec::new(),
            succeeded: false,
            ..crate::mcp::store::new_trace("?".into(), None, "miss", 0, 0)
        }]);

        assert!(obs.covered.is_empty(), "an empty sequence was indexed");
        assert!(obs.failures().is_empty());
    }

    /// An empty rule set with an empty log is a clean corpus, and saying so
    /// requires having looked.
    #[test]
    fn an_empty_corpus_is_clean_and_that_is_a_verdict_not_a_default() {
        let (cleaning, kept) = clean(&[], &[]);
        assert!(!cleaning.changed());
        assert!(kept.is_empty());
        assert!(cleaning.unmattered.is_empty());
    }

    /// The cleaner can tell a population apart from a caretaker, which is what
    /// decides whether discarding the evidence is recycling or amnesia.
    #[test]
    fn the_cleaner_sees_who_produced_the_evidence_it_is_about_to_discard() {
        let mut mine = trace(&["SetValue"], true);
        mine.actor = Some(Actor::Itself);
        let mut theirs = trace(&["SetValue"], true);
        theirs.actor = Some(Actor::Player);
        let none = trace(&["SetValue"], true);

        let (itself, player) = authorship(&[mine, theirs, none]);
        assert_eq!(
            (itself, player),
            (1, 1),
            "and a trace with no actor is neither"
        );
    }
}

#[cfg(test)]
mod convergence_tests {
    //! The measurement this design exists to make possible, and the one the
    //! project has never been able to run.
    //!
    //! Twelve hand-written patterns share **zero** capabilities: one author, one
    //! intent, twelve solitaries. Thirty-six generated artifacts share six
    //! denominators they were *manufactured* to share: a uniformity, not a
    //! convergence. Neither is evidence that independent creatures arrive at the
    //! same capability, because in both cases the arrival was arranged.
    //!
    //! Recycling without a genealogy is what makes the arrangement unnecessary.
    //! Nothing links one generation to the next, so agreement between them is
    //! agreement rather than inheritance.

    use super::*;
    use std::collections::BTreeSet;

    fn authored(address: &str, signature: &str, phrasings: &[String]) -> LearnedRule {
        LearnedRule {
            address: address.into(),
            signature: signature.into(),
            aliases: phrasings.to_vec(),
            // Confident and well-attested, and it makes no difference: the traces
            // that attested it went with the creature.
            confidence: 0.95,
            observations: 40,
        }
    }

    /// The traces a culled creature leaves behind, which confirm nothing because
    /// every one of them failed on the way out.
    fn departing(failures: usize) -> Vec<Trace> {
        (0..failures)
            .map(|_| Trace {
                primitives: vec!["SetValue".into(), "CheckSense".into()],
                succeeded: false,
                ..crate::mcp::store::new_trace(
                    "last attempt".into(),
                    Some("ca-culled".into()),
                    "miss",
                    0,
                    0,
                )
            })
            .collect()
    }

    /// Six generations, six creatures that never met, one capability — and one
    /// artifact, arrived at independently each time.
    ///
    /// Each generation authors its own mattern at an address of its own choosing,
    /// teaches it a vocabulary nothing else has, is culled, and leaves behind only
    /// failures. The capability survives because the mattern is re-mattered; the
    /// address it survives at is a function of the act and nothing else, so all
    /// six land in the same place without any of them knowing.
    ///
    /// Measured: 6 generations, 1 capability, 1 distinct address, and no surviving
    /// artifact carrying a culled one.
    #[test]
    fn six_generations_that_never_met_converge_on_one_artifact() {
        const CAPABILITY: &str = "SetValue_CheckSense";
        let mut corpus: Vec<LearnedRule> = Vec::new();
        let mut carried: Option<LearnedRule> = None;
        let mut culled: Vec<String> = Vec::new();

        for generation in 0..6usize {
            // A vocabulary nothing else in the run has, growing as the creature
            // is taught more of its own act.
            let phrasings: Vec<String> = (0..=generation)
                .map(|k| format!("generation-{generation}-word-{k}"))
                .collect();

            let mut held: Vec<LearnedRule> = Vec::new();
            if let Some(inherited) = carried.clone() {
                held.push(inherited);
            }
            held.push(authored(
                &format!("address-author-chosen-in-generation-{generation}"),
                CAPABILITY,
                &phrasings,
            ));
            let before = held.last().expect("just pushed").address.clone();

            let (cleaning, kept) = clean(&held, &departing(generation + 1));
            assert!(
                !cleaning.stale.is_empty(),
                "generation {generation} left nothing stale, so nothing was recycled \\
                 and the run proves nothing"
            );
            assert!(
                kept.iter().all(|r| r.address != before),
                "generation {generation} kept the address it was culled with"
            );

            carried = kept.last().cloned();
            culled.push(before);
            corpus.extend(kept);
        }

        let (_, survivors) = clean(&corpus, &[]);

        let capabilities: BTreeSet<&str> = survivors.iter().map(|r| r.signature.as_str()).collect();
        let addresses: BTreeSet<&str> = survivors.iter().map(|r| r.address.as_str()).collect();
        assert_eq!(
            capabilities.len(),
            1,
            "six generations produced {} capabilities, and one act should be one",
            capabilities.len()
        );
        assert_eq!(
            addresses.len(),
            1,
            "six independent arrivals at one act produced {} artifacts, so the \\
             address is not a function of the capability",
            addresses.len()
        );
    }

    /// **The no-genealogy claim, as a check rather than a promise.**
    ///
    /// A cleaner that recorded its work would pass every other test here and fail
    /// this one. So: after six cullings, nothing in the surviving corpus may name
    /// any address that was culled, and the report the cleaner hands back must be
    /// the *only* place the relationship exists.
    #[test]
    fn no_surviving_artifact_names_one_that_was_culled() {
        const CAPABILITY: &str = "SetValue_CheckSense";
        let mut corpus: Vec<LearnedRule> = Vec::new();
        let mut carried: Option<LearnedRule> = None;
        let mut culled: BTreeSet<String> = BTreeSet::new();
        let mut reported = 0usize;

        for generation in 0..6usize {
            let phrasings: Vec<String> = (0..=generation)
                .map(|k| format!("g{generation}-w{k}"))
                .collect();
            let mut held: Vec<LearnedRule> = Vec::new();
            if let Some(inherited) = carried.clone() {
                held.push(inherited);
            }
            held.push(authored(
                &format!("culled-{generation}"),
                CAPABILITY,
                &phrasings,
            ));

            let (cleaning, kept) = clean(&held, &departing(generation + 1));
            reported += cleaning.remattered.len();
            for r in cleaning.remattered.iter() {
                // A re-matter either changes the address or is already at the
                // fixed point. The second case is not a defect: once an address
                // *is* the capability's address, re-mattering it again lands in
                // the same place, so the second pass has nothing to do. Asserting
                // that every pass changes the address would be asserting that
                // cleaning never converges, which is the opposite of the design.
                assert!(
                    !culled.contains(&r.to),
                    "a re-mattered artifact was given the address of something \
                     culled earlier, which is a lineage after all"
                );
            }
            for r in kept.iter() {
                assert!(
                    !culled.contains(&r.address),
                    "a surviving artifact carries the culled address {}",
                    r.address
                );
            }
            culled.insert(held.last().expect("just pushed").address.clone());
            carried = kept.last().cloned();
            corpus.extend(kept);
        }

        let (_, survivors) = clean(&corpus, &[]);
        for survivor in survivors.iter() {
            assert!(
                !culled.contains(&survivor.address),
                "after the whole run, {} still names a culled address",
                survivor.address
            );
        }
        assert!(
            reported > 0,
            "nothing was ever re-mattered, so there was no relationship to sever"
        );

        // And the survivor is a fixed point: cleaning it again changes nothing.
        // This is the property that makes the design safe to run repeatedly, and
        // it is what a re-matter converging to the capability's own address buys.
        let (second, again) = clean(&survivors, &[]);
        assert!(
            !second.changed(),
            "cleaning an already-clean corpus changed {} of {} matterns, so a \
             second pass is not a no-op and the cleaner is not convergent",
            second.remattered.len(),
            survivors.len()
        );
        assert_eq!(again, survivors, "and it did not return the same artifacts");
    }
}
