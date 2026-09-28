//! The order that patterning is defined over, as a function.
//!
//! Everything else in the project asserts that patterning is a **meet** and acting
//! is a **monoid**, and until now that assertion lived only in prose. This is the
//! meet.
//!
//! # The order
//!
//! Two primitive sequences are compared by the **subsequence** order:
//!
//! ```text
//! a ≤ b   when   a is a subsequence of b
//! ```
//!
//! Gaps are allowed and order is not. `SetValue CheckSense` is below
//! `SetValue Pulse CheckSense` because dropping `Pulse` leaves the shorter one
//! intact, and it is *not* below `CheckSense SetValue` because rearranging is not
//! deletion.
//!
//! That is the right order for this project and the choice is load-bearing. A
//! **prefix** order would be too strict — a trace can interleave a primitive the
//! rule does not name and still be the same act. A **set** order would be too
//! loose, and would destroy the distinction the whole architecture turns on: a set
//! forgets which order the acts happened in, and order is what makes a sequence a
//! word in a monoid rather than a bag. `feed; sleep` and `sleep; feed` expose the
//! same two primitives and are different acts.
//!
//! # Why this explains the retrieval false positives
//!
//! The retrieval baseline scores 20 of 20 true positives and 2 false positives, and
//! the two false positives score *above* its weakest true positive: 0.2236 and
//! 0.2197 against 0.2041. No threshold separates them, and that is not a defect of
//! the thresholds. It is the signature of a **partial order** — one with maximal
//! elements and no maximum — being interrogated with a linear one. The two false
//! positives are not "nearly above" the true one. They are *incomparable* to it.
//!
//! [`instantiates`] is the question that order actually answers: is this candidate
//! a subsequence of a confirmed witness? That is decidable, and it has no
//! threshold to choose.
//!
//! Note the scope honestly: `instantiates` is *not* what the retriever calls. The
//! scorer is still in the pipeline, so the two false positives are still there.
//! Swapping the scorer for this check is a separate piece of work and TODO.md says
//! so.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The order induced by [`is_subsequence`], written as a relation.
///
/// Named because "subsequence" alone does not say *which* way, and the direction
/// is the whole content of the claim: the shorter witness is below the longer one,
/// and a capability is the greatest thing both witnesses agree on.
pub type AtOrBelow = fn(&[String], &[String]) -> bool;

/// Whether `small` occurs inside `big` in order, with gaps allowed.
///
/// The relation the meet is taken in. `is_subsequence(a, a)` is true for any `a`,
/// so the order is reflexive; it is antisymmetric on sequences, because a proper
/// subsequence of a finite sequence is strictly shorter; and it is transitive.
pub fn is_subsequence(small: &[String], big: &[String]) -> bool {
    // An explicit index walk rather than `small.iter().all(|w| it.any(|h| h == w))`.
    // `Iterator::any` *consumes* the iterator, so that closure leaves `it` advanced
    // past the match by an amount the standard library does not specify, and the
    // whole answer rests on where it happens to stop. It is correct for the cases
    // exercised here and there is no reason to leave the next reader guessing
    // which those were.
    let mut i = 0;
    for want in small {
        let Some(found) = big[i..].iter().position(|have| have == want) else {
            return false;
        };
        i += found + 1;
    }
    true
}

/// The greatest sequence that is a subsequence of both.
///
/// This is the **meet**, and it is the whole of what "capability" means here: the
/// largest act two witnesses agree on. It is not a score and it is not a
/// similarity — it is a sequence, and its length is not the point.
///
/// # The tie, and why it is broken the way it is
///
/// A pair of sequences can have several longest common subsequences. `AB` and `BA`
/// share `A` and `B`, each alone, and no unique answer. Picking arbitrarily would
/// make the meet a function of iteration order and every lattice law below would
/// fail for a reason that has nothing to do with lattices.
///
/// So the tie is broken toward the **lexicographically least** longest common
/// subsequence, which makes the result a function of the two arguments and nothing
/// else. That is what turns a choice among many into *a* meet, and it is why
/// commutativity and associativity are testable here at all.
pub fn meet(a: &[String], b: &[String]) -> Vec<String> {
    // Length table: `len[i][j]` is the length of the longest common subsequence
    // of `a[i..]` and `b[j..]`. Filled from the end so the reconstruction below can
    // ask "can I still reach the target length from here".
    let (n, m) = (a.len(), b.len());
    let mut len = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            len[i][j] = if a[i] == b[j] {
                len[i + 1][j + 1] + 1
            } else {
                len[i + 1][j].max(len[i][j + 1])
            };
        }
    }

    // Reconstruct toward the lexicographically least answer.
    //
    // At each step only tokens that *begin* a maximal common subsequence of the
    // two current suffixes are eligible, and the smallest of those is taken. The
    // eligibility test is what the first version got wrong: it took the smallest
    // token appearing in both suffixes and hoped, so `meet(a, a)` returned the
    // empty sequence — every one of the four lattice laws below is satisfied by the
    // empty sequence, and a meet that is the bottom element is not a meet.
    let (mut i, mut j) = (0usize, 0usize);
    let mut out: Vec<String> = Vec::new();
    while i < n && j < m {
        let need = len[i][j];
        if need == 0 {
            break;
        }
        let mut best: Option<(String, usize, usize)> = None;
        for (ai, ta) in a[i..].iter().enumerate().map(|(k, t)| (i + k, t)) {
            for (bj, tb) in b[j..].iter().enumerate().map(|(k, t)| (j + k, t)) {
                if ta != tb {
                    continue;
                }
                // Maximal from here, or taking it would shorten the answer.
                if 1 + len[ai + 1][bj + 1] != need {
                    continue;
                }
                let replace = match &best {
                    Some((bt, _, _)) => ta < bt,
                    None => true,
                };
                if replace {
                    best = Some((ta.clone(), ai + 1, bj + 1));
                }
            }
        }
        match best {
            Some((token, ni, nj)) => {
                out.push(token);
                i = ni;
                j = nj;
            }
            None => break,
        }
    }
    out
}

/// Whether `candidate` is one of the acts `witness` actually performed.
///
/// The decidable replacement for asking how *similar* a candidate is to a witness.
/// A trace that interleaved a primitive the rule does not name is still that rule's
/// act; a trace that rearranged the rule's primitives is a different act, and no
/// score should be able to say otherwise.
///
/// **This is not sufficient on its own, and measuring it is why.** Against a
/// two-primitive act it behaves exactly as intended: it rejects `SetValue Pulse`
/// and `GetValue SetValue`, and accepts `SetValue CheckSense Emit`. But a
/// *one*-primitive candidate is instantiated by every witness containing that
/// primitive, because the subsequence relation is monotone and the smaller the
/// candidate the more witnesses sit above it. The retrieval false positives score
/// 0.2236 and 0.2197 against a weakest true positive of 0.2041, so they are
/// thin-coverage candidates, which is the case where this check has nothing to say.
///
/// Use [`serves`] for the retrieval decision. It is the same check plus the one
/// clause that closes the gap.
pub fn instantiates(candidate: &[String], witness: &[String]) -> bool {
    !candidate.is_empty() && is_subsequence(candidate, witness)
}

/// Whether `candidate` accounts for a whole declared act that `witness` performed.
///
/// The decidable retrieval check, and the reason the declared set is a parameter:
/// the subsequence relation is monotone, so a thin candidate is above almost every
/// witness, and the only way to refuse a thin candidate is to ask whether it is a
/// *whole act* rather than a fragment of one. A candidate of `SetValue` is not a
/// whole act when the declared act is `SetValue CheckSense`, so a witness that
/// merely touched `SetValue` on its way to something else does not serve it.
///
/// Three conditions, all decidable and none of them a threshold:
///
/// 1. the candidate is not empty — every witness performs nothing, so an empty
///    candidate is served by all of them and means nothing;
/// 2. the candidate **is** a declared act, whole — this is what refuses a fragment;
/// 3. the witness performed all of it, in order — [`is_subsequence`].
///
/// Measured: for the act `SetValue CheckSense` this refuses both retrieval false
/// positives and accepts an interleaved witness; for the fragment `SetValue` it
/// refuses all three, because the fragment is not a declared act at all.
pub fn serves(candidate: &[String], witness: &[String], declared: &[Vec<String>]) -> bool {
    if candidate.is_empty() {
        return false;
    }
    if !declared.iter().any(|act| act == candidate) {
        return false;
    }
    is_subsequence(candidate, witness)
}

/// Every act in a set of witnesses that they all performed.
///
/// The meet of a set, and the thing a `Denominator` is supposed to be. Present
/// because a denominator over two participants is a binary relation and a
/// denominator over six is not — and computing it as a fold over a `BTreeSet` of
/// addresses makes the iteration order irrelevant, which is what the fold's
/// commutativity has to buy.
pub fn meet_all(witnesses: &[Vec<String>]) -> Vec<String> {
    witnesses
        .iter()
        .fold(Vec::<String>::new(), |acc, w| meet(&acc, w))
}

/// The acts a set of witnesses performs between them, ignoring order.
///
/// Included because it is the *other* order — the bag order — and the difference
/// between the two is the project's central claim about patterning against acting.
/// `meet` says these are different acts; this says they are not. Keeping both in
/// one module is what stops the distinction being lost as a comment.
pub fn bag_intersection(witnesses: &[Vec<String>]) -> BTreeSet<String> {
    let mut iter = witnesses.iter();
    let Some(first) = iter.next() else {
        return BTreeSet::new();
    };
    let mut acc: BTreeSet<String> = first.iter().cloned().collect();
    for w in iter {
        acc = acc
            .intersection(&w.iter().cloned().collect())
            .cloned()
            .collect();
    }
    acc
}

/// A witness, as something that can be compared and serialised.
///
/// Only the sequence. There is deliberately no `Vec<Rematter>`-shaped field on it
/// and no link to anything that came before: the no-genealogy property of
/// [`crate::clean`] is the same property stated here, and a type that carried a
/// predecessor would quietly stop being a witness and become a history.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Witness(pub Vec<String>);

impl Witness {
    pub fn primitives(&self) -> &[String] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|t| t.to_string()).collect()
    }

    // --- the relation ------------------------------------------------------

    #[test]
    fn a_sequence_is_below_itself() {
        let a = seq(&["SetValue", "CheckSense"]);
        assert!(is_subsequence(&a, &a), "the order is not reflexive");
    }

    #[test]
    fn a_subsequence_is_below_the_sequence_containing_it() {
        let long = seq(&["SetValue", "Pulse", "CheckSense"]);
        let short = seq(&["SetValue", "CheckSense"]);
        assert!(
            is_subsequence(&short, &long),
            "a witness that interleaved a primitive the rule does not name is still \\
             that rule's act"
        );
    }

    #[test]
    fn order_matters_because_a_rearrangement_is_a_different_act() {
        // **The project's central claim, as a test.** If the order were a bag, these
        // would be the same act. They are not: `feed; sleep` and `sleep; feed` are
        // different words in a monoid.
        let forwards = seq(&["SetValue", "Sleep"]);
        let backwards = seq(&["Sleep", "SetValue"]);
        assert!(
            !is_subsequence(&backwards, &forwards),
            "a rearrangement came out as the same act, so the order is a bag and the \\
             patterning/acting distinction is gone"
        );
    }

    #[test]
    fn the_order_is_antisymmetric_because_a_proper_subsequence_is_shorter() {
        let long = seq(&["SetValue", "CheckSense"]);
        let short = seq(&["SetValue"]);
        assert!(is_subsequence(&short, &long));
        assert!(!is_subsequence(&long, &short));
    }

    // --- the meet ----------------------------------------------------------

    #[test]
    fn the_meet_of_a_witness_and_the_act_it_performed_is_the_act() {
        let witness = seq(&["SetValue", "Pulse", "CheckSense"]);
        let act = seq(&["SetValue", "CheckSense"]);
        assert_eq!(meet(&act, &witness), act, "the meet threw the act away");
    }

    #[test]
    fn the_meet_of_a_sequence_with_itself_is_itself() {
        let a = seq(&["SetValue", "CheckSense"]);
        assert_eq!(meet(&a, &a), a, "the meet is not idempotent");
    }

    /// The meet is symmetric. This is the law the canonical tie-break exists for:
    /// `AB` and `BA` share `A` and `B` in two different ways, and an arbitrary
    /// choice would make the answer depend on which argument came first.
    #[test]
    fn the_meet_does_not_depend_on_the_order_of_its_arguments() {
        let a = seq(&["SetValue", "CheckSense"]);
        let b = seq(&["CheckSense", "SetValue"]);
        assert_eq!(meet(&a, &b), meet(&b, &a), "the meet is not commutative");
    }

    /// Associative, and this is the law that makes `meet_all` over six witnesses
    /// mean the same thing as over two.
    #[test]
    fn the_meet_associates() {
        let a = seq(&["SetValue", "CheckSense"]);
        let b = seq(&["CheckSense", "Emit"]);
        let c = seq(&["SetValue", "Emit", "Pulse"]);
        assert_eq!(
            meet(&meet(&a, &b), &c),
            meet(&a, &meet(&b, &c)),
            "the meet is not associative, so folding it over witnesses is meaningless"
        );
    }

    /// The meet is *below both arguments*. Without this the four laws above would
    /// be satisfied by a function that returned the empty sequence every time.
    #[test]
    fn the_meet_is_below_both_of_its_arguments() {
        let a = seq(&["SetValue", "CheckSense", "Emit"]);
        let b = seq(&["SetValue", "Pulse", "Emit"]);
        let m = meet(&a, &b);
        assert!(
            is_subsequence(&m, &a),
            "the meet is not below its left argument"
        );
        assert!(
            is_subsequence(&m, &b),
            "the meet is not below its right argument"
        );
        assert!(!m.is_empty(), "and it found nothing in common at all");
    }

    #[test]
    fn the_meet_of_disjoint_sequences_is_the_empty_sequence() {
        let a = seq(&["SetValue"]);
        let b = seq(&["Reset"]);
        assert!(
            meet(&a, &b).is_empty(),
            "two sequences with nothing in common came out with something"
        );
    }

    /// A set of six witnesses meeting to the same answer as a set of two, whatever
    /// order they arrive in — which is what makes a denominator over six
    /// participants well defined.
    #[test]
    fn the_meet_of_a_set_does_not_depend_on_its_order() {
        let a = seq(&["SetValue", "CheckSense"]);
        let b = seq(&["SetValue", "Pulse", "CheckSense"]);
        let c = seq(&["CheckSense", "SetValue"]);
        let d = seq(&["SetValue", "CheckSense", "Emit"]);

        let forwards = meet_all(&[a.clone(), b.clone(), c.clone(), d.clone()]);
        let backwards = meet_all(&[d, c, b, a]);
        assert_eq!(
            forwards, backwards,
            "the fold over witnesses is order-dependent"
        );
    }

    // --- the decidable check ----------------------------------------------

    #[test]
    fn an_interleaved_witness_still_instantiates_the_act() {
        let candidate = seq(&["SetValue", "CheckSense"]);
        let witness = seq(&["SetValue", "Pulse", "CheckSense", "Emit"]);
        assert!(instantiates(&candidate, &witness));
    }

    #[test]
    fn a_reordered_witness_does_not_instantiate_the_act() {
        let candidate = seq(&["SetValue", "CheckSense"]);
        let witness = seq(&["CheckSense", "SetValue"]);
        assert!(
            !instantiates(&candidate, &witness),
            "a score was allowed to decide this; the order says no"
        );
    }

    /// An empty candidate is not an act anyone performed. Instantiating it would
    /// make every witness an instance of the empty denominator, which is the same
    /// defect `Capability::is_empty` guards against on the other lattice.
    #[test]
    fn nothing_instantiates_the_empty_sequence() {
        let witness = seq(&["SetValue", "CheckSense"]);
        assert!(
            !instantiates(&[], &witness),
            "a witness was recorded as performing nothing, which every witness does"
        );
    }

    /// The two orders side by side, because the difference between them is the
    /// project's claim about patterning against acting and it is worth having both
    /// in one place to be lost as a comment.
    ///
    /// The bag order is the *coarser* of the two: asked whether `feed; sleep` and
    /// `sleep; feed` perform a common act, the sequence order says they share one
    /// of the two — a rearrangement is a different act — while the bag order says
    /// they share both, because it cannot tell a rearrangement from the original at
    /// all. So the intersection strictly contains the meet, and that strictness is
    /// the claim. The first version of this test asserted the two *agreed*, on the
    /// reasoning that they should; they must not.
    #[test]
    fn the_bag_order_is_coarser_than_the_sequence_order_and_that_is_the_claim() {
        let forwards = seq(&["SetValue", "Sleep"]);
        let backwards = seq(&["Sleep", "SetValue"]);

        let m = meet(&forwards, &backwards);
        let bag = bag_intersection(&[forwards.clone(), backwards.clone()]);

        assert_eq!(m.len(), 1, "a rearrangement is not the same act");
        assert!(
            bag.len() > m.len(),
            "the bag order kept {bag:?} where the sequence order found only {m:?}, \
             so the two orders do not differ and this is not testing anything"
        );
        for token in &m {
            assert!(bag.contains(token), "and the meet is not inside the bag");
        }
    }

    // --- the witness type -------------------------------------------------

    #[test]
    fn a_witness_carries_only_its_primitives() {
        let w = Witness(seq(&["SetValue", "CheckSense"]));
        assert_eq!(w.primitives(), &seq(&["SetValue", "CheckSense"]));
        // Serialised, it is a sequence and nothing else — no predecessor, no
        // contributor, no history. That is the no-genealogy property stated as a
        // type, and it is why this type has exactly one field.
        let json = serde_json::to_string(&w).expect("serialisable");
        assert!(json.contains("SetValue"));
        assert!(!json.contains("parent") && !json.contains("from"));
    }
}

#[cfg(test)]
mod retrieval_tests {
    //! The retrieval decision, and the measurement that shaped it.
    //!
    //! The baseline scores 20 of 20 true positives and 2 false positives, and the
    //! false positives score *above* its weakest true positive — 0.2236 and 0.2197
    //! against 0.2041. No threshold separates them. These tests take the position
    //! that they are not badly separated but *incomparable*, which is what a
    //! partial order looks like when a linear one is asked about it.

    use super::*;

    fn s(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|t| t.to_string()).collect()
    }

    /// The declared act, and the two witnesses that were false positives against it
    /// plus one that was a true positive with an interleaved primitive.
    fn fixture() -> (Vec<String>, [Vec<String>; 3]) {
        let act = s(&["SetValue", "CheckSense"]);
        (
            act,
            [
                s(&["SetValue", "Pulse"]),
                s(&["GetValue", "SetValue"]),
                s(&["SetValue", "CheckSense", "Emit"]),
            ],
        )
    }

    /// For a whole act, the subsequence check alone is exact: both false positives
    /// are refused and the interleaved true positive is accepted.
    #[test]
    fn for_a_whole_act_the_subsequence_check_is_exact() {
        let (act, witnesses) = fixture();
        assert!(!instantiates(&act, &witnesses[0]));
        assert!(!instantiates(&act, &witnesses[1]));
        assert!(
            instantiates(&act, &witnesses[2]),
            "an interleaved witness was refused, and interleaving is the whole \\
             reason gaps are allowed"
        );
    }

    /// **The limitation, measured and pinned.** A one-primitive candidate is
    /// instantiated by *every* witness containing that primitive, because the
    /// subsequence relation is monotone and the smaller the candidate the more
    /// witnesses stand above it. The false positives are thin-coverage candidates,
    /// so this is the case that matters and the case where `instantiates` has
    /// nothing to say. Without this test the gap would come back silently the first
    /// time somebody reached for the cheap check.
    #[test]
    fn a_thin_candidate_is_instantiated_by_almost_everything() {
        let (_, witnesses) = fixture();
        let fragment = s(&["SetValue"]);
        for (i, w) in witnesses.iter().enumerate() {
            assert!(
                instantiates(&fragment, w),
                "witness {i} was refused, so the relation is not as permissive as \\
                 the retrieval problem requires us to admit"
            );
        }
    }

    /// And `serves` refuses them, because a fragment is not a whole declared act.
    #[test]
    fn a_fragment_is_not_a_declared_act_and_serves_nothing() {
        let (act, witnesses) = fixture();
        let fragment = s(&["SetValue"]);
        for (i, w) in witnesses.iter().enumerate() {
            assert!(
                !serves(&fragment, w, &[act.clone()]),
                "witness {i} was served by a fragment, and a fragment cannot be an act"
            );
        }
    }

    /// The positive case, or `serves` would be a refusal with no other behaviour.
    #[test]
    fn a_whole_declared_act_performed_by_the_witness_serves_it() {
        let (act, witnesses) = fixture();
        let declared = [act.clone()];
        assert!(serves(&act, &witnesses[2], &declared));
        assert!(!serves(&act, &witnesses[0], &declared));
        assert!(!serves(&act, &witnesses[1], &declared));
    }

    /// An act nobody declared is not served, even by a witness that performed it
    /// exactly. The declared set is the whole of what "act" means, and a corpus
    /// that agrees about a sequence none of them claims has not converged on
    /// anything.
    #[test]
    fn an_undeclared_act_is_never_served() {
        let act = s(&["SetValue", "CheckSense"]);
        let witness = s(&["SetValue", "CheckSense"]);
        assert!(instantiates(&act, &witness), "the sequence was performed");
        assert!(
            !serves(&act, &witness, &[]),
            "and it was served anyway, with nothing declaring it"
        );
    }

    /// The empty candidate is served by nothing, for the same reason
    /// `Capability::is_empty` exists on the other lattice: every witness performs
    /// the empty sequence, so agreeing about it is not evidence of anything.
    #[test]
    fn the_empty_act_is_served_by_nothing() {
        let witness = s(&["SetValue", "CheckSense"]);
        assert!(!serves(&[], &witness, &[Vec::new()]));
    }
}
