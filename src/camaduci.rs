//! ca™maduci: a digital pet whose care is recorded as traces.
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// The pet, its lifecycle, and how it is kept.
#[derive(Debug, Clone)]
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
        }
    }

    /// Restores a pet from recorded state, for resuming a session.
    pub fn restore(id: impl Into<String>, vitals: Vitals, quarantined: bool, history: BTreeMap<Care, u32>) -> Self {
        let mut p = Pet {
            id: id.into(),
            vitals,
            stage: Stage::Egg,
            quarantined,
            history,
        };
        p.stage = p.earned_stage();
        p
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
