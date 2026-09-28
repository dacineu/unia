//! The declared menu, and a gate that refuses to score against a menu it cannot
//! confirm.
//!
//! # What the first fixture run found, and it was not what the fixture was for
//!
//! The draft fixture assumed an eight-act menu: `feed`, `play`, `clean`, `sleep`,
//! `get_status`, `check_health`, `give_medicine`, `set_hunger`, on a resource
//! `creature-001`. All eight were reconstructed by its author from documentation,
//! because the real menu is only reachable from files it was forbidden to read.
//!
//! **Four of the eight do not exist.** `get_status`, `check_health`,
//! `give_medicine` and `set_hunger` appear nowhere in `src/` or `patterns/`. And
//! `creature-001` is not a manifest in the tree either — `patterns/actuator/`
//! holds exactly two: `filesystem.ure` and `smart_valve.ure`.
//!
//! So every `accepts` list in that fixture was scored against a fiction in four
//! places out of eight, and nothing said so. **This is precisely the
//! silent-disagreement failure `menu_ref` was added to prevent, demonstrated by
//! the fixture that motivated it.** The fix was in the right direction and the
//! first run is the evidence it was needed.
//!
//! # The real menu, and the finding underneath it
//!
//! The pet declares **four** acts: `feed`, `play`, `clean`, `sleep` — and they
//! are the variants of the `Care` enum in `src/camaduci.rs:272`.
//!
//! **They are not in any manifest.** They are Rust. A `.ure` declares
//! `action_primitives`, and no `.ure` in the tree declares the pet's acts.
//!
//! That is the finding, and it is deeper than the fixture:
//!
//! > **A creature cannot declare its own capabilities to a peer, because its
//! > capabilities are not in the artefact it would declare them in.**
//!
//! The vision has a creature *offering* and *receiving* self-primitives — and the
//! market exists (`broadcast_actuator`, `published`, `discover_resources`), and
//! `Handover`/`receive` exists, and the capability set is a field
//! (`learned: Vec<LearnedRule>`). None of it can carry the four acts, because they
//! are enum variants. **The offering axis of the vision has no payload for the
//! creature it was designed around.**
//!
//! This is also the same shape as everything else in `docs/HANDOVER.md` §3, one
//! level up: something declared, somewhere, that nothing can carry. `clean()` with
//! no caller. A champion written under a key no reader used. A fixture scored
//! against a menu that does not exist.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The acts the creature can actually perform.
///
/// Recorded here as data, with its provenance, because the whole failure this
/// module exists to prevent was a menu that nobody could point at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Menu {
    /// The resource these acts belong to.
    pub resource_id: String,
    /// The declared act ids, sorted.
    pub acts: Vec<String>,
    /// Where this list came from, in a form a reader can check.
    pub declared_in: String,
    /// Whether a `.ure` in the tree declares them, or they live only in Rust.
    ///
    /// **`false` for the creature today, and that flag is the finding.** A menu
    /// with `declared_in_manifest: false` cannot be offered to a peer, because
    /// there is no artefact to offer.
    pub declared_in_manifest: bool,
}

impl Menu {
    /// The pet's four acts, from `Care`.
    ///
    /// `declared_in` names the enum rather than a file path because the path
    /// moves and the enum is the thing; the line number is in the doc comment
    /// above and in `docs/FIXTURE-SPEC.md`.
    pub fn creature() -> Self {
        Menu {
            resource_id: "creature".to_string(),
            acts: vec!["clean".into(), "feed".into(), "play".into(), "sleep".into()],
            declared_in: "Care enum, src/camaduci.rs".to_string(),
            declared_in_manifest: false,
        }
    }

    /// Whether this menu declares an act.
    pub fn declares(&self, act: &str) -> bool {
        self.acts.iter().any(|a| a == act)
    }

    /// Every act a fixture's `accepts` list refers to, across all its rows.
    pub fn undeclared(&self, accepted: impl IntoIterator<Item = String>) -> BTreeSet<String> {
        accepted.into_iter().filter(|a| !self.declares(a)).collect()
    }
}

/// Why a fixture may not be scored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unscoreable {
    /// An `accepts` entry names an act no manifest declares. **The draft fixture
    /// hit this on four of its eight.**
    UndeclaredActs(BTreeSet<String>),
    /// A declared act no row ever accepts — the fixture does not test it.
    UntestedActs(BTreeSet<String>),
    /// The menu is not in a manifest, so it cannot be confirmed against one.
    MenuNotInManifest { declared_in: String },
}

/// Whether a fixture may be scored against a menu.
///
/// **Every one of these is a refusal, and refusing is the feature.** The first run
/// of the fixture scored cleanly against a menu that did not exist, and the only
/// reason anyone noticed is that the author volunteered the uncertainty. A gate
/// that has never refused anything has not been tested.
pub fn check(menu: &Menu, accepts_per_row: &[Vec<String>]) -> Result<(), Unscoreable> {
    if !menu.declared_in_manifest {
        return Err(Unscoreable::MenuNotInManifest {
            declared_in: menu.declared_in.clone(),
        });
    }
    let accepted: BTreeSet<String> = accepts_per_row.iter().flatten().cloned().collect();
    let undeclared = menu.undeclared(accepted.clone());
    if !undeclared.is_empty() {
        return Err(Unscoreable::UndeclaredActs(undeclared));
    }
    let untested: BTreeSet<String> = menu
        .acts
        .iter()
        .filter(|a| !accepted.contains(*a))
        .cloned()
        .collect();
    if !untested.is_empty() {
        return Err(Unscoreable::UntestedActs(untested));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The gate's job is to refuse, so most of these assert refusals.

    use super::*;

    /// **The finding, as a test.** The creature's four acts live in a Rust enum
    /// and in no manifest, so a menu that is not in a manifest cannot be scored
    /// against — and the offering axis of the vision has no payload.
    #[test]
    fn the_creature_menu_is_not_in_a_manifest_and_that_is_the_finding() {
        let m = Menu::creature();
        assert_eq!(m.acts, vec!["clean", "feed", "play", "sleep"]);
        assert!(
            !m.declared_in_manifest,
            "the pet declares its acts as enum variants, so there is no artefact \\
             to offer a peer and `receive` has nothing to receive"
        );
        assert_eq!(
            check(&m, &[vec!["feed".into()]]),
            Err(Unscoreable::MenuNotInManifest {
                declared_in: "Care enum, src/camaduci.rs".into()
            }),
            "and the gate refuses before it even looks at the answers"
        );
    }

    /// **The draft fixture, as a refusal.** Four of its eight acts were fiction.
    /// This is the exact shape of the failure, expressed as the thing that would
    /// have caught it.
    #[test]
    fn an_accepts_list_naming_an_undeclared_act_is_refused() {
        let mut m = Menu::creature();
        m.declared_in_manifest = true; // pretend a manifest exists, to reach the next check
        let draft = vec![
            "feed",
            "play",
            "clean",
            "sleep",
            "get_status",
            "check_health",
            "give_medicine",
            "set_hunger",
        ]
        .into_iter()
        .map(String::from)
        .collect::<BTreeSet<_>>();

        match check(&m, &[draft.into_iter().collect()]) {
            Err(Unscoreable::UndeclaredActs(found)) => {
                assert_eq!(
                    found,
                    ["check_health", "get_status", "give_medicine", "set_hunger"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                    "exactly the four that exist nowhere in src/ or patterns/"
                );
            }
            other => panic!("a fixture naming four non-existent acts was scoreable: {other:?}"),
        }
    }

    /// A declared act no row accepts means the fixture does not test it, and a
    /// score that ignores an untested act is a score with a hole in it.
    #[test]
    fn an_untested_declared_act_is_refused() {
        let mut m = Menu::creature();
        m.declared_in_manifest = true;
        assert_eq!(
            check(&m, &[vec!["feed".into()]]),
            Err(Unscoreable::UntestedActs(
                ["clean", "play", "sleep"]
                    .into_iter()
                    .map(String::from)
                    .collect()
            )),
            "three of the four acts are untested, and a mean over one tested act \\
             is not a score"
        );
    }

    /// And the positive, so the gate is not simply refusing everything: a menu in
    /// a manifest, with every declared act exercised, scores.
    #[test]
    fn a_confirmed_menu_with_full_coverage_is_scoreable() {
        let mut m = Menu::creature();
        m.declared_in_manifest = true;
        let rows = vec![
            vec!["feed".to_string()],
            vec!["play".to_string()],
            vec!["clean".to_string(), "feed".to_string()],
            vec!["sleep".to_string()],
        ];
        assert_eq!(check(&m, &rows), Ok(()), "and the gate allows it");
    }

    /// The gate's own test is that it refuses. A gate that has never refused
    /// anything has not been tested, and the first fixture run is the reason to
    /// write this one.
    #[test]
    fn the_gate_refuses_every_unconfirmed_menu() {
        let m = Menu::creature();
        let rows = vec![vec!["feed".to_string()]];
        assert!(check(&m, &rows).is_err());
    }
}
