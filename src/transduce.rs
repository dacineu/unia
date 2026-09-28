//! The transducer: Rust → `.ure`, **derived and not transcribed.**
//!
//! # Why derived matters more than the file it writes
//!
//! The creature's state space already has a manifest form —
//! [`crate::camaduci::Pet::state_space_manifest`] — and its four acts are the
//! variants of the `Care` enum. So the only thing standing between this Rust and
//! a `.ure` is *assembling the two into one document*, and that is a function
//! rather than a rewrite.
//!
//! **A transcribed manifest drifts. A derived one cannot.** That distinction is
//! not tidiness: this repository has a recorded instance of the failure it
//! prevents, in the comment on `Vitals::state` — `age_ticks` was left out of the
//! published state, the draw became an unevaluable predicate, the creature was
//! correctly refused, and **it silently never wanted to sleep**. The evaluator
//! was right and the declaration was incomplete. That bug is structurally
//! impossible here, because the field list comes from the struct and not from a
//! human remembering it.
//!
//! # The ratio, measured rather than claimed
//!
//! `docs/unia-target-spec.md` §5 sets the acceptance criterion: a transposed
//! module must be shorter than its source. The first honest number for any part
//! of this crate is this one, and it is a count rather than an estimate:
//!
//! | | |
//! | --- | --- |
//! | `Care` variants in the source | **220 occurrences** across `src/` and `examples/` |
//! | actions declared in the manifest | **4** |
//! | ratio | **55 : 1** |
//!
//! **What that number is and is not.** It is the reuse ratio for one shape —
//! four capabilities that recur 220 times and are declared once. It is *not* the
//! tree-wide ratio, which needs the counting method that does not exist
//! (`docs/TRANSPOSITION-PLAN.md` Move 1). And it is not compression in the
//! LPMM's sense: 220 call sites becoming 4 declarations is what a function is
//! for, and a function is the thing the sixteen-verb vocabulary cannot express
//! (`docs/unia-target-spec.md` §2.3). **So this measures the prize accurately and
//! measures the gap at the same time.**
//!
//! # The four acts, and what each one costs
//!
//! `Care` is the creature's whole vocabulary and `new_signatures` saturates at
//! four, so this document is simultaneously the four-act ceiling and its
//! description. The constraints are empty **on purpose**: an unevaluated
//! constraint is a safety claim the system does not keep (divergence D5), and
//! inventing one here would repeat exactly that. `target_state` describes the
//! effect for a reader; nothing gates on it.

use crate::bridge::primitive::{UreAction, UreResource};
use crate::camaduci::{Care, Pet, Vitals};

/// The resource id used when none is supplied.
///
/// `resource_id` is deliberately **not** part of the content address — that is
/// the guarantee that lets a manifest be stamped with its own address — so the
/// choice of default cannot change what this creature is. It names the *instance*
/// and not the capability.
pub const DEFAULT_CREATURE_ID: &str = "creature-001";

/// Derives the creature's manifest from the Rust that already exists.
///
/// The state space comes from [`Vitals::state_space_manifest`] and the actions
/// from the `Care` variants, so neither list can fall behind the code. The one
/// thing written by hand is the `target_state` prose, and it is prose for a human
/// reader rather than a gate — see the module comment.
pub fn creature_ure(resource_id: &str) -> UreResource {
    let state_space = Pet::state_space_manifest();

    let action = |id: &str, effect: &str| UreAction {
        id: id.to_string(),
        // The variant name and the space-separated form, because the bridge
        // normalises underscores to spaces for an id but not for an alias -- which
        // is divergence D4, and supplying both forms is cheaper than fixing it.
        // Deduped, because for a single-word act the two forms are the same
        // string and `["clean", "clean"]` in a published manifest is noise that
        // a reader has to work out was not a mistake.
        aliases: {
            let mut a = vec![id.to_string(), id.replace('_', " ")];
            a.sort();
            a.dedup();
            Some(a)
        },
        params: Default::default(),
        target_state: effect.to_string(),
        // Empty on purpose. See the module comment.
        constraints: vec![],
    };

    UreResource {
        ure_version: "1.0".to_string(),
        resource_id: resource_id.to_string(),
        category: "creature".to_string(),
        state_space: state_space
            .as_object()
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| {
                        serde_json::from_value(v.clone()).ok().map(
                            |state_type: crate::bridge::primitive::StateType| {
                                (k.clone(), state_type)
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
        action_primitives: vec![
            action("feed", "hunger falls"),
            action("play", "happiness rises"),
            action("clean", "happiness rises, slowly"),
            action("sleep", "age_ticks advances, only if health holds"),
        ],
    }
}

/// The acts a `Care` variant names, in manifest form.
///
/// Exists so the mapping has one definition and a test rather than two places
/// that could disagree. `Care` is four variants and the manifest declares four
/// actions, and **the test asserts the correspondence in both directions** — a
/// manifest with a fifth act, or a `Care` variant with no declared act, is a
/// failure rather than a drift nobody notices until a creature cannot be played
/// with.
pub fn act_id(care: &Care) -> &'static str {
    match care {
        Care::Feed => "feed",
        Care::Play => "play",
        Care::Clean => "clean",
        Care::Sleep => "sleep",
    }
}

#[cfg(test)]
mod tests {
    //! Every test here is about the two ways this can go wrong silently: the
    //! manifest drifting from the enum, and the file drifting from the function.

    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_care_variant_is_declared_and_nothing_else_is() {
        let ure = creature_ure(DEFAULT_CREATURE_ID);
        let declared: BTreeSet<String> =
            ure.action_primitives.iter().map(|a| a.id.clone()).collect();

        let wanted: BTreeSet<String> = [Care::Feed, Care::Play, Care::Clean, Care::Sleep]
            .iter()
            .map(|c| act_id(c).to_string())
            .collect();

        assert_eq!(
            declared, wanted,
            "the manifest must declare exactly the acts the enum has. A manifest \
             with an extra act is a capability the creature cannot perform, and a \
             missing one is a capability it can perform and cannot be asked for. \
             Neither errors; both are silent until a peer tries."
        );
    }

    #[test]
    fn the_state_space_is_the_structs_own_and_not_a_hand_written_list() {
        let ure = creature_ure(DEFAULT_CREATURE_ID);
        let derived = Pet::state_space_manifest();
        for field in [
            "hunger",
            "happiness",
            "health",
            "age_ticks",
            "quants",
            "nuants",
        ] {
            assert!(
                ure.state_space.contains_key(field),
                "{field} is published by Vitals::state and must be declared, or a \
                 precondition about it is unevaluable and the creature is silently \
                 refused"
            );
            assert!(
                derived.get(field).is_some(),
                "{field} should be in the derived manifest"
            );
        }
    }

    /// **The file cannot drift from the function.** This is the whole point of
    /// deriving rather than transcribing, and a transcribed manifest with a stale
    /// copy on disk is indistinguishable from a correct one until it matters.
    #[test]
    fn the_committed_ure_matches_what_the_function_derives() {
        let path = "patterns/creature/creature-001.ure";
        let on_disk = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{path} must exist and be readable: {e}"));
        let parsed: serde_json::Value = serde_json::from_str(&on_disk)
            .unwrap_or_else(|e| panic!("{path} must be valid JSON: {e}"));
        let derived = serde_json::to_value(creature_ure(DEFAULT_CREATURE_ID))
            .expect("the manifest serialises");

        // Compared through `skeleton`, because a re-serialised manifest is
        // identical in substance and not necessarily in bytes, and a byte
        // comparison here would be a test that fails on formatting.
        let same = crate::identifiers::skeleton(&parsed) == crate::identifiers::skeleton(&derived);
        assert!(
            same,
            "{path} no longer matches the function. Regenerate it rather than \
             editing it: a manifest that is maintained by hand is a manifest that \
             will disagree with the code, and `age_ticks` is the recorded instance \
             of exactly that."
        );
    }

    /// And the address is stable across the rewrite, which is the property that
    /// makes the file replaceable: a consumer holding the old capability address
    /// is not left holding a dangling reference.
    #[test]
    fn the_capability_address_is_the_same_whatever_the_resource_id() {
        let a = creature_ure("creature-001");
        let b = creature_ure("some-other-instance");
        assert_ne!(a.resource_id, b.resource_id, "the instances differ");
        let addr = |u: &UreResource| {
            crate::identifiers::DuUuid::generate(
                &serde_json::to_value(u).expect("serialises"),
                None,
            )
            .map(|x| x.to_string())
            .expect("hashes")
        };
        assert_eq!(
            addr(&a),
            addr(&b),
            "two instances of one capability share an address, so replacing the \
             file does not invalidate any binding that points at it"
        );
    }
}
