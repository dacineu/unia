//! ca(R)maduci: a digital pet whose care is recorded as traces.
//!
//! This module holds the pet as pure logic. It has no clock, no I/O, and no
//! renderer, which is what lets the same code drive a terminal example, a
//! browser client, and a test suite without any of them agreeing on how time
//! passes. `now` is always a parameter supplied by the caller.
//!
//! The design rationale and the prior art it draws on are in
//! `docs/camaduci.md`. Three properties matter enough to state here:
//!
//! * **Age advances on a completed sleep cycle, not on a clock.** A pet never put
//!   to sleep never leaves the egg. Progress is gated on a finished interaction
//!   rather than on elapsed time. The mechanic is taken from `tama96`.
//! * **Neglect is the absence of a call.** Nothing in this module can punish the
//!   pet; time passing is the only way its state worsens. That is what makes the
//!   obligation real rather than a rule the game enforces at will.
//! * **Death is a lifecycle transition, not a deletion.** A dead pet is retained,
//!   still inspectable, and its traces remain evidence.
//!
//! What this does not model: rendering, sound, score, or more than one pet. One
//! pet is one artifact with no peer, so nothing here demonstrates convergence.

use std::collections::{BTreeMap, BTreeSet};

/// The creature's declared state space.
///
/// Every field is a state variable a `.ure` manifest would declare, with the same
/// type and the same range. Keeping them in a struct rather than a manifest is
/// the one place this diverges from the design: the manifest form is exercised by
/// the corpus, and a struct keeps the arithmetic readable.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vitals {
    /// 1.0 is starving. Rises on every tick the pet is not fed.
    pub hunger: f64,
    /// Falls on every tick the pet is not played with.
    pub happiness: f64,
    /// Falls when hunger or happiness bottoms out, recovers slowly otherwise.
    pub health: f64,
    /// Completed sleep cycles. A stage cannot advance without one.
    pub age_ticks: u32,
}

impl Default for Vitals {
    fn default() -> Self {
        Vitals {
            hunger: 0.2,
            happiness: 0.6,
            health: 1.0,
            age_ticks: 0,
        }
    }
}

impl Vitals {
    /// Whether the pet is still alive.
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }

    /// A one-word description of how the pet is doing.
    pub fn mood(&self) -> &'static str {
        if !self.alive() {
            return "gone";
        }
        if self.health < 0.3 {
            return "sick";
        }
        if self.hunger > 0.8 {
            return "starving";
        }
        if self.happiness < 0.25 {
            return "lonely";
        }
        "content"
    }

    /// Advances one unit of time with no care given.
    ///
    /// Decay applies to the whole vector at once so the numbers a care operation
    /// sees are the numbers the next tick starts from.
    pub fn decay(&mut self) {
        self.hunger = (self.hunger + 0.15).min(1.0);
        self.happiness = (self.happiness - 0.10).max(0.0);
        // Health follows the worst of the two rather than tracking them
        // separately: a pet that is starving *and* lonely is in worse shape than
        // either alone would suggest, and one term captures that.
        let strain = self.hunger.max(1.0 - self.happiness);
        if strain > 0.7 {
            self.health = (self.health - 0.10).max(0.0);
        } else {
            self.health = (self.health + 0.02).min(1.0);
        }
    }
}

/// The growth stage, advanced only by a completed sleep cycle.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Egg,
    Hatchling,
    Juvenile,
    Adult,
}

impl Stage {
    /// The next stage, or `None` at the end of the line.
    pub fn next(self) -> Option<Stage> {
        match self {
            Stage::Egg => Some(Stage::Hatchling),
            Stage::Hatchling => Some(Stage::Juvenile),
            Stage::Juvenile => Some(Stage::Adult),
            Stage::Adult => None,
        }
    }

    /// Completed cycles needed in total to leave this stage.
    ///
    /// Cumulative rather than per-stage, so the thresholds read 1, 3, 5.
    pub fn cycles_to_advance(self) -> u32 {
        match self {
            Stage::Egg => 1,
            Stage::Hatchling => 3,
            Stage::Juvenile => 5,
            Stage::Adult => 0,
        }
    }

    /// The stage's name, as shown to a player.
    pub fn label(self) -> &'static str {
        match self {
            Stage::Egg => "egg",
            Stage::Hatchling => "hatchling",
            Stage::Juvenile => "juvenile",
            Stage::Adult => "adult",
        }
    }
}

/// A care operation. These are the primitives the pet exposes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Care {
    Feed,
    Play,
    Clean,
    Sleep,
}

impl Care {
    /// Parses an operation name, for a wire protocol.
    ///
    /// This is the wire form, not understanding. It recognises four words and
    /// nothing else, which is why a player who types "give it some kibble" is
    /// told the operation is unknown even though the creature has been fed that
    /// exact sentence many times. Understanding lives in
    /// [`Pet::understand`], which resolves free text through the vocabulary the
    /// creature has actually learned.
    pub fn parse(s: &str) -> Option<Care> {
        match s.to_ascii_lowercase().as_str() {
            "feed" => Some(Care::Feed),
            "play" => Some(Care::Play),
            "clean" => Some(Care::Clean),
            "sleep" => Some(Care::Sleep),
            _ => None,
        }
    }

    /// Every act the creature knows how to perform, as its own name.
    ///
    /// Offered when a request is not understood, so that a player who cannot be
    /// served is at least told what would be. A refusal that does not say what
    /// would work is the same as no answer.
    pub fn all() -> [Care; 4] {
        [Care::Feed, Care::Play, Care::Clean, Care::Sleep]
    }

    /// The primitive sequence this act reduces to.
    ///
    /// The single source of the mapping, so that recognising a sequence and
    /// producing one cannot drift apart. `apply` used to return this inline and
    /// nothing could read it back, which is why a learned rule could be
    /// recognised and then not acted on.
    pub fn signature(self) -> &'static str {
        match self {
            Care::Feed => "SetValue_CheckSense",
            // Was `SetValue_Emit`. `Emit` is not one of the sixteen declared
            // `UniversalPrimitive` variants, so playing with the creature recorded
            // a sequence the bridge could never resolve and the nucleus could
            // never dispatch. It survived all the way into a learned rule, as
            // `SetValue_Emit`, which is the case P2 exists to prevent. `Pulse` is
            // the honest name for a short outward emission.
            Care::Play => "SetValue_Pulse",
            Care::Clean => "SetValue",
            Care::Sleep => "SetValue_GetState",
        }
    }

    /// The state the creature declares, as `(field, description)`.
    ///
    /// The same list `Vitals` carries, kept beside the acts that change it so
    /// that a refusal can be assembled from declarations rather than from a
    /// hand-written sentence about them.
    pub fn declared_state() -> [(&'static str, &'static str); 3] {
        [
            ("hunger", "rises when I am not fed"),
            ("happiness", "falls when I am not played with"),
            ("health", "falls when the other two bottom out"),
        ]
    }

    /// The acts that change a declared field, by name.
    ///
    /// Note what is absent: no act *sets* a field to a value. Every one of them
    /// moves it as a side effect of doing something else, which is the fact the
    /// creature's question is about.
    pub fn changes(field: &str) -> Vec<&'static str> {
        // Probed from a mid-range reading rather than from `Vitals::default`.
        // From the default, `health` is already 1.0 and feeding it cannot raise
        // it, so the probe reported that nothing changes health — which is a
        // property of the starting value and not of the act. A reader asking
        // "what changes my health" must not be told "nothing" because the default
        // happened to be at the ceiling.
        let baseline = Vitals {
            hunger: 0.5,
            happiness: 0.5,
            health: 0.5,
            age_ticks: 0,
        };
        let mut out = Vec::new();
        for care in Care::all() {
            let mut probe = baseline;
            care.apply(&mut probe);
            if field_moved(field, &baseline, &probe) {
                out.push(care.label());
            }
        }
        out
    }

    /// The act a primitive sequence corresponds to, if it is one the creature
    /// knows how to perform.
    pub fn from_signature(signature: &str) -> Option<Care> {
        Care::all().into_iter().find(|c| c.signature() == signature)
    }

    /// The operation's name, as shown to a player.
    pub fn label(self) -> &'static str {
        match self {
            Care::Feed => "feed",
            Care::Play => "play",
            Care::Clean => "clean",
            Care::Sleep => "sleep",
        }
    }

    /// Applies the operation and returns the universal primitives it reduced to.
    ///
    /// The sequence is the induction key: two different sentences that reduced to
    /// the same sequence are evidence for one rule, which is why it is returned
    /// rather than merely logged.
    pub fn apply(self, v: &mut Vitals) -> Vec<String> {
        match self {
            Care::Feed => {
                v.hunger = (v.hunger - 0.5).max(0.0);
                v.health = (v.health + 0.05).min(1.0);
                Care::Feed
                    .signature()
                    .split('_')
                    .map(String::from)
                    .collect()
            }
            Care::Play => {
                v.happiness = (v.happiness + 0.4).min(1.0);
                Care::Play
                    .signature()
                    .split('_')
                    .map(String::from)
                    .collect()
            }
            Care::Clean => {
                v.happiness = (v.happiness + 0.1).min(1.0);
                v.health = (v.health + 0.03).min(1.0);
                Care::Clean
                    .signature()
                    .split('_')
                    .map(String::from)
                    .collect()
            }
            // Sleep is the only operation that advances age, and only if the pet
            // is alive to be put to sleep.
            Care::Sleep => {
                if v.alive() {
                    v.age_ticks += 1;
                    v.hunger = (v.hunger + 0.05).min(1.0);
                    v.happiness = (v.happiness + 0.05).min(1.0);
                }
                Care::Sleep
                    .signature()
                    .split('_')
                    .map(String::from)
                    .collect()
            }
        }
    }
}

/// Joins a list as prose: "a", "a and b", "a, b and c".
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// Reads one declared field out of a vitals reading.
fn field_value(field: &str, v: &Vitals) -> f64 {
    match field {
        "hunger" => v.hunger,
        "happiness" => v.happiness,
        "health" => v.health,
        _ => 0.0,
    }
}

fn field_moved(field: &str, before: &Vitals, after: &Vitals) -> bool {
    (field_value(field, before) - field_value(field, after)).abs() > f64::EPSILON
}

/// Terms too common to carry meaning about which act is meant.
///
/// Deliberately short and deliberately English-only. A Romanian request would
/// not be filtered by it, which is a known asymmetry rather than a decision: the
/// creature understands what it has been fed and has no way to know that "the"
/// is common in a language it has never encountered.
const FUNCTION_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "can", "could", "for", "from", "give",
    "has", "have", "he", "her", "him", "his", "i", "in", "is", "it", "its", "let", "me", "my",
    "no", "not", "of", "on", "or", "our", "please", "she", "should", "so", "than", "that", "the",
    "their", "them", "then", "there", "they", "this", "to", "up", "was", "we", "were", "what",
    "when", "which", "will", "with", "would", "you", "your",
];

/// What a creature made of a request.
///
/// Three outcomes, and keeping them apart is the whole point. Collapsing the
/// last two into one "unknown" is worse than useless: a creature then either
/// apologises for knowledge it has, or claims knowledge it lacks, and the player
/// stops asking either way. The middle case exists so that a player who cannot be
/// served is told what *would* be served, and can simply switch.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Understanding {
    /// The request reduced to an act. The exact wording was one it already had.
    Understood { care: Care },
    /// The act is one the creature performs, and this phrasing is among the ways
    /// it has been seen to be asked for it.
    UnderstoodByExperience { care: Care, matched: String },
    /// The act is one the creature performs, but this phrasing is not among the
    /// ways it has been asked. It is being told what would work, not refused.
    NeedsDifferentWords {
        known: Vec<String>,
        learned_examples: Vec<String>,
    },
    /// The request named one of the creature's own declared fields, and no act
    /// sets it to a value.
    ///
    /// This is the creature asking *why*, and it is the one refusal that is a
    /// statement about the architecture rather than about vocabulary. The field
    /// is declared, the creature can report it, and something changes it — but
    /// only as a side effect of a whole act. "Set its hunger to 0.2" is not a
    /// mis-phrasing of "feed it"; it is a request the declarations cannot
    /// satisfy, and the honest answer is to say so rather than to guess which of
    /// the two the player meant.
    Contradicted {
        field: String,
        declared: Vec<String>,
        changed_by: Vec<String>,
    },
    /// Nothing matches, and none of the wording is recognisable either. The
    /// creature says so as a fact rather than as a score, and still says what it
    /// has: being told "no" and nothing else is the same as not answering.
    NeverHeardOf {
        known: Vec<String>,
        learned_examples: Vec<String>,
    },
}

impl Understanding {
    /// Whether the creature can act on the request.
    pub fn acts(&self) -> Option<Care> {
        match self {
            Understanding::Understood { care }
            | Understanding::UnderstoodByExperience { care, .. } => Some(*care),
            _ => None,
        }
    }

    /// Whether this is a refusal to act, as opposed to a clarification.
    pub fn is_refusal(&self) -> bool {
        matches!(
            self,
            Understanding::NeedsDifferentWords { .. }
                | Understanding::NeverHeardOf { .. }
                | Understanding::Contradicted { .. }
        )
    }

    /// Whether the refusal is about the architecture rather than about wording.
    ///
    /// The distinction matters to whoever is reading it. A phrasing miss is
    /// fixed by saying it differently. A contradiction is not, and answering it
    /// with a list of phrasings would be answering a question nobody asked.
    pub fn is_contradiction(&self) -> bool {
        matches!(self, Understanding::Contradicted { .. })
    }

    /// The question the creature puts to the player, in its own terms.
    ///
    /// Two branches, because there are exactly two things that could be wrong:
    /// either the player meant a different field, or they think the creature can
    /// do something it cannot. A player can answer either in one word.
    pub fn why(&self) -> Option<String> {
        match self {
            Understanding::Contradicted {
                field,
                declared,
                changed_by,
            } => {
                let changers = if changed_by.is_empty() {
                    "nothing I do changes it".to_string()
                } else {
                    format!("what I do instead is {}", join_and(changed_by))
                };
                Some(format!(
                    "You asked me to set my {field}. I have {}. Nothing I do sets one \
                     of them to a value — {changers}. So either you mean one of those, \
                     or you have me confused with something else. Which?",
                    declared.join(", "),
                ))
            }
            _ => None,
        }
    }
}

/// A rule the creature has worked out for itself.
///
/// This is the creature's actual education. `history` records what it was *fed*,
/// which is a record of the player's actions and tells the creature nothing; a
/// learned rule is what it derived from those actions, and until one exists
/// somewhere the creature cannot be said to have evolved at all.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LearnedRule {
    /// Content address of the induced artifact. Two creatures that learned the
    /// same act share this, which is what makes them related rather than similar.
    pub address: String,
    /// The primitive sequence the act reduced to.
    pub signature: String,
    /// The phrasings it can be recognised by.
    pub aliases: Vec<String>,
    /// How well evidenced the rule is, in `0.0..=1.0`.
    pub confidence: f64,
    /// Distinct observations behind it.
    pub observations: usize,
}

/// The pet, its lifecycle, and how it is kept.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pet {
    /// Content address or human id this pet is recorded under.
    pub id: String,
    /// Its declared state.
    pub vitals: Vitals,
    /// The stage it has earned, recomputed from `age_ticks`.
    pub stage: Stage,
    /// A lifecycle value in the sense `crate::gc` uses one. Death moves the pet
    /// to quarantined; it does not delete it.
    pub quarantined: bool,
    /// How many times each operation has been performed, which is what induction
    /// groups by.
    pub history: BTreeMap<Care, u32>,
    /// What the creature has worked out for itself, most confident first.
    ///
    /// Empty for a creature that has not been taught, and *that* is the honest
    /// starting state: a kept creature with no learned rules has been played with
    /// and has learned nothing, which the two must not be confused for.
    pub learned: Vec<LearnedRule>,
}

impl Pet {
    /// A new pet, at the egg stage.
    pub fn new(id: impl Into<String>) -> Self {
        Pet {
            id: id.into(),
            vitals: Vitals::default(),
            stage: Stage::Egg,
            quarantined: false,
            history: BTreeMap::new(),
            learned: Vec::new(),
        }
    }

    /// Records what induction derived from the creature's own trace log.
    ///
    /// This is the feedback path. Without it the creature writes traces that
    /// something else reads and the creature never learns that it learned, so its
    /// evolution is invisible to it and the same on every restart.
    pub fn learn(&mut self, rules: Vec<LearnedRule>) {
        self.learned = rules;
    }

    /// Restores a pet from recorded state, for resuming a session.
    pub fn restore(
        id: impl Into<String>,
        vitals: Vitals,
        quarantined: bool,
        history: BTreeMap<Care, u32>,
    ) -> Self {
        let mut p = Pet {
            id: id.into(),
            vitals,
            stage: Stage::Egg,
            quarantined,
            history,
            learned: Vec::new(),
        };
        // The stage is earned, not stored: it is a function of completed sleep
        // cycles, so recomputing it is correct even if a saved stage disagreed.
        p.stage = p.earned_stage();
        p
    }

    /// Serialises the creature for storage.
    ///
    /// The saved form is the whole memory. There is no export, and a handed-over
    /// creature arrives carrying its mistakes along with its progress.
    /// Works out what a free-text request is asking for.
    ///
    /// Three stages, in order of how much the creature actually knows:
    /// the four wire words, then the phrasings it has been fed, then nothing.
    /// Phrasing is matched on **whole terms**, not substrings, for the reason
    /// `resolve_primitive` does it: `widget` contains `get` and `offset` contains
    /// `off`, so a substring test reads one word as another.
    ///
    /// It reads the creature's own `learned` rules, which is the point. The
    /// vocabulary that resolves a request is the vocabulary the creature earned
    /// from being kept, not a list hard-coded next to the parser.
    pub fn understand(&self, text: &str) -> Understanding {
        if let Some(care) = Care::parse(text) {
            return Understanding::Understood { care };
        }

        let terms = crate::bridge::primitive::tokenize_id(text);
        if terms.is_empty() {
            return Understanding::NeverHeardOf {
                known: Care::all().iter().map(|c| c.label().to_string()).collect(),
                learned_examples: self.heard_examples(8),
            };
        }

        // Longest match wins, so a phrasing that is a superset of another is
        // preferred over the shorter one it contains.
        let mut best: Option<(usize, Care, String)> = None;
        for rule in &self.learned {
            let Some(care) = Care::from_signature(&rule.signature) else {
                continue;
            };
            for alias in &rule.aliases {
                let alias_terms = crate::bridge::primitive::tokenize_id(alias);
                if alias_terms.is_empty() || !alias_terms.iter().all(|a| terms.contains(a)) {
                    continue;
                }
                let score = alias_terms.len();
                if best.as_ref().is_none_or(|(n, _, _)| score > *n) {
                    best = Some((score, care, alias.clone()));
                }
            }
        }

        let known = Care::all().iter().map(|c| c.label().to_string()).collect();
        if let Some((_, care, matched)) = best {
            return Understanding::UnderstoodByExperience { care, matched };
        }

        // Nothing matched. The honest next question is whether the creature has
        // heard *any* of this player's words, and that decides which of the two
        // refusals applies.
        //
        // This distinction is only available lexically, and that is a real limit
        // rather than an implementation detail. "give it a snack" and
        // "frobnicate the widget" are the same act to this creature: it has
        // never been shown either, and with no model it cannot tell that one is
        // a request for something it does and the other is not. Pretending
        // otherwise would be a guess wearing the costume of a distinction.
        // Before the vocabulary questions, the one that is about the
        // architecture rather than about wording. A request that names a declared
        // field is not a mis-phrasing; it is a request the declarations cannot
        // satisfy, because nothing here *sets* a field to a value.
        for (field, _) in Care::declared_state() {
            if terms.iter().any(|t| t == field) {
                return Understanding::Contradicted {
                    field: field.to_string(),
                    declared: Care::declared_state()
                        .iter()
                        .map(|(f, d)| format!("{f} ({d})"))
                        .collect(),
                    changed_by: Care::changes(field)
                        .into_iter()
                        .map(str::to_string)
                        .collect(),
                };
            }
        }

        let vocabulary = self.vocabulary_terms();
        let heard_any = terms.iter().any(|t| vocabulary.contains(t));

        if !heard_any {
            return Understanding::NeverHeardOf {
                known,
                learned_examples: self.heard_examples(8),
            };
        }

        Understanding::NeedsDifferentWords {
            known,
            learned_examples: self.heard_examples(8),
        }
    }

    /// The content terms the creature has any evidence for.
    ///
    /// Function words are dropped, because almost every English sentence shares
    /// them: counting "the" as an overlap would make every request look like one
    /// the creature had half-heard, and the distinction between the two refusals
    /// would collapse.
    fn vocabulary_terms(&self) -> BTreeSet<String> {
        self.learned
            .iter()
            .flat_map(|r| r.aliases.iter())
            .flat_map(|a| crate::bridge::primitive::tokenize_id(a))
            .filter(|t| !FUNCTION_WORDS.contains(&t.as_str()))
            .collect()
    }

    /// The phrasings the creature has been fed, most recent evidence first in
    /// whatever order the rules carry, capped so a refusal stays readable.
    fn heard_examples(&self, limit: usize) -> Vec<String> {
        let mut out: Vec<String> = self
            .learned
            .iter()
            .filter(|r| Care::from_signature(&r.signature).is_some())
            .flat_map(|r| r.aliases.clone())
            .collect();
        out.sort();
        out.dedup();
        out.truncate(limit);
        out
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }

    /// Restores a creature from its saved form.
    ///
    /// Returns `None` for anything unreadable rather than substituting a fresh
    /// creature, because silently replacing a pet that died with a live egg would
    /// erase the one part of it that cannot be regenerated.
    pub fn from_json(id: &str, text: &str) -> Option<Pet> {
        let mut p: Pet = serde_json::from_str(text).ok()?;
        p.id = id.to_string();
        p.stage = p.earned_stage();
        Some(p)
    }

    /// Performs one care operation, returning the primitives it reduced to.
    ///
    /// `None` means the pet is dead and accepted nothing, in which case the
    /// caller must not record a trace: a refused operation is not an
    /// interaction, and recording one would put an event in the training signal
    /// that never happened.
    pub fn tend(&mut self, care: Care) -> Option<Vec<String>> {
        if self.quarantined {
            return None;
        }
        let primitives = care.apply(&mut self.vitals);
        *self.history.entry(care).or_insert(0) += 1;
        self.stage = self.earned_stage();
        if !self.vitals.alive() {
            self.quarantined = true;
        }
        Some(primitives)
    }

    /// Lets one unit of time pass with no care at all.
    ///
    /// This is the escalation path. No intent arrived, so no primitive was
    /// dispatched, and the pet's state moved because nothing was served.
    pub fn neglect(&mut self) {
        if self.quarantined {
            return;
        }
        self.vitals.decay();
        if !self.vitals.alive() {
            self.quarantined = true;
        }
    }

    /// The stage the pet has earned from its completed sleep cycles.
    pub fn earned_stage(&self) -> Stage {
        let mut stage = Stage::Egg;
        loop {
            let Some(next) = stage.next() else { break };
            if self.vitals.age_ticks < stage.cycles_to_advance() {
                break;
            }
            stage = next;
        }
        stage
    }

    /// The care history as a phrase, for display.
    pub fn summary(&self) -> String {
        if self.history.is_empty() {
            return "nothing recorded".to_string();
        }
        self.history
            .iter()
            .map(|(c, n)| format!("{n}×{}", c.label()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_pet_is_an_egg() {
        let p = Pet::new("ca-001");
        assert_eq!(p.stage, Stage::Egg);
        assert!(p.vitals.alive());
        assert!(!p.quarantined);
    }

    #[test]
    fn feeding_reduces_hunger() {
        let mut v = Vitals {
            hunger: 0.9,
            ..Default::default()
        };
        Care::Feed.apply(&mut v);
        assert!(v.hunger < 0.9);
    }

    #[test]
    fn playing_raises_happiness() {
        let mut v = Vitals {
            happiness: 0.2,
            ..Default::default()
        };
        Care::Play.apply(&mut v);
        assert!(v.happiness > 0.2);
    }

    #[test]
    fn neglect_raises_hunger_and_lowers_happiness() {
        let mut v = Vitals::default();
        let (h0, p0) = (v.hunger, v.happiness);
        v.decay();
        assert!(v.hunger > h0);
        assert!(v.happiness < p0);
    }

    #[test]
    fn sleep_is_the_only_operation_that_advances_age() {
        let mut v = Vitals::default();
        for care in [Care::Feed, Care::Play, Care::Clean] {
            care.apply(&mut v);
        }
        assert_eq!(v.age_ticks, 0);
        Care::Sleep.apply(&mut v);
        assert_eq!(v.age_ticks, 1);
    }

    #[test]
    fn a_pet_that_never_sleeps_never_leaves_the_egg() {
        // Progress gated on a completed cycle, not on elapsed time.
        let mut p = Pet::new("ca-001");
        for _ in 0..50 {
            p.tend(Care::Feed);
            p.tend(Care::Play);
        }
        assert_eq!(p.stage, Stage::Egg);
        assert_eq!(p.vitals.age_ticks, 0);
    }

    #[test]
    fn sleep_cycles_advance_the_stage_at_cumulative_thresholds() {
        let mut p = Pet::new("ca-001");
        p.tend(Care::Sleep);
        assert_eq!(p.stage, Stage::Hatchling);
        p.tend(Care::Sleep);
        assert_eq!(p.stage, Stage::Hatchling, "two cycles is not yet three");
        p.tend(Care::Sleep);
        assert_eq!(p.stage, Stage::Juvenile);
        p.tend(Care::Sleep);
        p.tend(Care::Sleep);
        assert_eq!(p.stage, Stage::Adult);
    }

    #[test]
    fn the_final_stage_does_not_advance_further() {
        let mut p = Pet::new("ca-001");
        for _ in 0..40 {
            p.tend(Care::Sleep);
        }
        assert_eq!(p.stage, Stage::Adult);
        assert_eq!(Stage::Adult.next(), None);
    }

    #[test]
    fn long_neglect_kills_the_pet() {
        let mut p = Pet::new("ca-001");
        for _ in 0..60 {
            p.neglect();
        }
        assert!(!p.vitals.alive());
        assert!(p.quarantined);
    }

    #[test]
    fn a_dead_pet_is_retained_rather_than_deleted() {
        let mut p = Pet::new("ca-001");
        for _ in 0..60 {
            p.neglect();
        }
        assert_eq!(p.id, "ca-001");
        assert!(p.quarantined);
    }

    #[test]
    fn a_dead_pet_accepts_no_further_care() {
        let mut p = Pet::new("ca-001");
        for _ in 0..60 {
            p.neglect();
        }
        let before = p.vitals;
        assert_eq!(p.tend(Care::Feed), None);
        assert_eq!(p.vitals, before, "a refused operation must change nothing");
        assert_eq!(p.stage, Stage::Egg);
    }

    #[test]
    fn a_refused_operation_does_not_count_toward_the_history() {
        // A refused operation is not an interaction, and putting it in the
        // training signal would record an event that never happened.
        let mut p = Pet::new("ca-001");
        for _ in 0..60 {
            p.neglect();
        }
        p.tend(Care::Feed);
        assert!(p.history.is_empty());
    }

    #[test]
    fn a_tended_pet_outlives_a_neglected_one_over_the_same_span() {
        let mut kept = Pet::new("ca-001");
        let mut ignored = Pet::new("ca-002");
        for round in 0..30 {
            if round % 2 == 0 {
                kept.tend(Care::Feed);
            }
            if round % 3 == 0 {
                kept.tend(Care::Play);
            }
            if round % 4 == 3 {
                kept.tend(Care::Sleep);
            }
            ignored.neglect();
        }
        assert!(kept.vitals.alive());
        assert!(!ignored.vitals.alive());
        assert_eq!(kept.stage, Stage::Adult);
        assert_eq!(ignored.stage, Stage::Egg);
    }

    #[test]
    fn every_care_operation_reduces_to_a_primitive_sequence() {
        let mut v = Vitals::default();
        for care in [Care::Feed, Care::Play, Care::Clean, Care::Sleep] {
            assert!(
                !care.apply(&mut v).is_empty(),
                "{} reduced to nothing",
                care.label()
            );
        }
    }

    #[test]
    fn the_history_summarises_by_operation() {
        let mut p = Pet::new("ca-001");
        p.tend(Care::Feed);
        p.tend(Care::Feed);
        p.tend(Care::Sleep);
        assert_eq!(p.summary(), "2×feed, 1×sleep");
    }

    #[test]
    fn an_untended_pet_summarises_as_nothing_recorded() {
        assert_eq!(Pet::new("ca-001").summary(), "nothing recorded");
    }

    #[test]
    fn parses_every_care_name_and_rejects_an_unknown_one() {
        for care in [Care::Feed, Care::Play, Care::Clean, Care::Sleep] {
            assert_eq!(Care::parse(care.label()), Some(care));
        }
        assert_eq!(
            Care::parse("FEED"),
            Some(Care::Feed),
            "parsing is case-insensitive"
        );
        assert_eq!(Care::parse("burn"), None);
    }

    fn rule(address: &str, confidence: f64) -> LearnedRule {
        LearnedRule {
            address: address.into(),
            signature: "SetValue_CheckSense".into(),
            aliases: vec!["pour some kibble".into()],
            confidence,
            observations: 3,
        }
    }

    #[test]
    fn a_new_pet_knows_nothing() {
        // Kept is not the same as taught. A creature played with but never taught
        // has an empty education, and the two must not be confused.
        let p = Pet::new("ca-001");
        assert!(p.learned.is_empty());
    }

    #[test]
    fn learns_the_rules_it_is_given() {
        let mut p = Pet::new("ca-001");
        p.learn(vec![rule("addr-a", 0.8)]);
        assert_eq!(p.learned.len(), 1);
        assert_eq!(p.learned[0].address, "addr-a");
    }

    #[test]
    fn relearning_replaces_rather_than_accumulates() {
        // Induction is a function of the whole log, so re-deriving must not stack
        // duplicates of the same rule.
        let mut p = Pet::new("ca-001");
        p.learn(vec![rule("addr-a", 0.8)]);
        p.learn(vec![rule("addr-a", 0.9)]);
        assert_eq!(p.learned.len(), 1);
        assert_eq!(p.learned[0].confidence, 0.9);
    }

    #[test]
    fn survives_a_save_and_restore_with_what_it_learned() {
        let mut original = Pet::new("ca-001");
        original.tend(Care::Feed);
        original.tend(Care::Sleep);
        original.learn(vec![rule("addr-a", 0.7)]);

        let restored = Pet::from_json("ca-001", &original.to_json()).expect("round trips");
        assert_eq!(restored.id, "ca-001");
        assert_eq!(restored.vitals, original.vitals);
        assert_eq!(restored.learned, original.learned);
        assert_eq!(restored.stage, original.stage);
    }

    #[test]
    fn refuses_to_restore_a_corrupt_memory_rather_than_starting_afresh() {
        // Substituting a live egg for a creature that died would erase the one
        // part of it that cannot be regenerated, and the player would never learn
        // their pet had been swapped.
        assert!(Pet::from_json("ca-001", "{ not json").is_none());
        assert!(Pet::from_json("ca-001", "").is_none());
    }

    #[test]
    fn recomputes_an_earned_stage_on_restore() {
        // The stage is a function of sleep cycles, so a stored one that disagrees
        // is not trusted over the recomputed value.
        let mut p = Pet::new("ca-001");
        for _ in 0..3 {
            p.tend(Care::Sleep);
        }
        let json = p
            .to_json()
            .replace("\"stage\": \"egg\"", "\"stage\": \"hatchling\"");
        let restored = Pet::from_json("ca-001", &json).expect("round trips");
        assert_eq!(
            restored.stage,
            Stage::Juvenile,
            "recomputed, not read from the file"
        );
    }

    #[test]
    fn a_restored_pet_keeps_its_stage_and_history() {
        let mut original = Pet::new("ca-001");
        for _ in 0..3 {
            original.tend(Care::Sleep);
        }
        original.tend(Care::Feed);
        let restored = Pet::restore(
            "ca-001",
            original.vitals,
            original.quarantined,
            original.history.clone(),
        );
        assert_eq!(restored.stage, original.stage);
        assert_eq!(restored.summary(), original.summary());
    }
}

/// The question the pet asks, and the one its lineage is an answer to.
///
/// Not decoration. "Which came first" is a question about the root of a
/// derivation, and this crate has three honest answers to it rather than one,
/// which is a direct consequence of treating ancestry as a relation on content
/// addresses instead of a chain.
pub const MOTTO: &str =
    "Let's find together the answer to the ever question: what came first, the egg or the chicken?";

/// What the trace log can say about firstness.
///
/// The three cases are not degrees of confidence in one answer. They are three
/// different states of the evidence, and only the middle one is usually
/// interesting.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "case", rename_all = "snake_case")]
pub enum Firstness {
    /// Only one lineage exists, so the egg precedes the chicken trivially.
    ///
    /// True but uninteresting: a single pet always starts as an egg, so this says
    /// nothing that the stage counter does not already say.
    SingleLineage,
    /// Two or more lineages reached the same content address.
    ///
    /// The answer is *neither*, and the shared address is the egg of both. This
    /// is what convergent derivation means: two players who word their care the
    /// same way induce one artifact, so the chicken and the egg are the same
    /// thing and the ordering question dissolves rather than being decided.
    Converged {
        /// The address both lineages reached.
        shared: String,
        /// How many lineages reached it.
        lineages: usize,
    },
    /// The lineages share no address, so the log does not contain the answer.
    ///
    /// Reported rather than guessed. A shared *primitive* is not a shared
    /// ancestor, and treating the two as equivalent would invent a descent that
    /// never happened.
    Undetermined,
}

/// Decides what the induced candidates say about firstness.
///
/// @param addresses_by_player - The content addresses each lineage induced, in
///   the form `player -> [address, ...]`. Two players converging on one address
///   is the only evidence of a shared ancestor this function will accept.
pub fn firstness(addresses_by_player: &BTreeMap<String, Vec<String>>) -> Firstness {
    // Count how many distinct players reached each address. Counting players
    // rather than artifacts is the point: one player inducing the same rule
    // twice is repetition, not corroboration.
    let mut lineages_per_address: BTreeMap<&str, usize> = BTreeMap::new();
    for addresses in addresses_by_player.values() {
        let mut seen_by_this_player: BTreeMap<&str, ()> = BTreeMap::new();
        for a in addresses {
            seen_by_this_player.insert(a.as_str(), ());
        }
        for a in seen_by_this_player.keys() {
            *lineages_per_address.entry(a).or_insert(0) += 1;
        }
    }

    let converged = lineages_per_address
        .iter()
        .filter(|(_, n)| **n > 1)
        .max_by_key(|(a, n)| (**n, a.to_string()));

    match converged {
        Some((address, n)) => Firstness::Converged {
            shared: (*address).to_string(),
            lineages: *n,
        },
        // Exactly one lineage is the trivial case, and it is worth separating
        // from having no lineages at all: a single pet does precede its own
        // adulthood, whereas an empty log has no ordering to speak of.
        None if addresses_by_player.len() == 1 => Firstness::SingleLineage,
        None => Firstness::Undetermined,
    }
}

#[cfg(test)]
mod firstness_tests {
    use super::*;

    fn players(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        entries
            .iter()
            .map(|(p, addrs)| {
                (
                    (*p).to_string(),
                    addrs.iter().map(|a| (*a).to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn a_single_player_is_trivially_the_egg_first() {
        let f = firstness(&players(&[("ca-001", &["addr-a"])]));
        assert_eq!(f, Firstness::SingleLineage);
    }

    #[test]
    fn two_players_reaching_one_address_have_no_first() {
        // The interesting case. Content addressing collapses them, so the
        // chicken and the egg are the same artifact and the ordering question
        // has no answer rather than a preferred one.
        let f = firstness(&players(&[
            ("ca-001", &["shared"]),
            ("ca-002", &["shared"]),
        ]));
        assert_eq!(
            f,
            Firstness::Converged {
                shared: "shared".to_string(),
                lineages: 2
            }
        );
    }

    #[test]
    fn three_players_reaching_one_address_report_three_lineages() {
        let f = firstness(&players(&[
            ("ca-001", &["shared"]),
            ("ca-002", &["shared"]),
            ("ca-003", &["shared"]),
        ]));
        assert_eq!(
            f,
            Firstness::Converged {
                shared: "shared".to_string(),
                lineages: 3
            }
        );
    }

    #[test]
    fn one_player_repeating_a_rule_is_not_corroboration() {
        // Repetition is not a second lineage, and counting it as one would make
        // a single player's habits look like independent agreement.
        let f = firstness(&players(&[("ca-001", &["same", "same", "same"])]));
        assert_eq!(f, Firstness::SingleLineage);
    }

    #[test]
    fn two_players_reaching_different_addresses_leave_it_undetermined() {
        let f = firstness(&players(&[("ca-001", &["a"]), ("ca-002", &["b"])]));
        assert_eq!(f, Firstness::Undetermined);
    }

    #[test]
    fn a_shared_address_wins_over_unrelated_ones() {
        // The most corroborated address is the one that answers the question.
        let f = firstness(&players(&[
            ("ca-001", &["unique-a", "shared"]),
            ("ca-002", &["unique-b", "shared"]),
        ]));
        assert_eq!(
            f,
            Firstness::Converged {
                shared: "shared".to_string(),
                lineages: 2
            }
        );
    }

    #[test]
    fn no_players_at_all_is_undetermined() {
        assert_eq!(firstness(&BTreeMap::new()), Firstness::Undetermined);
    }

    #[test]
    fn the_motto_asks_the_question_it_can_answer() {
        assert!(MOTTO.contains("egg"));
        assert!(MOTTO.contains("chicken"));
    }
}

#[cfg(test)]
mod understanding_tests {
    use super::*;

    fn rule(signature: &str, aliases: &[&str]) -> LearnedRule {
        LearnedRule {
            address: "addr".into(),
            signature: signature.into(),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            confidence: 0.9,
            observations: aliases.len(),
        }
    }

    fn fed_pet() -> Pet {
        // A creature that has been fed several ways, and played with once.
        let mut p = Pet::new("ca-x");
        p.learn(vec![
            rule(
                "SetValue_CheckSense",
                &["pour some kibble", "serve the food", "refill the bowl"],
            ),
            rule("SetValue_Pulse", &["throw the ball"]),
        ]);
        p
    }

    mod understands {
        use super::*;

        #[test]
        fn a_wire_word_is_understood_without_any_evidence() {
            let p = Pet::new("ca-y");
            assert_eq!(
                p.understand("feed"),
                Understanding::Understood { care: Care::Feed }
            );
        }

        #[test]
        fn a_wire_word_is_matched_case_insensitively() {
            let p = Pet::new("ca-y");
            assert!(p.understand("SLEEP").acts().is_some());
        }

        #[test]
        fn a_learned_phrasing_resolves_to_the_act_it_was_fed_under() {
            let p = fed_pet();
            assert_eq!(
                p.understand("please pour some kibble now"),
                Understanding::UnderstoodByExperience {
                    care: Care::Feed,
                    matched: "pour some kibble".into(),
                }
            );
        }

        #[test]
        fn a_learned_phrasing_survives_surrounding_words() {
            // The player's sentence contains the act's phrasing and other words
            // besides; requiring the whole request to equal the alias would make
            // free text impossible.
            let p = fed_pet();
            assert_eq!(p.understand("pour some kibble").acts(), Some(Care::Feed));
            assert_eq!(
                p.understand("could you pour some kibble, it looks hungry")
                    .acts(),
                Some(Care::Feed)
            );
        }
    }

    mod distinguishes {
        use super::*;

        #[test]
        fn a_near_miss_says_what_would_work() {
            // The middle outcome: the creature has heard these words, just not in
            // this arrangement. "pour the kibble" shares its content terms with a
            // phrasing it knows, so it can say the request was nearly right.
            let p = fed_pet();
            let u = p.understand("pour the kibble in the bowl");

            assert!(u.is_refusal());
            assert_eq!(u.acts(), None);
            match u {
                Understanding::NeedsDifferentWords {
                    known,
                    learned_examples,
                } => {
                    assert!(known.contains(&"feed".to_string()), "{known:?}");
                    assert!(
                        learned_examples.contains(&"pour some kibble".to_string()),
                        "{learned_examples:?}"
                    );
                }
                other => panic!("expected NeedsDifferentWords, got {other:?}"),
            }
        }

        #[test]
        fn an_unrelated_request_says_it_has_never_heard_it() {
            // The far case. Nothing in the request resembles anything the
            // creature has been fed, so it says so rather than implying the
            // player was nearly right.
            let p = fed_pet();
            assert!(matches!(
                p.understand("frobnicate the widget"),
                Understanding::NeverHeardOf { .. }
            ));
        }

        #[test]
        fn it_cannot_tell_a_metaphor_from_nonsense_and_does_not_pretend_to() {
            // "give it a snack" is a request for something the creature does. With
            // no model it shares no vocabulary with what it has been fed, so it is
            // indistinguishable from nonsense. An earlier version of this test
            // asserted otherwise and was wrong: the distinction is not available
            // from the evidence the creature actually has, and claiming it would
            // be a guess dressed as a distinction. What it must do is say which
            // of the two it is, and offer the vocabulary either way.
            let p = fed_pet();
            for text in ["give it a snack", "frobnicate the widget"] {
                let u = p.understand(text);
                assert!(u.is_refusal(), "{text}: {u:?}");
                match u {
                    Understanding::NeverHeardOf {
                        known,
                        learned_examples,
                    } => {
                        assert_eq!(known.len(), 4, "{text}: {known:?}");
                        assert!(
                            learned_examples.contains(&"pour some kibble".to_string()),
                            "{text}: {learned_examples:?}"
                        );
                    }
                    other => panic!("{text}: expected NeverHeardOf, got {other:?}"),
                }
            }
        }

        #[test]
        fn a_creature_with_no_evidence_still_says_what_it_knows() {
            // Never heard of anything, but still not a bare "no": the four acts
            // are the whole of what it can do and saying so is more use than an
            // error.
            let p = Pet::new("ca-z");
            let u = p.understand("give it some kibble");
            assert!(u.is_refusal());
            match u {
                Understanding::NeverHeardOf {
                    known,
                    learned_examples,
                } => {
                    assert_eq!(known.len(), 4, "{known:?}");
                    assert!(learned_examples.is_empty(), "{learned_examples:?}");
                }
                other => panic!("expected NeverHeardOf, got {other:?}"),
            }
        }

        #[test]
        fn the_two_refusals_are_distinguishable_from_each_other() {
            // Conflating these is the failure the type exists to prevent: a
            // creature that cannot tell "nearly right" from "not addressed to
            // me" cannot tell the player whether trying again would help.
            let p = fed_pet();
            assert!(matches!(
                p.understand("pour the kibble in the bowl"),
                Understanding::NeedsDifferentWords { .. }
            ));
            assert!(matches!(
                p.understand("frobnicate the widget"),
                Understanding::NeverHeardOf { .. }
            ));
        }

        #[test]
        fn function_words_alone_do_not_make_a_request_look_half_heard() {
            // Every English sentence shares "the" and "it". Counting those as
            // overlap would make every request look nearly right, and the two
            // refusals would collapse into one.
            let p = fed_pet();
            assert!(matches!(
                p.understand("it is the widget"),
                Understanding::NeverHeardOf { .. }
            ));
        }

        #[test]
        fn an_empty_request_is_refused_rather_than_panicking() {
            let p = fed_pet();
            assert!(p.understand("   ").is_refusal());
        }
    }

    mod refuses_to_guess {
        use super::*;

        #[test]
        fn a_word_containing_another_act_s_name_does_not_match_it() {
            // The D6 trap, in a new place. `widget` contains no act word today,
            // but `sleep` inside `asleep` and `play` inside `player` are the live
            // cases, and a substring matcher would serve those.
            let p = fed_pet();
            assert_eq!(p.understand("the player is asleep").acts(), None);
        }

        #[test]
        fn an_act_it_never_learned_is_not_offered_as_known() {
            // The pet knows feeding and playing. Cleaning and sleeping it has
            // never been shown, so a learned phrase must not conjure them.
            let p = fed_pet();
            assert!(!matches!(
                p.understand("pour some kibble"),
                Understanding::UnderstoodByExperience {
                    care: Care::Clean,
                    ..
                }
            ));
        }

        #[test]
        fn a_rule_for_an_unknown_signature_is_ignored() {
            // A learned rule whose sequence is not an act the creature performs
            // is a real thing — induction produces them — and must not be
            // resolved into an act by taking the first match.
            let mut p = Pet::new("ca-w");
            p.learn(vec![rule("Toggle_Reset", &["flip the switch"])]);
            assert_eq!(p.understand("flip the switch").acts(), None);
        }
    }

    mod signatures_round_trip {
        use super::*;

        #[test]
        fn every_act_reads_back_from_its_own_sequence() {
            // The mapping is declared once. If `apply` and `from_signature` could
            // disagree, a learned rule would be recognised and then not acted on,
            // which is the unaddressable-signature bug this was written to close.
            for care in Care::all() {
                assert_eq!(Care::from_signature(care.signature()), Some(care));
            }
        }

        #[test]
        fn applying_an_act_produces_the_sequence_it_declares() {
            for care in Care::all() {
                let mut v = Vitals::default();
                let produced = care.apply(&mut v).join("_");
                assert_eq!(produced, care.signature(), "{}", care.label());
            }
        }
    }
}

#[cfg(test)]
mod primitive_vocabulary_tests {
    use super::*;
    use crate::bridge::primitive::UniversalPrimitive;

    /// Every name the game is capable of putting into a trace.
    fn declared_by_the_game() -> Vec<String> {
        Care::all()
            .iter()
            .flat_map(|c| c.signature().split('_').map(String::from))
            .collect()
    }

    #[test]
    fn every_primitive_the_game_emits_is_a_declared_one() {
        // P2, as an invariant rather than a promise. The game hands these to
        // `Trace.primitives`, which is the densest learning signal in the system
        // and the input to induction, so a name that is not in the vocabulary
        // becomes a learned rule the bridge can never resolve. `Emit` did exactly
        // that and reached a `LearnedRule` as `SetValue_Emit`.
        let names = declared_by_the_game();
        let mut seen = names.clone();
        seen.sort();
        seen.dedup();

        for name in &seen {
            let declared = serde_json::to_value(name)
                .ok()
                .and_then(|v| serde_json::from_value::<UniversalPrimitive>(v).ok())
                .is_some();
            assert!(
                declared,
                "the game emits {name:?}, which is not a UniversalPrimitive. \
                 A trace carrying it cannot be dispatched by anything."
            );
        }
    }

    #[test]
    fn applying_an_act_emits_only_declared_primitives() {
        // The same check on the actual output rather than on the declaration, so
        // a future edit to `apply` that bypasses `signature` is caught here.
        for care in Care::all() {
            let mut v = Vitals::default();
            for name in care.apply(&mut v) {
                let declared = serde_json::to_value(&name)
                    .ok()
                    .and_then(|v| serde_json::from_value::<UniversalPrimitive>(v).ok())
                    .is_some();
                assert!(declared, "{} emitted {name:?}", care.label());
            }
        }
    }

    #[test]
    fn the_sequence_the_game_records_is_the_sequence_it_declares() {
        // Guards against the declaration and the behaviour drifting apart, which
        // is how `SetValue_Emit` survived: nothing read one back from the other.
        for care in Care::all() {
            let mut v = Vitals::default();
            assert_eq!(
                care.apply(&mut v).join("_"),
                care.signature(),
                "{}",
                care.label()
            );
        }
    }
}

#[cfg(test)]
mod contradiction_tests {
    use super::*;

    fn pet() -> Pet {
        let mut p = Pet::new("ca-c");
        p.learn(vec![LearnedRule {
            address: "addr".into(),
            signature: Care::Feed.signature().into(),
            aliases: vec!["pour some kibble".into(), "feed the ca maduci".into()],
            confidence: 0.7,
            observations: 2,
        }]);
        p
    }

    #[test]
    fn naming_a_declared_field_asks_why_rather_than_listing_phrasings() {
        // The question, not a clarification. "Set its hunger to 0.2" is not a
        // mis-phrasing of "feed it", and answering it with a list of ways to say
        // feed would be answering a question nobody asked.
        let p = pet();
        let u = p.understand("set its hunger to 0.2");

        assert!(u.is_contradiction());
        assert!(u.is_refusal());
        assert_eq!(u.acts(), None);

        let why = u.why().expect("a contradiction always has a question");
        assert!(why.contains("set my hunger"), "{why}");
        assert!(why.contains("Nothing I do sets"), "{why}");
        assert!(why.contains("Which?"), "ends with a question: {why}");
    }

    #[test]
    fn the_question_names_both_branches_a_player_could_mean() {
        // Two things could be wrong — the wrong field, or the wrong creature —
        // and a player can answer either in one word.
        let p = pet();
        let why = p.understand("set its health to 1").why().unwrap();
        assert!(why.contains("you mean one of those"), "{why}");
        assert!(why.contains("confused with something else"), "{why}");
    }

    #[test]
    fn the_question_is_assembled_from_the_declarations() {
        // Built from what the creature declares, not from a hand-written sentence
        // about it, so a change to the state space changes the question.
        let p = pet();
        let why = p.understand("set its happiness to 0.9").why().unwrap();
        for (field, _) in Care::declared_state() {
            assert!(why.contains(field), "{why} does not name {field}");
        }
    }

    #[test]
    fn the_question_names_the_acts_that_do_change_the_field() {
        let p = pet();
        let why = p.understand("set its hunger to 0.2").why().unwrap();
        let changers = Care::changes("hunger");
        assert!(!changers.is_empty(), "something must change hunger");
        for label in changers {
            assert!(why.contains(label), "{why} omits {label}");
        }
    }

    #[test]
    fn no_act_accepts_a_value_and_that_is_the_point() {
        // The contradiction rests on a structural fact, not on a measurement:
        // `apply` takes no parameter, so no act can put a field where a caller
        // asked. The assignment below compiles *only* while that holds, so
        // adding a value to any act breaks the build here rather than quietly
        // making the creature's question wrong.
        //
        // This is the right way to say it because the first version of this test
        // tried to measure "did the act set the field" by probing, and produced a
        // predicate that was simply wrong: feeding left health at 1.0, which a
        // naive reading called a set. The fact is in the signature.
        let _unparameterised: fn(Care, &mut Vitals) -> Vec<String> = Care::apply;
    }

    #[test]
    fn every_declared_field_is_changed_by_at_least_one_act() {
        // If a field were changed by nothing, the creature could not maintain
        // it and the state space would be describing a fiction. This is the
        // check that the declared state and the act vocabulary agree.
        for (field, _) in Care::declared_state() {
            let changers = Care::changes(field);
            assert!(
                !changers.is_empty(),
                "{field} is declared but no act changes it"
            );
        }
    }

    #[test]
    fn a_contradiction_is_not_a_phrasing_miss() {
        // The distinction the type exists to keep: one is fixed by saying it
        // differently, the other is not.
        let p = pet();
        let contradiction = p.understand("set its hunger to 0.2");
        let phrasing = p.understand("pour the kibble in the bowl");

        assert!(contradiction.is_contradiction());
        assert!(!phrasing.is_contradiction());
        assert_eq!(contradiction.why().is_some(), true);
        assert_eq!(phrasing.why().is_some(), false);
        assert_ne!(contradiction, phrasing);
    }

    #[test]
    fn a_word_that_merely_contains_a_field_name_is_not_a_contradiction() {
        // `healthcare` contains `health`. Treating it as a request to set health
        // would fire the gate on ordinary words, which is the D6 trap again.
        let p = pet();
        assert!(!p
            .understand("the healthcare system is broken")
            .is_contradiction());
    }

    #[test]
    fn an_undeclared_field_is_not_detected_and_that_is_recorded() {
        // `brightness` is a real thing the creature does not have, and the
        // valve-manifest case is exactly this. It is indistinguishable from
        // gibberish without a model, so it falls through to NeverHeardOf rather
        // than being guessed at. Documented as a limit rather than papered over.
        let p = pet();
        let u = p.understand("set its brightness to eighty");
        assert!(!u.is_contradiction());
        assert!(matches!(u, Understanding::NeverHeardOf { .. }), "{u:?}");
    }
}
