//! Tests for trace induction.
//!
//! These assert the safety properties rather than just the happy path, because
//! the properties are the point: a system that promotes learned artifacts is
//! only safe if it refuses to promote thin evidence.

use super::*;
use crate::mcp::store::new_trace;

/// `escalated` chooses hit versus miss. `succeeded` is separate on purpose: a
/// miss that still produced an answer is a success that cost tokens, and
/// conflating the two was the bug the tests first caught.
fn trace(intent: &str, prims: &[&str], escalated: bool) -> Trace {
    let mut t = new_trace(
        intent.to_string(),
        None,
        if escalated { "miss" } else { "hit" },
        if escalated { 1200 } else { 0 },
        if escalated { 400 } else { 0 },
    );
    t.primitives = prims.iter().map(|s| s.to_string()).collect();
    t.succeeded = true;
    t
}

fn failed_trace(intent: &str, prims: &[&str]) -> Trace {
    let mut t = trace(intent, prims, true);
    t.succeeded = false;
    t
}

#[cfg(test)]
mod grouping {
    use super::*;

    #[test]
    fn groups_by_primitive_sequence_not_by_phrasing() {
        let traces = vec![
            trace("add a rust dependency", &["GetValue", "SetValue"], true),
            trace("bump the crate and refresh", &["GetValue", "SetValue"], true),
        ];
        let groups = group_traces(&traces);
        assert_eq!(groups.len(), 1, "two phrasings of one sequence are one rule");
        let g = &groups["GetValue_SetValue"];
        assert_eq!(g.observations(), 2);
        // Both escalated, and both succeeded: escalation costs tokens but is
        // still evidence that the sequence is the right one.
        assert_eq!(g.hits, 2);
        assert_eq!(g.misses, 0);
        assert!(g.tokens_saved > 0, "escalation is where the cost is recorded");
    }

    #[test]
    fn splits_differing_sequences() {
        let traces = vec![
            trace("shut the valve", &["SetValue"], true),
            trace("read the config", &["GetValue"], true),
        ];
        assert_eq!(group_traces(&traces).len(), 2);
    }

    #[test]
    fn ignores_traces_with_no_recorded_sequence() {
        let traces = vec![
            trace("no sequence recorded", &[], true),
            trace("also none", &[], true),
        ];
        assert!(group_traces(&traces).is_empty(),
                "a trace without primitives is not inducible and must not be guessed at");
    }
}

#[cfg(test)]
mod evidence_gate {
    use super::*;

    #[test]
    fn rejects_a_single_observation() {
        let g = group_traces(&[trace("do the thing once", &["SetValue"], true)]);
        let g = &g["SetValue"];
        assert!(!g.is_evidentiary(), "one observation is an anecdote");
        assert!(induce(g).is_err());
    }

    #[test]
    fn rejects_repeated_single_phrasing() {
        // Breadth without evidence the sequence is reliable: the same sentence
        // ten times is not ten observations of a rule.
        let traces: Vec<Trace> = (0..10)
            .map(|_| trace("do the thing", &["SetValue"], true))
            .collect();
        let g = &group_traces(&traces)["SetValue"];
        assert_eq!(g.observations(), 1);
        assert!(!g.is_evidentiary());
    }

    #[test]
    fn rejects_a_sequence_that_usually_fails() {
        let mut traces = vec![
            trace("first phrasing", &["SetValue"], false),
            trace("second phrasing", &["SetValue"], false),
        ];
        for i in 0..8 {
            traces.push(failed_trace(&format!("failing phrasing {i}"), &["SetValue"]));
        }
        let g = &group_traces(&traces)["SetValue"];
        assert!(g.failure_ratio() > MAX_FAILURE_RATIO);
        assert!(!g.is_evidentiary(), "a rule that mostly fails is not a rule");
    }

    #[test]
    fn confidence_rises_with_breadth_and_falls_with_failures() {
        let narrow = group_traces(&[
            trace("alpha", &["SetValue"], false),
            trace("beta", &["SetValue"], false),
        ]);
        let wide = group_traces(&[
            trace("alpha", &["SetValue"], false),
            trace("beta", &["SetValue"], false),
            trace("gamma", &["SetValue"], false),
            trace("delta", &["SetValue"], false),
        ]);
        let n = narrow["SetValue"].confidence();
        let w = wide["SetValue"].confidence();
        assert!(w > n, "more phrasings must mean more confidence: {w} vs {n}");

        let degraded = group_traces(&[
            trace("alpha", &["SetValue"], false),
            failed_trace("beta", &["SetValue"]),
        ]);
        assert!(degraded["SetValue"].confidence() < w);
    }
}

#[cfg(test)]
mod synthesis {
    use super::*;

    fn evidenced() -> Induction {
        group_traces(&[
            trace("add a rust dependency", &["GetValue", "SetValue"], true),
            trace("bump the crate and refresh the lock", &["GetValue", "SetValue"], true),
        ])["GetValue_SetValue"]
            .clone()
    }

    #[test]
    fn learned_aliases_are_the_observed_phrasings() {
        let c = induce(&evidenced()).expect("two clean phrasings are evidentiary");
        assert!(c.learned_aliases.contains(&"add a rust dependency".to_string()));
        assert!(c
            .learned_aliases
            .contains(&"bump the crate and refresh the lock".to_string()));
    }

    #[test]
    fn action_carries_both_underscore_and_space_forms() {
        // Divergence D4: the Bridge normalises underscores to spaces for the
        // action id but not for aliases, so both forms must be supplied.
        let c = induce(&evidenced()).unwrap();
        let a = &c.manifest.action_primitives[0];
        let aliases = a.aliases.as_ref().expect("aliases are always synthesised");
        assert!(aliases.contains(&"GetValue_SetValue".to_string()), "got {aliases:?}");
        assert!(aliases.contains(&"GetValue SetValue".to_string()), "got {aliases:?}");
    }

    #[test]
    fn resource_id_is_its_own_content_address() {
        let c = induce(&evidenced()).unwrap();
        assert_eq!(c.manifest.resource_id, c.du_uuid.to_string());
    }

    #[test]
    fn identical_rules_deduplicate_by_content_address() {
        let a = induce(&group_traces(&[
            trace("one phrasing", &["SetValue"], false),
            trace("another phrasing", &["SetValue"], false),
        ])["SetValue"])
        .unwrap();
        let b = induce(&group_traces(&[
            trace("completely different words", &["SetValue"], false),
            trace("yet another", &["SetValue"], false),
        ])["SetValue"])
        .unwrap();
        // The bodies differ only in aliases, so addresses must differ; the
        // addresses are equal only when the induced rule is identical.
        assert_ne!(a.du_uuid, b.du_uuid,
                   "different alias sets are different patterns, not one deduped");
    }

    #[test]
    fn candidate_is_never_emitted_as_champion() {
        let c = induce(&evidenced()).unwrap();
        assert_eq!(c.lifecycle, "candidate",
                   "promotion must be a separate, deliberate step");
    }

    #[test]
    fn failures_become_recorded_preconditions() {
        let mut traces = vec![
            trace("alpha", &["SetValue"], false),
            trace("beta", &["SetValue"], false),
        ];
        traces.push(failed_trace("gamma", &["SetValue"]));
        let c = induce(&group_traces(&traces)["SetValue"]).unwrap();
        assert!(!c.manifest.action_primitives[0].constraints.is_empty(),
                "an observed failure should be recorded as a precondition");
    }

    #[test]
    fn tokens_saved_is_carried_from_hits() {
        let mut t = trace("hit phrasing one", &["SetValue"], false);
        t.tokens_in = 500;
        let mut t2 = trace("hit phrasing two", &["SetValue"], false);
        t2.tokens_out = 250;
        let g = &group_traces(&[t, t2])["SetValue"];
        assert_eq!(g.tokens_saved, 750);
    }

    #[test]
    fn induce_all_skips_unevidentiary_groups() {
        let traces = vec![
            trace("lone phrasing", &["SetValue"], true),
            trace("alpha", &["GetValue"], false),
            trace("beta", &["GetValue"], false),
        ];
        let all = induce_all(&traces).expect("one evidentiary group must induce");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].signature, "GetValue");
    }
}
