//! The small-model boundary, and the rule that a mock may not report a
//! measurement.
//!
//! # What was wrong here, and why it was a type problem
//!
//! [`SlmResponse`] used to carry `tokens_used: usize` and `reasoning_steps:
//! usize`, and [`MockSlm`] filled them with `30`, `450` and `1`, `3` — literals,
//! for a `format!` that consumed no tokens and took no steps. The values were
//! also *ordered the way a real engine's would be*, so they were not obviously
//! fabricated to a reader: `DEEP_REASONING` reported 450 and 3, and a player or
//! a test would correctly infer that the hard question cost more. It did not.
//!
//! One test asserted `reasoning_steps > 1`, so a suite existed whose only
//! content was confirming that a mock lies convincingly. That is a test that
//! passes when the fabrication is *good*.
//!
//! **The fix is not setting the numbers honestly, because there are no honest
//! numbers.** There is no engine here. So the type is changed so the claim is
//! unrepresentable: [`Measured`] exists, and only a real engine can populate it.
//! A mock returns `None`, which every consumer must handle, and there is no
//! longer a value in the tree that means "the model used 450 tokens" without
//! something having measured it.

use crate::orchestrator::{ActivationVector, BehavioralVector};

/// Where an answer came from, and therefore what may be claimed about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provenance {
    /// Fabricated locally. Carries no measurement and must not pretend to.
    Mock,
    /// Produced by a named engine, which reported its own cost.
    Consulted {
        /// Whatever the caller can name the engine as. A peer has to be able to
        /// tell a model from a mock, and "some endpoint" is not a name.
        model: String,
    },
}

impl Provenance {
    /// Whether anything behind this answer was measured.
    pub fn is_measured(&self) -> bool {
        matches!(self, Provenance::Consulted { .. })
    }
}

/// A cost that was actually observed.
///
/// Separate from [`SlmResponse`] rather than optional fields on it, because an
/// optional field is a field a caller can read without noticing it is absent, and
/// `0` and `None` are both plausible-looking numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measured {
    /// Tokens the engine reported consuming.
    pub tokens: usize,
    /// Engine-reported intermediate steps, when the engine reports any. Many do
    /// not, and a missing count is not a zero.
    pub engine_steps: Option<usize>,
}

/// An answer, and exactly what is known about how it was produced.
#[derive(Debug, Clone, PartialEq)]
pub struct SlmResponse {
    pub text: String,
    pub provenance: Provenance,
    /// `None` unless [`Provenance::Consulted`]. A mock has no measurement to
    /// report, so it reports none.
    pub measured: Option<Measured>,
}

impl SlmResponse {
    /// A fabricated answer, and it says so.
    pub fn mock(text: String) -> Self {
        SlmResponse {
            text,
            provenance: Provenance::Mock,
            measured: None,
        }
    }

    /// An answer from a named engine that reported its cost.
    pub fn consulted(model: &str, text: String, measured: Measured) -> Self {
        SlmResponse {
            text,
            provenance: Provenance::Consulted {
                model: model.to_string(),
            },
            measured: Some(measured),
        }
    }

    /// The observed token cost, or `None` when nothing observed one.
    ///
    /// `Option` rather than a zero, because a zero would be a claim that the
    /// engine was free and would silently pass a budget check that nothing
    /// actually performed.
    pub fn tokens(&self) -> Option<usize> {
        self.measured.map(|m| m.tokens)
    }
}

/// A local stand-in for an engine.
///
/// It exists so the system can be exercised without one, and it is built so that
/// exercising it cannot produce a false claim: [`SlmResponse::mock`] is the only
/// constructor it uses, and the shape of a request still changes the answer, so
/// the mode is observable — it just has no price.
pub struct MockSlm;

impl MockSlm {
    pub fn new() -> Self {
        Self
    }

    /// Answers without an engine, and reports nothing it did not measure.
    ///
    /// The three modes still produce different text, because a mock that ignored
    /// its configuration would not exercise the code that reads it. What it no
    /// longer does is attach a number to the difference.
    pub fn execute(
        &self,
        user_input: &str,
        activation: &ActivationVector,
    ) -> Result<SlmResponse, Box<dyn std::error::Error>> {
        let text = match activation.behavioral_mode {
            BehavioralVector::Quickest => format!("[direct] {}", user_input),
            BehavioralVector::Smartest => {
                format!("[analysis -> validation] {}", user_input)
            }
            BehavioralVector::Direct => format!("[result] {}", user_input),
        };
        Ok(SlmResponse::mock(text))
    }
}

#[cfg(test)]
mod tests {
    //! The contract is a negative one, so most of these tests assert that
    //! something is *absent*, which is unusual and deliberate.

    use super::*;

    fn activation(mode: BehavioralVector, budget: usize) -> ActivationVector {
        ActivationVector {
            system_prompt: String::new(),
            token_budget: budget,
            is_deep_dive: false,
            behavioral_mode: mode,
        }
    }

    /// **A mock cannot report a cost, in either direction.**
    ///
    /// Not "reports an honest one" — there is no honest one. The point is that
    /// the number is not there to be wrong.
    #[test]
    fn a_mock_reports_no_measurement() {
        let r = MockSlm::new()
            .execute(
                "what is 2 + 2",
                &activation(BehavioralVector::Smartest, 10_000),
            )
            .expect("the mock always answers");
        assert_eq!(r.measured, None, "nothing ran, so nothing was measured");
        assert_eq!(r.tokens(), None);
        assert!(!r.provenance.is_measured());
        assert_eq!(r.provenance, Provenance::Mock);
    }

    /// The old suite asserted `reasoning_steps > 1` on the mock's output. It
    /// passed when the fabrication was *good*, which means it was a test of the
    /// lie. This is the same test turned around: the claim must now be absent.
    #[test]
    fn the_response_type_has_no_step_count_to_assert_on() {
        let r = MockSlm::new()
            .execute("q", &activation(BehavioralVector::Smartest, 1))
            .expect("answers");
        // Nothing here can be asserted to be greater than one, because the field
        // is gone rather than zero. This is asserted by construction: the
        // response has exactly three fields and none of them is a step count.
        assert_eq!(
            [
                r.text.len() > 0,
                r.measured.is_none(),
                !r.provenance.is_measured()
            ],
            [true, true, true]
        );
    }

    /// **A budget cannot be checked against a mock, and the check is now
    /// honest about that.** `MockSlm` used to return `TOKEN_BUDGET_EXCEEDED` by
    /// comparing a fabricated cost against the budget — a budget refusal
    /// manufactured by a number that did not exist. The mock now has no cost, so
    /// it cannot claim to have exceeded anything.
    #[test]
    fn a_mock_answers_a_request_its_budget_could_never_have_refused() {
        let r = MockSlm::new()
            .execute("q", &activation(BehavioralVector::Smartest, 1))
            .expect("no fabricated cost means no fabricated refusal");
        assert!(r.tokens().is_none());
    }

    /// A consulted answer is distinguishable from a mock, and carries the cost
    /// its engine reported. This is the shape a real integration has to fill, so
    /// it is pinned now rather than when the integration arrives.
    #[test]
    fn a_consulted_answer_is_named_and_carries_its_own_cost() {
        let r = SlmResponse::consulted(
            "some-model",
            "four".into(),
            Measured {
                tokens: 412,
                engine_steps: None,
            },
        );
        assert_eq!(r.tokens(), Some(412));
        assert!(r.provenance.is_measured());
        assert_eq!(
            r.provenance,
            Provenance::Consulted {
                model: "some-model".into()
            }
        );
        // And a missing step count is `None`, not zero: many engines report no
        // steps, and a zero would be a claim that it took none.
        assert_eq!(r.measured.unwrap().engine_steps, None);
    }

    /// **A reader can always tell which kind of answer it has.** Every consumer
    /// that prints a cost has to branch, and this is the test that the branch is
    /// available.
    #[test]
    fn the_two_kinds_of_answer_are_never_equal() {
        let mock = SlmResponse::mock("four".into());
        let real = SlmResponse::consulted(
            "m",
            "four".into(),
            Measured {
                tokens: 1,
                engine_steps: None,
            },
        );
        assert_ne!(
            mock, real,
            "same text, different provenance, not the same answer"
        );
    }

    /// The mode still changes the answer, so the code that reads the mode is
    /// still exercised. What changed is that the difference has no price.
    #[test]
    fn the_mode_is_observable_but_priceless() {
        let s = MockSlm::new();
        let fastest = s
            .execute("q", &activation(BehavioralVector::Quickest, 9_999))
            .expect("answers");
        let smartest = s
            .execute("q", &activation(BehavioralVector::Smartest, 9_999))
            .expect("answers");
        assert_ne!(fastest.text, smartest.text, "the mode is still visible");
        assert_eq!(
            (fastest.measured, smartest.measured),
            (None, None),
            "and neither claims what the difference cost"
        );
    }
}
