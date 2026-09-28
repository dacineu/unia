//! Generation of a synthetic `.ure` corpus.
//!
//! The corpus this repository shipped had **twelve artifacts with zero shared
//! capabilities**. That is a real measurement and it is a dead end: with nothing
//! in common, `gather` finds no denominators, the escalation rate cannot be
//! computed, and convergence has nothing to converge *on*. The cold start has
//! the same problem from the other side — induction needs traces, and traces need
//! artifacts to have been called.
//!
//! So the corpus is generated rather than hand-written. That is a deliberate
//! trade and it has a cost, stated here so it is not discovered later:
//!
//! - **It is not evidence about the world.** Every artifact below is a plausible
//!   fiction. A measurement taken on it shows the *machinery* works, and says
//!   nothing about whether the machinery is right for a real valve.
//! - **It is reproducible and seeded**, so two runs produce the same corpus and a
//!   regression is a real regression rather than sampling noise.
//! - **It shares primitives on purpose.** The overlap is the point: a corpus of
//!   unrelated artifacts cannot exercise convergence, and convergence is the
//!   claim.
//!
//! The generator is deliberately *not* clever. It emits declared state spaces,
//! overlapping actions, and phrasings in more than one language, because those
//! are the three things the retrieval, induction and convergence paths each need
//! something to do. Anything it emits that looks like real-world knowledge is
//! incidental.

use serde_json::{json, Value};
use std::collections::BTreeSet;

/// A seeded generator. Deterministic for a given seed, which is what makes a
/// corpus regression a regression.
pub struct Generator {
    state: u64,
}

impl Generator {
    /// A generator with a fixed seed.
    ///
    /// The default is a named constant rather than a random seed so that `cargo
    /// test` and a developer's machine produce the same corpus.
    pub fn seeded(seed: u64) -> Self {
        Generator {
            // Avoids the all-zero state, in which xorshift never moves.
            state: seed | 1,
        }
    }

    /// A small deterministic PRNG.
    ///
    /// xorshift64, written out rather than pulled in as a dependency. It is not
    /// cryptographically anything and does not need to be: the requirement is
    /// that the same seed gives the same corpus on every machine, and that no
    /// two draws inside a run collide in a way that clusters the sample.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

/// A capability profile: one shape of action structure.
///
/// Profiles rather than "kinds with overlapping actions", because `gather`
/// converges on **exact equality of the action set** and not on similarity. Two
/// artifacts that share two of three actions do not interpenetrate. So the unit
/// of overlap in this corpus is a whole profile, and an artifact's profile
/// determines its action ids exactly.
///
/// The `unpaired` profiles exist on purpose. A corpus in which everything
/// converges is as useless as one in which nothing does: the first cannot
/// distinguish convergence from a uniform generator, the second cannot show the
/// rule discriminating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub name: &'static str,
    pub category: &'static str,
    /// The state fields this profile declares, as `(name, type, values)`.
    pub fields: &'static [(&'static str, &'static str, &'static [&'static str])],
    /// The actions, as `(id, phrasing table key)`.
    pub actions: &'static [(&'static str, &'static str)],
    /// Whether artifacts of this profile are expected to find each other.
    pub paired: bool,
}

const P_ACTUATOR: Profile = Profile {
    name: "actuator-basic",
    category: "actuator",
    fields: &[
        ("level", "float", &[]),
        ("status", "enum", &["open", "closed", "fault"]),
    ],
    actions: &[
        ("set_level", "set_level"),
        ("read_status", "read_status"),
        ("reset", "reset"),
    ],
    paired: true,
};

const P_SENSOR: Profile = Profile {
    name: "sensor-basic",
    category: "sensor",
    fields: &[
        ("reading", "float", &[]),
        ("status", "enum", &["online", "stale", "fault"]),
    ],
    actions: &[
        ("read_reading", "read_reading"),
        ("check_status", "check_status"),
    ],
    paired: true,
};

const P_MEMORY: Profile = Profile {
    name: "memory-basic",
    category: "memory",
    fields: &[
        ("occupancy", "enum", &["empty", "partial", "full"]),
        ("level", "float", &[]),
    ],
    actions: &[("clear", "clear"), ("set_level", "set_level_mem")],
    paired: true,
};

const P_COMPUTE: Profile = Profile {
    name: "compute-basic",
    category: "compute",
    fields: &[("load", "float", &[]), ("reading", "float", &[])],
    actions: &[("set_load", "set_load"), ("read_reading", "read_reading")],
    paired: true,
};

/// A superset of the actuator profile. Shares two of three actions with it and
/// therefore converges with nobody, which is the point of including it.
const P_ACTUATOR_WIDE: Profile = Profile {
    name: "actuator-wide",
    category: "actuator",
    fields: &[
        ("level", "float", &[]),
        ("status", "enum", &["open", "closed", "fault"]),
        ("reading", "float", &[]),
    ],
    actions: &[
        ("set_level", "set_level"),
        ("read_status", "read_status"),
        ("reset", "reset"),
        ("read_reading", "read_reading"),
    ],
    paired: false,
};

/// A strict subset of the actuator profile, and likewise unpaired.
const P_ACTUATOR_MIN: Profile = Profile {
    name: "actuator-minimal",
    category: "actuator",
    fields: &[("level", "float", &[])],
    actions: &[("set_level", "set_level")],
    paired: false,
};

/// Every profile, in a fixed order so the corpus is deterministic.
pub const PROFILES: &[Profile] = &[
    P_ACTUATOR,
    P_SENSOR,
    P_MEMORY,
    P_COMPUTE,
    P_ACTUATOR_WIDE,
    P_ACTUATOR_MIN,
];

/// Phrasings for an action, in two languages.
///
/// The Romanian is real and overlaps the hand-written corpus, so a generated
/// corpus and the shipped one can be compared rather than being two unrelated
/// things. Two languages is the minimum that can show the cross-lingual claim.
fn phrasing(key: &str) -> &'static [(&'static str, &'static str)] {
    match key {
        "set_level" => &[
            ("set the level", "seteaza nivelul"),
            ("adjust the level", "ajusteaza nivelul"),
            ("dial it to", "pune-l la"),
        ],
        "set_level_mem" => &[
            ("set the level", "seteaza nivelul"),
            ("reserve some of it", "rezerva spatiu"),
        ],
        "set_load" => &[
            ("set the load", "limiteaza sarcina"),
            ("throttle it", "reduce sarcina"),
        ],
        "read_status" => &[
            ("check the status", "verifica starea"),
            ("is it faulted", "este defect?"),
            ("report the status", "raporteaza starea"),
        ],
        "check_status" => &[
            ("check the status", "verifica starea"),
            ("is it online", "este online?"),
            ("report the status", "raporteaza starea"),
        ],
        "read_reading" => &[
            ("read the reading", "citeste valoarea"),
            ("what is the reading", "ce valoare este?"),
            ("report the value", "raporteaza valoarea"),
        ],
        "reset" => &[
            ("reset it", "reseteaza"),
            ("shut it down", "opreste-l"),
            ("emergency stop", "oprire de urgenta"),
        ],
        "clear" => &[
            ("clear it", "goleste"),
            ("empty it", "golește-l"),
            ("purge the contents", "sterge continutul"),
        ],
        _ => &[],
    }
}

/// One generated action, before it is written into a manifest.
struct Action {
    id: String,
    aliases: Vec<String>,
    target_state: String,
    constraints: Vec<String>,
}

/// The state-space entries a profile declares, as manifest JSON.
fn state_space(profile: &Profile) -> serde_json::Map<String, Value> {
    let mut out = serde_json::Map::new();
    for (name, kind, values) in profile.fields {
        let spec = if *kind == "enum" {
            json!({ "type": "enum", "values": values })
        } else {
            json!({ "type": "float", "range": [-100.0, 100.0], "unit": "units" })
        };
        out.insert(name.to_string(), spec);
    }
    out
}

/// The precondition an action carries, derived from what the profile declares.
///
/// Derived rather than written per action because the first version hard-coded
/// `status != 'fault'`, which handed the memory profile a precondition on a field
/// it does not have — and an unevaluable precondition is refused by the gate, so
/// the corpus would have measured the refusal path instead of convergence. A
/// test now holds this invariant.
fn precondition(profile: &Profile) -> Vec<String> {
    if let Some((_, kind, values)) = profile.fields.iter().find(|(n, _, _)| *n == "status") {
        if *kind == "enum" {
            let bad = values.last().copied().unwrap_or("fault");
            return vec![format!("status != '{bad}'")];
        }
    }
    if profile.fields.iter().any(|(n, _, _)| *n == "occupancy") {
        return vec!["occupancy != 'full'".to_string()];
    }
    Vec::new()
}

/// The actions a profile declares.
fn actions_for(profile: &Profile) -> Vec<Action> {
    let guard = precondition(profile);
    profile
        .actions
        .iter()
        .map(|(id, key)| Action {
            id: id.to_string(),
            aliases: phrasing(key)
                .iter()
                .flat_map(|(en, ro)| [en.to_string(), ro.to_string()])
                .collect(),
            target_state: profile
                .fields
                .first()
                .map(|(n, _, _)| n.to_string())
                .unwrap_or_else(|| "state".to_string()),
            constraints: guard.clone(),
        })
        .collect()
}

/// Generates one manifest for a profile.
///
/// Two artifacts of the same profile differ in their complexity score, their
/// guidance, and therefore their content address, while sharing every action id
/// exactly. That is the shape the convergence rule is looking for: **the same
/// structure reached from different words**, which is the project's central claim
/// expressed as a fixture.
pub fn manifest(g: &mut Generator, profile: &Profile, index: usize) -> Value {
    let actions = actions_for(profile);
    let mut declared = Vec::new();
    for a in &actions {
        declared.push(json!({
            "id": a.id,
            "aliases": a.aliases,
            "params": {},
            "target_state": a.target_state,
            "constraints": a.constraints,
        }));
    }

    json!({
        "ure_version": "1.0",
        "category": profile.category,
        "external_id": format!("gen-{}-{:03}", profile.name, index),
        // Varies per instance, so two artifacts of one profile do not collide
        // into the same content address. The variation is in the *surface* and
        // the complexity, never in the action ids.
        "complexity_score": 0.10 + (g.below(90) as f64) / 100.0,
        "guidance": format!(
            "A generated {} artifact (instance {}). Synthetic, and not evidence \
             about anything real; see src/corpus.rs for why it exists.",
            profile.name, index
        ),
        "state_space": state_space(profile),
        "action_primitives": declared,
    })
}

/// A whole corpus: `per_profile` instances of every profile.
///
/// Both paired and unpaired profiles are emitted. A corpus where everything
/// converges would be as uninformative as the twelve-artifact one it replaces,
/// because a uniform generator produces a uniform result and a uniform result
/// cannot be told apart from a rule that accepts everything.
pub fn corpus(seed: u64, per_profile: usize) -> Vec<(String, Value)> {
    let mut g = Generator::seeded(seed);
    let mut out = Vec::new();
    for profile in PROFILES {
        for i in 0..per_profile {
            let m = manifest(&mut g, profile, i);
            let name = m["external_id"].as_str().unwrap_or("gen").to_string();
            out.push((name, m));
        }
    }
    out
}

#[cfg(test)]
mod corpus_tests {
    use super::*;

    #[test]
    fn the_same_seed_produces_the_same_corpus() {
        // Without this, every measurement on it is sampling noise.
        assert_eq!(corpus(7, 3), corpus(7, 3));
    }

    #[test]
    fn a_different_seed_produces_a_different_corpus() {
        assert_ne!(corpus(7, 3), corpus(8, 3));
    }

    #[test]
    fn instances_of_one_profile_share_every_action_id() {
        // The property the convergence rule needs. `gather` groups on exact
        // equality of the action set, so two artifacts that differ by a single
        // action find each other never.
        for profile in PROFILES {
            let mut g = Generator::seeded(3);
            let reference: Vec<String> = actions_for(profile).into_iter().map(|a| a.id).collect();
            for i in 0..5 {
                let m = manifest(&mut g, profile, i);
                let got: Vec<String> = m["action_primitives"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| a["id"].as_str().unwrap().to_string())
                    .collect();
                assert_eq!(got, reference, "{} instance {i}", profile.name);
            }
        }
    }

    #[test]
    fn instances_of_one_profile_are_distinct_artifacts() {
        // Same structure, different content address. If they collided, the corpus
        // would be several copies of one artifact and convergence would be
        // counting files rather than derivations.
        let mut g = Generator::seeded(5);
        let ids: BTreeSet<String> = (0..12)
            .map(|i| {
                let m = manifest(&mut g, &P_ACTUATOR, i);
                m["external_id"].as_str().unwrap().to_string()
            })
            .collect();
        assert_eq!(ids.len(), 12, "instances collapsed into fewer artifacts");
    }

    #[test]
    fn every_action_tests_only_a_field_the_profile_declares() {
        // A constraint on an undeclared field is unevaluable, and the gate refuses
        // it. A corpus that tripped this would be measuring the refusal path.
        for profile in PROFILES {
            let names: Vec<&str> = profile.fields.iter().map(|(n, _, _)| *n).collect();
            for a in actions_for(profile) {
                for c in &a.constraints {
                    let field = c.split_whitespace().next().unwrap();
                    assert!(
                        names.contains(&field),
                        "{}: constraint {c:?} tests undeclared {field}",
                        profile.name
                    );
                }
            }
        }
    }

    #[test]
    fn every_action_has_phrasings_in_two_languages() {
        // One language cannot show the cross-lingual claim, which is the reason
        // for generating a corpus at all.
        for profile in PROFILES {
            for a in actions_for(profile) {
                assert!(
                    a.aliases.len() >= 4,
                    "{}::{} has {} aliases",
                    profile.name,
                    a.id,
                    a.aliases.len()
                );
            }
        }
    }

    #[test]
    fn paired_and_unpaired_profiles_both_exist() {
        // A corpus where everything converges is as uninformative as the
        // twelve-artifact one it replaces, because a uniform generator produces a
        // uniform result.
        assert!(PROFILES.iter().any(|p| p.paired));
        assert!(
            PROFILES.iter().any(|p| !p.paired),
            "every profile is paired, so nothing fails to converge and the rule \
             is untested"
        );
    }

    #[test]
    fn an_unpaired_profile_actually_differs_from_the_one_it_extends() {
        // The claim in the comment on the profile. If a later edit made these
        // identical, the unpaired profiles would quietly become paired and the
        // corpus would be uniform again.
        let ids =
            |p: &Profile| -> Vec<String> { actions_for(p).into_iter().map(|a| a.id).collect() };
        assert_ne!(ids(&P_ACTUATOR), ids(&P_ACTUATOR_WIDE));
        assert_ne!(ids(&P_ACTUATOR), ids(&P_ACTUATOR_MIN));
    }

    #[test]
    fn the_generated_corpus_says_it_is_synthetic() {
        // A reader who meets one of these in a results table must be able to tell
        // from the artifact itself.
        for (_, m) in corpus(1, 1) {
            assert!(m["guidance"].as_str().unwrap().contains("Synthetic"));
        }
    }

    #[test]
    fn every_manifest_declares_the_format_version() {
        for (_, m) in corpus(1, 2) {
            assert_eq!(m["ure_version"], "1.0");
        }
    }
}
