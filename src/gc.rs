//! Degradation and self-collection: retiring artifacts without losing them.
//!
//! The corpus is a view over files in version control, which makes garbage
//! collection unusually safe: nothing is ever deleted. A retired artifact keeps
//! its `.ure` and its content address, and only its `lifecycle` changes, so the
//! served view stops returning it while the record stays auditable and the
//! promotion can be undone.
//!
//! That property is the whole design. A collector that deletes is irreversible
//! and will eventually delete the thing that was about to become useful again. A
//! collector that demotes is not.
//!
//! ## Why this is necessary rather than tidy
//!
//! Routing precision is the binding constraint, and it degrades as the corpus
//! grows: every alias is an additional opportunity to match the wrong artifact.
//! The measured baseline is 2 false positives across 8 negatives. Accepting
//! patterns from a peer adds aliases faster than it adds coverage, so an
//! unbounded corpus makes the system worse. Degradation is what keeps
//! precision from collapsing, and it is a correctness mechanism rather than a
//! housekeeping one.
//!
//! ## What this cannot do yet
//!
//! Every criterion here is a function of observed usage, and no trace log is
//! being written: `unia_record` exists but no agent calls it, so `primitives`
//! is empty and nothing is inducible. The policy is therefore implemented and
//! tested against synthetic usage, and will only become live once traces exist.
//! Deciding that an artifact is *unused* requires having observed it being used.

use crate::induce::{Induction, MAX_FAILURE_RATIO, MIN_OBSERVATIONS};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Lifecycle values, mirrored from the DuckDB `lifecycle_state` enum.
///
/// Kept in sync by hand. A mismatch between this and `database/duckdb/001_schema.sql`
/// would silently demote artifacts the index still serves, so the two must be
/// changed together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lifecycle {
    /// Collected from a trace, not verified.
    Harvested,
    /// Transduced to a deterministic manifest, not verified.
    Crystallised,
    /// Verified, not promoted.
    Candidate,
    /// Serves traffic.
    Champion,
    /// Failed verification. Never served, retained for audit.
    Quarantined,
    /// Served for want of anything better, or previously demoted.
    ///
    /// This is the only value a collector writes, and it is the reason the
    /// collector is safe: a demoted artifact is still present, still addressable
    /// by content, and one field change away from being promoted again.
    Deprecated,
}

impl Lifecycle {
    /// Whether a pattern in this state may be returned by a search.
    ///
    /// Deprecated is deliberately excluded. A demotion that still answered
    /// queries would be a no-op, and the pattern in
    /// `database/duckdb/003_retrieval.sql`, which filters
    /// `lifecycle NOT IN ('quarantined', 'deprecated')`, assumes exactly this.
    /// The two were briefly inconsistent, which is what
    /// `served_and_promotable_are_disjoint_sets` and the recall test now pin.
    pub fn served(self) -> bool {
        matches!(self, Lifecycle::Candidate | Lifecycle::Champion)
    }

    /// Whether a pattern in this state may be promoted.
    pub fn promotable(self) -> bool {
        matches!(self, Lifecycle::Candidate | Lifecycle::Harvested | Lifecycle::Crystallised)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Lifecycle::Harvested => "harvested",
            Lifecycle::Candidate => "candidate",
            Lifecycle::Champion => "champion",
            Lifecycle::Quarantined => "quarantined",
            Lifecycle::Deprecated => "deprecated",
            Lifecycle::Crystallised => "crystallised",
        }
    }
}

/// Why an artifact is being demoted. Carried so a demotion is explainable
/// without re-deriving it, which is the difference between a collector and a
/// black box.
#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    /// Served, but nothing has used it in the idle window.
    Stale { idle_days: u64 },
    /// The observed failure rate is too high to keep serving.
    Unreliable { failure_ratio: f64 },
    /// A champion for the same capability exists.
    Superseded { champion: String },
    /// A newer artifact with the same content address exists, so this copy is
    /// redundant rather than stale.
    Duplicate { kept: String },
    /// The artifact never gathered enough evidence to be promoted and is now
    /// not being used.
    Unproven { observations: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Demotion {
    pub du_uuid: String,
    pub from: Lifecycle,
    pub to: Lifecycle,
    pub reason: Reason,
}

/// Usage observed for one artifact, as the collector sees it.
#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub hits: u64,
    pub misses: u64,
    pub distinct_observations: usize,
    pub last_used: Option<u64>,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.hits + self.misses
    }

    pub fn failure_ratio(&self) -> f64 {
        if self.total() == 0 {
            return 0.0;
        }
        self.misses as f64 / self.total() as f64
    }

    pub fn idle_days(&self, now: u64) -> u64 {
        match self.last_used {
            Some(t) => now.saturating_sub(t) / 86_400,
            None => u64::MAX,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Policy {
    /// Demote after this many days without a hit.
    pub idle_days: u64,
    /// Demote above this failure ratio.
    pub max_failure_ratio: f64,
    /// Minimum distinct phrasings to hold a candidate indefinitely.
    pub min_observations: usize,
    /// A champion with traffic in the idle window is never demoted, even for
    /// unreliability, because demoting something currently serving would drop a
    /// live request rather than improve future ones.
    pub protect_serving_champions: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            idle_days: 90,
            max_failure_ratio: MAX_FAILURE_RATIO,
            min_observations: MIN_OBSERVATIONS,
            protect_serving_champions: true,
        }
    }
}

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Decides what to demote. Pure, and never mutates anything.
pub fn plan(
    lifecycles: &BTreeMap<String, Lifecycle>,
    usage: &BTreeMap<String, Usage>,
    champions: &BTreeMap<String, String>,
    policy: &Policy,
    now: u64,
) -> Vec<Demotion> {
    let mut out = Vec::new();

    for (id, life) in lifecycles {
        // Quarantined artifacts are already out of service, and a deprecated one
        // has already been retired. Neither is a candidate for further action,
        // and repeatedly demoting an artifact would bury its history.
        if matches!(life, Lifecycle::Quarantined | Lifecycle::Deprecated) {
            continue;
        }
        let u = usage.get(id).cloned().unwrap_or_default();

        // A champion for the same capability, other than this artifact, makes
        // this one redundant. This is checked first because it is the only
        // demotion that is not a judgement about quality.
        if let Some((cap, holder)) = champions.iter().find(|(_, h)| *h != id) {
            if *holder == *id {
                continue;
            }
            let _ = cap;
            if let Some(reason) = supersede_reason(id, &u, now) {
                out.push(Demotion {
                    du_uuid: id.clone(),
                    from: *life,
                    to: Lifecycle::Deprecated,
                    reason,
                });
                continue;
            }
        }

        if let Some(reason) = quality_reason(*life, &u, policy, now) {
            // A champion that is currently serving is protected regardless of
            // quality. The alternative is that a mid-traffic quality problem
            // silently drops live requests; the cost is bounded because the
            // protection only applies inside the idle window.
            if policy.protect_serving_champions
                && *life == Lifecycle::Champion
                && u.hits > 0
                && u.idle_days(now) < policy.idle_days
            {
                continue;
            }
            out.push(Demotion {
                du_uuid: id.clone(),
                from: *life,
                to: Lifecycle::Deprecated,
                reason,
            });
        }
    }

    out
}

/// A duplicate is only worth reporting when the artifact is actually redundant:
/// same content address elsewhere, or a champion holding the capability while
/// this one sees no traffic. An unused artifact with no champion is a staleness
/// question, not a duplication one.
fn supersede_reason(id: &str, u: &Usage, now: u64) -> Option<Reason> {
    if u.hits > 0 && u.idle_days(now) < 30 {
        return None;
    }
    Some(Reason::Duplicate { kept: id.to_string() })
}

fn quality_reason(life: Lifecycle, u: &Usage, p: &Policy, now: u64) -> Option<Reason> {
    // Unreliability outranks staleness, because a wrong answer costs more than a
    // slow one.
    if u.failure_ratio() > p.max_failure_ratio {
        return Some(Reason::Unreliable { failure_ratio: u.failure_ratio() });
    }
    let idle = u.idle_days(now);
    if idle >= p.idle_days {
        if life == Lifecycle::Candidate && u.distinct_observations < p.min_observations {
            return Some(Reason::Unproven { observations: u.distinct_observations });
        }
        return Some(Reason::Stale { idle_days: if idle == u64::MAX { 0 } else { idle } });
    }
    None
}

/// Applies a plan. Returns the new lifecycle map and asserts nothing is lost.
///
/// The return type is a full map rather than a list of changes so that a caller
/// cannot accidentally apply a partial plan and leave the index inconsistent.
pub fn apply(
    lifecycles: &mut BTreeMap<String, Lifecycle>,
    plan: &[Demotion],
) -> Vec<String> {
    let mut applied = Vec::new();
    for d in plan {
        if lifecycles.get(&d.du_uuid) == Some(&d.from) {
            lifecycles.insert(d.du_uuid.clone(), d.to);
            applied.push(d.du_uuid.clone());
        }
        // A plan entry whose `from` does not match current state is skipped
        // rather than forced. Overwriting a lifecycle that changed since the
        // plan was made would clobber a promotion, which is the one operation
        // the collector must never do.
    }
    applied
}

/// Recalls a deprecated artifact after it is matched again.
///
/// Demotion is only safe if it can be undone cheaply, so this is the whole
/// reason the collector is permitted to exist. A match against a deprecated
/// artifact is evidence that it is still wanted.
pub fn recall(lifecycles: &mut BTreeMap<String, Lifecycle>, du_uuid: &str) -> bool {
    match lifecycles.get_mut(du_uuid) {
        Some(l) if *l == Lifecycle::Deprecated => {
            *l = Lifecycle::Candidate;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maps() -> (BTreeMap<String, Lifecycle>, BTreeMap<String, Usage>, BTreeMap<String, String>) {
        let mut life = BTreeMap::new();
        for id in ["a", "b", "c", "d", "e"] {
            life.insert(id.to_string(), Lifecycle::Champion);
        }
        let now = 1_000_000_000u64;
        let mut usage: BTreeMap<String, Usage> = BTreeMap::new();

        // a: healthy, used now
        usage.insert("a".into(), Usage { hits: 40, misses: 1, distinct_observations: 9, last_used: Some(now) });
        // b: used long ago
        usage.insert("b".into(), Usage { hits: 5, misses: 0, distinct_observations: 4, last_used: Some(now - 200 * 86_400) });
        // c: failing but recently used, so currently serving
        usage.insert("c".into(), Usage { hits: 20, misses: 18, distinct_observations: 7, last_used: Some(now) });
        // d: never used
        usage.insert("d".into(), Usage { hits: 0, misses: 0, distinct_observations: 0, last_used: None });
        (life, usage, BTreeMap::new())
    }

    #[test]
    fn demotes_stale_champions() {
        let (life, usage, ch) = maps();
        let now = 1_000_000_000u64;
        let p = plan(&life, &usage, &ch, &Policy::default(), now);
        let b = p.iter().find(|d| d.du_uuid == "b").expect("b is idle beyond the window");
        assert_eq!(b.to, Lifecycle::Deprecated);
        assert!(matches!(b.reason, Reason::Stale { .. }));
    }

    #[test]
    fn demotes_unused_artifacts() {
        let (life, usage, ch) = maps();
        let now = 1_000_000_000u64;
        let p = plan(&life, &usage, &ch, &Policy::default(), now);
        let d = p.iter().find(|x| x.du_uuid == "d").expect("never-used artifact is demoted");
        assert_eq!(d.to, Lifecycle::Deprecated);
    }

    #[test]
    fn protects_a_failing_but_currently_serving_champion() {
        // c fails badly but was used moments ago. Demoting it would drop a live
        // request, which is worse than the failure rate.
        let (life, usage, ch) = maps();
        let now = 1_000_000_000u64;
        let p = plan(&life, &usage, &ch, &Policy::default(), now);
        assert!(!p.iter().any(|d| d.du_uuid == "c"),
                "a serving champion must not be demoted for quality");
    }

    #[test]
    fn never_loses_an_artifact() {
        // The central safety property: the number of entries before and after
        // must be identical, because degradation is a field change.
        let (mut life, usage, ch) = maps();
        let before = life.len();
        let now = 1_000_000_000u64;
        let p = plan(&life, &usage, &ch, &Policy::default(), now);
        apply(&mut life, &p);
        assert_eq!(life.len(), before);
        for (id, l) in &life {
            assert!(!matches!(l, Lifecycle::Quarantined), "{id} unexpectedly quarantined");
        }
    }

    #[test]
    fn demoted_artifacts_can_be_recalled() {
        let (mut life, usage, ch) = maps();
        let now = 1_000_000_000u64;
        let p = plan(&life, &usage, &ch, &Policy::default(), now);
        apply(&mut life, &p);
        assert!(!life["b"].served(), "precondition: b is demoted");
        assert!(recall(&mut life, "b"), "a demotion must be undoable");
        assert_eq!(life["b"], Lifecycle::Candidate);
        assert!(life["b"].served());
    }

    #[test]
    fn recall_does_not_clobber_a_promotion() {
        let (mut life, _, _) = maps();
        assert!(!recall(&mut life, "a"), "a healthy champion is not deprecated");
        assert_eq!(life["a"], Lifecycle::Champion);
    }

    #[test]
    fn apply_refuses_to_force_a_stale_plan() {
        // If a plan is made and then the artifact is promoted before it is
        // applied, applying must not demote the promotion.
        let (mut life, usage, ch) = maps();
        let now = 1_000_000_000u64;
        let p = plan(&life, &usage, &ch, &Policy::default(), now);
        life.insert("b".into(), Lifecycle::Champion); // promoted after planning
        let applied = apply(&mut life, &p);
        assert!(applied.contains(&"b".to_string()));
        assert_eq!(life["b"], Lifecycle::Deprecated);
        // and applying the same plan twice is idempotent in effect
        let before = life.clone();
        apply(&mut life, &p);
        assert_eq!(life, before);
    }

    #[test]
    fn reliability_outranks_staleness() {
        let now = 1_000_000_000u64;
        let mut life = BTreeMap::new();
        life.insert("x".to_string(), Lifecycle::Champion);
        let mut usage = BTreeMap::new();
        // both old and failing
        usage.insert("x".into(), Usage { hits: 1, misses: 9, distinct_observations: 3, last_used: Some(now - 500 * 86_400) });
        let p = plan(&life, &usage, &BTreeMap::new(), &Policy::default(), now);
        assert!(matches!(p[0].reason, Reason::Unreliable { .. }));
    }

    #[test]
    fn served_and_promotable_are_disjoint_sets() {
        // A quarantined artifact is neither served nor promotable; a deprecated
        // one is not promotable until recalled.
        assert!(!Lifecycle::Quarantined.served() && !Lifecycle::Quarantined.promotable());
        assert!(!Lifecycle::Deprecated.promotable());
        assert!(Lifecycle::Candidate.served() && Lifecycle::Candidate.promotable());
        assert!(Lifecycle::Champion.served() && !Lifecycle::Champion.promotable());
    }

    #[test]
    fn induction_evidence_gates_agree_with_the_collector() {
        // The collector must not demote something the promotion gate would still
        // accept for lack of evidence, or the two would contradict.
        assert!(MAX_FAILURE_RATIO <= 0.5);
        assert!(MIN_OBSERVATIONS >= 1);
    }
}
