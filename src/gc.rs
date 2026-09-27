//! Degradation and self-collection, distributed across the actuator mesh.
//!
//! # Correction to the centralised design
//!
//! The first version of this module kept every artifact's usage in a central
//! table and ran a global `plan()` over it. That is a reporting architecture,
//! not an inference one, and it was wrong for this project: `ActuatorDriver`
//! exposes `execute()`, so an actuator is *itself* the observation point. It
//! knows it was called, when, and whether the call succeeded. Routing that
//! knowledge to a supervisor in order to obtain a decision is strictly more
//! work than deciding locally, and it introduces a single point of observation
//! loss — an actuator connected to a nucleus that has gone away is, to a
//! central collector, indistinguishable from one that is genuinely idle.
//!
//! So the unit of collection is the artifact. Each carries a `SelfReport`, runs
//! `decide()` on its own state, and proposes only its own transition. The
//! nucleus applies proposals. There is no global scan and no central usage
//! table, and `decide` needs nothing but the report and a policy.
//!
//! # The gap this exposes
//!
//! Moving the decision into the artifact makes a distinction unavoidable that
//! the centralised version hid: **an artifact can have no usage because it is
//! unwanted, or because it is unreachable.** Those demand opposite responses.
//!
//! The prototype already produces the second case. In the retrieval baseline,
//! `write output to results.txt` matched nothing, because the action id is
//! `write_file` and containment requires the literal `write file`. That
//! pattern is not idle. It is mis-indexed, and it will report zero usage
//! forever.
//!
//! A collector that cannot tell those apart will retire exactly the artifacts
//! that need repair, and will do so silently, because the symptom of a routing
//! bug and the symptom of an unwanted artifact are both "no hits". `Reachability`
//! below is the discriminator, and `Reachability::Unknown` is a deliberate
//! third state: an artifact that has not been matched *and* cannot be shown to
//! be reachable must not be retired on that basis alone.
//!
//! # What stays centralised, and why
//!
//! Nothing about *observation* is centralised. The evidence gate in
//! `crate::induce` remains the precondition for promotion, and it is enforced by
//! whichever node performs the promotion. That is a property of the
//! decomposition: a mesh in which any attached node can synthesise and promote
//! arbitrary executables is the ClawHavoc surface, and a content-addressed
//! corpus with a uniform evidence gate is the mitigation. Distribution of
//! observation does not require distribution of trust, and this is the line.

use crate::induce::MAX_FAILURE_RATIO;
use std::time::{SystemTime, UNIX_EPOCH};

/// Lifecycle values, mirrored from the DuckDB `lifecycle_state` enum.
///
/// Kept in sync by hand. A drift between this and
/// `database/duckdb/001_schema.sql` would silently retire artifacts the index
/// still serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lifecycle {
    /// Collected from a trace, not verified.
    Harvested,
    /// Transduced to a deterministic manifest, not verified.
    Crystallised,
    /// Verified, attached, not promoted.
    Candidate,
    /// Serves traffic.
    Champion,
    /// Failed verification. Never served, retained for audit.
    Quarantined,
    /// Detached from its nucleus. Not served, not promotable, still present.
    Deprecated,
}

impl Lifecycle {
    /// Whether a search may return an artifact in this state.
    ///
    /// Deprecated is excluded: a retirement that still answered queries would be
    /// a no-op, and `database/duckdb/003_retrieval.sql` filters
    /// `lifecycle NOT IN ('quarantined', 'deprecated')`. The two must agree.
    pub fn served(self) -> bool {
        matches!(self, Lifecycle::Candidate | Lifecycle::Champion)
    }

    /// Whether a pattern in this state may be promoted.
    pub fn promotable(self) -> bool {
        matches!(
            self,
            Lifecycle::Harvested | Lifecycle::Crystallised | Lifecycle::Candidate
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Lifecycle::Harvested => "harvested",
            Lifecycle::Crystallised => "crystallised",
            Lifecycle::Candidate => "candidate",
            Lifecycle::Champion => "champion",
            Lifecycle::Quarantined => "quarantined",
            Lifecycle::Deprecated => "deprecated",
        }
    }
}

/// Can this artifact still be reached by a query?
///
/// This is the discriminator between "unwanted" and "broken", and the reason
/// zero usage is not sufficient evidence of retirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reachability {
    /// Shown to be reachable: a query has matched it.
    Verified,
    /// Shown to be unreachable: it was queried and no alias or action id
    /// matched. A routing defect, not a demand defect.
    Unreachable,
    /// Never matched, and no test has established whether it can be.
    ///
    /// The default, and deliberately not `Unreachable`. Most artifacts in a
    /// young corpus are here, and retiring on absence of evidence would empty it.
    Unknown,
}

/// Everything an artifact knows about itself. This is the entire input to a
/// retirement decision; no other artifact's state is consulted.
#[derive(Debug, Clone, PartialEq)]
pub struct SelfReport {
    pub du_uuid: String,
    pub lifecycle: Lifecycle,
    /// Connected to a live nucleus.
    pub attached: bool,
    pub hits: u64,
    pub misses: u64,
    /// Distinct observed phrasings, which is the evidence breadth the induction
    /// gate counts.
    pub observations: usize,
    pub last_used: Option<u64>,
    pub reachability: Reachability,
}

impl Default for SelfReport {
    fn default() -> Self {
        Self {
            du_uuid: String::new(),
            lifecycle: Lifecycle::Harvested,
            attached: false,
            hits: 0,
            misses: 0,
            observations: 0,
            last_used: None,
            reachability: Reachability::Unknown,
        }
    }
}

impl SelfReport {
    pub fn new(du_uuid: impl Into<String>) -> Self {
        Self {
            du_uuid: du_uuid.into(),
            ..Default::default()
        }
    }

    /// Whether a search may return this artifact.
    pub fn served(&self) -> bool {
        self.lifecycle.served()
    }

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

    /// Records a call. This is the whole observation mechanism: a driver counts
    /// its own invocations, locally, and needs no coordinator to do it.
    pub fn record(&mut self, ok: bool, now: u64) {
        if ok {
            self.hits += 1;
        } else {
            self.misses += 1;
        }
        self.last_used = Some(now);
    }

    /// Marks the artifact as having been matched, which promotes reachability
    /// from Unknown to Verified.
    pub fn observe_match(&mut self) {
        self.reachability = Reachability::Verified;
    }
}

/// Why an artifact proposes its own retirement.
#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    /// Failing, or idle, while still connected.
    Unproductive { failure_ratio: f64, idle_days: u64 },
    /// The nucleus it was attached to is gone.
    Detached,
    /// A newer artifact supersedes it. Rare, since identity is content-derived.
    Superseded { by: String },
    /// Cannot be reached, so it is a routing defect rather than an unwanted
    /// capability. Retiring it would hide the defect, so it is flagged instead.
    Unreachable { note: &'static str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    pub du_uuid: String,
    pub from: Lifecycle,
    pub to: Lifecycle,
    pub reason: Reason,
    /// A defect to repair rather than a capability to retire. The caller may
    /// quarantine it, but must not delete it and must not treat it as evidence
    /// that the capability is unwanted.
    pub is_defect: bool,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub idle_days: u64,
    pub max_failure_ratio: f64,
    /// Never retire an artifact whose reachability has not been established.
    /// The default is the safe one, and changing it is how a collector starts
    /// eating mis-indexed patterns.
    pub retire_unknown: bool,
    /// A connected artifact is not retired on quality alone, because detaching
    /// something mid-traffic drops a live request.
    pub protect_attached: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            idle_days: 90,
            max_failure_ratio: MAX_FAILURE_RATIO,
            retire_unknown: false,
            protect_attached: true,
        }
    }
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// An artifact's proposal about itself. Pure; touches nothing but its own report.
///
/// Returns `None` when the artifact should keep its current lifecycle, which is
/// the common case and not a failure.
pub fn decide(report: &SelfReport, policy: &Policy, now: u64) -> Option<Proposal> {
    // Already out of service. Re-proposing would bury the history.
    if matches!(
        report.lifecycle,
        Lifecycle::Quarantined | Lifecycle::Deprecated
    ) {
        return None;
    }

    let propose = |to: Lifecycle, reason: Reason, is_defect: bool| {
        Some(Proposal {
            du_uuid: report.du_uuid.clone(),
            from: report.lifecycle,
            to,
            reason,
            is_defect,
        })
    };

    // A routing defect is reported, not retired. Retiring an unreachable
    // artifact destroys the evidence that it was unreachable, which is the only
    // way anyone finds out the routing is broken.
    if report.reachability == Reachability::Unreachable {
        return propose(
            Lifecycle::Quarantined,
            Reason::Unreachable {
                note: "matched by no query; repair the aliases or action id rather than \
                       treating this as unwanted",
            },
            true,
        );
    }

    // Unproven and unestablished. Absence of a match is not evidence of absence
    // of demand, and this is the state most artifacts in a young corpus are in.
    if report.reachability == Reachability::Unknown && !policy.retire_unknown && report.hits == 0 {
        return None;
    }

    let idle = report.idle_days(now);
    let failing = report.failure_ratio() > policy.max_failure_ratio;
    let stale = idle >= policy.idle_days;

    if report.attached {
        // Detachment is a separate question from productivity. An attached
        // artifact that is failing or idle is a candidate for retirement, but
        // only if protection is off, because the alternative is a live request
        // losing its capability.
        if policy.protect_attached {
            return None;
        }
    } else {
        // Not attached: the mesh has let it go, so record the detachment.
        return propose(Lifecycle::Deprecated, Reason::Detached, false);
    }

    if failing || stale {
        return propose(
            Lifecycle::Deprecated,
            Reason::Unproductive {
                failure_ratio: report.failure_ratio(),
                idle_days: if idle == u64::MAX { 0 } else { idle },
            },
            false,
        );
    }

    None
}

/// Reconciles proposals from several copies of the same artifact.
///
/// Copies can disagree, because each has only its own observation window. The
/// reconciliation rule is monotone toward the less destructive action: a
/// proposal to retire loses to a proposal to keep, always. This prevents a node
/// with a narrow window from retiring a capability that a node with a broader
/// window is actively using, and it is why disagreement cannot compound.
pub fn reconcile(proposals: &[Proposal], reports: &[SelfReport]) -> Vec<Proposal> {
    let by_uuid: std::collections::BTreeMap<&str, &SelfReport> =
        reports.iter().map(|r| (r.du_uuid.as_str(), r)).collect();
    let mut out: Vec<Proposal> = Vec::new();
    for p in proposals {
        // If any copy of this artifact has actually been matched, it is wanted,
        // and no retirement proposed by another copy may stand.
        let wanted_elsewhere = proposals
            .iter()
            .filter(|q| q.du_uuid == p.du_uuid)
            .any(|q| by_uuid.get(q.du_uuid.as_str()).is_some_and(|r| r.hits > 0));
        if wanted_elsewhere {
            continue;
        }
        if !out.iter().any(|q| q.du_uuid == p.du_uuid) {
            out.push(p.clone());
        }
    }
    out
}

/// Applies a proposal. Returns whether the lifecycle actually changed.
///
/// A proposal whose `from` no longer matches is skipped rather than forced:
/// overwriting a lifecycle that changed in between would clobber a promotion,
/// which is the one operation the collector must never perform.
pub fn apply(report: &mut SelfReport, p: &Proposal) -> bool {
    if report.lifecycle != p.from {
        return false;
    }
    report.lifecycle = p.to;
    // Connection follows the transition: anything leaving service is detached.
    report.attached = p.to.served();
    true
}

/// Recalls a retired artifact after it is matched again.
///
/// Cheap undo is the precondition for permitting a collector to exist at all.
pub fn recall(report: &mut SelfReport) -> bool {
    if report.lifecycle != Lifecycle::Deprecated {
        return false;
    }
    report.lifecycle = Lifecycle::Candidate;
    report.attached = true;
    report.reachability = Reachability::Verified;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_000_000_000;
    const DAY: u64 = 86_400;

    fn verified_used(id: &str) -> SelfReport {
        SelfReport {
            du_uuid: id.into(),
            lifecycle: Lifecycle::Champion,
            attached: true,
            hits: 40,
            misses: 1,
            observations: 9,
            last_used: Some(NOW),
            reachability: Reachability::Verified,
        }
    }

    #[test]
    fn record_is_the_whole_observation_mechanism() {
        // No coordinator is involved: a driver counts its own calls.
        let mut r = SelfReport::new("a");
        assert_eq!(r.total(), 0);
        r.record(true, NOW);
        r.record(false, NOW);
        assert_eq!(r.hits, 1);
        assert_eq!(r.misses, 1);
        assert_eq!(r.failure_ratio(), 0.5);
        assert_eq!(r.last_used, Some(NOW));
    }

    #[test]
    fn decide_needs_no_other_artifacts_state() {
        // The signature is the point: there is no corpus argument.
        let r = verified_used("a");
        let p = Policy::default();
        let _ = decide(&r, &p, NOW);
    }

    #[test]
    fn healthy_artifact_proposes_nothing() {
        assert!(decide(&verified_used("a"), &Policy::default(), NOW).is_none());
    }

    #[test]
    fn unknown_reachability_is_never_retired() {
        // The case that centralised collection hid: a young, unexercised corpus
        // is mostly Unknown, and retiring on it would empty the corpus.
        let r = SelfReport {
            lifecycle: Lifecycle::Candidate,
            attached: true,
            reachability: Reachability::Unknown,
            ..SelfReport::new("u")
        };
        assert!(
            decide(&r, &Policy::default(), NOW).is_none(),
            "absence of a match is not evidence of absence of demand"
        );
    }

    #[test]
    fn unreachable_is_a_defect_not_a_retirement() {
        let r = SelfReport {
            lifecycle: Lifecycle::Champion,
            attached: true,
            reachability: Reachability::Unreachable,
            observations: 0,
            last_used: None,
            ..SelfReport::new("b")
        };
        let p = decide(&r, &Policy::default(), NOW).expect("unreachable is reported");
        assert!(p.is_defect);
        assert_eq!(p.to, Lifecycle::Quarantined);
        match p.reason {
            Reason::Unreachable { .. } => {}
            other => panic!("expected Unreachable, got {other:?}"),
        }
    }

    #[test]
    fn attached_artifact_is_protected_from_retirement() {
        // Fails badly, used moments ago. Detaching it drops a live request.
        let r = SelfReport {
            hits: 20,
            misses: 18,
            last_used: Some(NOW),
            ..verified_used("c")
        };
        assert!(decide(&r, &Policy::default(), NOW).is_none());
    }

    #[test]
    fn protection_can_be_disabled_and_then_it_is_retired() {
        // 5 hits, 30 misses: a 0.857 failure ratio, unambiguously failing.
        let r = SelfReport {
            hits: 5,
            misses: 30,
            last_used: Some(NOW),
            ..verified_used("c")
        };
        let policy = Policy {
            protect_attached: false,
            ..Policy::default()
        };
        let p = decide(&r, &policy, NOW).expect("protection off, so it is retired");
        assert!(matches!(p.reason, Reason::Unproductive { .. }));
    }

    #[test]
    fn detached_artifact_records_the_detachment() {
        let mut r = verified_used("d");
        r.attached = false;
        r.lifecycle = Lifecycle::Candidate;
        let p = decide(&r, &Policy::default(), NOW).expect("detached artifacts retire");
        assert_eq!(p.to, Lifecycle::Deprecated);
        assert_eq!(p.reason, Reason::Detached);
    }

    #[test]
    fn retirement_is_reversible() {
        let mut r = SelfReport {
            attached: false,
            lifecycle: Lifecycle::Candidate,
            ..verified_used("d")
        };
        let p = decide(&r, &Policy::default(), NOW).unwrap();
        assert!(apply(&mut r, &p));
        assert_eq!(r.lifecycle, Lifecycle::Deprecated);
        assert!(!r.served(), "a retired artifact must not answer queries");
        assert!(recall(&mut r));
        assert_eq!(r.lifecycle, Lifecycle::Candidate);
        assert!(r.served());
    }

    #[test]
    fn a_stale_promotion_is_not_clobbered() {
        let mut r = verified_used("e");
        let stale = Proposal {
            du_uuid: "e".into(),
            from: Lifecycle::Candidate,
            to: Lifecycle::Deprecated,
            reason: Reason::Detached,
            is_defect: false,
        };
        assert!(
            !apply(&mut r, &stale),
            "proposal built from a different state must not force"
        );
        assert_eq!(r.lifecycle, Lifecycle::Champion);
    }

    #[test]
    fn reconciliation_prefers_keeping_when_copies_disagree() {
        // Node 1 saw a 30-day window with no hits and proposes retirement. Node 2
        // saw 4000 hits. The retirement must lose.
        let mut busy = verified_used("f");
        busy.hits = 4000;
        let quiet = SelfReport {
            hits: 0,
            last_used: Some(NOW - 200 * DAY),
            ..verified_used("f")
        };
        let p1 = Proposal {
            du_uuid: "f".into(),
            from: Lifecycle::Champion,
            to: Lifecycle::Deprecated,
            reason: Reason::Unproductive {
                failure_ratio: 0.0,
                idle_days: 200,
            },
            is_defect: false,
        };
        let out = reconcile(&[p1.clone()], &[quiet, busy]);
        assert!(
            out.is_empty(),
            "a node with hits vetoes a retirement from a quiet window"
        );
    }

    #[test]
    fn served_and_promotable_stay_disjoint() {
        assert!(!Lifecycle::Quarantined.served() && !Lifecycle::Quarantined.promotable());
        assert!(!Lifecycle::Deprecated.served() && !Lifecycle::Deprecated.promotable());
        assert!(Lifecycle::Candidate.served() && Lifecycle::Candidate.promotable());
        assert!(Lifecycle::Champion.served() && !Lifecycle::Champion.promotable());
    }

    #[test]
    fn connection_is_authoritative_and_not_derived_from_lifecycle() {
        // The two are independent facts. A candidate that lost its nucleus is
        // not attached; a harvested artifact that a node wired up is. Deriving
        // connection from lifecycle would make one of these unrepresentable,
        // and would give the module two sources of truth for liveness, which is
        // how the Rust predicate came to disagree with the SQL filter earlier.
        let mut r = SelfReport {
            lifecycle: Lifecycle::Candidate,
            attached: false,
            ..SelfReport::new("x")
        };
        assert!(!r.attached);
        assert!(r.lifecycle.served(), "lifecycle says served...");
        assert!(
            !r.attached,
            "...but the connection says detached, and connection wins"
        );

        let mut h = SelfReport {
            lifecycle: Lifecycle::Harvested,
            attached: true,
            ..SelfReport::new("y")
        };
        assert!(h.attached, "an unverified artifact can still be connected");
        // Harvested is promotable: an artifact collected from a trace should be
        // promotable once it gathers evidence. The connection says nothing about
        // whether the evidence is sufficient.
        assert!(h.lifecycle.promotable());
        h.attached = false;
    }
}
