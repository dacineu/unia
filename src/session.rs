//! Turning a session with a language model into evidence a creature can learn from.
//!
//! # Why this module exists
//!
//! A trace and a conversation look alike and are not the same thing. A trace
//! records one resolved act: an intent, the primitives it reduced to, and whether
//! it worked. A conversation records a stream of turns, most of which are not
//! acts at all — they are thinking out loud, refusals, questions, and
//! corrections. Handing a whole transcript to induction would teach the creature
//! that every sentence it ever read is a rule.
//!
//! So the unit here is smaller and it is the **correction**: the moment a person
//! says the machine did the wrong thing and says what the right thing was. That
//! is the densest learning signal in a session, and it is the one a trace can
//! honestly represent.
//!
//! # What it produces, and what it does not
//!
//! This module extracts corrections and confirmations. It does **not** emit
//! traces, and the reason is not conservatism.
//!
//! A trace's value is its `primitives` field: the sequence an intent reduced to.
//! Without a defined primitive vocabulary there is no sequence to record, and a
//! trace carrying a made-up one would be indistinguishable from a real one to
//! everything downstream. That gap is divergence D6, where `resolve_primitive`
//! reaches three of twenty variants and silently returns `SetValue` for the rest.
//!
//! What this produces instead is a [`Lesson`]: a correction or a confirmation
//! carrying its own words, the turn it answered, and how much evidence backs it.
//! When the vocabulary is defined, a lesson becomes a trace by one function call.
//! Nothing here is thrown away, and nothing here pretends.
//!
//! # What a creature can learn from this
//!
//! Very little on its own, and that is stated rather than oversold. A creature
//! can learn the shape of an instruction — that a particular phrasing was wrong
//! and another was right — but it cannot yet learn *what to do*, because doing
//! requires the primitive the vocabulary has not defined. The session teaches
//! vocabulary. It does not teach capability.

use serde::{Deserialize, Serialize};

/// One turn in a transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    /// `human`, `assistant`, or anything else the transcript used.
    pub speaker: String,
    pub text: String,
}

impl Turn {
    /// Whether this turn came from a person.
    pub fn is_human(&self) -> bool {
        self.speaker.eq_ignore_ascii_case("human") || self.speaker.eq_ignore_ascii_case("user")
    }

    /// Whether this turn came from a model.
    pub fn is_assistant(&self) -> bool {
        !self.is_human()
    }
}

/// A single piece of learnable evidence drawn from a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lesson {
    /// What the person said, in their own words. This is the phrasing that would
    /// become an alias, and it is the reason the module preserves wording rather
    /// than normalising it.
    pub phrasing: String,
    /// Which act this answers, when it answered one.
    pub answers: Option<String>,
    /// What kind of evidence it is.
    pub kind: LessonKind,
    /// How many turns in the session corroborate it. One is an anecdote.
    pub weight: usize,
}

/// The kinds of evidence a session yields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonKind {
    /// The model did the wrong thing and the person said so.
    ///
    /// The densest signal in a session. A correction names both what was wrong and
    /// what should have happened, and it carries the person's own phrasing for it.
    Correction,
    /// The model did something and the person accepted it.
    ///
    /// Weaker than a correction, and deliberately so: acceptance is consistent
    /// with a great many outcomes, while a correction rules one out.
    Confirmation,
}

/// Phrases that mark a turn as correcting the previous one.
///
/// Kept as a list of lowercase prefixes rather than a parser because a correction
/// in real transcripts is overwhelmingly a sentence *beginning* with one of
/// these, and a substring scan would also fire on "that is not quite what I meant
/// earlier in the thread" — a remark about something else entirely.
const CORRECTION_MARKERS: &[&str] = &[
    "no,",
    "no.",
    "not right",
    "not correct",
    "that's wrong",
    "that is wrong",
    "instead,",
    "rather,",
    "actually,",
    "wrong,",
    "no —",
    "no –",
    "try ",
    "use ",
    "don't ",
    "do not ",
    "should be ",
    "shouldn't ",
];

// Deliberately absent: "that is not" and "not quite". Both read as corrections at
// the start of a turn and both appear constantly in retrospective remarks about
// earlier in the thread -- "that is not quite what I meant earlier" is a comment
// on the past, not a correction of the turn in front of it. A prefix test cannot
// tell those apart, so the ambiguous markers are dropped rather than tuned, and
// the cost is that some genuine corrections phrased that way are missed. Missing
// evidence is recoverable; invented evidence is not.

/// Whether a turn reads as a correction of the one before it.
pub fn is_correction(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return false;
    }
    CORRECTION_MARKERS.iter().any(|m| t.starts_with(m))
}

/// Whether a turn reads as an acceptance of the one before it.
///
/// Deliberately conservative. Silence is not acceptance, and neither is a turn
/// that merely contains positive words — "great, now do the same thing to the
/// other file" is a new instruction, not an endorsement of the last act.
pub fn is_confirmation(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    const ACKNOWLEDGEMENTS: &[&str] = &[
        "yes", "yep", "yeah", "ok", "okay", "correct", "right", "perfect", "that works",
        "works", "good", "nice", "thanks", "thank you", "exactly",
    ];
    ACKNOWLEDGEMENTS.iter().any(|a| {
        t == *a
            || t.starts_with(&format!("{a}, "))
            || t.starts_with(&format!("{a}."))
            || t.starts_with(&format!("{a}!"))
    })
}

/// Extracts the learnable lessons from a session.
///
/// Walks the turns in order and pairs each human turn with the model turn it
/// answers. A human turn following a model turn is a response to it; a human turn
/// following another human turn is a follow-up and answers nothing, which is
/// recorded honestly rather than attached to whatever came before.
pub fn lessons(turns: &[Turn]) -> Vec<Lesson> {
    let mut out: Vec<Lesson> = Vec::new();

    for (i, turn) in turns.iter().enumerate() {
        if !turn.is_human() {
            continue;
        }
        // The turn this one answers, if the previous turn was the model's.
        let answers = match i.checked_sub(1).and_then(|p| turns.get(p)) {
            Some(prev) if prev.is_assistant() => Some(prev.text.clone()),
            _ => None,
        };

        let kind = if is_correction(&turn.text) {
            LessonKind::Correction
        } else if answers.is_some() && is_confirmation(&turn.text) {
            LessonKind::Confirmation
        } else {
            continue;
        };

        // Corroboration is counted by identical phrasing, which is the same rule
        // the evidence gate in `crate::induce` applies. Two people correcting a
        // model in the same words is corroboration; the same person correcting
        // twice in a row is repetition.
        let phrasing = turn.text.trim().to_string();
        match out.iter_mut().find(|l| l.phrasing == phrasing) {
            Some(existing) => existing.weight += 1,
            None => out.push(Lesson {
                phrasing,
                answers,
                kind,
                weight: 1,
            }),
        }
    }

    out
}

/// What a session yielded, stated as counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SessionSummary {
    pub turns: usize,
    /// Turns from a person.
    pub human_turns: usize,
    pub corrections: usize,
    pub confirmations: usize,
    /// Distinct phrasings, which is the number the evidence gate cares about
    /// rather than the number of turns.
    pub distinct_learnable: usize,
}

/// Counts what a session contains.
///
/// The distinct count is reported beside the raw one because they diverge sharply
/// in real transcripts: a long session is mostly repetition of a handful of
/// phrasings, and quoting the turn count as evidence would overstate it by an
/// order of magnitude.
pub fn summarise(turns: &[Turn]) -> SessionSummary {
    let found = lessons(turns);
    SessionSummary {
        turns: turns.len(),
        human_turns: turns.iter().filter(|t| t.is_human()).count(),
        corrections: found.iter().filter(|l| l.kind == LessonKind::Correction).count(),
        confirmations: found
            .iter()
            .filter(|l| l.kind == LessonKind::Confirmation)
            .count(),
        distinct_learnable: found.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn turn(speaker: &str, text: &str) -> Turn {
        Turn {
            speaker: speaker.into(),
            text: text.into(),
        }
    }

    pub(super) fn session() -> Vec<Turn> {
        vec![
            turn("human", "add a doc comment to the parser"),
            turn("assistant", "Added `/// Parses one statement` above the function."),
            turn("human", "no, the comment should describe the return value, not the input"),
            turn("assistant", "Rewritten to describe the return value."),
            turn("human", "perfect"),
            turn("human", "now run the tests"),
            turn("assistant", "12 passed."),
            turn("human", "yes"),
        ]
    }

}

#[cfg(test)]
mod describe_turns {
    use super::tests::turn;
    use super::*;

    #[test]
    fn recognises_a_human_turn_under_either_convention() {
        // Transcripts disagree about this spelling, and getting it wrong would
        // silently drop every lesson in a file.
        assert!(turn("human", "x").is_human());
        assert!(turn("user", "x").is_human());
        assert!(turn("HUMAN", "x").is_human());
    }

    #[test]
    fn treats_anything_else_as_the_model() {
        assert!(turn("assistant", "x").is_assistant());
        assert!(turn("bot", "x").is_assistant());
    }
}

#[cfg(test)]
mod describe_markers {
    use super::*;

    #[test]
    fn recognises_a_correction_at_the_start_of_a_turn() {
        for text in [
            "no, use tabs",
            "not right — the return value",
            "instead, pass it by reference",
            "that's wrong",
            "don't mutate the input",
            "should be a slice",
        ] {
            assert!(is_correction(text), "{text:?} should read as a correction");
        }
    }

    #[test]
    fn does_not_fire_on_a_correction_word_used_elsewhere() {
        // A substring scan would fire here and invent a correction that was never
        // made. Real transcripts are full of remarks about earlier parts of the
        // thread.
        assert!(!is_correction("that is not quite what I meant earlier in the thread"));
        assert!(!is_correction("the error was wrong about the path"));
        assert!(!is_correction(""));
    }

    #[test]
    fn recognises_an_acknowledgement() {
        for text in ["yes", "ok", "perfect", "thanks!", "correct."] {
            assert!(is_confirmation(text), "{text:?} should read as an acknowledgement");
        }
    }

    #[test]
    fn does_not_treat_a_new_instruction_as_acceptance() {
        // "Great, now do the same to the other file" praises the last act and then
        // asks for another one. Counting it as confirmation would teach the
        // creature that a new instruction is an endorsement of the previous one.
        assert!(!is_confirmation("great, now do the same to the other file"));
        assert!(!is_confirmation("do the same for the parser"));
    }
}

#[cfg(test)]
mod describe_lessons {
    use super::tests::{session, turn};
    use super::*;

    #[test]
    fn extracts_a_correction_with_the_turn_it_answers() {
        let found = lessons(&session());
        let c = found
            .iter()
            .find(|l| l.kind == LessonKind::Correction)
            .expect("the session contains a correction");
        assert!(c.answers.as_deref().unwrap().contains("Added"));
        assert!(c.phrasing.contains("return value"));
    }

    #[test]
    fn extracts_a_confirmation() {
        let found = lessons(&session());
        assert!(found.iter().any(|l| l.kind == LessonKind::Confirmation));
    }

    #[test]
    fn ignores_a_human_turn_that_answers_another_human() {
        // Two people talking to each other is not the model being corrected, and
        // attaching the second to whatever preceded it would invent a lesson.
        let turns = vec![
            turn("human", "first"),
            turn("human", "no, do the other one instead"),
        ];
        assert!(lessons(&turns).iter().all(|l| l.answers.is_none()));
    }

    #[test]
    fn counts_repeated_identical_wording_as_one_observation() {
        // The same rule the evidence gate applies, and for the same reason: five
        // identical corrections are one piece of evidence.
        let turns = vec![
            turn("assistant", "did a thing"),
            turn("human", "no, do it differently"),
            turn("assistant", "redid it"),
            turn("human", "no, do it differently"),
            turn("assistant", "redid it again"),
            turn("human", "no, do it differently"),
        ];
        let found = lessons(&turns);
        assert_eq!(found.len(), 1, "one distinct phrasing");
        assert_eq!(found[0].weight, 3, "but three observations of it");
    }

    #[test]
    fn distinct_wording_produces_distinct_lessons() {
        let turns = vec![
            turn("assistant", "a"),
            turn("human", "no, use a map instead"),
            turn("assistant", "b"),
            turn("human", "no, sort the keys first"),
        ];
        assert_eq!(lessons(&turns).len(), 2);
    }

    #[test]
    fn extracts_nothing_from_an_empty_session() {
        assert!(lessons(&[]).is_empty());
    }

    #[test]
    fn extracts_nothing_when_a_model_only_agrees_with_itself() {
        let turns = vec![turn("assistant", "I will do that"), turn("assistant", "Done.")];
        assert!(lessons(&turns).is_empty());
    }

    #[test]
    fn a_correction_with_no_prior_turn_still_counts() {
        // Someone opening a session by saying "no, do it like this" is evidence
        // even though there is nothing to correct.
        let turns = vec![turn("human", "no, always use snake_case here")];
        let found = lessons(&turns);
        assert_eq!(found.len(), 1);
        assert!(found[0].answers.is_none());
    }
}

#[cfg(test)]
mod describe_summary {
    use super::tests::{session, turn};
    use super::*;

    #[test]
    fn reports_corrections_and_confirmations_apart() {
        let s = summarise(&session());
        assert_eq!(s.corrections, 1);
        assert_eq!(s.confirmations, 2);
    }

    #[test]
    fn distinguishes_turns_from_distinct_phrasings() {
        // A long session is mostly repetition of a few phrasings, and quoting the
        // turn count as evidence would overstate it by an order of magnitude.
        let mut turns = Vec::new();
        for _ in 0..20 {
            turns.push(turn("assistant", "ok then"));
            turns.push(turn("human", "no, try again"));
        }
        let s = summarise(&turns);
        assert_eq!(s.turns, 40);
        assert_eq!(s.corrections, 1);
        assert_eq!(s.distinct_learnable, 1);
    }

    #[test]
    fn summarises_an_empty_session_as_zero() {
        assert_eq!(
            summarise(&[]),
            SessionSummary {
                turns: 0,
                human_turns: 0,
                corrections: 0,
                confirmations: 0,
                distinct_learnable: 0,
            }
        );
    }
}
