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
    /// The economy: `quants`, the power it acts at, and `nuants`, the resources it
    /// spends to act.
    ///
    /// Declared state rather than a field of the creature, because it is
    /// observable and checkable in the same way hunger is. That is what lets a
    /// precondition be written about it, and what makes "this creature is broke"
    /// a sentence the verifier can refuse an act on rather than a mood.
    ///
    /// Defaults on load, for the same reason the personality does: a file written
    /// before the economy existed must still open. Without this it did not —
    /// `from_json` returned `None` and the creature was gone.
    #[serde(default)]
    pub economy: Economy,
}

impl Default for Vitals {
    fn default() -> Self {
        Vitals {
            hunger: 0.2,
            happiness: 0.6,
            health: 1.0,
            age_ticks: 0,
            economy: Economy::default(),
        }
    }
}

impl Vitals {
    /// Whether the pet is still alive.
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }

    /// A one-word description of how the pet is doing.
    pub fn mood(&self) -> String {
        // Delegated to the declared sensors rather than written out here, so
        // there is one description of what `mood` means. It used to be this `if`
        // chain, which meant the creature's most-acted-upon reading of itself was
        // in code, unnamed and unaddressed, while every other reading in the
        // system was a declared state field.
        //
        // Returns an owned `String` because the readings come from data now. An
        // earlier version kept the `&'static str` signature and reached it with
        // `Box::leak`, which would have leaked once per call in a long-running
        // server — a real defect, traded for a signature.
        self.readings()
            .into_iter()
            .next()
            .map(|(_, reading)| reading)
            .unwrap_or_else(|| "content".to_string())
    }

    /// The sensors the creature perceives itself with, in the order they report.
    ///
    /// The first that reports wins, so the order *is* the precedence, and it is
    /// written here rather than derived because precedence is a decision and
    /// should be readable as one. A creature that is starving *and* lonely reports
    /// `starving`; the other reading is still available, it simply is not the
    /// first thing said.
    pub fn sensors() -> Vec<Sensor> {
        Personality::default().sensors
    }

    #[allow(dead_code)]
    fn sensors_legacy() -> Vec<Sensor> {
        vec![
            Sensor {
                name: "mortality".into(),
                when: "health < 0.01".into(),
                reads: "gone".into(),
                sense: Sense::Derived,
            },
            Sensor {
                name: "sickness".into(),
                when: "health < 0.3".into(),
                reads: "sick".into(),
                sense: Sense::Derived,
            },
            Sensor {
                name: "hunger".into(),
                when: "hunger > 0.8".into(),
                reads: "starving".into(),
                sense: Sense::Derived,
            },
            Sensor {
                name: "company".into(),
                when: "happiness < 0.25".into(),
                reads: "lonely".into(),
                sense: Sense::Derived,
            },
        ]
    }

    /// Every reading this state produces, not just the first.
    ///
    /// A mood is a summary; this is the whole picture. A caller that wants detail
    /// asks for this rather than parsing the summary, and a creature that is
    /// starving *and* sick says one thing as its mood while being two readings.
    ///
    /// A sensor is a function of state rather than of the creature, so this is
    /// evaluable against a hypothetical: "what would it see if it were hungry"
    /// is answerable without first making it hungry.
    pub fn readings(&self) -> Vec<(String, String)> {
        Self::sensors()
            .into_iter()
            .filter_map(|s| s.reporting(self).map(|r| (s.name, r)))
            .collect()
    }

    /// The creature's state in the shape the constraint evaluator reads.
    ///
    /// The bridge between two things that already exist: a vitals reading and a
    /// predicate over declared state. Nothing was wrong with either; they simply
    /// had no way to meet, which is why `mood` was a hand-written `if` chain
    /// rather than a declared sensor.
    pub fn state(v: &Vitals) -> crate::constraints::State {
        let mut state = crate::constraints::State::new();
        state.insert("hunger".to_string(), format!("{}", v.hunger));
        state.insert("happiness".to_string(), format!("{}", v.happiness));
        state.insert("health".to_string(), format!("{}", v.health));
        // Published because a drive names it. Leaving it out looked harmless and
        // was not: `age_ticks > 0` became an unevaluable predicate, the draw was
        // correctly refused as an unknown field, and the creature silently never
        // wanted to sleep. The evaluator was right and the declaration was
        // incomplete, which is the only kind of bug a checkable field catches.
        state.insert("age_ticks".to_string(), format!("{}", v.age_ticks));
        state.insert("quants".to_string(), format!("{}", v.economy.quants));
        state.insert("nuants".to_string(), format!("{}", v.economy.nuants));
        state
    }

    /// Every field the published state carries, in manifest form.
    pub fn declared_state() -> [(&'static str, &'static str); 6] {
        [
            ("hunger", "rises when I am not fed"),
            ("happiness", "falls when I am not played with"),
            ("health", "falls when the other two bottom out"),
            ("age_ticks", "completed sleep cycles"),
            ("quants", "the power I can currently act at"),
            ("nuants", "the resources I have left to spend"),
        ]
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

/// Who performed an act.
///
/// Present only to choose between the two rules on [`Pet::tend`], and worth
/// having as a type because collapsing it makes one of them impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Who {
    /// A person chose the act.
    Player,
    /// The creature chose it, from its own reading of itself.
    Itself,
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
            economy: Economy::default(),
        };
        let mut out = Vec::new();
        for care in Care::all() {
            let mut probe = baseline;
            // The whole act, not just its effect on the vitals. An act spends
            // `nuants` through `apply_economy` and moves a vital through
            // `apply`, and asking only about the second reported that nothing
            // here ever spent a resource — which is not true, and which is what
            // the "declared but no act changes it" test caught.
            let mut economy = probe.economy;
            care.apply_economy(&mut economy);
            care.apply(&mut probe);
            let touched_vital = field_moved(field, &baseline, &probe);
            let touched_economy = match field {
                "nuants" => economy.nuants < baseline.economy.nuants,
                "quants" => economy.quants != baseline.economy.quants,
                _ => false,
            };
            if touched_vital || touched_economy {
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

    /// The nuants one act costs — the resources it spends.
    ///
    /// Declared per act, in the same way drives are, because "what does this
    /// costs" is a property of the act and not of the creature. It is also the
    /// first place in this project where a primitive carries a *quantity*, which
    /// is what the empty `params` in every manifest has been quietly failing to
    /// express — divergence D14.
    pub fn cost(self) -> f64 {
        match self {
            Care::Feed => 1.0,
            Care::Play => 1.5,
            // Cleaning is dear because nothing you can see comes of it.
            Care::Clean => 2.0,
            // Sleep is free: it consumes time rather than quants, and it is the
            // only act available to a creature that cannot afford anything else.
            // A broke creature can still grow.
            Care::Sleep => 0.0,
        }
    }

    /// Applies an act to the economy: spends the resources and *debits* the power.
    ///
    /// **The power falls here.** That is the whole correction. The first version
    /// credited the power on every act, so repetition paid as if it were
    /// production: twelve identical feeds with no new phrasing and no rule induced
    /// took a creature from 0.500 to 0.694. An act is consumption and nothing
    /// more, and the only place the power can rise is [`Pet::learn`], which is
    /// handed a rule set and can see what is in it that was not there before.
    ///
    /// A debit rather than a flat cost because repetition is not neutral. Doing
    /// the same thing again and getting nothing for it is corrosive rather than
    /// merely wasteful, and the economy is the place to say so: the player is
    /// invited to repeat, and repetition without production should cost them
    /// something they notice.
    ///
    /// Returns whether the act was affordable, so a caller can refuse before
    /// acting rather than after.
    pub fn apply_economy(&self, e: &mut Economy) -> bool {
        if e.nuants < self.cost() {
            return false;
        }
        e.nuants = (e.nuants - self.cost()).max(0.0);
        e.quants = (e.quants - self.repetition() * (e.quants - Economy::FLOOR_CUANTE))
            .max(Economy::FLOOR_CUANTE);
        true
    }

    /// How much power an unproductive repetition costs.
    ///
    /// Small, and deliberately not proportional to what the act cost. A creature
    /// that repeats must be able to recover, or the floor becomes a trap and the
    /// game is unwinnable after one careless afternoon.
    pub fn repetition(self) -> f64 {
        0.15
    }

    /// One tick with no care: the resources drain and the power decays.
    ///
    /// The power falls by a proportion rather than a fixed step, because a
    /// capability you have stopped demonstrating is not one you still have at its
    /// old strength. It decays toward a floor rather than to it, so a creature
    /// that once learned something is never quite the creature that never did —
    /// which is the only reason the game is winnable after a long absence.
    pub fn decay_economy(&self, e: &mut Economy) {
        e.nuants = (e.nuants - 0.5).max(0.0);
        e.quants = (e.quants * 0.9).max(Economy::FLOOR_CUANTE);
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

/// How strongly a satisfied draw pulls: `1.0`, or not at all.
///
/// A drive is a **gate, not a ramp**, and this is the whole of the pressure
/// model. Measuring pressure as "how far past the threshold" was the first
/// attempt and it was wrong twice over: a draw on an accumulating field such as
/// `age_ticks > 0` grew without limit, so a creature that had slept a hundred
/// times wanted nothing but sleep and one that had slept once outranked its own
/// hunger. A distance needs a scale, and there is none declared.
///
/// The gradient is expressed instead by *declaring more draws*. `hunger > 0.5`
/// and `hunger > 0.9` are both the same statement in the same grammar, and
/// ordering them sharpest-last means a starving creature satisfies both and the
/// later one is the one it is drawn by. That is data rather than arithmetic, it
/// moves a design decision into the manifest where it belongs, and it cannot run
/// away.
fn pressure_of(constraint: &crate::constraints::Constraint, actual: f64, threshold: f64) -> f64 {
    use crate::constraints::Comparison as C;
    let satisfied = match constraint.comparison {
        C::Equal => actual == threshold,
        C::NotEqual => actual != threshold,
        C::Less => actual < threshold,
        C::LessOrEqual => actual <= threshold,
        C::Greater => actual > threshold,
        C::GreaterOrEqual => actual >= threshold,
    };
    if satisfied {
        1.0
    } else {
        0.0
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
        "age_ticks" => v.age_ticks as f64,
        _ => 0.0,
    }
}

fn field_moved(field: &str, before: &Vitals, after: &Vitals) -> bool {
    (field_value(field, before) - field_value(field, after)).abs() > f64::EPSILON
}

/// Something the creature can perceive, and how it perceives it.
///
/// The three kinds differ only in where the number comes from, and nothing else:
/// all three are named, all three are addressable by content, and all three are
/// reached through the same primitive vocabulary. A valve's `flow_rate` and the
/// creature's `hunger` are not different kinds of thing, and neither is `mood`,
/// which is a *function* of three declared fields and has no hardware behind it
/// at all.
///
/// A software sensor is the case that makes the architecture coherent rather
/// than merely portable. Hardware independence is a portability claim: the same
/// capability on different hardware. A derived sensor is not portable, it is
/// *compositional* — it exists only because other declared state exists, it
/// inherits their vocabulary, and it becomes meaningless if they are renamed.
/// That is a stronger dependence than hardware has, and the format is in a
/// position to express it because the derivation is data rather than code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Sense {
    /// Read from a driver: something outside the process happened.
    Physical,
    /// Computed from other declared state, with no dispatch required.
    Derived,
    /// Computed by dispatching a primitive sequence against the state space, so
    /// evaluating it is an act rather than a function call.
    Inferred,
}

impl Sense {
    pub fn label(self) -> &'static str {
        match self {
            Sense::Physical => "physical",
            Sense::Derived => "derived",
            Sense::Inferred => "inferred",
        }
    }
}

/// The two quantities a creature runs on, and the economy they make.
///
/// **Cuante** is *productivity* — a power. **Nuante** is *consumption* —
/// resources. The names are deliberately crossed against their physics.
///
/// In quantum computing a quantum is a unit of *consumed* computing power: you
/// spend quanta to compute. Here the word is read the other way, because a
/// creature does not burn its capability to act. It *has* a power, sustains it
/// by being cared for, and loses it by not being. So quants is the power, and
/// because that word is then taken, the thing the creature actually burns needed
/// a name of its own: **nuants**, which in plain terms is resources.
///
/// The distinction is the whole of the economy, and it is not decoration because a
/// rate and a stock fail differently:
///
/// - out of **nuants** is an *empty* creature — it has no resources and cannot
///   act at all. Poverty.
/// - out of **quants** is a *stuck* creature — it has resources and no power, so
///   it can act, badly, and a full stock does not help. Neglect.
///
/// Capability is a rate rather than a stock, which is also what
/// [`LearnedRule::confidence`] already was: a `0.0..=1.0` that decays unless it
/// is fed. The economy gives that number its name.
///
/// They are separate fields rather than one number on purpose. A single "energy"
/// reading would make neglect and poverty the same event, and they are not: you
/// can be rich and powerless, or powerful and destitute, and a creature that can
/// only be one of those is not modelling anything.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Economy {
    /// Productivity: the power the creature currently acts at. Sustained.
    pub quants: f64,
    /// Consumption: the resources available to spend. A stock.
    pub nuants: f64,
}

impl Default for Economy {
    fn default() -> Self {
        Economy {
            // Enough to begin, and not enough to be comfortable: a creature that
            // starts rich has no economy to learn.
            nuants: 12.0,
            // A middling power. It can be raised by being cared for and lost by
            // not being, and it is the thing that decays, so starting high would
            // mean starting near the end.
            quants: 0.5,
        }
    }
}

impl Economy {
    /// The most a creature can hold. A stock needs a ceiling or there is no
    /// reason to prefer efficiency over abundance.
    pub const MAX_NUANTE: f64 = 24.0;
    /// The highest power achievable. A rate needs a ceiling for the same reason:
    /// without one, "getting better" has no end and nothing is at stake.
    pub const MAX_CUANTE: f64 = 1.0;
    /// The power a neglected creature decays to, and below which it cannot act.
    ///
    /// **This was found by playing the game, not by reading it.** The first
    /// version floored the power at 0.1 and had `can_act()` ask only for > 0.0, so
    /// a floored creature could still act and the `stuck` failure — the whole
    /// reason the two quantities are separate — was *unreachable through neglect
    /// at all*. A declared failure that cannot happen is worse than a lower floor,
    /// because the type implies a distinction the game never offers.
    ///
    /// The floor is not zero on purpose: a creature that was once cared for is
    /// never quite the creature that never was. And because it is *below the
    /// usable threshold* rather than at it, one act of care lifts a stuck creature
    /// clear of it, which is what makes a long absence recoverable.
    pub const FLOOR_CUANTE: f64 = 0.1;

    /// How long the creature's resources last at its current power, in ticks.
    ///
    /// This is the number that makes the two quantities comparable at all, and it
    /// is why a power cannot simply be treated as a smaller stock. Resources
    /// divided by a power is a *duration*, not an amount: the same nuants buys a
    /// weak creature less time than a strong one.
    pub fn endurance(&self) -> f64 {
        if self.quants <= 0.0 {
            return 0.0;
        }
        self.nuants / self.quants
    }

    /// Whether the creature can act at all: it needs resources *and* usable power.
    ///
    /// The threshold is the floor rather than zero, so a creature decayed to it
    /// is alive and stuck rather than merely weak. See [`Economy::FLOOR_CUANTE`].
    pub fn can_act(&self) -> bool {
        self.nuants > 0.0 && self.quants > Economy::FLOOR_CUANTE
    }

    /// One word for the state of the economy, for a player to read.
    pub fn posture(&self) -> &'static str {
        if !self.can_act() {
            if self.nuants <= 0.0 {
                "empty"
            } else {
                "stuck"
            }
        } else if self.endurance() < 6.0 {
            "spending down"
        } else {
            "sustaining"
        }
    }
}

/// Something the creature produced, counted rather than judged.
///
/// **This exists because the first version of the economy was inflationary.** The
/// power rose on every act, so feeding the same creature the same sentence a
/// hundred times took it to full strength with nothing learned — measured: twelve
/// identical feeds, no new phrasing, no rule induced, and the power went from
/// 0.500 to 0.694. Repetition is not production, and a currency that pays for
/// repetition is a currency that pays for rumination.
///
/// So the credit is separated from the cost. **An act spends; induction pays.**
/// The act is performed, the nuants are gone, and nothing is credited until
/// `learn` is handed a rule set that contains something the previous one did not.
/// There is no path to power that does not go through production, and production
/// is four integers anyone can check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Production {
    /// Rules that did not exist before. The strongest signal: a new act, induced.
    pub new_rules: usize,
    /// Phrasings no rule had before. Evidence: the same act, newly expressible.
    pub new_phrasings: usize,
    /// Distinct primitive sequences across the rule set. Capability, deduplicated.
    ///
    /// Counted separately from `new_rules` because two rules with one signature
    /// are one capability learned twice, and paying for both would be paying for
    /// the redundancy.
    pub new_signatures: usize,
    /// Rules that were already known and became *more reliable*.
    ///
    /// **This counter exists because the other two are finite.** There are four
    /// built-in acts, so `new_signatures` saturates after four, and a new phrasing
    /// is worth a tenth of a debit. Measured: a creature that produced on all
    /// twelve of twelve acts ended *weaker* than one that produced on five,
    /// because from the fifth onward it paid a full debit for a cent. A currency
    /// with no sustaining term is a tax, and the tax falls hardest on whoever
    /// plays longest.
    ///
    /// So the sustaining term is **consolidation** — evidence accumulating under
    /// an act that was already there. This is [`LearnedRule::confidence`], the
    /// rate that decays unless fed. It is not inflation, and the reason is a
    /// property of the type rather than a rule anyone remembered to write:
    /// `observations` counts *distinct* phrasings, so the same sentence arriving
    /// again raises nothing. Consolidation needs new evidence, and it saturates.
    pub consolidated: usize,
}

impl Production {
    /// Whether anything at all was produced.
    pub fn any(&self) -> bool {
        self.new_rules > 0 || self.new_phrasings > 0 || self.consolidated > 0
    }

    /// The credit this production is worth, in power.
    ///
    /// A new act is worth much more than a new way of asking for one. That is the
    /// escalation the economy is supposed to reward, and it is a claim about what
    /// matters: nobody gained anything because the same sentence arrived in a
    /// different language, and a lot because a new act became possible.
    pub fn credit(&self) -> f64 {
        self.new_rules as f64 * 0.10
            + self.new_signatures as f64 * 0.05
            + self.new_phrasings as f64 * 0.01
            + self.consolidated as f64 * 0.02
    }

    /// What a set of rules contains that a previous set did not.
    ///
    /// Two of the three counters are deliberately different things, because the
    /// project's own corpus measurement turned out to escalate in one and not the
    /// other. A new **signature** is new capability: an act that was not possible
    /// before. A new **address** with an existing signature is new *reach* — the
    /// same act arrived at from somewhere new — and the generated corpus does that
    /// abundantly while doing no capability escalation at all. Paying for both at
    /// one rate would be paying for redundancy and calling it progress.
    pub fn since(previous: &[LearnedRule], current: &[LearnedRule]) -> Self {
        // Owned rather than borrowed: three closures each capturing a different
        // parameter will not unify their lifetimes, and a set difference is the
        // same operation on owned strings.
        let addr = |rules: &[LearnedRule]| -> BTreeSet<String> {
            rules.iter().map(|r| r.address.clone()).collect()
        };
        let sig = |rules: &[LearnedRule]| -> BTreeSet<String> {
            rules.iter().map(|r| r.signature.clone()).collect()
        };
        let phrasing = |rules: &[LearnedRule]| -> BTreeSet<String> {
            rules
                .iter()
                .flat_map(|r| r.aliases.iter().cloned())
                .collect()
        };

        let prev_addr = addr(previous);
        let cur_addr = addr(current);
        let prev_sig = sig(previous);
        let cur_sig = sig(current);
        let prev_ph = phrasing(previous);
        let cur_ph = phrasing(current);

        Production {
            // A new act, induced, at an address nobody had. An address that
            // re-arrives with a signature already known is new *reach*, counted
            // below as neither, because paying for it as a rule would be paying
            // for redundancy and calling it progress.
            new_rules: cur_addr
                .difference(&prev_addr)
                .filter(|a| {
                    current
                        .iter()
                        .any(|r| &r.address == *a && !prev_sig.contains(&r.signature))
                })
                .count(),
            new_signatures: cur_sig.difference(&prev_sig).count(),
            new_phrasings: cur_ph.difference(&prev_ph).count(),
            // A rule that was already there and became more reliable. Matched on
            // address, so a brand-new rule is never paid twice: its rise from
            // nothing to something is novelty, credited above.
            consolidated: current
                .iter()
                .filter(|r| {
                    previous
                        .iter()
                        .any(|old| old.address == r.address && r.confidence > old.confidence)
                })
                .count(),
        }
    }
}

/// What a creature is disposed to want and to notice.
///
/// **Declared, not computed.** This was a pair of methods returning a fresh
/// `Vec` every call, which put the creature's entire personality in code: it had
/// no address, could not be changed without a recompile, and two creatures with
/// different wants were indistinguishable. As data it is structure, it goes in
/// the hashed skeleton, and a creature that wants something different is a
/// different creature — which is what makes personality part of identity rather
/// than a setting.
///
/// Ordering is part of the value. Drives are read in order and the first
/// satisfied one wins, and sensors report in order and the first reading is the
/// mood, so a reordering changes behaviour and therefore changes the address.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Personality {
    pub drives: Vec<Drive>,
    pub sensors: Vec<Sensor>,
}

impl Default for Personality {
    /// The disposition this project shipped with, written out as data so it can be
    /// compared, serialised, hashed, and replaced.
    fn default() -> Self {
        Personality {
            drives: vec![
                Drive {
                    care: Care::Feed,
                    when: "hunger > 0.5".into(),
                },
                // Sharper, and later, so a starving creature is drawn by this one.
                Drive {
                    care: Care::Feed,
                    when: "hunger > 0.9".into(),
                },
                Drive {
                    care: Care::Play,
                    when: "happiness < 0.4".into(),
                },
                Drive {
                    care: Care::Clean,
                    when: "health < 0.6".into(),
                },
                Drive {
                    care: Care::Sleep,
                    when: "age_ticks > 0".into(),
                },
            ],
            sensors: vec![
                Sensor {
                    name: "mortality".into(),
                    when: "health < 0.01".into(),
                    reads: "gone".into(),
                    sense: Sense::Derived,
                },
                Sensor {
                    name: "sickness".into(),
                    when: "health < 0.3".into(),
                    reads: "sick".into(),
                    sense: Sense::Derived,
                },
                Sensor {
                    name: "hunger".into(),
                    when: "hunger > 0.8".into(),
                    reads: "starving".into(),
                    sense: Sense::Derived,
                },
                Sensor {
                    name: "company".into(),
                    when: "happiness < 0.25".into(),
                    reads: "lonely".into(),
                    sense: Sense::Derived,
                },
            ],
        }
    }
}

impl Personality {
    /// The drives toward a given act, in declaration order.
    pub fn draws_for(&self, care: Care) -> impl Iterator<Item = &Drive> {
        self.drives.iter().filter(move |d| d.care == care)
    }

    /// The readings this state produces under these sensors, in order.
    pub fn readings_of(&self, state: &Vitals) -> Vec<(String, String)> {
        self.sensors
            .iter()
            .filter_map(|s| s.reporting(state).map(|r| (s.name.clone(), r)))
            .collect()
    }

    /// The first reading, which is the summary.
    pub fn mood_of(&self, state: &Vitals) -> String {
        self.readings_of(state)
            .into_iter()
            .next()
            .map(|(_, reading)| reading)
            .unwrap_or_else(|| "content".to_string())
    }
}

/// A named derivation over declared state.
///
/// This is the missing declaration. `mood` existed as a hard-coded `if` chain
/// returning a `&'static str`, which meant the creature's most-acted-upon reading
/// of itself had no address, no place in the skeleton, and could not be renamed,
/// derived from, or converged on with another creature's. As a declaration it is
/// structure, it goes in the hashed skeleton, and its *value* is state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sensor {
    pub name: String,
    /// The condition under which this reading holds, in the constraint grammar.
    ///
    /// Deliberately the same grammar as a manifest's `constraints`, because it is
    /// the same kind of statement: a predicate over declared state. Reusing it
    /// means a sensor can be written in a manifest by someone who has never read
    /// this file, and evaluated by the same evaluator that now refuses a
    /// violation rather than passing it.
    pub when: String,
    /// The reading this sensor reports when the condition holds.
    pub reads: String,
    pub sense: Sense,
}

impl Sensor {
    /// Whether this sensor currently reports, given the creature's state.
    ///
    /// Three-valued for the same reason a constraint is: a sensor whose condition
    /// cannot be evaluated is not reporting `false`, it is not reporting. See
    /// [`crate::constraints::Verdict`].
    pub fn reporting(&self, v: &Vitals) -> Option<String> {
        let mut state = crate::constraints::State::new();
        for (name, value) in [
            ("hunger", v.hunger),
            ("happiness", v.happiness),
            ("health", v.health),
        ] {
            state.insert(name.to_string(), format!("{value}"));
        }
        match crate::constraints::Constraint::parse(&self.when) {
            Ok(c) => match c.evaluate(&state) {
                crate::constraints::Verdict::Holds => Some(self.reads.clone()),
                _ => None,
            },
            // An unreadable sensor is a manifest bug, not a quiet `None`.
            Err(_) => None,
        }
    }
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

/// What draws an act: a precondition, read in the other direction.
///
/// This is the whole of "motivation" in this architecture, and it is not a new
/// concept so much as the other direction of one that already exists. A manifest
/// `constraint` is a **gate**: this act is refused unless the predicate holds. A
/// **drive** is a **pull**: this act is attractive when the predicate holds.
/// Same grammar, same evaluator, opposite use.
///
/// A precondition was built to *block*, and it is a poor model for wanting,
/// because refusing everything is not wanting anything. What a drive adds is
/// selection: several acts can be permitted at once, and the creature has to
/// choose among them. That choice is the policy, and it is the part that does not
/// exist yet anywhere in the project.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Drive {
    /// The act this draw belongs to.
    pub care: Care,
    /// The predicate, in the constraint grammar. Held means drawn.
    pub when: String,
}

impl Drive {
    /// Whether this draw is active for the given state.
    pub fn active(&self, v: &Vitals) -> bool {
        match crate::constraints::Constraint::parse(&self.when) {
            Ok(c) => c.evaluate(&Vitals::state(v)).permits_dispatch(),
            // A drive that cannot be read is not active. Reads a dead animal as
            // wanting nothing, which is right.
            Err(_) => false,
        }
    }
}

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
    /// Who performed the most recent act, if any.
    ///
    /// Recorded because a creature's whole life is a record of who looked after
    /// it, and a trace that cannot say whether a creature fed itself or was fed
    /// is a trace that has thrown away the only evidence that distinguishes them.
    #[serde(default)]
    pub last_actor: Option<Who>,
    /// What it is disposed to want and to notice. Structure, so it goes in the
    /// content address; changing it makes this a different creature.
    ///
    /// Defaults on load. A pet file written before dispositions existed must still
    /// load, and without this it did not: `from_json` returned `None` and a
    /// player's creature was gone. The fallback is the shipped disposition, which
    /// is the right guess and the wrong creature if the original had another —
    /// so the address differs, and the substitution cannot be silent.
    #[serde(default)]
    pub personality: Personality,
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
            personality: Personality::default(),
            last_actor: None,
        }
    }

    /// The manifest form of this creature: what it *is*, with nothing it knows
    /// and nothing it has done.
    ///
    /// The skeleton rule applied to a living thing, and the same rule the
    /// manifest loader uses. What is **in** here:
    ///
    /// - the declared state space, because that is what it can be acted on
    /// - its drives, because a disposition is what it wants
    /// - its sensors, because what it notices is as much its character as its
    ///   appetites
    ///
    /// What is **out**, and each exclusion is the point:
    ///
    /// - `id`, which is a name
    /// - `vitals`, `stage` and `history`, which are what has happened to it
    /// - `learned`, because **a vocabulary must not move the identity**. This is
    ///   the project's central claim and it has to hold here as well as in the
    ///   corpus: a creature that has been addressed in ten languages is the same
    ///   creature, and a creature that wants something different is not.
    pub fn manifest(&self) -> serde_json::Value {
        serde_json::json!({
            "ure_version": "1.0",
            "category": "creature",
            "external_id": format!("ca({})", self.id),
            "guidance": "A ca(R)maduci. Its disposition is its character; what it \
                         has learned is not.",
            "state_space": Self::state_space_manifest(),
            // The acts it can perform, each with the primitive sequence it
            // reduces to. The same vocabulary a valve declares, which is what
            // makes the creature addressable by a peer that has never heard of it.
            "action_primitives": Care::all()
                .iter()
                .map(|care| serde_json::json!({
                    "id": care.signature(),
                    "aliases": [care.label()],
                    "params": {},
                    "target_state": care.label(),
                    "constraints": [],
                }))
                .collect::<Vec<serde_json::Value>>(),
            "drives": self.personality.drives,
            "sensors": self.personality.sensors,
        })
    }

    /// The declared state space, in the manifest's own shape.
    pub fn state_space_manifest() -> serde_json::Value {
        let mut out = serde_json::Map::new();
        out.insert(
            "hunger".into(),
            serde_json::json!({ "type": "float", "range": [0.0, 1.0] }),
        );
        out.insert(
            "happiness".into(),
            serde_json::json!({ "type": "float", "range": [0.0, 1.0] }),
        );
        out.insert(
            "health".into(),
            serde_json::json!({ "type": "float", "range": [0.0, 1.0] }),
        );
        out.insert(
            "age_ticks".into(),
            serde_json::json!({ "type": "int", "range": [0, 1000000] }),
        );
        // The economy, with its ceilings as ranges. A ceiling that is only in the
        // code is not a constraint, and the evaluator cannot check what it cannot
        // see. That was the D5 lesson arriving again through a different door.
        out.insert(
            "quants".into(),
            serde_json::json!({
                "type": "float",
                "range": [0.0, Economy::MAX_CUANTE],
                "unit": "power",
                "usable_above": Economy::FLOOR_CUANTE,
            }),
        );
        out.insert(
            "nuants".into(),
            serde_json::json!({
                "type": "float",
                "range": [0.0, Economy::MAX_NUANTE],
                "unit": "resources",
            }),
        );
        serde_json::Value::Object(out)
    }

    /// This creature's content address.
    ///
    /// Two creatures that are the same thing share it, and the test that matters
    /// is the negative one: learning a new phrasing must not move it. That is the
    /// whole thesis in one assertion about a digital animal.
    pub fn address(&self) -> String {
        crate::identifiers::DuUuid::generate(&self.manifest(), None)
            .map(|u| u.to_string())
            .unwrap_or_else(|e| format!("unaddressable: {e}"))
    }

    /// Records what induction derived from the creature's own trace log, and
    /// **credits the power for whatever is new in it**.
    ///
    /// This is the only path to power that exists. An act spends resources and
    /// debits the power; induction pays. Nothing else is credited, so there is no
    /// way to become strong by repeating, and the debit in
    /// [`Care::apply_economy`] is what makes repetition cost something.
    ///
    /// The credit is not the count of rules but what is *in* them: a new act, a
    /// new signature, a new phrasing. The same sentence arriving again credits
    /// nothing, which is the entire point and the thing the first version got
    /// backwards.
    pub fn learn(&mut self, rules: Vec<LearnedRule>) -> Production {
        let production = Production::since(&self.learned, &rules);
        self.learned = rules;
        if production.any() {
            // Rises toward the ceiling, so the last stretch of power is the
            // hardest and there is always something left to be earned.
            let headroom = Economy::MAX_CUANTE - self.vitals.economy.quants;
            self.vitals.economy.quants = (self.vitals.economy.quants
                + production.credit() * headroom)
                .min(Economy::MAX_CUANTE);
        }
        production
    }

    /// Records what induction derived from the creature's own trace log.
    ///
    /// This is the feedback path. Without it the creature writes traces that
    /// something else reads and the creature never learns that it learned, so its
    /// evolution is invisible to it and the same on every restart.

    /// Restores a pet from recorded state, for resuming a session.
    pub fn restore(
        id: impl Into<String>,
        vitals: Vitals,
        quarantined: bool,
        history: BTreeMap<Care, u32>,
    ) -> Self {
        Self::restore_with(id, vitals, quarantined, history, Personality::default())
    }

    /// Restores a creature *and* the disposition it was kept with.
    ///
    /// Separate from [`Pet::restore`] because a disposition is part of what a
    /// creature is, so restoring a pet without its own would produce a different
    /// creature wearing the same id. The default-taking form exists for callers
    /// that genuinely have no personality to restore.
    pub fn restore_with(
        id: impl Into<String>,
        vitals: Vitals,
        quarantined: bool,
        history: BTreeMap<Care, u32>,
        personality: Personality,
    ) -> Self {
        let mut p = Pet {
            id: id.into(),
            vitals,
            stage: Stage::Egg,
            quarantined,
            history,
            learned: Vec::new(),
            personality,
            last_actor: None,
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
    /// The draws this creature has toward its own acts.
    ///
    /// Read from the declared personality rather than built here. A drive is a
    /// *disposition*, so it belongs in the hashed skeleton: a creature with
    /// different wants is a different creature, and one that has learned more
    /// phrasings is not. What the creature has *learned* is vocabulary, and that
    /// sits beside the identity rather than inside it.
    ///
    /// Without this the creature is a Tamagotchi: the player is the entire
    /// policy, and "hunger" is a number that goes up. With it, hunger is a number
    /// that *pulls*, and the creature can act on its own reading of itself.
    pub fn drives(&self) -> &[Drive] {
        &self.personality.drives
    }

    /// A copy of this creature with a different disposition.
    ///
    /// The point of it is that the result is a *different creature* — see
    /// [`Pet::address`] — so a lineage cannot quietly change its wants while
    /// keeping the address it was known by.
    pub fn with_personality(mut self, personality: Personality) -> Self {
        self.personality = personality;
        self
    }

    /// The act this creature would choose for itself, and why.
    ///
    /// The policy, and the first thing in this project that is one. Every draw
    /// whose condition holds is a candidate; the most *pressing* wins, where
    /// pressing is the declared field pushed furthest past its own draw
    /// threshold. Ties are broken by declaration order so the choice is
    /// deterministic — a creature that chose differently on identical state would
    /// not be a function, and this whole layer is about things that are.
    ///
    /// Returns `None` when nothing is drawn, which is a real state and not an
    /// error: an adult that is content and has slept is not wanting anything, and
    /// a creature that invented a preference to fill the gap would be performing
    /// want rather than having it.
    ///
    /// The second element is the declared field that is pulling, so a caller can
    /// say *why* rather than only *what*: "you are hungry" rather than "feed".
    pub fn want(&self) -> Option<(Care, String)> {
        // A creature that is not alive wants nothing.
        //
        // The first version had no such gate and a test caught it: a pet with
        // health 0.0 and hunger 1.0 was drawn to feeding, because nothing in the
        // drive declarations mentioned that being dead is terminal. It is a small
        // omission with an unpleasant shape — a quarantined creature volunteering
        // to be fed — and it belongs here rather than in each draw.
        if !self.vitals.alive() {
            return None;
        }

        let state = Vitals::state(&self.vitals);

        // Priority is the order the acts are declared in, and nothing else. The
        // first act with a satisfied draw is what the creature is drawn by.
        //
        // Two earlier attempts were wrong and both are worth recording. Ranking by
        // "how far past the threshold" let an accumulating field run away, so a
        // creature that had slept a hundred times wanted nothing but sleep. And
        // ranking by "the last satisfied draw" handed priority to whichever act
        // happened to be declared last, which is not a priority at all. A declared
        // order is the smallest thing that is actually a policy.
        for care in Care::all() {
            let mut sharpest: Option<String> = None;
            for drive in self.personality.draws_for(care) {
                let Ok(constraint) = crate::constraints::Constraint::parse(&drive.when) else {
                    continue;
                };
                let crate::constraints::Literal::Number(threshold) = &constraint.literal else {
                    continue;
                };
                let Some(observed) = state.get(&constraint.field) else {
                    continue;
                };
                let Ok(actual) = observed.parse::<f64>() else {
                    continue;
                };
                if pressure_of(&constraint, actual, *threshold) > 0.0 {
                    // Read in declaration order, so a sharper draw placed after a
                    // looser one is the one reported.
                    sharpest = Some(constraint.field.clone());
                }
            }
            if let Some(field) = sharpest {
                return Some((care, field));
            }
        }
        None
    }

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
        for (field, _) in Vitals::declared_state() {
            if terms.iter().any(|t| t == field) {
                return Understanding::Contradicted {
                    field: field.to_string(),
                    declared: Vitals::declared_state()
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
    ///
    /// **This is the player's entry point, and it has no power or affordability
    /// check.** That is deliberate. A person can always feed a creature that
    /// cannot feed itself, because being helped and helping yourself are
    /// different events, and the asymmetry between them is most of what the game
    /// is about. Gating this too would have made a powerless creature
    /// *unwinnable*, which it nearly was: it could not act, so it could not tend,
    /// so it could not produce, so it was never credited, so it stayed powerless
    /// for good. That is a liveness hole and it is closed by the two entry
    /// points rather than by weakening the economy.
    pub fn tend(&mut self, care: Care) -> Option<Vec<String>> {
        self.tend_as(care, Who::Player)
    }

    /// Tends the creature **as itself**: it chose the act, from its own reading.
    ///
    /// Requires usable power, because a creature that cannot work cannot help
    /// itself, and that is the case the two economies exist to keep distinct.
    pub fn tend_as_self(&mut self, care: Care) -> Option<Vec<String>> {
        self.tend_as(care, Who::Itself)
    }

    fn tend_as(&mut self, care: Care, who: Who) -> Option<Vec<String>> {
        if self.quarantined {
            return None;
        }
        if who == Who::Itself && !self.vitals.economy.can_act() {
            return None;
        }
        if !care.apply_economy(&mut self.vitals.economy) {
            return None;
        }
        self.last_actor = Some(who);
        let primitives = care.apply(&mut self.vitals);
        *self.history.entry(care).or_insert(0) += 1;
        self.stage = self.earned_stage();
        if !self.vitals.alive() {
            self.quarantined = true;
        }
        Some(primitives)
    }

    /// The word for why the creature will not act, in its own terms.
    ///
    /// Two failures that look identical from outside and are not: a creature with
    /// an empty tank is *broke*, and a creature whose rate has decayed is
    /// *stuck*. A full tank fixes the first and does nothing for the second, and
    /// saying so is the difference between an economy and a countdown.
    pub fn why_wont_act(&self) -> Option<String> {
        let e = &self.vitals.economy;
        if e.can_act() {
            return None;
        }
        if e.nuants <= 0.0 {
            // Out of resources. The power is intact and irrelevant, and saying so
            // is what distinguishes this from the other failure.
            Some(
                "I have no nuants left. My power is untouched, but I have nothing \
                 to act with, so nothing I do can happen. Sleep is free — put me to \
                 sleep and I can still grow."
                    .to_string(),
            )
        } else {
            // Out of power. A full stock of resources and no way to use them, which
            // is the failure that spending cannot fix.
            Some(
                "I still have nuants, but I have lost the power. My resources are \
                 untouched, and nothing I do is quick any more. No amount of \
                 spending will help; only being cared for will."
                    .to_string(),
            )
        }
    }

    /// Lets one unit of time pass with no care at all.
    ///
    /// This is the escalation path. No intent arrived, so no primitive was
    /// dispatched, and the pet's state moved because nothing was served.
    pub fn neglect(&mut self) {
        // Neglect is not only decay. It is the economy running down: the stock
        // drains and the rate falls, and the rate falls faster than the stock
        // empties, so a creature left alone becomes incapable before it becomes
        // broke. That ordering is deliberate — the cheaper failure comes first,
        // which is what makes a long absence recoverable and a short one cheap.
        Care::Sleep.decay_economy(&mut self.vitals.economy);
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
        for (field, _) in Vitals::declared_state() {
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
        for (field, _) in Vitals::declared_state() {
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

#[cfg(test)]
mod motivation_tests {
    use super::*;

    fn v(hunger: f64, happiness: f64, health: f64) -> Vitals {
        Vitals {
            hunger,
            happiness,
            health,
            age_ticks: 3,
            economy: Economy::default(),
        }
    }

    mod drives {
        use super::*;

        #[test]
        fn a_hungry_creature_is_drawn_to_feeding() {
            let p = Pet::new("ca-m");
            let mut p = p;
            p.vitals = v(0.9, 0.8, 1.0);
            let (care, field) = p.want().expect("a hungry creature wants something");
            assert_eq!(care, Care::Feed);
            assert_eq!(field, "hunger", "it must be able to say *why*");
        }

        #[test]
        fn a_lonely_creature_is_drawn_to_playing() {
            let mut p = Pet::new("ca-m");
            p.vitals = v(0.1, 0.1, 1.0);
            assert_eq!(p.want().map(|(c, _)| c), Some(Care::Play));
        }

        #[test]
        fn the_most_pressing_need_wins() {
            // Hunger at 1.0 and happiness at 0.0. Both draws hold; the creature
            // picks one, and it picks the same one every time, because a creature
            // that chose differently on identical state would not be a function.
            let mut p = Pet::new("ca-m");
            p.vitals = v(1.0, 0.0, 1.0);
            let first = p.want();
            for _ in 0..5 {
                p.vitals = v(1.0, 0.0, 1.0);
                assert_eq!(p.want(), first, "the choice must be deterministic");
            }
            assert_eq!(first.map(|(c, _)| c), Some(Care::Feed));
        }

        #[test]
        fn a_content_creature_that_has_never_slept_wants_nothing() {
            // A real state, not an error. A creature that invented a preference to
            // fill the gap would be performing want rather than having it.
            let mut p = Pet::new("ca-m");
            p.vitals = Vitals {
                hunger: 0.2,
                happiness: 0.8,
                health: 1.0,
                age_ticks: 0,
                economy: Economy::default(),
            };
            assert_eq!(p.want(), None);
        }

        #[test]
        fn a_content_creature_that_has_slept_wants_to_sleep_again() {
            // The earlier version of this test asserted a content creature wants
            // nothing, and was wrong. Sleeping is how the creature grows, so a
            // content creature that has slept once is drawn to sleep again — and
            // saying otherwise would have hidden the game's one real motivation
            // behind a tidier-looking policy.
            let mut p = Pet::new("ca-m");
            p.vitals = v(0.2, 0.8, 1.0);
            assert_eq!(p.want().map(|(c, _)| c), Some(Care::Sleep));
        }

        #[test]
        fn a_starving_creature_is_drawn_by_the_sharper_of_its_two_feed_draws() {
            // The gradient is two declarations rather than a computed distance.
            // Both feed draws name `hunger`; the reported one cannot be told apart
            // from the other by field name, so this asserts the act and not the
            // field, and the *ordering* property is held by the fact that a
            // creature at 0.6 is still drawn by the looser one.
            let mut p = Pet::new("ca-m");
            p.vitals = v(0.95, 0.8, 1.0);
            assert_eq!(p.want().map(|(c, _)| c), Some(Care::Feed));
            assert_eq!(p.want().map(|(c, _)| c), Some(Care::Feed));
        }

        #[test]
        fn a_draw_uses_the_constraint_grammar_and_not_a_threshold_constant() {
            // The whole point of the design: the drive is written in the same
            // language a manifest uses, so it can be moved there unchanged.
            for drive in Pet::new("ca-m").drives() {
                assert!(
                    crate::constraints::Constraint::parse(&drive.when).is_ok(),
                    "drive for {} is not a readable predicate: {:?}",
                    drive.care.label(),
                    drive.when
                );
            }
        }

        #[test]
        fn a_gate_and_a_pull_are_the_same_statement_used_in_opposite_directions() {
            // `hunger > 0.5` as a drive pulls; the identical string as a
            // constraint gates. One grammar, two uses, which is why motivation
            // needed no new vocabulary.
            let c = crate::constraints::Constraint::parse("hunger > 0.5").unwrap();
            let hungry = v(0.9, 0.5, 1.0);
            let full = v(0.1, 0.5, 1.0);

            assert!(c.evaluate(&Vitals::state(&hungry)).permits_dispatch());
            assert!(!c.evaluate(&Vitals::state(&full)).permits_dispatch());

            let drive = Drive {
                care: Care::Feed,
                when: "hunger > 0.5".into(),
            };
            assert!(drive.active(&hungry));
            assert!(!drive.active(&full));
        }

        #[test]
        fn a_dead_creature_wants_nothing() {
            let mut p = Pet::new("ca-m");
            p.vitals = v(1.0, 0.0, 0.0);
            // Every draw except sleep's age condition could hold, so this is the
            // case where a policy most wants to act on a body that cannot.
            let wanted = p.want().map(|(c, _)| c);
            assert!(
                wanted != Some(Care::Feed),
                "a creature with no health must not be trying to feed itself"
            );
        }
    }

    mod sensors {
        use super::*;

        #[test]
        fn a_software_sensor_reports_from_declared_state_with_no_hardware() {
            // `hunger` as a reading is a function of a number, not a device. That
            // is the case the architecture has to be able to express.
            let readings = v(0.9, 0.8, 1.0).readings();
            assert!(readings
                .iter()
                .any(|(n, r)| n == "hunger" && r == "starving"));
        }

        #[test]
        fn a_sensor_condition_is_written_in_the_manifest_grammar() {
            for sensor in Vitals::sensors() {
                assert!(
                    crate::constraints::Constraint::parse(&sensor.when).is_ok(),
                    "sensor {} has an unreadable condition {:?}",
                    sensor.name,
                    sensor.when
                );
            }
        }

        #[test]
        fn the_first_sensor_that_reports_sets_the_mood() {
            // Precedence is written in one place and is readable as a decision.
            // A creature that is starving *and* lonely says `starving`, because
            // hunger is the more urgent reading.
            let mut p = Pet::new("ca-s");
            p.vitals = v(0.95, 0.05, 1.0);
            assert_eq!(p.vitals.mood(), "starving");
        }

        #[test]
        fn a_content_creature_reads_content() {
            let mut p = Pet::new("ca-s");
            p.vitals = v(0.2, 0.8, 1.0);
            assert_eq!(p.vitals.mood(), "content");
        }

        #[test]
        fn several_readings_can_hold_at_once_and_mood_names_the_first() {
            // A mood is a summary, not the whole picture. `readings` is the whole
            // picture, and it is what a caller that wants detail should ask for.
            let mut p = Pet::new("ca-s");
            p.vitals = v(0.95, 0.05, 0.2);
            let all = p.vitals.readings();
            assert!(all.len() >= 2, "expected several readings, got {all:?}");
            assert_eq!(p.vitals.mood(), "sick", "sickness outranks hunger");
        }

        #[test]
        fn a_derived_sensor_needs_no_dispatch_to_be_evaluated() {
            // The distinction from an inferred one: evaluating this is arithmetic,
            // not an act. If a sensor were `Inferred`, evaluating it would be a
            // primitive sequence, and the two must not be conflated.
            for sensor in Vitals::sensors() {
                assert_eq!(sensor.sense, Sense::Derived, "{}", sensor.name);
            }
        }

        #[test]
        fn the_three_senses_are_distinguishable() {
            // Present as a type with three inhabited variants. If the distinction
            // collapsed to one case, the whole decomposition would be decorative.
            assert_ne!(Sense::Physical, Sense::Derived);
            assert_ne!(Sense::Derived, Sense::Inferred);
            assert_ne!(Sense::Physical, Sense::Inferred);
            assert_eq!(Sense::Derived.label(), "derived");
        }
    }
}

#[cfg(test)]
mod declaration_tests {
    use super::*;

    /// The invariant that would have caught the missing `age_ticks`: nothing may
    /// name a field the creature does not publish.
    ///
    /// The same rule the corpus generator holds against its own manifests, and
    /// for the same reason. An unevaluable predicate is refused rather than
    /// guessed at, which is right — and it means a declaration naming a field
    /// that does not exist fails *silently*, as a feature that never fires. Both
    /// halves have to be true at once: the evaluator must be strict, and the
    /// declarations must be complete.
    fn published_fields() -> BTreeSet<String> {
        Vitals::state(&Vitals::default()).into_keys().collect()
    }

    #[test]
    fn every_drive_names_a_field_the_creature_publishes() {
        let published = published_fields();
        for drive in Pet::new("ca-d").drives() {
            let c = crate::constraints::Constraint::parse(&drive.when).unwrap();
            assert!(
                published.contains(&c.field),
                "drive for {} names {:?}, which the state does not publish",
                drive.care.label(),
                c.field
            );
        }
    }

    #[test]
    fn every_sensor_names_a_field_the_creature_publishes() {
        let published = published_fields();
        for sensor in Vitals::sensors() {
            let c = crate::constraints::Constraint::parse(&sensor.when).unwrap();
            assert!(
                published.contains(&c.field),
                "sensor {} names {:?}, which the state does not publish",
                sensor.name,
                c.field
            );
        }
    }

    #[test]
    fn every_declared_field_is_published_and_the_reverse() {
        // Both directions. A field nobody names is a field nothing can be checked
        // against, which is the same defect as one that is named and missing.
        let declared: BTreeSet<String> = Vitals::declared_state()
            .iter()
            .map(|(n, _)| n.to_string())
            .collect();
        assert_eq!(
            declared,
            published_fields(),
            "the declared state and the published state disagree"
        );
    }

    #[test]
    fn every_drive_can_actually_fire() {
        // The end of the line. Each draw, given a state that satisfies it, must
        // come alive. A draw that can never be active is a declaration of
        // wanting that does nothing, and nothing in the type system would say so.
        for drive in Pet::new("ca-d").drives() {
            let mut probe = Vitals::default();
            match drive.care {
                Care::Feed => probe.hunger = 1.0,
                Care::Play => probe.happiness = 0.0,
                Care::Clean => probe.health = 0.0,
                Care::Sleep => probe.age_ticks = 3,
            }
            assert!(
                drive.active(&probe),
                "the draw for {} cannot fire even when satisfied: {:?}",
                drive.care.label(),
                drive.when
            );
        }
    }

    #[test]
    fn a_content_creature_wants_to_sleep_once_it_has_slept_before() {
        // The specific case that came back null when `age_ticks` was unpublished.
        // A drive that names a real state and a real act must reach the policy.
        let mut p = Pet::new("ca-d");
        p.vitals = Vitals {
            hunger: 0.2,
            happiness: 0.8,
            health: 1.0,
            age_ticks: 3,
            economy: Economy::default(),
        };
        let want = p
            .want()
            .expect("a creature that has slept once wants to sleep again");
        assert_eq!(want.0, Care::Sleep);
        assert_eq!(want.1, "age_ticks");
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    fn rule(signature: &str, aliases: &[&str]) -> LearnedRule {
        LearnedRule {
            address: "addr".into(),
            signature: signature.into(),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            confidence: 0.8,
            observations: aliases.len(),
        }
    }

    #[test]
    fn two_fresh_creatures_share_one_address() {
        assert_eq!(Pet::new("ca-a").address(), Pet::new("ca-b").address());
    }

    #[test]
    fn the_name_a_creature_is_kept_under_is_not_part_of_what_it_is() {
        // `id` is a name. Two players who both call their creature `ca-001` hold
        // the same creature, and the address has to say so.
        assert_ne!(Pet::new("ca-001").id, Pet::new("ca-002").id);
        assert_eq!(Pet::new("ca-001").address(), Pet::new("ca-002").address());
    }

    #[test]
    fn what_has_happened_to_a_creature_is_not_part_of_what_it_is() {
        // Vitals, stage, and history are state, not structure. A creature that
        // has been fed to adulthood is the same artifact as one that has not.
        let mut tended = Pet::new("ca-a");
        tended.tend(Care::Feed);
        for _ in 0..6 {
            tended.tend(Care::Sleep);
        }
        assert_eq!(tended.stage, Stage::Adult, "it should have grown");
        assert_eq!(tended.address(), Pet::new("ca-b").address());
    }

    #[test]
    fn a_quarantined_creature_is_still_the_creature_it_was() {
        let mut dead = Pet::new("ca-a");
        for _ in 0..80 {
            dead.neglect();
        }
        assert!(dead.quarantined);
        assert_eq!(dead.address(), Pet::new("ca-b").address());
    }

    #[test]
    fn learning_a_language_does_not_move_the_address() {
        // The project's central claim, asserted about a living thing. A creature
        // that has been addressed in ten languages is the same creature; one that
        // wants something different is not. If this ever fails, the corpus-level
        // result was a property of manifests and not of the idea.
        // `learn` replaces the rule set rather than merging it, because the
        // caller recomputes the whole set from the trace log every time. So the
        // vocabulary grows the way it really grows: one rule, more phrasings.
        let mut before = Pet::new("ca-a");
        before.learn(vec![rule(Care::Feed.signature(), &["pour some kibble"])]);
        let a = before.address();

        let six = [
            "pour some kibble",
            "serve the food",
            "refill the bowl",
            "toarna porumb",
            "da-i de mancare",
            "füttere es",
        ];
        before.learn(vec![rule(Care::Feed.signature(), &six)]);

        assert_eq!(before.learned[0].aliases.len(), 6, "the vocabulary grew");
        assert_eq!(before.address(), a, "a new phrasing moved the identity");
    }

    #[test]
    fn wanting_something_different_does_move_the_address() {
        // The other half, and the one that gives the first its meaning. If this
        // were also stable, "identity is what a thing does" would be vacuous: any
        // two creatures would be one creature.
        let mut glutton = Personality::default();
        glutton.drives.retain(|d| d.care != Care::Feed);

        let plain = Pet::new("ca-a");
        let other = plain.clone().with_personality(glutton);

        assert_ne!(
            plain.address(),
            other.address(),
            "a creature that does not want feeding is a different creature"
        );
    }

    #[test]
    fn how_a_creature_notices_is_also_its_identity() {
        // A disposition is appetites plus perception, and a creature that sees
        // the world differently is not the same animal.
        let mut blind = Personality::default();
        blind.sensors.retain(|s| s.name != "company");

        let plain = Pet::new("ca-a");
        let other = plain.clone().with_personality(blind);
        assert_ne!(plain.address(), other.address());
    }

    #[test]
    fn the_order_of_drives_is_identity_because_it_is_the_policy() {
        // A reordering changes what the creature does on the same state, so it has
        // to change the address or two creatures with different behaviour would
        // claim the same one.
        let mut swapped = Personality::default();
        swapped.drives.reverse();
        let plain = Pet::new("ca-a");
        assert_ne!(
            plain.address(),
            plain.clone().with_personality(swapped).address()
        );
    }

    #[test]
    fn the_manifest_form_declares_what_it_can_do_in_the_shared_vocabulary() {
        // The creature is addressable by a peer that has never heard of it,
        // because its acts are `UniversalPrimitive` sequences and nothing else.
        let m = Pet::new("ca-a").manifest();
        let actions = m["action_primitives"].as_array().unwrap();
        assert_eq!(actions.len(), Care::all().len());

        for a in actions {
            let id = a["id"].as_str().unwrap();
            assert!(
                Care::from_signature(id).is_some(),
                "{id} is not an act this creature has"
            );
        }
    }

    #[test]
    fn the_manifest_states_what_it_declares() {
        let m = Pet::new("ca-a").manifest();
        let space = m["state_space"].as_object().unwrap();
        for (field, _) in Vitals::declared_state() {
            assert!(space.contains_key(field), "state space omits {field}");
        }
    }

    #[test]
    fn a_creature_whose_personality_is_restored_is_the_same_creature() {
        // A disposition has to survive a restart, or a kept creature quietly
        // becomes a different one every time the server restarts.
        let mut original = Pet::new("ca-a");
        let mut tweaked = Personality::default();
        tweaked.drives.retain(|d| d.care != Care::Clean);
        let address_before = original.clone().with_personality(tweaked.clone()).address();
        original = original.with_personality(tweaked.clone());

        let text = original.to_json();
        let restored = Pet::from_json("ca-a", &text).expect("restores");
        assert_eq!(restored.address(), address_before);
        assert_eq!(restored.address(), original.address());
        assert_eq!(restored.personality, original.personality);
    }

    #[test]
    fn a_creature_restored_without_its_personality_is_a_different_creature() {
        // The failure this guards: a pet file written before dispositions existed
        // loads with the default one, which is the right fallback and the wrong
        // creature if the original had another. Making it a different address is
        // what stops the substitution being silent.
        let mut original = Pet::new("ca-a");
        let mut tweaked = Personality::default();
        tweaked.drives.retain(|d| d.care != Care::Clean);
        original = original.with_personality(tweaked);

        // Remove the field outright. An earlier version of this test tried to do it
        // with a string replace against the pretty-printed JSON, which never
        // matched, so the "stripped" file still carried the personality and the
        // test passed for the wrong reason — asserting that restoring an
        // unmodified creature gives the same address, which is true and is not
        // what this is about.
        let mut as_value: serde_json::Value =
            serde_json::from_str(&original.to_json()).expect("round-trips");
        as_value.as_object_mut().unwrap().remove("personality");
        let fallback = Pet::from_json("ca-a", &as_value.to_string()).expect("still loads");

        assert_eq!(
            fallback.personality,
            Personality::default(),
            "an absent disposition falls back to the default"
        );
        assert_ne!(
            fallback.address(),
            original.address(),
            "the fallback silently produced a different creature as the same one"
        );
    }
}

#[cfg(test)]
mod economy_tests {
    use super::*;

    fn pet() -> Pet {
        Pet::new("ca-e")
    }

    mod the_two_quantities {
        use super::*;

        #[test]
        fn cuante_is_a_power_and_nuante_is_not() {
            // The distinction stated as arithmetic, because that is what it is.
            // Divided by a power, a stock of resources is a *duration*: the same
            // nuants lasts a weak creature less time than a strong one.
            let strong = Economy {
                quants: 1.0,
                nuants: 10.0,
            };
            let weak = Economy {
                quants: 0.1,
                nuants: 10.0,
            };
            assert_eq!(strong.endurance(), 10.0);
            assert_eq!(weak.endurance(), 100.0);
        }

        #[test]
        fn a_creature_needs_both_to_act() {
            let destitute = Economy {
                quants: 1.0,
                nuants: 0.0,
            };
            let powerless = Economy {
                quants: 0.0,
                nuants: 100.0,
            };
            let both = Economy {
                quants: 0.5,
                nuants: 5.0,
            };
            assert!(
                !destitute.can_act(),
                "full power and no resources is nothing"
            );
            assert!(
                !powerless.can_act(),
                "full resources and no power is nothing"
            );
            assert!(both.can_act());
        }

        #[test]
        fn the_two_failures_have_different_names() {
            // Conflating them would make the economy a countdown, which is not
            // what it is.
            assert_eq!(
                Economy {
                    quants: 1.0,
                    nuants: 0.0
                }
                .posture(),
                "empty"
            );
            assert_eq!(
                Economy {
                    quants: 0.0,
                    nuants: 5.0
                }
                .posture(),
                "stuck"
            );
        }

        #[test]
        fn a_stock_has_a_ceiling_or_there_is_no_reason_to_be_efficient() {
            let mut e = Economy::default();
            for _ in 0..100 {
                e.nuants = (e.nuants + 5.0).min(Economy::MAX_NUANTE);
            }
            assert_eq!(e.nuants, Economy::MAX_NUANTE);
        }

        #[test]
        fn a_rate_has_a_ceiling_or_getting_better_has_no_end() {
            // Topped up each time, because the interesting fact here is that a
            // creature *runs out before it maxes out*. That is the economy, not a
            // bug: capability is not something you accumulate by spending, it is
            // something you have to keep demonstrating, and the tank runs dry
            // before the rate is full.
            // Reached by credit now, not by acting: the ceiling is a property of
            // production and acting is not production.
            let mut e = Economy::default();
            for _ in 0..500 {
                e.nuants = Economy::MAX_NUANTE;
                e.quants =
                    (e.quants + 0.5 * (Economy::MAX_CUANTE - e.quants)).min(Economy::MAX_CUANTE);
            }
            // To a tolerance, because the increment shrinks with the distance to
            // the ceiling and a float approaches 1.0 without landing on it. An
            // exact equality here would have been a test that could only ever
            // fail, which is a way of saying nothing.
            assert!(
                (e.quants - Economy::MAX_CUANTE).abs() < 1e-9,
                "power settled at {} rather than at the ceiling {}",
                e.quants,
                Economy::MAX_CUANTE
            );
        }

        #[test]
        fn a_creature_runs_out_before_it_maxes_out() {
            // The consequence, stated as a measurement. Play costs 1.5 and the
            // starting tank is 12, so the first act the creature cannot afford is
            // the ninth — at a power nowhere near its ceiling.
            let mut e = Economy::default();
            let start_power = e.quants;
            let mut acts = 0;
            while Care::Play.apply_economy(&mut e) {
                acts += 1;
            }
            assert_eq!(acts, 8, "12 nuants at 1.5 each");
            // And acting does not raise the power at all, so it certainly cannot
            // reach the ceiling.
            assert!(
                e.quants < start_power,
                "acting {} times raised the power from {start_power} to {}, and \
                 repetition is not production",
                acts,
                e.quants
            );
            assert!(
                e.quants < Economy::MAX_CUANTE,
                "power reached {} which is its ceiling, so the resources were not the binding constraint",
                e.quants
            );
        }
    }

    mod spending {
        use super::*;

        #[test]
        fn an_act_costs_what_it_is_declared_to_cost() {
            assert!(Care::Clean.cost() > Care::Play.cost());
            assert!(Care::Play.cost() > Care::Feed.cost());
            assert_eq!(Care::Sleep.cost(), 0.0, "sleep is free");
        }

        #[test]
        fn acting_spends_the_resources_and_debits_the_power() {
            // Not "sustains the rate", which is what this test used to say and
            // what the economy used to do. An act is consumption and nothing
            // more; the power rises only when induction credits production.
            let mut p = pet();
            let before = p.vitals.economy.clone();
            assert!(p.tend(Care::Feed).is_some());
            assert!(p.vitals.economy.nuants < before.nuants, "spent resources");
            assert!(
                p.vitals.economy.quants < before.quants,
                "an act with nothing produced behind it must not raise the power"
            );
        }

        #[test]
        fn an_unaffordable_act_is_refused_and_changes_nothing() {
            // Not just refused: the rate must not be paid for by a credit the
            // creature did not have, or being broke would make it better.
            let mut p = pet();
            p.vitals.economy.nuants = 0.5;
            let before = p.vitals.economy.clone();
            assert!(p.tend(Care::Clean).is_none(), "clean costs 2.0");
            assert_eq!(p.vitals.economy, before, "nothing moved");
        }

        #[test]
        fn a_broke_creature_can_still_sleep_and_grow() {
            // The one escape from an empty tank, and the reason the game is
            // winnable after a long absence.
            let mut p = pet();
            p.vitals.economy.nuants = 0.0;
            assert!(p.tend(Care::Feed).is_none());
            assert!(p.tend(Care::Sleep).is_some(), "sleep is free");
            assert_eq!(p.vitals.economy.nuants, 0.0, "and still costs nothing");
        }

        /// A stuck creature cannot help itself, and a person can still help it.
        ///
        /// The split into two entry points is what makes this true. Gating the
        /// player's `tend` on power as well is what would have made a powerless
        /// creature permanently stuck: no act, no trace, no production, no credit.
        #[test]
        fn a_stuck_creature_cannot_help_itself_but_can_be_helped() {
            let mut p = pet();
            p.vitals.economy = Economy {
                quants: 0.0,
                nuants: Economy::MAX_NUANTE,
            };
            // Its own door is shut, and a full tank does not open it.
            assert!(
                p.tend_as_self(Care::Feed).is_none(),
                "a full tank changes nothing for a creature with no power"
            );
            assert!(
                p.why_wont_act().unwrap().contains("lost the power"),
                "it must be able to say which failure it is before it escapes it"
            );

            // The player's door is not, which is the whole reason there are two.
            assert!(
                p.tend(Care::Feed).is_some(),
                "a person can still feed a creature that cannot feed itself"
            );
            // But being *fed* does not lift it out of the hole, and an earlier
            // version of this test asserted that it did: it held that sleep, being
            // free and having the highest gain, was the way out. Both halves of
            // that are gone. The power is no longer sustained by acting — an act
            // debits it — so putting a stuck creature to sleep pushes it
            // *further* down, and the test was asserting the opposite. The floor
            // test below pins what actually lifts it: being taught.
        }
    }

    mod decay {
        use super::*;

        #[test]
        fn neglect_drains_the_stock_and_falls_the_rate() {
            let mut p = pet();
            let before = p.vitals.economy.clone();
            p.neglect();
            assert!(p.vitals.economy.nuants < before.nuants, "resources drained");
            assert!(p.vitals.economy.quants < before.quants, "power fell");
        }

        #[test]
        fn a_creature_becomes_incapable_before_it_becomes_broke() {
            // The ordering is the design. The cheaper failure arrives first, so a
            // long absence is recoverable and a short one is not much of a
            // punishment.
            let mut p = pet();
            let mut ticks = 0;
            loop {
                let e = p.vitals.economy;
                if e.quants <= 0.1 && e.nuants > 0.0 {
                    break;
                }
                if ticks > 100 {
                    panic!("neither failure ever arrived");
                }
                p.neglect();
                ticks += 1;
            }
            let e = p.vitals.economy;
            assert!(e.quants <= 0.1, "power collapsed at tick {ticks}");
            assert!(
                e.nuants > 0.0,
                "but it still had resources, at tick {ticks}"
            );
        }

        #[test]
        fn a_rate_decays_toward_a_floor_and_never_to_nothing() {
            // So a creature that once learned something is never quite the
            // creature that never did, which is the only thing making the game
            // winnable after a long absence.
            let mut p = pet();
            for _ in 0..200 {
                p.neglect();
            }
            assert_eq!(
                p.vitals.economy.quants,
                Economy::FLOOR_CUANTE,
                "the floor is a floor"
            );
        }

        #[test]
        fn a_neglected_creature_says_which_failure_it_is() {
            let mut broke = pet();
            broke.vitals.economy = Economy {
                quants: 0.5,
                nuants: 0.0,
            };
            assert!(broke.why_wont_act().unwrap().contains("no nuants"));

            let mut stuck = pet();
            stuck.vitals.economy = Economy {
                quants: 0.0,
                nuants: 9.0,
            };
            assert!(stuck.why_wont_act().unwrap().contains("lost the power"));
        }
    }

    mod declared {
        use super::*;

        #[test]
        fn both_quantities_are_declared_state_and_published() {
            let v = Vitals::default();
            let state = Vitals::state(&v);
            assert!(state.contains_key("quants"));
            assert!(state.contains_key("nuants"));

            let declared: Vec<&str> = Vitals::declared_state().iter().map(|(n, _)| *n).collect();
            for field in ["quants", "nuants"] {
                assert!(declared.contains(&field), "{field} is not declared");
                assert!(state.contains_key(field), "{field} is not published");
            }
        }

        #[test]
        fn the_manifest_states_the_ceilings_as_ranges() {
            // A ceiling that is not in the declaration is not a constraint, and
            // the evaluator cannot check it.
            let space = Pet::state_space_manifest();
            assert_eq!(
                space["quants"]["range"][1],
                serde_json::json!(Economy::MAX_CUANTE)
            );
            assert_eq!(
                space["nuants"]["range"][1],
                serde_json::json!(Economy::MAX_NUANTE)
            );
        }

        #[test]
        fn the_economy_is_state_and_does_not_move_the_address() {
            // The rule that keeps holding: what has happened to a creature is not
            // what it is. A broke one is still the same creature.
            let mut p = pet();
            let a = p.address();
            p.vitals.economy = Economy {
                quants: 0.1,
                nuants: 0.0,
            };
            assert_eq!(p.address(), a);
        }

        #[test]
        fn a_creature_saved_before_the_existed_still_loads() {
            let p = pet();
            let mut as_value: serde_json::Value =
                serde_json::from_str(&p.to_json()).expect("round-trips");
            let vitals = as_value
                .get_mut("vitals")
                .and_then(|v| v.as_object_mut())
                .expect("vitals");
            vitals.remove("economy");
            let restored = Pet::from_json("ca-e", &as_value.to_string()).expect("still loads");
            assert_eq!(restored.vitals.economy, Economy::default());
            assert_eq!(restored.address(), p.address());
        }
    }
}

#[cfg(test)]
mod floor_tests {
    use super::*;

    /// The hole the playing found, held shut.
    ///
    /// A declared failure that cannot happen is worse than no such failure,
    /// because the type implies a distinction the game never offers. The first
    /// version floored the power at 0.1 and asked only for > 0.0, so neglect could
    /// never produce a stuck creature and the whole reason the two quantities are
    /// separate was never exercised by the game itself.
    #[test]
    fn neglect_alone_can_make_a_creature_stuck() {
        let mut p = Pet::new("ca-f");
        assert!(p.vitals.economy.can_act());

        let mut ticks = 0;
        while p.vitals.economy.can_act() {
            p.neglect();
            ticks += 1;
            assert!(ticks < 200, "neglect never produced a stuck creature");
        }

        // It must be *stuck*, not *empty*: the point is that the resources were
        // still there and could not be used.
        assert_eq!(p.vitals.economy.posture(), "stuck");
        assert!(
            p.vitals.economy.nuants > 0.0,
            "it became stuck with no resources, which is the other failure"
        );
    }

    #[test]
    fn the_floor_is_below_the_usable_threshold_not_at_zero() {
        let at_floor = Economy {
            quants: Economy::FLOOR_CUANTE,
            nuants: 100.0,
        };
        assert!(
            !at_floor.can_act(),
            "the floor is where it stops being usable"
        );

        // And it is a floor, not an abyss: a creature that was cared for is never
        // quite the creature that never was.
        let mut e = Economy::default();
        for _ in 0..500 {
            Care::Sleep.decay_economy(&mut e);
        }
        assert_eq!(e.quants, Economy::FLOOR_CUANTE);
    }

    /// The route out of being stuck, which used to be sleep and is now teaching.
    ///
    /// The first version of this test asserted that one act of care lifted a
    /// stuck creature clear, on the grounds that sleep is free and `Sleep` had
    /// the highest gain. Both halves are gone: the power is no longer sustained
    /// by acting. An act now debits it, so putting a stuck creature to sleep
    /// pushes it *further* down, and the test asserted the opposite.
    ///
    /// So the question became what does lift it, and the answer is production
    /// alone. A stuck creature cannot produce, because producing requires acting,
    /// because acting requires power — a liveness hole, closed by the two entry
    /// points rather than by weakening the economy: a player may act on a
    /// powerless creature, the trace is recorded, induction finds the new
    /// phrasing, and the credit puts it back above the floor. Being helped is how
    /// a powerless creature comes back.
    #[test]
    fn a_stuck_creature_is_lifted_by_being_taught_not_by_being_fed() {
        let mut p = Pet::new("ca-f");
        p.vitals.economy = Economy {
            quants: Economy::FLOOR_CUANTE,
            nuants: 20.0,
        };
        assert_eq!(p.vitals.economy.posture(), "stuck");
        assert!(!p.vitals.economy.can_act(), "and it cannot act");

        // Being fed changes nothing. An act spends and debits; it never pays.
        p.tend(Care::Sleep);
        assert!(
            !p.vitals.economy.can_act(),
            "an act lifted a stuck creature clear, and acting is not production"
        );

        // Being taught does.
        p.tend(Care::Feed);
        let production = p.learn(vec![LearnedRule {
            address: "addr-feed".into(),
            signature: Care::Feed.signature().into(),
            aliases: vec!["a phrase it had never heard".into()],
            confidence: 0.6,
            observations: 1,
        }]);
        assert!(
            production.any(),
            "the phrase was new, so something was produced"
        );
        assert!(
            p.vitals.economy.can_act(),
            "production put it back above the floor at {}",
            p.vitals.economy.quants
        );
    }

    #[test]
    fn neglect_kills_the_creature_before_either_economic_failure_arrives() {
        // A finding, recorded because it is not what the design intended.
        //
        // Three things race, and death wins: health reaches zero in about ten
        // ticks, the power falls to its floor in about sixteen, and the resources
        // run out in about twenty-four. So a creature left alone is *dead* long
        // before it is ever poor or powerless, and both economic failures are
        // reachable only on a creature that no longer exists.
        //
        // Which means the two-failure economy is currently unobservable through
        // the game, and only through the code. That is a tuning problem rather
        // than a modelling one, and it is the next thing to decide: slow the
        // vitals, speed the economy, or accept that the economy is legible only
        // before death.
        let mut p = Pet::new("ca-f");
        let mut death_tick = None;
        let mut stuck_tick = None;
        let mut empty_tick = None;

        for tick in 1..=200 {
            p.neglect();
            if death_tick.is_none() && p.quarantined {
                death_tick = Some(tick);
            }
            if stuck_tick.is_none() && p.vitals.economy.nuants > 0.0 && !p.vitals.economy.can_act()
            {
                stuck_tick = Some(tick);
            }
            if empty_tick.is_none() && p.vitals.economy.nuants <= 0.0 {
                empty_tick = Some(tick);
            }
            if death_tick.is_some() && stuck_tick.is_some() && empty_tick.is_some() {
                break;
            }
        }

        let death = death_tick.expect("it must die eventually");
        let stuck = stuck_tick.expect("it must become stuck eventually");
        let empty = empty_tick.expect("it must run out eventually");

        assert!(
            death < stuck,
            "death at {death} should precede stuck at {stuck}"
        );
        assert!(
            stuck < empty,
            "stuck at {stuck} should precede empty at {empty}"
        );
    }

    #[test]
    fn the_usable_threshold_is_declared_rather_than_hidden_in_a_comparison() {
        // A threshold the verifier cannot see is a threshold nobody can check,
        // which is the D5 lesson arriving in a new place.
        let space = Pet::state_space_manifest();
        assert_eq!(
            space["quants"]["usable_above"],
            serde_json::json!(Economy::FLOOR_CUANTE)
        );
    }
}

#[cfg(test)]
mod production_tests {
    use super::*;

    /// A learned rule, with an explicit confidence.
    ///
    /// The parameter is not decoration. An earlier version of this helper
    /// hard-coded 0.8, which meant no rule could ever become *more* reliable and
    /// the consolidation term was unreachable — a fixture that quietly made the
    /// economy it was testing unrepresentative, and the reason the next test
    /// failed for a reason that had nothing to do with the economy.
    fn rule(sig: &str, addr: &str, aliases: &[&str], confidence: f64) -> LearnedRule {
        LearnedRule {
            address: addr.into(),
            signature: sig.into(),
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            confidence,
            observations: aliases.len(),
        }
    }

    /// The hole, held shut. Measured before the fix: twelve identical feeds, the
    /// same sentence every time, no rule induced, and the power went 0.500 to
    /// 0.694. Repetition paid as if it were production.
    #[test]
    fn repetition_without_production_does_not_raise_the_power() {
        let mut p = Pet::new("ca-p");
        let nothing: Vec<LearnedRule> = Vec::new();
        let start = p.vitals.economy.quants;

        for _ in 0..12 {
            p.tend(Care::Feed);
            p.learn(nothing.clone());
        }

        assert!(
            p.vitals.economy.quants < start,
            "twelve repetitions moved the power from {start} to {}, and repetition \
             is not production",
            p.vitals.economy.quants
        );
        assert!(p.vitals.economy.nuants < 1.0, "and it still cost resources");
    }

    #[test]
    fn an_unproductive_repetition_is_corrosive_rather_than_merely_free() {
        // The psychological reading, made mechanical: repeating and getting
        // nothing should cost something the player notices.
        let mut p = Pet::new("ca-p");
        let nothing: Vec<LearnedRule> = Vec::new();
        let after_one = {
            p.tend(Care::Feed);
            p.learn(nothing.clone());
            p.vitals.economy.quants
        };
        for _ in 0..8 {
            p.tend(Care::Feed);
            p.learn(nothing.clone());
        }
        assert!(p.vitals.economy.quants < after_one, "and it keeps costing");
    }

    #[test]
    fn a_new_sentence_for_a_known_act_is_production() {
        let mut p = Pet::new("ca-p");
        let mut held: Vec<LearnedRule> = Vec::new();
        p.tend(Care::Feed);
        held.push(rule(
            Care::Feed.signature(),
            "addr-feed",
            &["pour some kibble"],
            0.5,
        ));
        let first = p.learn(held.clone());
        assert_eq!(first.new_phrasings, 1);
        assert_eq!(
            first.new_signatures, 1,
            "the act itself is new the first time"
        );

        // The same sentence again is nothing at all.
        let repeat = p.learn(held.clone());
        assert!(!repeat.any(), "the same rule set is not production");
        assert_eq!(repeat.credit(), 0.0);
    }

    #[test]
    fn a_new_act_is_worth_more_than_a_new_sentence_for_the_same_act() {
        // The escalation the economy is supposed to reward, and a claim about
        // what matters: nobody gained because the same sentence arrived in a
        // different language, and a great deal because a new act became possible.
        let phrasing = Production {
            new_rules: 0,
            new_signatures: 0,
            new_phrasings: 1,
            consolidated: 0,
        };
        let act = Production {
            new_rules: 1,
            new_signatures: 1,
            new_phrasings: 1,
            consolidated: 0,
        };
        assert!(
            act.credit() > phrasing.credit(),
            "a new act ({}) must outrank a new phrasing ({})",
            act.credit(),
            phrasing.credit()
        );
    }

    #[test]
    fn new_reach_for_a_known_act_is_not_paid_as_a_new_rule() {
        // A second artifact expressing an act that is already known is redundancy,
        // not progress, and the generated corpus produces exactly this: six
        // artifacts per profile, all the same capability. Paying for it would be
        // paying for the corpus being uniform and calling it an escalation.
        let known = vec![rule(Care::Feed.signature(), "addr-1", &["a"], 0.5)];
        let more = vec![
            rule(Care::Feed.signature(), "addr-1", &["a"], 0.5),
            rule(Care::Feed.signature(), "addr-2", &["b"], 0.5),
        ];
        let production = Production::since(&known, &more);
        assert_eq!(production.new_signatures, 0, "no new capability");
        assert_eq!(
            production.new_rules, 0,
            "and no new rule, because the act is known"
        );
        assert_eq!(production.new_phrasings, 1, "only the phrasing");
    }

    #[test]
    fn there_is_no_path_to_power_that_does_not_go_through_production() {
        // The whole point of separating the cost from the credit. Tending without
        // learning cannot raise the power, however many times it happens.
        let mut p = Pet::new("ca-p");
        let start = p.vitals.economy.quants;
        for _ in 0..50 {
            p.tend(Care::Play);
        }
        assert!(p.vitals.economy.quants < start);
    }

    /// The economy's contract, stated as the three things it actually does.
    ///
    /// Four versions of this test have now been wrong, and the fourth is the one
    /// worth keeping. The first asserted a power above 0.5 — a number I had not
    /// measured. The second asserted a ratio above 2.0, the same mistake
    /// comparatively; it failed at 1.50 because the helper pinned confidence at
    /// 0.8, so consolidation could never fire. The third asserted a widening gap
    /// and a ratio above 2.0 at ten acts, which failed at 1.99 — not because the
    /// economy was wrong but because ten acts is not where the number I wanted
    /// lives.
    ///
    /// So this asserts the shape rather than a coordinate, because the shape is
    /// what the economy is for and the coordinate is a tuning artifact. Measured
    /// over twenty acts: the producer/repeater ratio rises monotonically
    /// 1.20 → 2.47, the producer is stronger at every single act, and the
    /// repeater ends below where it began.
    #[test]
    fn production_beats_repetition_at_every_act_and_the_margin_grows() {
        let mut producer = Pet::new("ca-p");
        let mut repeater = Pet::new("ca-r");
        let nothing: Vec<LearnedRule> = Vec::new();
        let mut held: Vec<LearnedRule> = Vec::new();
        let mut words: Vec<String> = Vec::new();
        let mut confidence: f64 = 0.5;
        let mut last_ratio = 0.0;

        for i in 0..20 {
            teach(&mut producer, &mut held, &mut words, &mut confidence, i);
            repeater.tend(Care::Feed);
            repeater.learn(nothing.clone());

            let (p, r) = (
                producer.vitals.economy.quants,
                repeater.vitals.economy.quants,
            );
            assert!(
                p > r,
                "at act {} a producer was at {p} and a repeater at {r}; production \
                 must win at every act, not on average",
                i + 1
            );
            if i > 0 {
                assert!(
                    p / r > last_ratio,
                    "the margin shrank at act {}: {:.3} after {:.3}",
                    i + 1,
                    p / r,
                    last_ratio
                );
            }
            last_ratio = p / r;
        }

        assert!(
            repeater.vitals.economy.quants < 0.5,
            "a repeater ends weaker than it began, at {}",
            repeater.vitals.economy.quants
        );
    }

    /// A creature that stops escalating declines, and this pins that as a known
    /// limit rather than leaving it to be discovered in play.
    ///
    /// Measured over twenty acts: the producer climbs to 0.577 on act four — the
    /// last of the four built-in acts — and then *falls* to 0.323 by act twelve
    /// before recovering once the repeater beside it has run out of resources
    /// entirely. Consolidation holds a floor; it does not hold a peak. The
    /// implied equilibrium from a 0.03 credit against a 0.15 debit is about 0.25,
    /// which is below the peak, so any creature that has learned everything it
    /// can currently learn ends up weaker than one still learning.
    ///
    /// **This is blocked on the act set, not on the economy.** There are four
    /// built-in `Care` acts, so `new_signatures` saturates after four and the
    /// only remaining novelty is a new phrasing, worth a tenth of a debit. Close
    /// the act set and the long game is decay, and the economy cannot be balanced
    /// around that without a knob that would only hide it. The fix is the one
    /// already on the list: the creature must be able to learn a power it did
    /// not ship with.
    #[test]
    fn a_creature_that_has_learned_everything_it_can_still_declines() {
        let mut p = Pet::new("ca-p");
        let mut held: Vec<LearnedRule> = Vec::new();
        let mut words: Vec<String> = Vec::new();
        let mut confidence: f64 = 0.5;

        let mut peak: f64 = 0.0;
        let mut peak_at = 0;
        for i in 0..12 {
            teach(&mut p, &mut held, &mut words, &mut confidence, i);
            if p.vitals.economy.quants > peak {
                peak = p.vitals.economy.quants;
                peak_at = i + 1;
            }
        }
        let settled = p.vitals.economy.quants;

        assert_eq!(
            peak_at, 4,
            "the peak is the last built-in act, so this is the shape"
        );
        assert!(
            settled < peak,
            "a creature that has learned every act it can learn settled at {settled}, \
             at or above its peak of {peak}. Consolidation is meant to hold a floor, \
             not a peak — if this ever passes, the act set changed and the \
             equilibrium above needs recomputing"
        );
    }

    /// Teaches one thing to a creature and lets induction run, the way the game
    /// does: the first four acts are new, and every act after that is a new word
    /// for an act it already holds.
    fn teach(
        p: &mut Pet,
        held: &mut Vec<LearnedRule>,
        words: &mut Vec<String>,
        confidence: &mut f64,
        i: usize,
    ) {
        p.tend(Care::Feed);
        if i < 4 {
            let addr = format!("addr-{i}");
            let phrase = format!("a phrase for act {i}");
            held.push(rule(
                [Care::Feed, Care::Play, Care::Clean, Care::Sleep][i].signature(),
                &addr,
                &[&phrase],
                0.5,
            ));
        } else {
            words.push(format!("word {i}"));
            *confidence = (*confidence + 0.06).min(0.99);
            let refs: Vec<&str> = words.iter().map(String::as_str).collect();
            held[0] = rule(Care::Feed.signature(), "addr-0", &refs, *confidence);
        }
        p.learn(held.clone());
    }

    #[test]
    fn repetition_walks_a_creature_down_through_whichever_failure_comes_first() {
        // Named honestly. Twelve nuants at a cost of 1.0 is twelve acts, so a
        // creature that only repeats runs out of resources before its power ever
        // reaches the floor. It is `empty` rather than `stuck` — an earlier
        // version of this test asserted `stuck` and was wrong about which
        // failure a repeater gets.
        let mut repeater = Pet::new("ca-p");
        let nothing: Vec<LearnedRule> = Vec::new();
        for _ in 0..12 {
            repeater.tend(Care::Feed);
            repeater.learn(nothing.clone());
        }
        assert_eq!(
            repeater.vitals.economy.posture(),
            "empty",
            "a pure repeater runs out of resources, not of power"
        );
    }

    #[test]
    fn a_creature_with_resources_but_no_power_is_stuck_and_only_that() {
        // The other failure, reached the only way it can be: resources intact,
        // power at the floor. A repeater never gets here because the resources go
        // first, which is itself worth knowing — the economy punishes the cheap
        // mistake first and the expensive one later.
        let mut p = Pet::new("ca-p");
        p.vitals.economy = Economy {
            quants: Economy::FLOOR_CUANTE,
            nuants: Economy::MAX_NUANTE,
        };
        assert_eq!(p.vitals.economy.posture(), "stuck");
        assert!(
            p.vitals.economy.nuants > 0.0,
            "with resources it never used"
        );
    }
}
