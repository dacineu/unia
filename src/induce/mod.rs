//! Induction: turns recorded LLM interactions into candidate `.ure` actuators.
//!
//! This is the loop that was missing. Traces were recorded and champions could
//! be set, but nothing connected the two, so the system accumulated logs rather
//! than learning from them.
//!
//! ## What is being learned, and what is not
//!
//! unia does not learn weights. It has no parameters and no gradient, so it will
//! never be an LLM, and nothing here should be read as an attempt to build one.
//! What it learns is a **rule**: a sequence of universal primitives, with the
//! observed phrasings bound to it as aliases. That is a deliberate choice
//! rather than a limitation to apologise for, and it follows the strongest
//! result in the agent-skills literature — NSI's finding that agents fail when
//! they "reason from scratch at every step or repeat fixed scripts." The fix is
//! to induce rules with explicit control flow, not to memorise one phrasing.
//!
//! Learning rules rather than weights is also why this needs no GPU. The
//! induction below is integer and string work over a trace log.
//!
//! ## The safety property
//!
//! The dominant risk in a system that promotes learned artifacts is that a
//! wrong pattern becomes fast, silent and reused. This module is built to make
//! that hard rather than merely unlikely:
//!
//!   * Candidates start at `candidate`, never `champion`. Promotion is a
//!     separate, deliberate step.
//!   * A candidate is only emitted when the evidence clears `MIN_OBSERVATIONS`
//!     distinct phrasings. One observation is not a pattern, it is an anecdote.
//!   * Confidence is reported, and callers are expected to gate on it.
//!   * Observed failures become `constraints` rather than being discarded, so a
//!     precondition is learned instead of being re-paid for on every call.

use crate::bridge::primitive::{UreAction, UreResource};
use crate::identifiers::DuUuid;
use crate::mcp::store::Trace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Below this many distinct phrasings, a group is an anecdote and no candidate is
/// emitted. The learned alias set is the evidence, so a single observation would
/// yield an actuator that matches exactly one sentence and generalises to
/// nothing.
pub const MIN_OBSERVATIONS: usize = 2;

/// Observed failures at or above this ratio block promotion. A sequence that
/// fails more often than it succeeds is not a rule.
///
/// Failure means the interaction did not produce a usable result, which is
/// `Trace::succeeded`, not `Trace::outcome`. An escalation to a provider is a
/// success that cost tokens, and counting it as a failure would reject almost
/// every group, because escalation is what every interaction looks like before a
/// pattern exists to serve it.
pub const MAX_FAILURE_RATIO: f64 = 0.5;

/// A set of traces that resolved to the same primitive sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct Induction {
    /// Stable key for the group: the primitive sequence, underscore-joined.
    pub signature: String,
    /// Every distinct phrasing observed for this sequence. These become the
    /// action's aliases, which is the whole point: the agent learns the
    /// vocabulary users actually use, not the one the author guessed.
    pub observations: BTreeSet<String>,
    pub hits: usize,
    pub misses: usize,
    pub tokens_saved: u64,
}

impl Induction {
    pub fn observations(&self) -> usize {
        self.observations.len()
    }

    pub fn total(&self) -> usize {
        self.hits + self.misses
    }

    pub fn failure_ratio(&self) -> f64 {
        if self.total() == 0 {
            return 1.0;
        }
        self.misses as f64 / self.total() as f64
    }

    /// Evidence strength in `0.0..=1.0`.
    ///
    /// Two independent factors, because either alone is insufficient. Many
    /// observations of one phrasing give breadth without evidence the sequence
    /// is reliable; a clean record of one phrasing gives reliability without
    /// breadth. Confidence is deliberately multiplicative so a candidate must
    /// earn both, and is capped by the failure ratio so a sequence that often
    /// fails can never reach a promotable confidence.
    pub fn confidence(&self) -> f64 {
        // Log-scaled so it is monotone in the evidence but never saturates.
        // A breadth term of observations/MIN_OBSERVATIONS, clamped to 1, reads 1.0
        // the moment the minimum is met, which makes the fourth distinct
        // phrasing worth exactly as much as the second and stops the score
        // distinguishing a twice-seen rule from a heavily used one.
        let o = self.observations().max(1) as f64;
        let breadth = o.ln() / (o.ln() + 1.0);
        let reliability = 1.0 - self.failure_ratio();
        (breadth * reliability).clamp(0.0, 1.0)
    }

    /// Whether this group is eligible to become a candidate at all.
    pub fn is_evidentiary(&self) -> bool {
        self.observations() >= MIN_OBSERVATIONS && self.failure_ratio() <= MAX_FAILURE_RATIO
    }
}

/// Groups traces by the primitive sequence they resolved to.
///
/// The sequence, not the phrasing, is the grouping key. Two different sentences
/// that reduce to the same primitives are evidence for one rule; the same
/// sentence that resolved to different primitives at different times is not
/// evidence of anything, and is split accordingly.
pub fn group_traces(traces: &[Trace]) -> BTreeMap<String, Induction> {
    let mut groups: BTreeMap<String, Induction> = BTreeMap::new();

    for t in traces {
        if t.primitives.is_empty() {
            // A trace with no primitive sequence recorded cannot be inducted
            // from. Skipping rather than guessing is the point: an actuator
            // induced from a guess is the failure mode this module exists to
            // prevent.
            continue;
        }
        let signature = t.primitives.join("_");
        let entry = groups
            .entry(signature.clone())
            .or_insert_with(|| Induction {
                signature,
                observations: BTreeSet::new(),
                hits: 0,
                misses: 0,
                tokens_saved: 0,
            });
        let intent = t.intent.trim().to_lowercase();
        if !intent.is_empty() {
            entry.observations.insert(intent);
        }
        if t.succeeded {
            entry.hits += 1;
            entry.tokens_saved += t.tokens_in + t.tokens_out;
        } else {
            entry.misses += 1;
        }
    }

    groups
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    /// Content address. Identical bodies produce identical addresses, so two
    /// groups that induce the same rule deduplicate without a comparison.
    pub du_uuid: Uuid,
    pub du_uuid_version: &'static str,
    pub signature: String,
    pub manifest: UreResource,
    /// Phrasings learned, as distinct from any alias the author wrote.
    pub learned_aliases: Vec<String>,
    pub confidence: f64,
    pub observations: usize,
    pub hits: usize,
    pub misses: usize,
    pub tokens_saved: u64,
    /// Always `candidate`. Promotion is a separate decision, and a function
    /// that can emit a champion has no gate at all.
    pub lifecycle: &'static str,
}

/// Induces a candidate actuator from one group.
///
/// Returns `None` when the group is not evidentiary, which is the expected
/// outcome for most groups and not an error.
pub fn induce(group: &Induction) -> Result<Candidate, String> {
    if !group.is_evidentiary() {
        return Err(format!(
            "group {} has {} distinct phrasing(s), {} failure ratio: not evidentiary",
            group.signature,
            group.observations(),
            group.failure_ratio()
        ));
    }

    // The action id is the primitive sequence. It is stable across phrasings
    // by construction, which is what makes two different sentences the same
    // rule.
    let aliases: Vec<String> = group.observations.iter().cloned().collect();

    let action = UreAction {
        id: group.signature.clone(),
        // The learned phrasings, plus the underscore and space forms of the
        // signature. The Bridge normalises underscores to spaces for the id but
        // not for aliases, which is divergence D4, so both forms are supplied
        // rather than relying on that asymmetry to resolve itself.
        aliases: Some({
            let mut a = aliases.clone();
            a.push(group.signature.clone());
            a.push(group.signature.replace('_', " "));
            a.sort();
            a.dedup();
            a
        }),
        params: serde_json::from_value(json!({})).unwrap_or_default(),
        target_state: "completed = true".to_string(),
        // Failures in this group are the evidence for a precondition. Recording
        // them is not the same as evaluating them, which is divergence D5.
        constraints: if group.misses > 0 {
            vec![format!("observed_failure_rate < {}", MAX_FAILURE_RATIO)]
        } else {
            vec![]
        },
    };

    let manifest = UreResource {
        ure_version: "1.0".to_string(),
        // Provisional: the content address is computed below and the manifest is
        // re-stamped, because a resource_id that is not its own address breaks
        // deduplication.
        resource_id: "pending".to_string(),
        category: "skill".to_string(),
        state_space: Default::default(),
        action_primitives: vec![action],
    };

    let value = serde_json::to_value(&manifest)
        .map_err(|e| format!("could not serialise manifest: {e}"))?;
    let du_uuid =
        DuUuid::generate(&value, None).map_err(|e| format!("could not derive DU-UUID: {e}"))?;

    let mut manifest = manifest;
    manifest.resource_id = du_uuid.to_string();

    Ok(Candidate {
        du_uuid,
        du_uuid_version: "4",
        signature: group.signature.clone(),
        manifest,
        learned_aliases: aliases,
        confidence: group.confidence(),
        observations: group.observations(),
        hits: group.hits,
        misses: group.misses,
        tokens_saved: group.tokens_saved,
        lifecycle: "candidate",
    })
}

/// Induces every evidentiary group, deduplicating by content address.
///
/// Two distinct phrasings of the same sequence produce one candidate, not two,
/// because the content address is a function of the body.
pub fn induce_all(traces: &[Trace]) -> Result<Vec<Candidate>, String> {
    let groups = group_traces(traces);
    let mut by_address: BTreeMap<Uuid, Candidate> = BTreeMap::new();
    for group in groups.values() {
        if let Ok(c) = induce(group) {
            by_address.insert(c.du_uuid, c);
        }
    }
    Ok(by_address.into_values().collect())
}

#[cfg(test)]
mod tests;
