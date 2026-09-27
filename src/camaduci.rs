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

use std::collections::BTreeMap;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Care {
    Feed,
    Play,
    Clean,
    Sleep,
}

impl Care {
    /// Parses an operation name, for a wire protocol.
    pub fn parse(s: &str) -> Option<Care> {
        match s.to_ascii_lowercase().as_str() {
            "feed" => Some(Care::Feed),
            "play" => Some(Care::Play),
            "clean" => Some(Care::Clean),
            "sleep" => Some(Care::Sleep),
            _ => None,
        }
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
                vec!["SetValue".into(), "CheckSense".into()]
            }
            Care::Play => {
                v.happiness = (v.happiness + 0.4).min(1.0);
                vec!["SetValue".into(), "Emit".into()]
            }
            Care::Clean => {
                v.happiness = (v.happiness + 0.1).min(1.0);
                v.health = (v.health + 0.03).min(1.0);
                vec!["SetValue".into()]
            }
            // Sleep is the only operation that advances age, and only if the pet
            // is alive to be put to sleep.
            Care::Sleep => {
                if v.alive() {
                    v.age_ticks += 1;
                    v.hunger = (v.hunger + 0.05).min(1.0);
                    v.happiness = (v.happiness + 0.05).min(1.0);
                }
                vec!["SetValue".into(), "GetState".into()]
            }
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
    pub fn restore(id: impl Into<String>, vitals: Vitals, quarantined: bool, history: BTreeMap<Care, u32>) -> Self {
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
        let mut v = Vitals { hunger: 0.9, ..Default::default() };
        Care::Feed.apply(&mut v);
        assert!(v.hunger < 0.9);
    }

    #[test]
    fn playing_raises_happiness() {
        let mut v = Vitals { happiness: 0.2, ..Default::default() };
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
            assert!(!care.apply(&mut v).is_empty(), "{} reduced to nothing", care.label());
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
        assert_eq!(Care::parse("FEED"), Some(Care::Feed), "parsing is case-insensitive");
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
        let json = p.to_json().replace("\"stage\": \"egg\"", "\"stage\": \"hatchling\"");
        let restored = Pet::from_json("ca-001", &json).expect("round trips");
        assert_eq!(restored.stage, Stage::Juvenile, "recomputed, not read from the file");
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
        let f = firstness(&players(&[("ca-001", &["shared"]), ("ca-002", &["shared"])]));
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
