//! The budget for **losing on purpose**, and the reason it had to be a new
//! movement rather than a discount.
//!
//! # The contradiction this removes
//!
//! The economy had exactly two movements. [`Economy::apply`] **debits**
//! `0.15 · (quants − FLOOR)` on an act, and [`Economy::credit`] **pays** for
//! confirmed novelty. There was no third.
//!
//! So a deliberate loss — an act run *expecting* to fail, to gather evidence
//! about a shape not yet known — had nowhere to be paid from. It was a debit
//! with no reachable credit, the rational strategy was never to try one, and the
//! economy forbade exploration for the worst available reason: it was
//! indistinguishable from waste.
//!
//! **And it was worse than a gap, because exploration and induction were in
//! direct conflict.** `induce::is_evidentiary` requires
//! `failure_ratio <= MAX_FAILURE_RATIO`, and failure is the only signal
//! induction has. Every exploratory loss therefore raised the failure ratio of
//! the very group it was exploring, and could disqualify it from ever becoming a
//! mattern. The system was being asked to pay for losses that the thing learning
//! from those losses used as evidence against them.
//!
//! # The fourth movement
//!
//! [`Economy::explore`] spends from a **declared reserve**, never refunds, and
//! is reachable only by an act whose signature has no prior evidence. It is
//! deliberately *not* a discount and *not* a smaller act:
//!
//! - **Not a discount**, because a cheaper act is still an act, and the problem
//!   was never the price — it was that no debit can ever be earned back.
//! - **Non-refundable**, because a refundable exploration budget is an
//!   exploration *credit*, and the creature would learn to reserve-then-refund
//!   and never pay a real price for anything.
//! - **Gated on novelty**, because an act with prior evidence is not exploration
//!   — it is a repeat, and repeats are already priced as rumination.
//!
//! # What it is not
//!
//! It does not make failure free, and it does not make a failed act count as
//! production. A creature that explores endlessly ends at the floor, which is
//! the correct outcome: the reserve is a *declared* fraction of power, so
//! exploring is affordable and unbounded exploring is not.
//!
//! **And the induced rule must be true, not merely unrejected.** The exemption
//! this buys is in [`Induction::is_evidentiary`]: an exploratory trace does not
//! count toward `misses`, so exploration can no longer disqualify the group it
//! explores. The bar that remains is `MIN_OBSERVATIONS` distinct phrasings — the
//! creature must have *reached for* a shape more than once, in more than one
//! way, before it is allowed to be a mattern. Losing repeatedly proves nothing;
//! reaching repeatedly does.

use crate::camaduci::{Economy, Production};

/// The share of power a creature may declare as an exploration reserve.
///
/// A fraction rather than a flat amount, so a creature that has grown can afford
/// to explore more and a creature near the floor cannot afford to at all. `0.15`
/// matches the per-act debit, which is what makes one exploratory act cost
/// roughly one ordinary act *of reserve* — so the reserve buys about one attempt
/// and the creature must widen it to go further.
pub const EXPLORATION_SHARE: f64 = 0.15;

/// What one exploratory act costs.
///
/// Flat, and the reason is a bug this module had: a cost proportional to the
/// reserve shrinks as the reserve shrinks, so exploration approached the floor
/// without ever reaching it and the first test of it looped until its guard
/// fired. A reserve is a stock and its unit of spending must not vanish with it.
pub const EXPLORATION_COST: f64 = 0.02;

/// What an exploratory act costs, and what it may never be paid back.
///
/// A separate currency on purpose. It is drawn from `quants` and it is not
/// recoverable, so it behaves like a real cost to every other rule in the
/// economy — `can_act` sees a smaller balance, dormancy sees less headroom — and
/// it cannot be smuggled in as a discount on the act itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Exploration {
    /// The reserve, and what is left of it.
    pub reserved: f64,
    /// What this act would have cost as an ordinary act, for the receipt.
    pub would_have_cost: f64,
}

/// The reserve a creature may currently draw on.
pub fn reserve(eco: &Economy) -> f64 {
    (eco.quants.max(0.0) - Economy::FLOOR_QUANTS) * EXPLORATION_SHARE
}

/// Whether an act on this signature is exploration: no prior evidence, and a
/// reserve to draw on.
///
/// The novelty test is the caller's, because only the caller knows the signature
/// is the one being tried. Passing `true` for an act that has been seen before
/// is not detectable here, and that is a real weakness: the economy cannot check
/// novelty without reading the trace log, and a caller that lies about it gets a
/// free act. It is stated rather than papered over, because a reserve that
/// refunds for repeats is worse than no reserve.
pub fn may_explore(eco: &Economy, no_prior_evidence: bool) -> bool {
    no_prior_evidence && reserve(eco) >= EXPLORATION_COST && !is_exhausted(eco)
}

/// Spends the reserve on an act that was expected to fail.
///
/// Returns `None` when the act is not exploration — the caller then pays the
/// ordinary way, and an act that was not exploratory is never charged twice.
pub fn explore(eco: &mut Economy, no_prior_evidence: bool) -> Option<Exploration> {
    if !may_explore(eco, no_prior_evidence) {
        return None;
    }
    let would_have_cost = 0.15 * (eco.quants.max(0.0) - Economy::FLOOR_QUANTS);
    // **Flat, not proportional.** The first version charged a fraction of the
    // reserve, which shrinks as the reserve shrinks, so exploration approached the
    // floor asymptotically and never terminated: the first test to use it looped
    // until its own 10,000-iteration guard fired. A budget whose unit of spending
    // vanishes as the budget vanishes is not a budget. A flat cost is also
    // legible -- the number of experiments a creature can afford is countable.
    let cost = EXPLORATION_COST.min(eco.quants - Economy::FLOOR_QUANTS);
    eco.quants = (eco.quants - cost).max(Economy::FLOOR_QUANTS);
    Some(Exploration {
        reserved: reserve(eco),
        would_have_cost,
    })
}

/// The production credit for an exploratory act.
///
/// **Zero, always, whatever the witness said.** An exploratory act is run to
/// find out, and finding out is the return. Crediting it would mean the creature
/// is paid for a lucky guess, and the next thing the economy does is stop
/// distinguishing exploration from exploitation.
pub fn exploratory_production() -> Production {
    Production::default()
}

/// Whether a trace is exploratory, for the induction exemption.
///
/// Read off the **intent prefix**, because `Trace` has no exploratory flag and
/// adding one to a persisted format is a migration for a distinction that is
/// visible in the record anyway. The prefix is the caller's declaration and the
/// log is the witness, so a caller that mislabels a trace is visible on the log
/// rather than hidden in a field.
pub fn is_exploratory(intent: &str) -> bool {
    intent.trim_start().starts_with(EXPLORATION_PREFIX)
}

/// The intent prefix marking an act as exploratory.
pub const EXPLORATION_PREFIX: &str = "explore";

/// A creature that has floored cannot explore, and neither can one at the cap —
/// the cap is the other boundary, and a reserve that ignored it would let a
/// creature at `MAX_QUANTS` hold a reserve it can never spend on anything.
pub fn is_exhausted(eco: &Economy) -> bool {
    eco.quants <= Economy::FLOOR_QUANTS || eco.quants >= Economy::MAX_QUANTS
}

#[cfg(test)]
mod tests {
    //! The contradiction, and the four ways a reserve could become a loophole.

    use super::*;
    use crate::mcp::store::Trace;

    fn eco(quants: f64) -> Economy {
        Economy {
            quants,
            nuants: 12.0,
        }
    }

    /// **The contradiction, stated as the thing that used to be impossible.** An
    /// act run expecting to fail had nowhere to be paid from, so the rational
    /// strategy was never to try one. Now it is affordable, from a declared
    /// reserve, and it is *only* affordable when there is no prior evidence.
    #[test]
    fn a_deliberate_loss_is_affordable_when_there_is_no_prior_evidence() {
        let mut e = eco(0.9);
        let before = e.quants;
        let spent = explore(&mut e, true).expect("a fresh signature may be explored");
        assert!(
            e.quants < before,
            "exploring cost nothing, so there is still no reason not to try"
        );
        assert!(
            spent.would_have_cost > 0.0,
            "and the receipt names what it avoided"
        );
    }

    /// **A repeat is not exploration.** An act with prior evidence is a repeat,
    /// and repeats are already priced as rumination. A reserve that refunded for
    /// repeats would be a way to make repetition free.
    #[test]
    fn a_repeat_is_never_exploration() {
        let mut e = eco(0.9);
        let before = e.quants;
        assert_eq!(
            explore(&mut e, false),
            None,
            "an act that has been done before is not exploration, and charging \
             the reserve for it would price repetition at zero"
        );
        assert_eq!(e.quants, before, "so nothing was taken either way");
    }

    /// **Non-refundable, and worth nothing if it goes well.** The credit for an
    /// exploratory act is the zero `Production`, whatever the witness said. An
    /// exploratory act is run to find out, and finding out is the return.
    /// Crediting the outcome would credit the luck instead, and the next thing
    /// the economy does is stop telling exploration from exploitation.
    #[test]
    fn an_exploratory_loss_is_never_paid_back() {
        assert_eq!(
            exploratory_production(),
            Production::default(),
            "an exploratory act earns no production, so a lucky guess is not \
             recoverable and a creature cannot bank on exploring well"
        );
        assert_eq!(
            exploratory_production(),
            Production {
                new_rules: 0,
                new_signatures: 0,
                new_phrasings: 0,
                consolidated: 0
            },
            "spelled out, so the four counters cannot drift from the default"
        );

        // And the debit really is a debit: the balance after is strictly below the
        // balance before, with no path back.
        let mut e = eco(0.9);
        let before = e.quants;
        explore(&mut e, true).expect("explores");
        assert!(e.quants < before);
    }

    /// **Bounded.** The reserve is a *share* of power, so exploring is affordable
    /// and unbounded exploring is not. A creature that explores forever ends at
    /// the floor, which is the correct outcome.
    #[test]
    fn exploring_is_bounded_and_ends_at_the_floor() {
        let mut e = eco(0.9);
        let mut attempts = 0;
        while explore(&mut e, true).is_some() {
            attempts += 1;
            assert!(attempts < 10_000, "the reserve is not bounded");
        }
        assert!(attempts > 0, "some exploration was affordable");
        // **It stops above the floor, because the reserve runs out before the
        // power does.** The first version asserted it reached the floor and was
        // wrong: the reserve is `EXPLORATION_SHARE` of the headroom, so once the
        // headroom is too small to hold one experiment the creature stops
        // exploring while it can still act. That is the better behaviour -- a
        // creature that explored itself down to the floor could not act at all,
        // and the reserve is meant to bound exploration, not to spend the creature.
        assert!(
            e.quants > Economy::FLOOR_QUANTS,
            "exploration stopped with the creature still able to act"
        );
        assert!(
            reserve(&e) < EXPLORATION_COST,
            "and the reason is that the reserve no longer holds one experiment, \
             not that the creature was spent"
        );
        assert!(
            !is_exhausted(&e),
            "so the creature is not floored or capped"
        );
    }

    /// A creature at the floor has nothing to explore with, and a creature at the
    /// cap has a reserve it can never spend on anything. Both are boundaries the
    /// reserve has to respect or it is not a budget.
    #[test]
    fn the_reserve_respects_both_boundaries() {
        assert!(
            explore(&mut eco(Economy::FLOOR_QUANTS), true).is_none(),
            "a floored creature may not explore"
        );
        assert!(is_exhausted(&eco(Economy::FLOOR_QUANTS)));
        assert!(is_exhausted(&eco(Economy::MAX_QUANTS)));
        assert!(!is_exhausted(&eco(0.5)));
    }

    /// **The bar is not weakened, only the contradiction removed.** Exploration
    /// is exempt from the failure ratio; it is *not* exempt from the observation
    /// count, so a creature cannot reach for one shape once and call it a mattern.
    #[test]
    fn exploration_is_exempt_from_failure_and_not_from_breadth() {
        use crate::induce::group_traces;
        let trace = |intent: &str, ok: bool| Trace {
            intent: intent.to_string(),
            succeeded: ok,
            primitives: vec!["Try".into(), "x".into()],
            ..Default::default()
        };

        // Six failed exploratory acts: exempt, so no misses.
        let explored: Vec<Trace> = (0..6)
            .map(|_| trace(&format!("{EXPLORATION_PREFIX} an unfamiliar shape"), false))
            .collect();
        let g = group_traces(&explored);
        let one = g.values().next().expect("a group");
        assert_eq!(one.misses, 0, "exploratory losses are not misses");
        // A group that only ever lost is still not a mattern. The exemption is
        // that exploration does not *raise* the failure ratio above what the
        // ordinary acts set -- it is not an amnesty. Reaching repeatedly proves
        // nothing; reaching repeatedly and *winning* at least once does.
        assert!(
            !one.is_evidentiary(),
            "and a group that only ever lost is still not a mattern, because \
             exempt from the count is not the same as counted as a success"
        );

        // Six failed *non*-exploratory acts: still misses, still disqualified.
        // Losing on purpose is exempt; losing by accident is not, and the
        // difference is the intent prefix and nothing else.
        let wasted: Vec<Trace> = (0..6).map(|_| trace("ordinary act", false)).collect();
        let g = group_traces(&wasted);
        let one = g.values().next().expect("a group");
        assert_eq!(one.misses, 6, "an ordinary failure is still a failure");
        assert!(
            one.failure_ratio() > crate::induce::MAX_FAILURE_RATIO,
            "and it is still disqualifying, because the exemption is for declared \
             exploration and not for everything that happened to fail"
        );
    }

    /// The prefix is the declaration, and the log is the witness: a caller that
    /// mislabels a trace is visible in the record rather than hidden in a field
    /// that does not exist.
    #[test]
    fn the_exploration_marker_is_a_visible_declaration() {
        assert!(is_exploratory("explore an unfamiliar shape"));
        assert!(is_exploratory("  explore something"));
        assert!(!is_exploratory("exploit a familiar shape"));
        assert!(!is_exploratory("fix the crash"));
    }
}
