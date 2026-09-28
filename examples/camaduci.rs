//! ca(R)maduci: a digital pet, and the smallest thing that exercises the whole loop.
//!
//! This is not a game yet. It is the part of one that touches the architecture:
//! a creature whose vitals are a declared state space, whose care operations are
//! primitives, and whose neglect is the *absence* of a call. That last property
//! is what makes it worth building — neglect is not a punishment the game
//! levies at will, it is simply what happens when nobody serves an intent.
//!
//! Three design points carry over from the research in `docs/camaduci.md`:
//!
//! * **Age advances on a completed sleep cycle, not on a clock.** A pet that is
//!   never put to sleep never reaches the next stage. Progress is gated on a
//!   finished interaction rather than on elapsed time.
//! * **Death is a lifecycle transition, not a deletion.** The pet is retained
//!   and inspectable, and the traces that killed it stay as evidence.
//! * **The clock is injected.** `now` is a parameter, not a call to the system
//!   clock, so the whole thing is deterministic and testable. That is the one
//!   thing a game and a fixture both need, which is why the game-or-fixture
//!   question does not have to be settled first.
//!
//! What it does *not* demonstrate: escalation rate at any interesting corpus
//! size, cross-model execution, or convergence between nodes. One pet is one
//! artifact with no peer to converge with.

use std::collections::BTreeMap;
use unia::mcp::store::{Store, Trace};

/// The creature's declared state space.
///
/// Every field here is a state variable a `.ure` manifest would declare, with
/// the same type and the same range. Keeping them in a struct rather than in a
/// manifest is the one place this example diverges from the design, and it is
/// deliberate: the manifest form is exercised by the corpus, and a struct keeps
/// the arithmetic readable.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vitals {
    /// 1.0 is starving. Rises on every tick the pet is not fed.
    hunger: f64,
    /// Falls on every tick the pet is not played with.
    happiness: f64,
    /// Falls when hunger or happiness bottoms out, and recovers slowly otherwise.
    health: f64,
    /// Completed sleep cycles. A stage cannot advance without one.
    age_ticks: u32,
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
    fn alive(&self) -> bool {
        self.health > 0.0
    }

    /// A one-word description of how the pet is doing, for the demo output.
    fn mood(&self) -> &'static str {
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

    /// Advances one unit of time.
    ///
    /// Decay is applied to the whole vector at once rather than per action, so
    /// the numbers a care operation sees are the numbers the next tick will
    /// start from.
    fn decay(&mut self) {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Egg,
    Hatchling,
    Juvenile,
    Adult,
}

impl Stage {
    /// The next stage, or `None` at the end of the line.
    fn next(self) -> Option<Stage> {
        match self {
            Stage::Egg => Some(Stage::Hatchling),
            Stage::Hatchling => Some(Stage::Juvenile),
            Stage::Juvenile => Some(Stage::Adult),
            Stage::Adult => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Stage::Egg => "egg",
            Stage::Hatchling => "hatchling",
            Stage::Juvenile => "juvenile",
            Stage::Adult => "adult",
        }
    }

    /// Sleep cycles needed to leave this stage.
    fn cycles_to_advance(self) -> u32 {
        match self {
            Stage::Egg => 1,
            Stage::Hatchling => 3,
            Stage::Juvenile => 5,
            Stage::Adult => 0,
        }
    }
}

/// A care operation. These are the primitives the pet exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Care {
    Feed,
    Play,
    Clean,
    Sleep,
}

impl Care {
    fn label(self) -> &'static str {
        match self {
            Care::Feed => "feed",
            Care::Play => "play",
            Care::Clean => "clean",
            Care::Sleep => "sleep",
        }
    }

    /// Applies the operation and returns the primitive sequence it reduced to.
    ///
    /// The sequence is the induction key: two different sentences that reduced to
    /// the same sequence are evidence for one rule, which is why it is returned
    /// rather than merely logged.
    fn apply(self, v: &mut Vitals) -> Vec<String> {
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
            // Sleep is the only operation that advances age, and it does so only
            // if the pet is alive to be put to sleep.
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

/// The pet, its lifecycle, and the trace log it writes.
struct Pet {
    id: String,
    vitals: Vitals,
    stage: Stage,
    /// A lifecycle value in the sense `unia::gc` uses one. Death moves the pet
    /// to `Quarantined`; it does not delete it, and the traces that killed it
    /// remain readable.
    quarantined: bool,
}

impl Pet {
    fn new(id: &str) -> Self {
        Pet {
            id: id.to_string(),
            vitals: Vitals::default(),
            stage: Stage::Egg,
            quarantined: false,
        }
    }

    /// Performs one care operation and records it as a trace.
    ///
    /// A dead pet accepts no operations and records nothing, which is what makes
    /// neglect a real consequence rather than a cosmetic one.
    fn tend(&mut self, care: Care, now: u64) -> Option<Vec<String>> {
        if self.quarantined {
            return None;
        }
        let primitives = care.apply(&mut self.vitals);
        if !self.vitals.alive() {
            self.quarantined = true;
        }
        Some(primitives)
    }

    /// Lets one unit of time pass with no care at all.
    ///
    /// This is the escalation path. No intent arrives, so no primitive is
    /// dispatched, and the pet's state moves because nothing was served.
    fn neglect(&mut self) {
        if self.quarantined {
            return;
        }
        self.vitals.decay();
        if !self.vitals.alive() {
            self.quarantined = true;
        }
    }

    /// The stage the pet has earned, given its completed sleep cycles.
    fn earned_stage(&self) -> Stage {
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
}

/// The shape a care history takes, which is what induction consumes.
type CareHistory = BTreeMap<Care, u32>;

/// Counts how often each operation was performed.
///
/// The counts are what `unia::induce` groups by: the primitive sequence is the
/// key and the observed phrasings become the aliases, so a history of many feeds
/// and few plays is evidence for a feeding rule and not for a play one.
fn summarise(history: &BTreeMap<Care, u32>) -> String {
    history
        .iter()
        .map(|(c, n)| format!("{}×{}", c.label(), n))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Builds a trace for one care operation, in the form `Store::record` writes.
fn trace_for(pet: &Pet, care: Care, primitives: &[String], succeeded: bool, now: u64) -> Trace {
    Trace {
        ts: now,
        intent: format!("{} the ca maduci", care.label()),
        resource_id: Some(pet.id.clone()),
        outcome: "hit".to_string(),
        tokens_in: 0,
        tokens_out: 0,
        primitives: primitives.to_vec(),
        succeeded,
    }
}

/// Runs the demonstration: a well-tended pet and a neglected one, side by side.
fn main() {
    let root = std::env::temp_dir().join("unia-camaduci-demo");
    let _ = std::fs::remove_dir_all(&root);
    let mut store = Store::open(&root);

    println!("ca(R)maduci — the same creature under two ways of being kept.\n");

    // A tended pet, tended on a schedule the demo drives.
    let mut kept = Pet::new("ca-001");
    let mut kept_history: CareHistory = BTreeMap::new();
    let mut now = 1_000u64;
    const ROUNDS: u32 = 24;

    for round in 0..ROUNDS {
        now += 60;
        if round % 2 == 0 {
            let care = Care::Feed;
            if let Some(p) = kept.tend(care, now) {
                *kept_history.entry(care).or_insert(0) += 1;
                let t = trace_for(&kept, care, &p, true, now);
                let _ = store.record(t);
            }
        }
        if round % 3 == 0 {
            let care = Care::Play;
            if let Some(p) = kept.tend(care, now) {
                *kept_history.entry(care).or_insert(0) += 1;
                let t = trace_for(&kept, care, &p, true, now);
                let _ = store.record(t);
            }
        }
        if round % 4 == 3 {
            let care = Care::Sleep;
            if let Some(p) = kept.tend(care, now) {
                *kept_history.entry(care).or_insert(0) += 1;
                let t = trace_for(&kept, care, &p, true, now);
                let _ = store.record(t);
            }
        }
    }
    kept.stage = kept.earned_stage();

    // The same number of rounds, with nobody doing anything.
    let mut ignored = Pet::new("ca-002");
    for _ in 0..ROUNDS {
        now += 60;
        ignored.neglect();
    }
    ignored.stage = ignored.earned_stage();

    report(&kept, &kept_history, "tended");
    report(&ignored, &BTreeMap::new(), "neglected");

    println!("\nThe traces are what induction would read:");
    println!("  ca-001  {}", summarise(&kept_history));
    println!("  ca-002  nothing recorded, so nothing to induce from");

    println!(
        "\nThis is the point of the example: both pets received the same number of\n\
         ticks. One has a trace log, the other has none. The difference is not\n\
         attention, it is that care was expressed as an intent and served by a\n\
         primitive. Nothing here demonstrates an escalation rate, and one pet is\n\
         not a corpus."
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// Prints one pet's state.
fn report(pet: &Pet, history: &CareHistory, label: &str) {
    println!("{label}:");
    println!("  stage       {}", pet.stage.label());
    println!("  age ticks   {}", pet.vitals.age_ticks);
    println!(
        "  vitals      hunger {:.2}  happiness {:.2}  health {:.2}",
        pet.vitals.hunger, pet.vitals.happiness, pet.vitals.health
    );
    println!("  mood        {}", pet.vitals.mood());
    println!(
        "  lifecycle   {}",
        if pet.quarantined {
            "quarantined (retained, not deleted)"
        } else {
            "served"
        }
    );
    if !history.is_empty() {
        println!("  care        {}", summarise(history));
    }
    println!();
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
        // The mechanic taken from tama96: progress is gated on a completed
        // cycle, not on elapsed time.
        let mut p = Pet::new("ca-001");
        for _ in 0..50 {
            p.tend(Care::Feed, 1);
            p.tend(Care::Play, 1);
        }
        assert_eq!(p.earned_stage(), Stage::Egg);
        assert_eq!(p.vitals.age_ticks, 0);
    }

    #[test]
    fn sleep_cycles_advance_the_stage_one_step_at_a_time() {
        // The thresholds are cumulative counts of completed cycles, not
        // additional cycles per stage: one leaves the egg, three reach juvenile,
        // five reach adult.
        let mut p = Pet::new("ca-001");
        p.tend(Care::Sleep, 1);
        assert_eq!(p.earned_stage(), Stage::Hatchling);
        p.tend(Care::Sleep, 2);
        assert_eq!(
            p.earned_stage(),
            Stage::Hatchling,
            "two cycles is not yet three"
        );
        p.tend(Care::Sleep, 3);
        assert_eq!(p.earned_stage(), Stage::Juvenile);
        p.tend(Care::Sleep, 4);
        p.tend(Care::Sleep, 5);
        assert_eq!(p.earned_stage(), Stage::Adult);
    }

    #[test]
    fn the_final_stage_does_not_advance_further() {
        let mut p = Pet::new("ca-001");
        for i in 0..40 {
            p.tend(Care::Sleep, i);
        }
        assert_eq!(p.earned_stage(), Stage::Adult);
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
        // Death is a lifecycle transition. The pet still exists and can still be
        // inspected, and its traces remain readable.
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
        assert_eq!(p.tend(Care::Feed, 1), None);
        assert_eq!(p.vitals, before, "a refused operation must change nothing");
    }

    #[test]
    fn a_tended_pet_outlives_a_neglected_one_over_the_same_span() {
        let mut kept = Pet::new("ca-001");
        let mut ignored = Pet::new("ca-002");
        for round in 0..30u32 {
            if round % 2 == 0 {
                kept.tend(Care::Feed, round as u64);
            }
            if round % 3 == 0 {
                kept.tend(Care::Play, round as u64);
            }
            if round % 4 == 3 {
                kept.tend(Care::Sleep, round as u64);
            }
            ignored.neglect();
        }
        assert!(kept.vitals.alive());
        assert!(!ignored.vitals.alive());
    }

    #[test]
    fn every_care_operation_records_a_primitive_sequence() {
        // The sequence is the induction key, so an operation that reduced to
        // nothing would be uninducible.
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
    fn the_trace_carries_the_sequence_and_the_resource() {
        let mut p = Pet::new("ca-001");
        let prims = p.tend(Care::Feed, 42).unwrap();
        let t = trace_for(&p, Care::Feed, &prims, true, 42);
        assert_eq!(t.resource_id.as_deref(), Some("ca-001"));
        assert_eq!(t.primitives, prims);
        assert_eq!(t.ts, 42);
        assert!(t.succeeded);
    }

    #[test]
    fn the_demo_history_summarises_by_operation() {
        let mut h: CareHistory = BTreeMap::new();
        *h.entry(Care::Feed).or_insert(0) += 1;
        *h.entry(Care::Feed).or_insert(0) += 1;
        *h.entry(Care::Sleep).or_insert(0) += 1;
        assert_eq!(summarise(&h), "feed×2, sleep×1");
    }
}
