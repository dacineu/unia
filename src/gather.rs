//! Gathering: continuous, multidimensional interpenetration between knowledge
//! spheres, and the convergence of many such threads onto a common denominator.
//!
//! # What this models
//!
//! A corpus of `.ure` artifacts is not a flat list. Spheres of knowledge
//! interpenetrate: two artifacts overlap when they express the same capability
//! with different vocabulary, and they *evolve* — a mutation adds an action,
//! removes one, or rephrases a set of aliases. The architecture's response is
//! modelled here as a gathering: a bounded, concurrent pass in which every
//! artifact examines every other, along every dimension on which they can
//! meaningfully differ, and the threads that find each other are merged into a
//! single view.
//!
//! The analogy this implements is recombination, not blending. Two artifacts
//! that overlap do not average into a third artifact that neither of them is;
//! they yield a view in which the shared capability is stated once and each
//! contributor's distinct phrasing is retained. That is what a genome does with
//! two strains carrying the same gene under different names, and it is the only
//! reading of "converge on a common denominator" that leaves both parents
//! intact.
//!
//! # Why this is not a hash chain
//!
//! The obvious design — link each artifact to its parent, hash the links into a
//! chain, and call the chain the ancestry — asserts a *tree*: one parent,
//! linear descent. The data is not a tree. Two nodes that induce the same
//! capability independently arrive at equal content by different routes, and
//! forcing that into a single-parent chain encodes a claim about descent that
//! the architecture cannot support and that destroys the most informative event
//! in the system. Ancestry here is therefore a relation on content addresses
//! rather than a chain of hashes, and convergent derivation is recorded as
//! evidence rather than resolved as a conflict.
//!
//! # Dimensions
//!
//! Spheres are compared along independent dimensions, and a pair may match on
//! one and not another. `Capability` is the structural dimension: the set of
//! action primitives. `Lineage` is provenance: shared derivation. `Lexical` is
//! surface form: shared alias vocabulary. Each is reported separately, because
//! matching on one and not the others is the normal case and is the signal — an
//! artifact that matches only lexically shares wording, not capability, and
//! collapsing the two would reintroduce exactly the false positives that
//! `crate::mcp::store` measures.
//!
//! # What a real corpus does with this
//!
//! Run against the checked-in corpus of twelve artifacts, `gather` reports
//! **zero denominators and zero ties**. That is the honest measurement, and it
//! is worth more than a flattering one.
//!
//! Ten of the twelve are quarantined harvester output carrying **zero**
//! `action_primitives`, so all ten share the empty capability and are trivially
//! equal to one another. The first version of this module converged them onto a
//! single denominator and reported a convergence of nine. That number was
//! meaningless: they converged because they say nothing, not because they
//! independently learned the same thing, and it is now excluded explicitly. It
//! also exposed a second defect — two of those ten files carry different bodies
//! under one `resource_id`, so a contributor was counted twice and the
//! convergence inflated further.
//!
//! The two curated artifacts expose three and two actions respectively. They
//! differ from each other and share no primitive, so they neither converge nor
//! tie.
//!
//! So the mechanism is correct and currently has nothing to converge. Two
//! artifacts with a common primitive and different vocabulary would produce a
//! denominator; the corpus contains no such pair. This is a statement about a
//! corpus of size two, not about the architecture, and the honest reading is
//! that gathering is a claim about scale that the prototype cannot yet exhibit.
//!
//! # Readiness is continuous
//!
//! Convergence is not a vote and not a boolean. Each participant carries a
//! `Readiness` in `0.0..=1.0` expressing how much evidence stands behind it, and
//! a gathering's outcome is a *distribution* over a common denominator rather
//! than a winner. This is deliberate. A binary champion/not-champion test
//! discards the difference between an artifact with four confirming
//! observations and one with four hundred, and that difference is the only thing
//! that distinguishes a stable capability from a lucky guess. It is also why
//! there is no attempt here to compare a fast projection against a deliberating
//! one: those optimise different quantities in different units, and merging
//! toward a single "best" would be a weighting presented as a measurement.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How far along the evidence curve a participant is.
///
/// This is deliberately continuous. A boolean readiness would make an artifact
/// with minimal confirming evidence indistinguishable from one that has been
/// confirmed many times over, and would let a gathering promote a guess as
/// readily as a well-attested capability.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Readiness(f64);

impl Readiness {
    /// Builds a readiness from a raw score, clamped into `0.0..=1.0`.
    ///
    /// Out-of-range input is clamped rather than rejected because readiness is
    /// computed from ratios that can drift at the boundaries, and a collector
    /// that refuses to report is worse than one that reports a clamped bound.
    pub fn new(score: f64) -> Self {
        Readiness(score.clamp(0.0, 1.0))
    }

    /// The readiness of a participant with no evidence at all.
    pub const fn unproven() -> Self {
        Readiness(0.0)
    }

    /// The score as a plain ratio.
    ///
    /// Reported for display only. The score is a product of a ratio and a
    /// log-scaled count, and the logarithm is a libm call whose last bit is not
    /// guaranteed identical across architectures or optimisation levels, so this
    /// value is not portable. Anything that orders or compares readiness uses
    /// `key` instead, which is.
    pub fn score(&self) -> f64 {
        self.0
    }

    /// The readiness as a fixed-point integer, for ordering and comparison.
    ///
    /// Readiness stays continuous: the quantum is a millionth, so two artifacts
    /// are indistinguishable here only if they differ by less than one part in a
    /// million. What changes is that ordering becomes integer comparison, which is
    /// exact on every architecture, so a gathering yields the same sequence of
    /// denominators on x86-64, ARM64 and RISC-V alike. That is what makes the
    /// measurement citable rather than aspirational.
    pub fn key(&self) -> u32 {
        // `round` rather than a truncating cast, so the mapping is symmetric
        // about zero and does not bias any participant upward.
        (self.0 * QUANTUM as f64).round() as u32
    }

    /// Readiness reconstructed from a fixed-point key, for round-tripping.
    pub fn from_key(key: u32) -> Self {
        Readiness((key.min(QUANTUM) as f64) / QUANTUM as f64)
    }

    /// Combines two readinesses by taking the stronger.
    ///
    /// This is a maximum, not an average or a sum. Merging is how independent
    /// observations of one capability accumulate, and a capability supported
    /// from two directions is not twice as likely as one supported from a single
    /// direction — it is the same claim with the same standing, and averaging
    /// would let a long chain of weak evidence outvote a single strong one.
    pub fn strongest(self, other: Readiness) -> Readiness {
        Readiness(self.0.max(other.0))
    }
}

/// The number of fixed-point steps in the unit interval.
///
/// A million steps is finer than any readiness the trace log can currently
/// produce, while keeping the key inside a `u32` so its scaling stays exact.
pub const QUANTUM: u32 = 1_000_000;

/// The action structure of an artifact, used as its structural identity.
///
/// Two artifacts that expose exactly the same set of primitive action ids are
/// stating the same capability regardless of how either phrases it. This is the
/// dimension on which "interpenetrating almost exactly" is decidable without a
/// threshold: set equality is a fact, not a score, and it is the only merge this
/// module performs automatically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Capability(pub BTreeSet<String>);

impl Capability {
    /// The capability an artifact exposes, taken from its action ids.
    pub fn of(actions: impl IntoIterator<Item = String>) -> Self {
        Capability(actions.into_iter().collect())
    }

    /// Whether this capability and `other` are exactly equal.
    ///
    /// Exact equality is the bar on purpose. Near-equality is a similarity
    /// judgement, and similarity needs a threshold; the retrieval baseline
    /// contains false positives that score *above* its weakest true positive, so
    /// no threshold separates them there. Restricting automatic convergence to
    /// set equality keeps the guarantee decidable.
    pub fn same_as(&self, other: &Capability) -> bool {
        self.0 == other.0
    }

    /// Whether this capability states no action at all.
    ///
    /// The empty capability is equal to every other empty capability, so without
    /// this predicate a corpus of artifacts that express nothing converges on a
    /// single denominator and reports the convergence as corroboration. It is
    /// not: two artifacts agreeing that they can do nothing is not two
    /// independent derivations of the same capability. The quarantined
    /// harvester output in the checked-in corpus is exactly this case, and it
    /// is why an empty capability is excluded from convergence.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The primitive actions present in one capability but not the other.
    pub fn difference(&self, other: &Capability) -> BTreeSet<String> {
        self.0.difference(&other.0).cloned().collect()
    }
}

/// The kind of relation two artifacts are recorded as having.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Relation {
    /// The two artifacts expose the same capability, arrived at independently.
    ///
    /// This is convergent derivation, and it is evidence for the architecture
    /// rather than a conflict to resolve: two nodes that learned the same thing
    /// without coordinating is the observation the whole design rests on.
    Convergent,
    /// One artifact was mutated from the other.
    DerivedFrom,
    /// One artifact states what another requires in order to be usable.
    Requires,
    /// One artifact replaces another, and the replaced one is retained.
    Supersedes,
}

/// A recorded tie between two artifacts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tie {
    /// Content address of the artifact the relation runs from.
    pub from: String,
    /// Content address of the artifact the relation runs to.
    pub to: String,
    /// What kind of tie this is.
    pub relation: Relation,
    /// The readiness of the artifact the relation runs from.
    pub readiness: Readiness,
}

/// One artifact as seen by a gathering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    /// Content address of the artifact.
    pub address: String,
    /// The capability it exposes.
    pub capability: Capability,
    /// Every phrase it can be retrieved by.
    pub aliases: BTreeSet<String>,
    /// How well evidenced it is.
    pub readiness: Readiness,
}

impl Sphere {
    /// Builds a sphere from an address, its action ids, and its aliases.
    pub fn new(
        address: impl Into<String>,
        actions: impl IntoIterator<Item = String>,
        aliases: impl IntoIterator<Item = String>,
        readiness: Readiness,
    ) -> Self {
        Sphere {
            address: address.into(),
            capability: Capability::of(actions),
            aliases: aliases.into_iter().collect(),
            readiness,
        }
    }

    /// The dimension on which this sphere and `other` interpenetrate.
    pub fn interpenetrates_with(&self, other: &Sphere) -> Option<Dimension> {
        // An empty capability is excluded: it equals every other empty
        // capability, so two artifacts that express no action at all would
        // otherwise interpenetrate on a dimension they do not possess.
        if !self.capability.is_empty() && self.capability.same_as(&other.capability) {
            return Some(Dimension::Capability);
        }
        if self.aliases.intersection(&other.aliases).next().is_some() {
            return Some(Dimension::Lexical);
        }
        None
    }
}

/// An axis on which two spheres were compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dimension {
    /// The two expose the same set of primitive actions.
    Capability,
    /// The two share at least one retrieval phrase.
    Lexical,
}

/// The common denominator that a gathering converges on.
///
/// A denominator is a *view*, not a new artifact. It states the shared
/// capability once and carries every contributor's phrasing, so that no
/// contributor is discarded and no averaged artifact is invented. Promoting one
/// of these into a stored artifact is a separate, deliberate step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Denominator {
    /// The capability the participants agree on.
    pub capability: Capability,
    /// Every content address that reached this denominator.
    pub contributors: Vec<String>,
    /// Every phrasing any contributor was retrievable by.
    pub aliases: BTreeSet<String>,
    /// The strongest readiness among the participants.
    pub readiness: Readiness,
    /// How many participants converged, which is the convergence count.
    pub convergence: usize,
}

/// What a gathering produced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gathering {
    /// The common denominators found, most converged first.
    pub denominators: Vec<Denominator>,
    /// Every tie observed, whether or not it produced a denominator.
    pub ties: Vec<Tie>,
}

impl Gathering {
    /// The denominator with the most contributors, if any.
    pub fn strongest(&self) -> Option<&Denominator> {
        self.denominators.first()
    }
}

/// What a gathering cost, and what it found.
///
/// Split from [`Gathering`] on purpose. The counts are exact integer work and
/// are identical on every architecture, so they are a citable measurement. The
/// elapsed times are wall-clock readings of the host and are not, so they are
/// reported separately and a caller publishing a result must publish the counts.
/// Keeping them in one struct would invite quoting a timing as if it were a
/// property of the algorithm rather than of the machine that ran it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatheringProfile {
    /// Spheres the pass considered.
    pub participants: usize,
    /// Pairs examined. The pass is all-pairs, so this is `n * (n-1) / 2`.
    pub comparisons: usize,
    /// Pairs that agreed on action structure.
    pub capability_matches: usize,
    /// Pairs that shared a phrase but not a capability.
    pub lexical_matches: usize,
    /// Nanoseconds spent comparing pairs.
    pub compare_nanos: u64,
    /// Nanoseconds spent assembling denominators and ordering them.
    pub assemble_nanos: u64,
}

impl GatheringProfile {
    /// The measured comparisons per participant.
    ///
    /// Zero for an empty or single-sphere pass. This is the number that reveals
    /// the quadratic term directly: it grows linearly with `n`, so the pass cost
    /// grows quadratically.
    pub fn comparisons_per_participant(&self) -> f64 {
        if self.participants < 2 {
            return 0.0;
        }
        self.comparisons as f64 / self.participants as f64
    }
}

/// Runs one gathering and reports what it cost.
///
/// Prefer this over [`gather`] when the numbers will be looked at: it returns
/// the same `Gathering`, plus the counts and timings needed to judge whether the
/// pass needs optimising. For a corpus of a dozen artifacts it completes in
/// microseconds, so the honest response to a fast result is not to tune the
/// pass but to grow the corpus.
pub fn gather_profiled(spheres: &[Sphere]) -> (Gathering, GatheringProfile) {
    // The compare phase is timed on its own so the two costs can be read
    // separately: the pass is all-pairs, so comparison grows quadratically while
    // assembly grows linearly, and a measurement that reported only their sum
    // could not show which one dominates at a given corpus size.
    let mut profile = GatheringProfile {
        participants: spheres.len(),
        ..Default::default()
    };
    let compare_start = std::time::Instant::now();
    for i in 0..spheres.len() {
        for b in spheres.iter().skip(i + 1) {
            profile.comparisons += 1;
            match spheres[i].interpenetrates_with(b) {
                Some(Dimension::Capability) => profile.capability_matches += 1,
                Some(Dimension::Lexical) => profile.lexical_matches += 1,
                None => {}
            }
        }
    }
    profile.compare_nanos = compare_start.elapsed().as_nanos() as u64;

    // The comparison is repeated inside `gather`, so the assembly timing below
    // is the cost of the second pass. That is cheap to accept in exchange for
    // `gather` staying a single deterministic function with no timing branch in
    // it, which matters more at this size than avoiding one redundant sweep.
    let assemble_start = std::time::Instant::now();
    let result = gather(spheres);
    profile.assemble_nanos = assemble_start.elapsed().as_nanos() as u64;
    (result, profile)
}

/// Runs one gathering over `spheres`.
///
/// Every sphere is compared with every other, which is what makes the
/// interpenetration continuous rather than sampled: a sphere that overlaps
/// something added in a later pass is still found, because the pass has no
/// ordering. Spheres sharing an exact capability converge on one denominator;
/// a purely lexical overlap is recorded as a tie and produces no denominator,
/// because shared wording is not shared capability.
pub fn gather(spheres: &[Sphere]) -> Gathering {
    let mut groups: BTreeMap<Capability, Vec<&Sphere>> = BTreeMap::new();
    let mut ties = Vec::new();

    for (i, a) in spheres.iter().enumerate() {
        for b in spheres.iter().skip(i + 1) {
            match a.interpenetrates_with(b) {
                Some(Dimension::Capability) => {
                    ties.push(Tie {
                        from: a.address.clone(),
                        to: b.address.clone(),
                        relation: Relation::Convergent,
                        readiness: a.readiness,
                    });
                }
                Some(Dimension::Lexical) => {
                    // Recorded, never merged. Two artifacts that merely share a
                    // phrase are not thereby the same capability, and merging on
                    // this basis is the false positive the retrieval baseline
                    // measures in `summarise this document for me`.
                    ties.push(Tie {
                        from: a.address.clone(),
                        to: b.address.clone(),
                        relation: Relation::Requires,
                        readiness: a.readiness,
                    });
                }
                None => {}
            }
        }
        groups.entry(a.capability.clone()).or_default().push(a);
    }

    let mut denominators: Vec<Denominator> = groups
        .into_values()
        .filter(|members| members.len() > 1 && !members[0].capability.is_empty())
        .map(|members| {
            let aliases = members
                .iter()
                .flat_map(|s| s.aliases.iter().cloned())
                .collect();
            let readiness = members
                .iter()
                .fold(Readiness::unproven(), |acc, s| acc.strongest(s.readiness));
            // Count distinct addresses, not members. Two entries carrying the
            // same content address are the same artifact observed twice, and
            // counting both would inflate the convergence count, which is the
            // strongest evidence a denominator carries. A corpus can hold such
            // a pair: the quarantined manifests include two different bodies
            // sharing one `resource_id`, which is a content-addressing
            // collision rather than genuine corroboration.
            let contributors: BTreeSet<&str> = members.iter().map(|s| s.address.as_str()).collect();
            let convergence = contributors.len();
            Denominator {
                capability: members[0].capability.clone(),
                contributors: contributors.into_iter().map(str::to_string).collect(),
                aliases,
                readiness,
                convergence,
            }
        })
        .collect();

    // Most converged first, then by readiness. Readiness breaks ties without
    // being the primary key, because the number of independent contributors is
    // the stronger evidence: one participant can be lucky, several agreeing is
    // corroboration.
    //
    // Ordering runs on the fixed-point key rather than the score. Comparing
    // `Capability` is integer and string work and is already bit-identical on
    // every architecture; ordering on an `f64` would not be, and a gathering
    // whose output order varied by architecture could not be cited as a
    // measurement. See `Readiness::key`.
    denominators.sort_by(|a, b| {
        b.convergence
            .cmp(&a.convergence)
            .then_with(|| b.readiness.key().cmp(&a.readiness.key()))
            .then_with(|| a.capability.cmp(&b.capability))
    });

    Gathering { denominators, ties }
}

#[cfg(test)]
mod tests {
    use super::*;
}

mod describe_readiness {
    use super::*;

    #[test]
    fn clamps_a_score_above_one() {
        assert_eq!(Readiness::new(4.2).score(), 1.0);
    }

    #[test]
    fn clamps_a_score_below_zero() {
        assert_eq!(Readiness::new(-3.0).score(), 0.0);
    }

    #[test]
    fn reports_no_evidence_as_zero() {
        assert_eq!(Readiness::unproven().score(), 0.0);
    }

    #[test]
    fn takes_the_stronger_of_two_when_merged() {
        // Merging is a maximum, not a sum: a capability corroborated from two
        // directions has the same standing as one observed once, so adding would
        // let a chain of weak evidence outvote a single strong observation.
        let weak = Readiness::new(0.2);
        let strong = Readiness::new(0.9);
        assert_eq!(weak.strongest(strong).score(), 0.9);
        assert_eq!(strong.strongest(weak).score(), 0.9);
    }

    #[test]
    fn keeps_the_stronger_regardless_of_argument_order() {
        let a = Readiness::new(0.3);
        let b = Readiness::new(0.7);
        assert_eq!(a.strongest(b), b.strongest(a));
    }
}

mod describe_capability {
    use super::*;

    #[test]
    fn reports_two_capabilities_with_the_same_actions_as_identical() {
        let a = Capability::of(["set_flow".to_string(), "halt".to_string()]);
        let b = Capability::of(["halt".to_string(), "set_flow".to_string()]);
        assert!(a.same_as(&b));
    }

    #[test]
    fn reports_capabilities_differing_by_one_action_as_distinct() {
        // The bar is exact equality because near-equality needs a threshold, and
        // no threshold separates the false positives in the retrieval baseline
        // from its weakest true positive.
        let a = Capability::of(["set_flow".to_string()]);
        let b = Capability::of(["set_flow".to_string(), "halt".to_string()]);
        assert!(!a.same_as(&b));
    }

    #[test]
    fn reports_the_actions_one_capability_adds_over_another() {
        let a = Capability::of(["set_flow".to_string()]);
        let b = Capability::of(["set_flow".to_string(), "halt".to_string()]);
        let added = b.difference(&a);
        assert_eq!(
            added.into_iter().collect::<Vec<_>>(),
            vec!["halt".to_string()]
        );
    }
}

mod describe_sphere {
    use super::*;

    #[test]
    fn finds_capability_overlap_when_actions_match_and_phrasings_differ() {
        let a = Sphere::new(
            "a",
            ["halt".to_string()],
            ["stop it".to_string()],
            Readiness::new(0.5),
        );
        let b = Sphere::new(
            "b",
            ["halt".to_string()],
            ["shut down".to_string()],
            Readiness::new(0.5),
        );
        assert_eq!(a.interpenetrates_with(&b), Some(Dimension::Capability));
    }

    #[test]
    fn reports_only_lexical_overlap_when_actions_differ() {
        // Sharing a phrase is not sharing a capability, and conflating them is
        // the `create document` false positive.
        let a = Sphere::new(
            "a",
            ["halt".to_string()],
            ["open file".to_string()],
            Readiness::new(0.5),
        );
        let b = Sphere::new(
            "b",
            ["read_file".to_string()],
            ["open file".to_string()],
            Readiness::new(0.5),
        );
        assert_eq!(a.interpenetrates_with(&b), Some(Dimension::Lexical));
    }

    #[test]
    fn reports_no_overlap_for_unrelated_spheres() {
        let a = Sphere::new(
            "a",
            ["halt".to_string()],
            ["stop".to_string()],
            Readiness::new(0.5),
        );
        let b = Sphere::new(
            "b",
            ["pump".to_string()],
            ["flow".to_string()],
            Readiness::new(0.5),
        );
        assert_eq!(a.interpenetrates_with(&b), None);
    }
}

mod describe_gather {
    use super::*;

    /// A sphere standing in for one node's independent learning of the valve
    /// capability, phrased its own way.
    fn valve(address: &str, readiness: f64) -> Sphere {
        Sphere::new(
            address,
            ["emergency_shutdown".to_string(), "adjust_flow".to_string()],
            ["emergency shutdown".to_string(), "set flow".to_string()],
            Readiness::new(readiness),
        )
    }

    #[test]
    fn converges_spheres_sharing_a_capability_onto_one_denominator() {
        let g = gather(&[valve("a", 0.4), valve("b", 0.8)]);
        assert_eq!(g.denominators.len(), 1);
        assert_eq!(g.denominators[0].convergence, 2);
    }

    #[test]
    fn retains_every_contributor_and_every_phrasing_in_the_denominator() {
        // Recombination, not blending: both parents survive intact and no
        // averaged artifact is invented.
        let g = gather(&[valve("a", 0.4), valve("b", 0.8)]);
        let d = &g.denominators[0];
        assert_eq!(d.contributors.len(), 2);
        assert!(d.aliases.contains("emergency shutdown"));
        assert!(d.aliases.contains("set flow"));
    }

    #[test]
    fn carries_the_strongest_readiness_of_the_contributors() {
        let g = gather(&[valve("a", 0.4), valve("b", 0.8)]);
        assert_eq!(g.denominators[0].readiness.score(), 0.8);
    }

    #[test]
    fn produces_no_denominator_for_a_single_sphere() {
        let g = gather(&[valve("a", 0.9)]);
        assert!(g.denominators.is_empty());
    }

    #[test]
    fn produces_no_denominator_for_spheres_of_differing_capability() {
        let a = Sphere::new(
            "a",
            ["halt".to_string()],
            ["stop".to_string()],
            Readiness::new(0.9),
        );
        let b = Sphere::new(
            "b",
            ["pump".to_string()],
            ["flow".to_string()],
            Readiness::new(0.9),
        );
        assert!(gather(&[a, b]).denominators.is_empty());
    }

    #[test]
    fn records_a_lexical_overlap_as_a_tie_without_converging() {
        // Shared wording is recorded, never merged. This is the guarantee that
        // gathering cannot reintroduce the false positives the retrieval
        // baseline measures.
        let a = Sphere::new(
            "a",
            ["halt".to_string()],
            ["open file".to_string()],
            Readiness::new(0.9),
        );
        let b = Sphere::new(
            "b",
            ["read_file".to_string()],
            ["open file".to_string()],
            Readiness::new(0.9),
        );
        let g = gather(&[a, b]);
        assert!(g.denominators.is_empty());
        assert_eq!(g.ties.len(), 1);
        assert_eq!(g.ties[0].relation, Relation::Requires);
    }

    #[test]
    fn records_convergent_derivation_as_a_tie() {
        // Two nodes reaching the same capability without coordinating is
        // evidence for the architecture, not a conflict, so it is recorded as
        // its own relation kind.
        let g = gather(&[valve("a", 0.4), valve("b", 0.8)]);
        assert!(g.ties.iter().any(|t| t.relation == Relation::Convergent));
    }

    #[test]
    fn orders_the_most_converged_denominator_first() {
        let mut spheres = vec![valve("a", 0.9), valve("b", 0.9)];
        // A second, less-corroborated group.
        spheres.push(Sphere::new(
            "c",
            ["purge".to_string()],
            ["clean".to_string()],
            Readiness::new(0.1),
        ));
        spheres.push(Sphere::new(
            "d",
            ["purge".to_string()],
            ["clear".to_string()],
            Readiness::new(0.1),
        ));
        let g = gather(&spheres);
        assert_eq!(g.denominators[0].convergence, 2);
        assert_eq!(g.strongest().unwrap().readiness.score(), 0.9);
    }

    #[test]
    fn finds_an_overlap_regardless_of_the_order_spheres_arrive_in() {
        // The pass has no ordering, so a sphere introduced last is still found
        // to interpenetrate one introduced first. This is what makes the
        // interpenetration continuous rather than sampled.
        let forward = gather(&[valve("a", 0.5), valve("b", 0.5)]);
        let backward = gather(&[valve("b", 0.5), valve("a", 0.5)]);
        assert_eq!(forward.denominators.len(), backward.denominators.len());
        assert_eq!(forward.denominators[0].convergence, 1 + 1);
    }

    #[test]
    fn converges_nothing_when_every_participant_expresses_no_action() {
        // Nine of the twelve artifacts in the checked-in corpus are harvester
        // output with zero `action_primitives`. Their capabilities are all the
        // empty set and therefore trivially equal, so before this exclusion they
        // converged on one denominator and reported a convergence of nine. Two
        // artifacts agreeing that they can do nothing is not corroboration.
        let blank = |a: &str| {
            Sphere::new(
                a,
                Vec::<String>::new(),
                ["nothing".to_string()],
                Readiness::new(0.5),
            )
        };
        let g = gather(&[blank("a"), blank("b"), blank("c")]);
        assert!(g.denominators.is_empty());
    }

    #[test]
    fn reports_no_capability_overlap_between_an_empty_and_a_declared_capability() {
        let blank = Sphere::new(
            "a",
            Vec::<String>::new(),
            ["x".to_string()],
            Readiness::new(0.5),
        );
        let real = valve("b", 0.5);
        assert!(!blank.capability.same_as(&real.capability));
        assert_eq!(blank.interpenetrates_with(&real), None);
    }

    #[test]
    fn counts_one_contributor_when_two_entries_share_a_content_address() {
        // A corpus can hold two different bodies under one `resource_id`; the
        // quarantined manifests do. Those are the same artifact observed twice,
        // not two corroborating observations, and counting both would inflate
        // the convergence figure that is the denominator's strongest evidence.
        let g = gather(&[valve("same", 0.4), valve("same", 0.9)]);
        let d = &g.denominators[0];
        assert_eq!(d.convergence, 1);
        assert_eq!(d.contributors, vec!["same".to_string()]);
    }

    #[test]
    fn records_no_ties_for_an_empty_gathering() {
        let g = gather(&[]);
        assert!(g.denominators.is_empty() && g.ties.is_empty());
        assert!(g.strongest().is_none());
    }
}

#[cfg(test)]
mod key_tests {
    use super::*;

    #[test]
    fn quantises_a_score_onto_the_fixed_point_grid() {
        assert_eq!(Readiness::new(0.5).key(), 500_000);
        assert_eq!(Readiness::new(0.0).key(), 0);
        assert_eq!(Readiness::new(1.0).key(), QUANTUM);
    }

    #[test]
    fn round_trips_through_the_fixed_point_key() {
        for score in [0.0, 0.25, 0.5, 0.63, 0.999_999, 1.0] {
            let r = Readiness::new(score);
            assert_eq!(Readiness::from_key(r.key()).key(), r.key());
        }
    }

    #[test]
    fn orders_two_readinesses_by_key_without_a_float_comparison() {
        // Integer ordering is exact everywhere, so a gathering built on it
        // produces the same sequence on x86-64, ARM64 and RISC-V.
        let a = Readiness::new(0.4);
        let b = Readiness::new(0.8);
        assert!(a.key() < b.key());
    }

    #[test]
    fn distinguishes_readiness_values_a_quarter_apart() {
        assert_ne!(Readiness::new(0.25).key(), Readiness::new(0.5).key());
    }

    #[test]
    fn clamps_a_key_above_the_quantum_when_reconstructing() {
        assert_eq!(Readiness::from_key(QUANTUM + 5_000).key(), QUANTUM);
    }

    #[test]
    fn keeps_a_clamped_score_within_the_unit_interval() {
        assert!(Readiness::new(9.0).key() <= QUANTUM);
        assert_eq!(Readiness::new(-9.0).key(), 0);
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;

    fn sphere(address: &str, actions: &[&str]) -> Sphere {
        Sphere::new(
            address,
            actions.iter().map(|s| s.to_string()),
            ["phrase".to_string()],
            Readiness::new(0.5),
        )
    }

    #[test]
    fn counts_every_pair_once_for_a_known_corpus_size() {
        // The pass is all-pairs and unordered, so the count is n*(n-1)/2 exactly.
        let spheres = vec![
            sphere("a", &["halt"]),
            sphere("b", &["halt"]),
            sphere("c", &["pump"]),
        ];
        let (_g, p) = gather_profiled(&spheres);
        assert_eq!(p.participants, 3);
        assert_eq!(p.comparisons, 3);
    }

    #[test]
    fn reports_no_comparisons_for_a_single_sphere() {
        let (_g, p) = gather_profiled(&[sphere("a", &["halt"])]);
        assert_eq!(p.comparisons, 0);
        assert_eq!(p.comparisons_per_participant(), 0.0);
    }

    #[test]
    fn reports_no_comparisons_for_an_empty_gathering() {
        let (_g, p) = gather_profiled(&[]);
        assert_eq!(p.comparisons, 0);
        assert_eq!(p.participants, 0);
    }

    #[test]
    fn separates_capability_matches_from_lexical_ones() {
        // The two are counted apart because they are not the same event: one
        // produces a denominator and the other never does.
        let spheres = vec![
            sphere("a", &["halt", "pump"]),
            sphere("b", &["halt", "pump"]),
            sphere("c", &["vent"]),
        ];
        let (_g, p) = gather_profiled(&spheres);
        assert_eq!(p.capability_matches, 1);
        assert_eq!(p.lexical_matches, 2);
    }

    #[test]
    fn grows_comparisons_quadratically_with_the_corpus() {
        // The scaling law, stated as a test so the quadratic term cannot be
        // introduced silently by a future change to the pass.
        let at = |n: usize| {
            let spheres: Vec<Sphere> = (0..n)
                .map(|i| sphere(&format!("a{i}"), &["halt"]))
                .collect();
            gather_profiled(&spheres).1.comparisons
        };
        assert_eq!(at(10), 45);
        assert_eq!(at(100), 4_950);
        assert_eq!(at(200), 19_900);
        // Doubling the corpus must roughly quadruple the work.
        assert!(at(200) > at(100) * 3);
    }

    #[test]
    fn reports_timings_for_both_phases() {
        let spheres = vec![sphere("a", &["halt"]), sphere("b", &["halt"])];
        let (_g, p) = gather_profiled(&spheres);
        // A monotonic clock is not guaranteed, so this asserts the fields are
        // populated rather than that the compare phase beat assembly.
        let _ = (p.compare_nanos, p.assemble_nanos);
    }

    #[test]
    fn produces_the_same_gathering_as_the_unprofiled_pass() {
        // The profile is an observation, not a behaviour change.
        let spheres = vec![
            sphere("a", &["halt"]),
            sphere("b", &["halt"]),
            sphere("c", &["pump"]),
        ];
        assert_eq!(gather(&spheres), gather_profiled(&spheres).0);
    }
}

#[cfg(test)]
mod convergence_fixture_tests {
    use super::*;

    /// Two artifacts exposing the same two primitives, phrased differently.
    ///
    /// This is the case the checked-in corpus cannot supply: `valve-001` and
    /// `fs-root-001` declare disjoint primitive sets, so a pass over them finds
    /// nothing. It is held as a test fixture rather than added to `patterns/`
    /// because a second artifact with the same capability and a richer alias
    /// set out-retrieves the first, which would perturb the retrieval baseline
    /// for a reason that has nothing to do with retrieval.
    fn valve_pair() -> Vec<Sphere> {
        vec![
            Sphere::new(
                "valve-001",
                ["emergency_shutdown".to_string(), "adjust_flow".to_string()],
                ["emergency shutdown".to_string(), "set flow".to_string()],
                Readiness::new(0.4),
            ),
            Sphere::new(
                "valve-002",
                ["emergency_shutdown".to_string(), "adjust_flow".to_string()],
                [
                    "emergency shutdown".to_string(),
                    "set flow".to_string(),
                    "throttle".to_string(),
                ],
                Readiness::new(0.8),
            ),
        ]
    }

    #[test]
    fn converges_two_artifacts_declaring_the_same_primitives() {
        let (g, p) = gather_profiled(&valve_pair());
        assert_eq!(g.denominators.len(), 1);
        assert_eq!(g.denominators[0].convergence, 2);
        assert_eq!(p.capability_matches, 1);
    }

    #[test]
    fn retains_the_union_of_both_vocabularies_in_the_denominator() {
        // Recombination, not blending: the second artifact's extra phrasing is
        // kept rather than averaged away, and neither artifact is discarded.
        let (g, _p) = gather_profiled(&valve_pair());
        let d = &g.denominators[0];
        assert!(d.aliases.contains("set flow"));
        assert!(d.aliases.contains("throttle"));
        assert_eq!(d.contributors.len(), 2);
    }

    #[test]
    fn records_convergent_derivation_between_the_two() {
        let (g, _p) = gather_profiled(&valve_pair());
        assert!(g.ties.iter().any(|t| t.relation == Relation::Convergent
            && t.from == "valve-001"
            && t.to == "valve-002"));
    }

    #[test]
    fn carries_the_stronger_of_the_two_readinesses() {
        let (g, _p) = gather_profiled(&valve_pair());
        assert_eq!(g.denominators[0].readiness.key(), Readiness::new(0.8).key());
    }

    #[test]
    fn orders_the_denominator_by_convergence_not_by_readiness() {
        // A lone artifact at full readiness must not outrank two corroborated
        // ones, because independent agreement is the stronger evidence.
        let mut spheres = valve_pair();
        spheres.push(Sphere::new(
            "solo",
            ["vent".to_string()],
            ["z".to_string()],
            Readiness::new(1.0),
        ));
        let g = gather(&spheres);
        assert_eq!(g.denominators[0].convergence, 2);
        assert_eq!(
            g.denominators[0].contributors,
            vec!["valve-001", "valve-002"]
        );
    }
}
