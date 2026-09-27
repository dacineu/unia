//! Does a player's care actually reach induction?
//!
//! This is the claim the ca™maduci browser client rests on: a click becomes a
//! primitive sequence, the sequence becomes a trace, the trace becomes a
//! candidate artifact, and the phrasings the player used become that candidate's
//! aliases. Nothing in the crate closed this loop before, so the test drives the
//! same path a browser does and asserts on what comes out the far end.
//!
//! The important detail is that the gate counts *distinct phrasings*, not
//! occurrences. Five clicks of the same button with the same wording are one
//! observation and cannot clear `MIN_OBSERVATIONS`; five clicks worded five
//! different ways are five observations and do. That is the gate working as
//! specified, and it is why the client sends the player's own words.

use unia::induce::induce_all;
use unia::mcp::store::{Store, Trace};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// A self-cleaning directory for the trace log under test.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "unia-pet-induction-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).expect("scratch dir");
        Scratch(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Writes a trace log the way the ca™maduci server does.
fn write_log(dir: &Path, lines: &[(&str, &[&str])]) {
    let body: String = lines
        .iter()
        .map(|(intent, prims)| {
            let prims: Vec<String> = prims.iter().map(|p| format!("\"{p}\"")).collect();
            format!(
                "{{\"intent\":\"{intent}\",\"resource_id\":\"ca-001\",\"outcome\":\"hit\",\
                 \"primitives\":[{}],\"succeeded\":true}}\n",
                prims.join(",")
            )
        })
        .collect();
    std::fs::write(dir.join("traces.jsonl"), body).expect("write log");
}

const FEED: &[&str] = &["SetValue", "CheckSense"];

#[test]
fn a_repeated_click_with_identical_wording_never_clears_the_gate() {
    // One phrasing is one observation however many times it recurs, so no
    // candidate can be induced and the rule cannot be promoted on repetition
    // alone.
    let dir = Scratch::new();
    let repeated: Vec<(&str, &[&str])> = (0..9).map(|_| ("feed the ca maduci", FEED)).collect();
    write_log(dir.path(), &repeated);

    let store = Store::open(dir.path());
    assert_eq!(store.traces().len(), 9);
    assert!(induce_all(store.traces()).unwrap().is_empty());
}

#[test]
fn the_same_button_worded_differently_induces_a_rule() {
    // This is the loop the browser client closes. The action is identical every
    // time; only the wording varies, which is what a real player supplies.
    let dir = Scratch::new();
    write_log(
        dir.path(),
        &[
            ("give it dinner", FEED),
            ("pour some kibble", FEED),
            ("serve the food", FEED),
            ("refill the bowl", FEED),
            ("she needs feeding", FEED),
        ],
    );

    let store = Store::open(dir.path());
    let candidates = induce_all(store.traces()).expect("induction ran");
    assert_eq!(candidates.len(), 1, "one rule from one primitive sequence");

    let c = &candidates[0];
    assert_eq!(c.signature, "SetValue_CheckSense");
    assert_eq!(c.learned_aliases.len(), 5, "every phrasing becomes an alias");
    for phrase in ["give it dinner", "pour some kibble", "she needs feeding"] {
        assert!(
            c.learned_aliases.iter().any(|a| a == phrase),
            "alias {phrase} was not learned: {:?}",
            c.learned_aliases
        );
    }
}

#[test]
fn different_actions_induce_different_rules() {
    let dir = Scratch::new();
    write_log(
        dir.path(),
        &[
            ("give it dinner", FEED),
            ("pour some kibble", FEED),
            ("serve the food", FEED),
            ("throw a ball", &["SetValue", "Emit"]),
            ("fetch the ball", &["SetValue", "Emit"]),
        ],
    );

    let store = Store::open(dir.path());
    let candidates = induce_all(store.traces()).expect("induction ran");
    assert_eq!(candidates.len(), 2, "one rule per primitive sequence");

    let mut signatures: Vec<&str> = candidates.iter().map(|c| c.signature.as_str()).collect();
    signatures.sort();
    assert_eq!(signatures, vec!["SetValue_CheckSense", "SetValue_Emit"]);
}

#[test]
fn a_trace_with_no_primitive_sequence_is_not_induced_from() {
    // Skipping rather than guessing is the point of the module: a rule induced
    // from a guess is the failure mode it exists to prevent.
    let dir = Scratch::new();
    let mut body = String::new();
    for phrase in ["one", "two", "three"] {
        body.push_str(&format!(
            "{{\"intent\":\"{phrase}\",\"resource_id\":\"ca-001\",\"outcome\":\"hit\"}}\n"
        ));
    }
    std::fs::write(dir.path().join("traces.jsonl"), body).unwrap();

    let store = Store::open(dir.path());
    assert_eq!(store.traces().len(), 3);
    assert_eq!(store.malformed_traces(), 0, "the lines are valid, just sequenceless");
    assert!(induce_all(store.traces()).unwrap().is_empty());
}

#[test]
fn two_players_using_the_same_wordings_induce_the_same_address() {
    // Content addressing is what makes the loop safe to run across a mesh: two
    // people who phrase their care identically converge on one artifact rather
    // than two, so the rule is stored once and shared.
    let phrases = [("give it dinner", FEED), ("pour some kibble", FEED)];

    let mut addresses = Vec::new();
    for who in ["ca-001", "ca-002"] {
        let dir = Scratch::new();
        let body: String = phrases
            .iter()
            .map(|(intent, prims)| {
                let prims: Vec<String> = prims.iter().map(|p| format!("\"{p}\"")).collect();
                format!(
                    "{{\"intent\":\"{intent}\",\"resource_id\":\"{who}\",\"outcome\":\"hit\",\
                     \"primitives\":[{}],\"succeeded\":true}}\n",
                    prims.join(",")
                )
            })
            .collect();
        std::fs::write(dir.path().join("traces.jsonl"), body).unwrap();

        let store = Store::open(dir.path());
        let c = induce_all(store.traces()).unwrap();
        assert_eq!(c.len(), 1);
        addresses.push(c[0].du_uuid);
    }

    assert_eq!(
        addresses[0], addresses[1],
        "identical phrasings must produce one address, not two"
    );
}

#[test]
fn a_neglected_pet_leaves_nothing_to_induce_from() {
    // The counterpart to the loop above: a pet nobody tends produces no traces at
    // all, so there is no evidence and no rule. Neglect is the absence of a
    // record, not a record of absence.
    let dir = Scratch::new();
    write_log(dir.path(), &[]);
    let store = Store::open(dir.path());
    assert!(store.traces().is_empty());
    assert!(induce_all(store.traces()).unwrap().is_empty());
}

#[test]
fn the_store_exposes_its_traces_for_induction() {
    // `induce_all` takes a slice and nothing in the crate could supply one, so a
    // populated trace log could not reach the module that learns from it. This
    // asserts the accessor exists and returns what was written, in order.
    let dir = Scratch::new();
    write_log(dir.path(), &[("first", FEED), ("second", FEED)]);
    let store = Store::open(dir.path());
    let by_intent: BTreeMap<&str, usize> = store
        .traces()
        .iter()
        .enumerate()
        .map(|(i, t)| (t.intent.as_str(), i))
        .collect();
    assert_eq!(by_intent.get("first"), Some(&0));
    assert_eq!(by_intent.get("second"), Some(&1));
}

#[test]
fn a_trace_from_the_browser_format_parses_without_loss() {
    // Exactly the line the ca™maduci server writes, including the fields it fills
    // in from the player's own wording.
    let line = r#"{"ts":1790541480,"intent":"give it dinner","resource_id":"ca-001","outcome":"hit","tokens_in":0,"tokens_out":0,"primitives":["SetValue","CheckSense"],"succeeded":true}"#;
    let t: Trace = serde_json::from_str(line).expect("parses");
    assert_eq!(t.intent, "give it dinner");
    assert_eq!(t.primitives, vec!["SetValue", "CheckSense"]);
    assert_eq!(t.tokens_in, 0, "tiers 1 and 2 cost no tokens");
    assert!(t.succeeded);
}
