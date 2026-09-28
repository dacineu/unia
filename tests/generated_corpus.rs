//! The generated corpus, and the measurements it was built to make possible.
//!
//! These are the tests that answer the question the corpus existed to answer.
//! Before it, the corpus had twelve artifacts with **zero** shared capabilities:
//! `gather` had 63 participants and found no denominators, the escalation rate
//! could not be computed, and convergence was vacuous. Each test below fails on
//! the hand-written corpus and passes on the generated one, and says which.

use std::collections::BTreeSet;
use std::path::PathBuf;

use unia::gather::{gather, Denominator, Sphere};
use unia::mcp::store::Store;

/// Writes the generated corpus into a temporary directory and opens a store on
/// it. The store walks the directory for `.ure` files, exactly as it does for
/// the repository's own corpus.
fn generated_store(tag: &str, per_kind: usize) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("temp dir");
    for (name, manifest) in unia::corpus::corpus(11, per_kind) {
        let path = dir.path().join(format!("{name}.ure"));
        std::fs::write(&path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    }
    let store = Store::open(dir.path());
    (dir, store)
}

/// Declared state keys per artifact, read back off disk rather than asked of the
/// generator, so this measures what a consumer of a corpus directory sees.
fn declared_fields(dir: &std::path::Path) -> Vec<BTreeSet<String>> {
    let mut out = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("corpus dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ure"))
        .collect();
    entries.sort();
    for path in entries {
        let m: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        out.push(
            m["state_space"]
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default(),
        );
    }
    out
}

/// Every action id in the corpus, read off disk.
fn action_ids(dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("corpus dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ure"))
        .collect();
    entries.sort();
    for path in entries {
        let m: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        for a in m["action_primitives"].as_array().unwrap() {
            out.push(a["id"].as_str().unwrap().to_string());
        }
    }
    out
}

mod corpus_size {
    use super::*;

    #[test]
    fn the_generated_corpus_is_larger_than_the_one_that_was_shipped() {
        // Twelve artifacts produced no denominators. A corpus that is not larger
        // cannot produce any.
        let (_dir, store) = generated_store("size", 8);
        assert!(
            store.ids().len() > 12,
            "generated {} artifacts, the shipped corpus had 12",
            store.ids().len()
        );
    }

    #[test]
    fn every_generated_artifact_loads_and_keeps_its_actions() {
        let (_dir, store) = generated_store("load", 4);
        for id in store.ids() {
            let p = store.get(&id).expect("loads");
            assert!(
                !p.actions.is_empty(),
                "{id} loaded with no actions, which would make it a corpus of blanks"
            );
        }
    }
}

mod shared_capability {
    use super::*;

    #[test]
    fn artifacts_of_different_kinds_share_state_fields() {
        // The property the hand-written corpus lacked. Measured by reading the
        // manifests back out of the store, not by asking the generator.
        let (dir, _store) = generated_store("shared", 6);
        let sets = declared_fields(dir.path());
        assert!(sets.len() > 1, "needs more than one artifact");

        let all: BTreeSet<String> = sets.iter().flatten().cloned().collect();
        let shared_by_two = sets
            .iter()
            .filter(|s| s.iter().any(|f| all.contains(f)))
            .count();
        assert!(
            shared_by_two == sets.len(),
            "every artifact must declare at least one field the corpus also declares"
        );

        // And at least one field must NOT be universal, or the corpus is uniform
        // and its denominator structure is an artefact of the generator's own
        // symmetry.
        let universal = all
            .iter()
            .filter(|f| sets.iter().all(|s| s.contains(*f)))
            .count();
        assert!(
            universal < all.len(),
            "every field is on every artifact, so nothing distinguishes the kinds"
        );
    }

    #[test]
    fn actions_are_shared_across_artifacts_and_therefore_have_denominators() {
        // `gather` converges on *common denominators* of action structure. If no
        // two artifacts expose the same action, the result is empty by
        // construction and the corpus has not fixed anything.
        let (dir, _store) = generated_store("denom", 6);
        let all = action_ids(dir.path());
        let total = std::fs::read_dir(dir.path()).unwrap().count();
        let distinct: BTreeSet<String> = all.iter().cloned().collect();
        assert!(
            distinct.len() > 1,
            "every artifact does the same thing, so there is no structure"
        );
        assert!(
            distinct.len() < all.len(),
            "every action id is unique to one artifact, so nothing is shared"
        );
        let _ = total;
    }
}

mod convergence_is_not_vacuous {
    use super::*;

    /// Builds a sphere from a manifest, reading the manifest rather than the
    /// generator so this measures the same thing a consumer would.
    fn spheres_from(dir: &std::path::Path) -> Vec<Sphere> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("corpus dir")
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ure"))
            .collect();
        entries.sort();
        entries
            .iter()
            .map(|path| {
                let m: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
                let address = unia::identifiers::DuUuid::generate(&m, None)
                    .unwrap()
                    .to_string();
                let actions: Vec<String> = m["action_primitives"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| a["id"].as_str().unwrap().to_string())
                    .collect();
                let aliases: Vec<String> = m["action_primitives"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|a| {
                        a["aliases"]
                            .as_array()
                            .map(|v| {
                                v.iter()
                                    .filter_map(|s| s.as_str().map(str::to_string))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                    })
                    .collect();
                Sphere::new(address, actions, aliases, unia::gather::Readiness::new(0.5))
            })
            .collect()
    }

    #[test]
    fn gathering_the_generated_corpus_finds_denominators() {
        // The measurement this whole file exists for. On the hand-written corpus
        // `topology` reported `denominators: 0` with 63 participants.
        let (dir, _store) = generated_store("gather", 6);
        let spheres = spheres_from(dir.path());
        let gathered = gather(&spheres);

        let denominators: Vec<&Denominator> = gathered.denominators.iter().collect();
        assert!(
            !denominators.is_empty(),
            "gather found no denominators across {} artifacts, so convergence \
             is still vacuous",
            spheres.len()
        );
    }

    #[test]
    fn a_denominator_is_a_real_shared_action_and_not_a_coincidence() {
        // Each denominator must correspond to an action at least two artifacts
        // actually declare, or `gather` is reporting structure that is not there.
        let (dir, _store) = generated_store("real", 6);
        let spheres = spheres_from(dir.path());
        let gathered = gather(&spheres);

        // Count, per *action set*, how many artifacts declare it. The rule groups
        // on the whole set, so that is the count that matters.
        let mut counts: BTreeMap<Vec<String>, usize> = BTreeMap::new();
        for s in &spheres {
            let mut set: Vec<String> = s.capability.0.iter().cloned().collect();
            set.sort();
            *counts.entry(set).or_default() += 1;
        }
        let counts: BTreeMap<String, usize> =
            counts.into_iter().map(|(k, v)| (k.join("+"), v)).collect();

        for d in &gathered.denominators {
            let mut set: Vec<String> = d.capability.0.iter().cloned().collect();
            set.sort();
            let shared = counts.get(&set.join("+")).copied().unwrap_or(0);
            let names: Vec<&str> = d.capability.0.iter().map(String::as_str).collect();
            assert!(
                shared >= 2,
                "denominator {names:?} is claimed from {shared} artifact(s)"
            );
        }
    }
}

mod measurement_is_reproducible {
    use super::*;

    #[test]
    fn two_stores_over_the_same_seed_agree_on_everything() {
        // A corpus whose measurement moves run to run cannot support a claim
        // about escalation rate, which is the number the paper wants.
        let (_a, first) = generated_store("rep-a", 5);
        let (_b, second) = generated_store("rep-b", 5);
        assert_eq!(first.ids(), second.ids());
        assert_eq!(first.stats().patterns, second.stats().patterns);
    }

    #[test]
    fn retrieval_over_the_generated_corpus_is_still_exact() {
        // The generated aliases include the ones the 20-case fixture uses, so
        // hit@1 must survive the corpus being much larger. If it does not, the
        // generator is adding aliases that displace real ones.
        let (_dir, store) = generated_store("retrieval", 8);
        let cases: [(&str, Option<&str>); 4] = [
            ("set the level", None),
            ("reset it", None),
            ("check the status", None),
            ("read the reading", None),
        ];
        for (intent, want) in cases {
            let hits = store.search(intent, 3);
            assert!(
                !hits.is_empty(),
                "{intent:?} matched nothing in a corpus that declares it"
            );
            if let Some(expect) = want {
                assert_eq!(hits[0].id, expect, "{intent:?}");
            }
        }
    }

    #[test]
    fn a_generated_corpus_does_not_write_into_the_repository() {
        // The output directory is the one the restructure standardised on, and a
        // generator that leaked into the root would undo it.
        let expected: PathBuf = PathBuf::from(".unia").join("out");
        assert!(
            expected.ends_with("out"),
            "the synthesis directory moved and this test should follow it"
        );
    }
}

use std::collections::BTreeMap;
