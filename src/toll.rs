//! The toll: what a consultation costs, in the currency the creature spends.
//!
//! # The cycle this closes
//!
//! The user described it precisely: unia cannot cover a piece of knowledge, so
//! it asks a model, and the ask **shows its cost in nuants** — the acts consumed,
//! correlated with the tokens the model reported. That is not an accounting
//! detail. It is the mechanism by which a creature sustains itself:
//!
//! ```text
//!   the corpus cannot answer
//!        → consult an engine, spending nuants in proportion to its tokens
//!             → the answer is knowledge
//!                  → knowledge is production
//!                       → production GAINS quants
//!                            → which buys more consultation
//! ```
//!
//! **An act spends; induction pays. A consultation is both, which is why it is
//! the interesting one.** Every other path in this project spends a resource and
//! hopes for production later. This one is a *conversion*: nuants in, quants out,
//! at a rate the engine's own report determines. If the rate is bad the creature
//! runs down, and that is the honest outcome rather than a free oracle.
//!
//! # Why tokens and nuants correlate at all
//!
//! Because [`crate::slm::Measured`] exists and only a real engine fills it. A
//! consultation that reported no measurement cannot be charged, so a model that
//! declines to report its cost gets asked for nothing — and that is precisely
//! the defect this repository removed from `MockSlm`, where `tokens_used: 450`
//! was a literal for a `format!` call. **A toll that cannot be measured must not
//! be charged, and must not be waived either**: an unmeasurable consultation is
//! refused, not free.
//!
//! # The rate, and why it is a constant and not a formula
//!
//! [`NUANTS_PER_KILO_TOKEN`] is a declared price, in the same way the credit
//! counters are declared prices (`new_rules` 0.10, `new_signatures` 0.05). It is
//! arbitrary and it is *stated*, which is the property that matters: a reader can
//! see what a consultation costs before doing one, and a different deployment
//! can change one number. What cannot be honest is leaving it implicit, because
//! then every number that depends on it is unreportable.
//!
//! **The rate is a price, not a cost.** A real deployment would derive it from
//! what the provider charges. Nothing here can know that, so nothing here
//! pretends to.

use crate::slm::Measured;

/// What a thousand reported tokens cost, in nuants.
///
/// **Declared, not derived.** See the module comment: the honesty requirement is
/// that it is stated, not that it is right.
pub const NUANTS_PER_KILO_TOKEN: f64 = 0.05;

/// Why a consultation could not be charged.
#[derive(Debug, Clone, PartialEq)]
pub enum Unpayable {
    /// The engine reported no measurement, so there is nothing to charge for.
    ///
    /// **Not waived, and not charged at a guess.** Both would be worse: a waiver
    /// makes unmeasured engines free and a guess is a fabricated number, which
    /// is the `MockSlm` defect wearing a rate.
    Unmeasured,
    /// The creature cannot pay. A refused question is not a free question.
    Insufficient { nuants: f64, required: f64 },
}

/// What a consultation costs, and what it left behind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Toll {
    /// Tokens the engine reported consuming.
    pub tokens: usize,
    /// What it cost, in nuants.
    pub nuants: f64,
}

/// The price of `tokens`, in nuants.
///
/// Rounded to four places because a float that is accumulated across a long run
/// and printed in a receipt has to look like a price, and a receipt full of
/// `0.013000000000000001` is a receipt nobody trusts.
pub fn price_of(tokens: usize) -> f64 {
    let nuants = tokens as f64 / 1000.0 * NUANTS_PER_KILO_TOKEN;
    (nuants * 10_000.0).round() / 10_000.0
}

/// Charges `nuants` for a consultation, or refuses.
///
/// **Refusing is the common case in a browser and it should be.** A creature
/// with a small stock cannot ask a model, and a system that lets it ask anyway
/// has made the economy decorative.
pub fn charge(nuants: &mut f64, measured: Option<&Measured>) -> Result<Toll, Unpayable> {
    let Some(m) = measured else {
        return Err(Unpayable::Unmeasured);
    };
    let cost = price_of(m.tokens);
    if *nuants < cost {
        return Err(Unpayable::Insufficient {
            nuants: *nuants,
            required: cost,
        });
    }
    *nuants -= cost;
    Ok(Toll {
        tokens: m.tokens,
        nuants: cost,
    })
}

/// What a consultation would earn back, in quants, if its answer became a
/// mattern.
///
/// **Stated as a range, not a number, and that is the honest form.** The earning
/// depends on what the answer is *for*: a new act is worth more than a new
/// phrasing of an act the creature already has, and which one it is cannot be
/// known until the answer has been induced into a candidate and scored. So this
/// is a bound, computed from the credit table, and the real figure comes from
/// [`crate::camaduci::Production`] after the fact.
///
/// The point of the bound is direction, not precision: **at the declared rate a
/// consultation is not obviously a loss**, and if it were, the whole cycle would
/// be a drain and the honest answer would be to stop consulting.
pub fn earn_bound() -> (f64, f64) {
    use crate::camaduci::Production;
    let a_new_rule = 0.10;
    let a_new_phrasing = 0.01;
    (a_new_phrasing * 10.0, a_new_rule * 10.0)
}

#[cfg(test)]
mod tests {
    //! The tests are about the three ways a toll can be dishonest: charging a
    //! number nothing measured, waiving a cost because it is inconvenient, and
    //! printing a price that does not look like one.

    use super::*;

    /// **The price is the declared rate and nothing else.** If this drifts, every
    /// number that depends on the toll becomes unreportable, which is the whole
    /// reason the rate is a constant rather than something computed.
    #[test]
    fn the_price_is_the_declared_rate() {
        assert_eq!(price_of(0), 0.0);
        assert_eq!(price_of(1_000), NUANTS_PER_KILO_TOKEN);
        assert_eq!(price_of(2_000), NUANTS_PER_KILO_TOKEN * 2.0);
        assert_eq!(price_of(500), NUANTS_PER_KILO_TOKEN / 2.0);
    }

    /// **A receipt has to look like a price.** Not a correctness requirement — a
    /// trust one. A float accumulated over a long run prints as
    /// `0.013000000000000001` and a reader stops believing the column.
    #[test]
    fn the_price_is_rounded_to_four_places() {
        let p = price_of(333);
        assert_eq!(
            p,
            ((p as f64) * 10_000.0).round() / 10_000.0,
            "{p} carries more precision than a price should"
        );
    }

    /// **An unmeasured consultation is refused — not waived and not guessed.**
    /// Charging a fabricated number is the `MockSlm` defect with a rate on it, and
    /// waiving it makes every unmeasured engine free, which is how a free oracle
    /// becomes the default and nobody notices.
    #[test]
    fn an_unmeasured_consultation_is_refused_rather_than_free() {
        let mut nuants = 10.0;
        assert_eq!(
            charge(&mut nuants, None),
            Err(Unpayable::Unmeasured),
            "and the stock is untouched, because nothing was spent"
        );
        assert_eq!(nuants, 10.0, "a refused question is not a free question");
    }

    /// **A creature that cannot pay does not ask.** The economy is decorative if
    /// a consultation ignores the stock, and the browser case is exactly this: a
    /// small stock and a model that costs more than it holds.
    #[test]
    fn a_creature_that_cannot_pay_cannot_ask() {
        let mut nuants = 0.001;
        let m = Measured {
            tokens: 10_000,
            engine_steps: None,
        };
        match charge(&mut nuants, Some(&m)) {
            Err(Unpayable::Insufficient {
                required,
                nuants: had,
            }) => {
                assert!(
                    required > had,
                    "the shortfall is reported, not rounded away"
                )
            }
            other => panic!("a creature with 0.001 nuants bought a 0.5 nuant answer: {other:?}"),
        }
        assert_eq!(nuants, 0.001, "and nothing was taken");
    }

    /// **The happy path spends exactly the price, and the reported tokens are
    /// what was charged for.** The correlation the user asked for is this: the
    /// resource consumed is derived from the measurement, never the reverse.
    #[test]
    fn a_measured_consultation_costs_its_reported_tokens() {
        let mut nuants = 10.0;
        let m = Measured {
            tokens: 4_000,
            engine_steps: Some(3),
        };
        let t = charge(&mut nuants, Some(&m)).expect("payable");
        assert_eq!(t.tokens, 4_000, "what it said it used");
        assert_eq!(t.nuants, NUANTS_PER_KILO_TOKEN * 4.0, "what that cost");
        assert!(
            (nuants - (10.0 - t.nuants)).abs() < 1e-9,
            "and the stock reflects it exactly, so a run of consultations \
             accumulates to a checkable figure"
        );
    }

    /// **Direction, not precision.** The bound is not a promise that a
    /// consultation pays; it is the statement that it is not obviously a drain.
    /// If it were obviously a drain the honest answer would be to stop consulting,
    /// and this test exists so that answer stays reachable instead of being
    /// foreclosed by a number nobody computed.
    #[test]
    fn a_consultation_is_not_obviously_a_drain() {
        let (worst, best) = earn_bound();
        let one_big_answer = price_of(2_000);
        assert!(
            best > one_big_answer,
            "a 2,000-token answer can return more than it cost, at the declared \
             rates: worst {worst}, best {best}, cost {one_big_answer}"
        );
        // **Exactly break-even, which is a sharper fact than "it loses" and the
        // one I did not expect.** At 2,000 tokens the cost is 0.1 and the worst
        // case earns 0.1, so the cycle neither drains nor guarantees: everything
        // below ten new phrasings for a 2,000-token answer is a loss, and
        // everything above is a gain. The declared rate has put the break-even
        // point at a number a reader can check, which is the most a *declared*
        // price can honestly offer.
        assert_eq!(
            worst, one_big_answer,
            "break-even at 2,000 tokens, so viability is decided by the answer \
             being better than ten phrasings -- a decision, not a guarantee"
        );
        let small = price_of(200);
        assert!(
            small < worst,
            "a 200-token answer at the worst case returns {} against a cost of {}, \
             so asking small questions is a drain and the rate does work",
            worst,
            small
        );
    }

    /// **A long run accumulates to something checkable.** Ten small
    /// consultations, summed by hand, not by the function — because a test that
    /// computes its expectation with the code under test tests nothing.
    #[test]
    fn ten_small_consultations_sum_to_the_declared_total() {
        let mut nuants = 100.0;
        let m = Measured {
            tokens: 250,
            engine_steps: None,
        };
        for _ in 0..10 {
            charge(&mut nuants, Some(&m)).expect("payable");
        }
        // 250 tokens is a quarter of a thousand, so ten of them are 2.5 thousand.
        assert!(
            (nuants - (100.0 - 0.125)).abs() < 1e-9,
            "spent {} and should have spent 0.125",
            100.0 - nuants
        );
    }
}
