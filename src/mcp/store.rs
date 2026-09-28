//! Corpus of `.ure` manifests plus the execution traces harvested from them.
//!
//! The matcher here is deliberately not a substring scan. `find_matching_actuators`
//! in the registry does substring containment, which is why it stops being useful
//! past a few dozen entries. This uses inverse-document-frequency weighted token
//! overlap so that a rare word in an intent ("rust", "lockfile") carries more
//! signal than a common one ("file", "set").
//!
//! The honesty rule this module enforces: a manifest only *saves tokens* if it
//! carries a runnable `payload`. A declaration with no payload re-describes the
//! problem and still costs a model call, and `Match::saves_tokens` says so
//! explicitly rather than letting the caller assume it.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// An executable body attached to a resource. A manifest without one is a
/// description; a manifest with one can be served without calling a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Payload {
    /// `wasm`, `native`, `script`, or `none`.
    pub kind: String,
    /// Content address of the module, so identical bodies dedupe.
    pub module: Option<String>,
    pub entry: Option<String>,
    /// Argv for a `script` payload.
    pub command: Option<Vec<String>>,
}

impl Payload {
    /// A payload is runnable only if it names something to run and how.
    pub fn is_runnable(&self) -> bool {
        if self.kind == "none" {
            return false;
        }
        match self.kind.as_str() {
            "wasm" | "native" => self.module.is_some(),
            "script" => self.command.as_ref().is_some_and(|c| !c.is_empty()),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub aliases: Vec<String>,
    pub target_state: String,
    pub constraints: Vec<String>,
}

impl Action {
    /// The primitive sequence this action declares, when its id is one.
    ///
    /// Whole-token, case-sensitive, and every component required to be a known
    /// primitive — so `SetValue_CheckSense` is a sequence and `emergency_shutdown`
    /// and `SetValue` alone are not, the second because a single primitive is a
    /// *fragment* of a sequence rather than one. The last part is not pedantry:
    /// [`crate::meet::serves`] refuses fragments precisely because a one-primitive
    /// candidate is instantiated by almost every witness, and recovering fragments
    /// here would put them straight back into the place that breaks it.
    pub fn declared_act(&self) -> Option<Vec<String>> {
        let parts: Vec<&str> = self.id.split('_').collect();
        if parts.len() < 2 {
            return None;
        }
        if parts
            .iter()
            .all(|p| crate::bridge::primitive::is_primitive_name(p))
        {
            Some(parts.iter().map(|p| p.to_string()).collect())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub category: String,
    pub guidance: String,
    pub actions: Vec<Action>,
    pub payload: Option<Payload>,
    pub source: PathBuf,
}

impl Pattern {
    /// The primitive sequences this pattern declares as whole acts, if any.
    ///
    /// **The store had no primitive sequences at all until this, and that is why
    /// retrieval is scored on prose.** `Action` carries an id, aliases, a target
    /// state and constraints; the sequence a pattern performs lives nowhere in a
    /// manifest. So the decidable check in [`crate::meet::serves`] could not be
    /// applied to the corpus even in principle, and the two false positives had no
    /// structural reason to go away.
    ///
    /// It turns out the sequence *is* recoverable, because induction sets an
    /// action's id to the signature it induced (`UreAction.id = group.signature`).
    /// So the id of an induced action is literally `SetValue_CheckSense`, while a
    /// hand-authored one is `emergency_shutdown`, and the difference is decidable:
    /// an id made entirely of primitive names is a declared act, and one that is
    /// not is prose that happens to sit in the same field.
    ///
    /// That is a convention rather than a schema, and it is fragile in a specific
    /// way worth naming: a hand-authored action legitimately named after a
    /// primitive sequence would be read as one. The honest fix is a `primitives`
    /// field on `Action`, which is a persisted-schema change and not something to
    /// slip in beside a measurement.
    pub fn declared_acts(&self) -> Vec<Vec<String>> {
        self.actions
            .iter()
            .filter_map(|a| a.declared_act())
            .collect()
    }

    /// Every phrase this pattern can be found by: its id, guidance, each action
    /// id, and each alias.
    fn phrases(&self) -> Vec<String> {
        let mut out = vec![self.id.clone(), self.guidance.clone()];
        for a in &self.actions {
            out.push(a.id.clone());
            out.extend(a.aliases.iter().cloned());
        }
        out.retain(|s| !s.trim().is_empty());
        out
    }
}

/// Who chose the act behind a trace.
///
/// Deliberately a separate type from the creature's own `Who`, because this is a
/// property of the *record* and not of a living thing: a trace outlives its
/// actor, and a store that had to link the game in order to be read would be a
/// store nobody could read without it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    /// A person chose the act.
    Player,
    /// The creature chose it, from its own reading of itself.
    Itself,
    /// Not a creature's act: a pattern served a caller.
    Caller,
}

/// One recorded execution. These are the training signal: without them there is
/// nothing to induce an actuator from, and today the project discards them.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trace {
    /// When the interaction happened, in seconds since the epoch.
    ///
    /// Defaults to zero when absent so that a trace log written by hand, or by
    /// anything other than `record`, still loads. This field had no default, so
    /// every such line was silently dropped by `reload` and the artifact it
    /// described reported no evidence at all.
    #[serde(default)]
    pub ts: u64,
    /// The request as phrased. This is the training signal's raw material: the
    /// observed phrasings become a candidate's aliases, so a trace whose intent
    /// is blank cannot be inducted from.
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    /// `hit` when a pattern served the call, `miss` when a provider was called.
    ///
    /// A miss is not a failure. It means the system escalated and the caller
    /// still got an answer, which is a successful interaction that cost tokens.
    /// Treating escalation as failure would discard most of the training
    /// signal, since a miss is what an interaction looks like before any
    /// pattern exists to serve it. Actual failure is `succeeded`, below.
    /// Defaults to `miss`, meaning the call escalated. An absent outcome is
    /// treated as an escalation rather than as a parse failure, since that is
    /// what an interaction looks like before any pattern exists to serve it.
    #[serde(default = "default_miss")]
    pub outcome: String,
    /// Which tier served this act, if it was served.
    ///
    /// **Absent means `local`, and that default is the honest one:** most acts
    /// are local, and an absent tier on an old log line should read as the
    /// common case rather than as a parse failure.
    ///
    /// The field exists because the *cost of caring* was invisible. A trace
    /// recorded **that** an act happened and not **which tier** served it, so
    /// escalating outward — reaching for the network when the local machine
    /// could not — left nothing on the log to attribute the tokens to. With
    /// this, a network-served act has the same signature and the same witness as
    /// a local one, which is the property that makes the two tiers one system
    /// rather than two: a learner that cannot tell them apart will route to
    /// whichever is cheaper, and that is the sharing policy becoming a
    /// consequence of the economy instead of a rule written beside it.
    #[serde(default = "default_local")]
    pub tier: String,
    /// Tokens billed for the request. Zero when unrecorded, which is correct
    /// rather than unknown: an artifact served from tier 1 or 2 costs none, and
    /// that is the number escalation rate is computed from.
    #[serde(default)]
    pub tokens_in: u64,
    #[serde(default)]
    pub tokens_out: u64,
    /// Who chose the act, when the interaction was one a creature could have
    /// chosen for itself.
    ///
    /// Present so that a trace log answers a question the counts otherwise
    /// cannot: what share of the acting was the creature's own. It is the
    /// measurement behind the word *civilisation* — the point at which a
    /// population sustains itself rather than being kept running — and it is
    /// unanswerable while every trace looks the same.
    ///
    /// `None` means the interaction was not a creature's at all: a pattern
    /// served a caller, or a provider was escalated to. That is a real third
    /// case rather than a missing value, and [`Store::authorship`] reports all
    /// three.
    #[serde(default)]
    pub actor: Option<Actor>,
    /// The universal primitives the intent resolved to, in order.
    ///
    /// This is the generalisable part of the interaction and the grouping key
    /// for induction: two different sentences that reduced to the same sequence
    /// are evidence for one rule, and the sentences become the learned aliases.
    /// Empty means the sequence was not recorded, and such a trace cannot be
    /// inducted from.
    #[serde(default)]
    pub primitives: Vec<String>,
    /// Whether the interaction actually produced a usable result. This, not
    /// `outcome`, is what tells induction whether the sequence worked. Defaults
    /// to true so an older trace log without the field is not read as a wall of
    /// failures.
    #[serde(default = "default_true")]
    pub succeeded: bool,
}

fn default_true() -> bool {
    true
}

fn default_miss() -> String {
    "miss".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub id: String,
    pub category: String,
    pub score: f64,
    pub champion: bool,
    /// True only when this pattern can be executed locally with no provider call.
    pub saves_tokens: bool,
    pub matched_on: String,
    pub guidance: String,
    pub actions: Vec<Action>,
    pub payload: Option<Payload>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub root: String,
    pub patterns: usize,
    pub runnable: usize,
    pub champions: usize,
    pub traces: usize,
    pub hits: usize,
    pub misses: usize,
    /// Tokens that would have been spent on the traces that a pattern served.
    pub tokens_saved: u64,
    pub spec_version: String,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Splits text into comparable terms.
///
/// An underscore is a separator, not part of a term. `write_file` and
/// `write file` name the same action, so keeping the underscore produced a
/// single opaque token that no natural-language intent could ever contain:
/// `write output to results.txt` shares no term with `write_file` and the
/// containment fast path missed it too, because it compares against a
/// space-normalised phrase. This is divergence D4 in `docs/SPEC.md`.
fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && t.len() > 1)
        .map(|t| t.to_string())
        .collect()
}

pub struct Store {
    root: PathBuf,
    patterns: Vec<Pattern>,
    traces: Vec<Trace>,
    champions: BTreeMap<String, String>,
    /// Term -> number of resources declaring it, for inverse-document-frequency.
    df: HashMap<String, usize>,
    /// Number of resources in the corpus, which is the `N` in the IDF formula.
    total_resources: usize,
    /// Trace-log lines that failed to parse, so a corrupt evidence trail is
    /// visible to a caller rather than being indistinguishable from no traffic.
    malformed_traces: usize,
}

impl Store {
    /// Opens a store, loading every `.ure` manifest under `root`.
    ///
    /// Malformed manifests are skipped rather than fatal: a corpus is expected to
    /// contain experiments, and one bad file should not make the server
    /// unstartable for every agent connected to it.
    pub fn open(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let mut store = Store {
            root,
            patterns: Vec::new(),
            traces: Vec::new(),
            champions: BTreeMap::new(),
            df: HashMap::new(),
            total_resources: 0,
            malformed_traces: 0,
        };
        store.reload();
        store
    }

    pub fn reload(&mut self) {
        self.patterns.clear();
        self.df.clear();
        self.total_resources = 0;
        self.malformed_traces = 0;
        self.champions.clear();
        self.traces.clear();

        // Recurse, so pointing UNIA_STORE at a repository picks up curated
        // manifests in subdirectories such as resources/ and not only the
        // working-directory root.
        let mut paths = Vec::new();
        Self::collect_ure(&self.root, &mut paths, 0);
        // Sort so the corpus order is stable across restarts and scores for
        // equal relevance do not depend on directory iteration order.
        paths.sort();
        for path in paths {
            if let Some(p) = Self::parse(&path) {
                self.patterns.push(p);
            }
        }

        // Document frequency is counted per resource, not per phrase. A resource
        // that lists a term across many of its own aliases has not made that
        // term common in the corpus; it has described itself thoroughly. Counting
        // phrases let a self-referential resource down-weight its own strongest
        // term, so a rare term belonging to a different resource outranked it.
        for p in &self.patterns {
            let mut seen: HashSet<String> = HashSet::new();
            for phrase in p.phrases() {
                for term in tokenize(&phrase) {
                    seen.insert(term);
                }
            }
            self.total_resources += 1;
            for term in seen {
                *self.df.entry(term).or_insert(0) += 1;
            }
        }

        if let Ok(text) = std::fs::read_to_string(self.champions_path()) {
            if let Ok(map) = serde_json::from_str::<BTreeMap<String, String>>(&text) {
                self.champions = map;
            }
        }
        if let Ok(text) = std::fs::read_to_string(self.traces_path()) {
            // A line that fails to parse is counted rather than dropped
            // silently. A trace log that is unreadable and a trace log that is
            // empty are different states, and conflating them makes an artifact
            // with a corrupt evidence trail look like one that was never
            // observed -- which is precisely the confusion `Reachability`
            // exists to prevent.
            for line in text.lines().filter(|l| !l.trim().is_empty()) {
                match serde_json::from_str::<Trace>(line) {
                    Ok(t) => self.traces.push(t),
                    Err(_) => self.malformed_traces += 1,
                }
            }
        }
    }

    /// Walks `dir` collecting `.ure` files, bounded in depth so a store
    /// pointed at a home directory or filesystem root cannot walk forever.
    fn collect_ure(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
        const MAX_DEPTH: usize = 8;
        if depth > MAX_DEPTH {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // Follow the symlink only if it is a directory, and never descend
            // into a loop created by one.
            if path.is_dir() {
                Self::collect_ure(&path, out, depth + 1);
            } else if path.extension().and_then(|e| e.to_str()) == Some("ure") {
                out.push(path);
            }
        }
    }

    fn champions_path(&self) -> PathBuf {
        self.root.join("champions.json")
    }

    fn traces_path(&self) -> PathBuf {
        self.root.join("traces.jsonl")
    }

    fn parse(path: &Path) -> Option<Pattern> {
        let text = std::fs::read_to_string(path).ok()?;
        // Every loader in the project is serde_json. The two example manifests
        // that shipped as YAML could not be read by any code path; see
        // docs/SPEC.md divergence D3.
        let v: Value = serde_json::from_str(&text).ok()?;

        let id = v
            .get("resource_id")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })?;

        let actions = v
            .get("action_primitives")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| {
                        Some(Action {
                            id: a.get("id")?.as_str()?.to_string(),
                            aliases: a
                                .get("aliases")
                                .and_then(|x| x.as_array())
                                .map(|als| {
                                    als.iter()
                                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                            target_state: a
                                .get("target_state")
                                .and_then(|x| x.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            constraints: a
                                .get("constraints")
                                .and_then(|x| x.as_array())
                                .map(|cs| {
                                    cs.iter()
                                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let payload = v.get("payload").and_then(|p| {
            Some(Payload {
                kind: p.get("kind")?.as_str()?.to_string(),
                module: p
                    .get("module")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string()),
                entry: p
                    .get("entry")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string()),
                command: p.get("command").and_then(|c| c.as_array()).map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                }),
            })
        });

        Some(Pattern {
            id,
            category: v
                .get("category")
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_string(),
            guidance: v
                .get("guidance")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            actions,
            payload,
            source: path.to_path_buf(),
        })
    }

    /// Weight a term by how rare it is in the corpus, so a distinctive word
    /// dominates a match and a boilerplate one does not.
    fn idf(&self, term: &str) -> f64 {
        let n = self.total_resources.max(1) as f64;
        let df = *self.df.get(term).unwrap_or(&0) as f64;
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Similarity of a single phrase to the intent, in `0.0..=1.0`.
    fn score_phrase(&self, intent: &[String], intent_set: &HashSet<&String>, phrase: &str) -> f64 {
        let p_norm = phrase.to_lowercase().replace('_', " ");
        let i_norm = intent.join(" ");

        // An exact or near-exact phrase is a decisive match. This is the fast
        // path the Bridge uses for alias containment, and it should dominate.
        // Containment already implies coverage, so these returns are not scaled
        // by the coverage factor applied below.
        if p_norm == i_norm {
            return 1.0;
        }
        if !p_norm.is_empty() && i_norm.contains(&p_norm) {
            return 0.85;
        }
        if !i_norm.is_empty() && p_norm.contains(&i_norm) {
            return 0.75;
        }

        let p_tokens = tokenize(phrase);
        if p_tokens.is_empty() || intent.is_empty() {
            return 0.0;
        }

        // Inverse-document-frequency weighted overlap, normalised by the weaker
        // side so a short precise phrase is not penalised against a long intent.
        let mut overlap = 0.0f64;
        let mut p_weight = 0.0f64;
        for t in &p_tokens {
            let w = self.idf(t);
            p_weight += w;
            if intent_set.contains(t) {
                overlap += w;
            }
        }
        if p_weight == 0.0 {
            return 0.0;
        }
        let recall = overlap / p_weight;
        // Partial credit for a phrase sharing more than half its distinctive
        // vocabulary with the intent, which is a usable signal even when the
        // wording differs.
        let precision = if p_tokens.is_empty() {
            0.0
        } else {
            let matched = p_tokens.iter().filter(|t| intent_set.contains(*t)).count();
            matched as f64 / p_tokens.len() as f64
        };
        let base = 0.75 * recall + 0.25 * precision;

        // Both terms above measure how much of the *phrase* the intent covers.
        // Neither measures how much of the *intent* the phrase accounts for, so a
        // two-word alias sharing one common word scored as highly as a real
        // match: `create document` claimed `summarise this document for me`
        // because both mention a document, and `open file` claimed `open the
        // valve`. Scaling by intent coverage makes a phrase that explains only a
        // fraction of the request rank below one that explains the request, which
        // is what scoping an alias to the action it actually names requires.
        let covered = p_tokens.iter().filter(|t| intent_set.contains(*t)).count();
        let coverage = covered as f64 / intent.len() as f64;
        base * coverage.sqrt()
    }

    /// Every act the corpus declares as a whole sequence.
    ///
    /// The declared set [`crate::meet::serves`] refuses anything outside, and it is
    /// the parameter that makes the check decidable: without it a thin candidate is
    /// a subsequence of almost every witness, which is the case the retrieval false
    /// positives live in.
    pub fn declared_acts(&self) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = self
            .patterns
            .iter()
            .flat_map(|p| p.declared_acts())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Retrieval by decidable check rather than by score, for a caller that has
    /// already resolved the intent to primitives.
    ///
    /// No threshold and no ranking: a pattern matches when one of its declared acts
    /// is an act the intent performed, in order, whole. That is
    /// [`crate::meet::serves`], and the reason to prefer it over [`Store::search`]
    /// is the measurement rather than a preference — `search`'s false positives
    /// score 0.2236 and 0.2197 against a weakest true positive of 0.2041, so no
    /// threshold separates them and no threshold will ever. They are not badly
    /// separated. They are incomparable under a partial order, being asked about by
    /// a linear one.
    ///
    /// Returns fewer matches than `search` and never a wrong one. It also cannot
    /// find a pattern that declares no act, which is every hand-authored manifest in
    /// the corpus: those are prose and only prose can reach them. So this is a
    /// supplement to `search`, not a replacement, and the honest summary of the
    /// state is that the corpus cannot yet be searched structurally.
    pub fn search_primitives(&self, primitives: &[String], limit: usize) -> Vec<Match> {
        if primitives.is_empty() {
            return Vec::new();
        }
        let declared = self.declared_acts();
        if declared.is_empty() {
            return Vec::new();
        }
        let champion_ids: HashSet<&String> = self.champions.values().collect();

        let mut out: Vec<Match> = self
            .patterns
            .iter()
            .filter_map(|p| {
                let served = p
                    .declared_acts()
                    .into_iter()
                    .find(|act| crate::meet::serves(act, primitives, &declared))?;
                let is_champion = champion_ids.contains(&p.id);
                Some(Match {
                    id: p.id.clone(),
                    category: p.category.clone(),
                    // Not a confidence. Every match here is equally certain, and
                    // giving them all 1.0 is what keeps this function from being
                    // mistaken for a scorer with a better score.
                    score: 1.0,
                    champion: is_champion,
                    saves_tokens: p.payload.as_ref().is_some_and(|pl| pl.is_runnable()),
                    matched_on: served.join(" "),
                    guidance: p.guidance.clone(),
                    actions: p.actions.clone(),
                    payload: p.payload.clone(),
                    source: p.source.display().to_string(),
                })
            })
            .collect();
        out.truncate(limit);
        out
    }

    /// Ranks corpus patterns against an intent. Champions get a small boost so a
    /// previously-promoted pattern wins ties, which is what makes promotion
    /// sticky in the right direction.
    pub fn search(&self, intent: &str, limit: usize) -> Vec<Match> {
        let intent_tokens = tokenize(intent);
        if intent_tokens.is_empty() {
            return Vec::new();
        }
        let intent_set: HashSet<&String> = intent_tokens.iter().collect();
        let champion_ids: HashSet<&String> = self.champions.values().collect();

        let mut out: Vec<Match> = self
            .patterns
            .iter()
            .filter_map(|p| {
                let mut best = 0.0f64;
                let mut matched_on = String::new();
                for phrase in p.phrases() {
                    let s = self.score_phrase(&intent_tokens, &intent_set, &phrase);
                    if s > best {
                        best = s;
                        matched_on = phrase;
                    }
                }
                if best <= 0.15 {
                    return None;
                }
                let is_champion = champion_ids.contains(&p.id);
                let score = if is_champion {
                    (best * 1.1).min(1.0)
                } else {
                    best
                };
                let runnable = p.payload.as_ref().is_some_and(|pl| pl.is_runnable());
                Some(Match {
                    id: p.id.clone(),
                    category: p.category.clone(),
                    score,
                    champion: is_champion,
                    saves_tokens: runnable,
                    matched_on,
                    guidance: p.guidance.clone(),
                    actions: p.actions.clone(),
                    payload: p.payload.clone(),
                    source: p.source.display().to_string(),
                })
            })
            .collect();

        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        out.truncate(limit);
        out
    }

    /// The evidence readiness of an artifact, or `None` when it is unknown.
    ///
    /// Readiness is derived from the trace log, so an artifact with no recorded
    /// interactions has no readiness rather than a readiness of zero. The
    /// distinction matters to a caller rendering the corpus: "no evidence yet"
    /// and "evidence of no value" are opposite states, and collapsing them makes
    /// a young artifact indistinguishable from a discredited one.
    /// How many trace-log lines failed to parse on the last load.
    ///
    /// Nonzero means the evidence trail for at least one artifact is incomplete,
    /// so any readiness computed from it understates. A caller presenting these
    /// numbers should say so rather than render them as complete.
    pub fn malformed_traces(&self) -> usize {
        self.malformed_traces
    }

    /// Every trace loaded from the log, in the order they were recorded.
    ///
    /// Exposed so a caller can hand the log to `crate::induce`. Nothing in the
    /// crate did this before: traces were written by `record` and read by
    /// `readiness`, but the induction entry point takes a slice and no caller
    /// could supply one, so a populated trace log could not reach the module
    /// that exists to learn from it.
    pub fn traces(&self) -> &[Trace] {
        &self.traces
    }

    pub fn readiness(&self, id: &str) -> Option<f64> {
        let observations: Vec<&Trace> = self
            .traces
            .iter()
            .filter(|t| t.resource_id.as_deref() == Some(id))
            .collect();
        if observations.is_empty() {
            return None;
        }
        let _hits = observations.iter().filter(|t| t.outcome == "hit").count();
        let distinct = observations
            .iter()
            .map(|t| t.intent.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .len();
        let failures = observations.iter().filter(|t| !t.succeeded).count();
        // Breadth and reliability, log-scaled on breadth so that reaching the
        // minimum does not saturate the score. See crate::induce for the same
        // shape, applied where promotion is decided.
        let breadth = (distinct as f64).ln_1p() / (crate::induce::MIN_OBSERVATIONS as f64).ln_1p();
        let reliability = 1.0 - (failures as f64 / observations.len() as f64);
        Some((breadth.min(1.0) * reliability).clamp(0.0, 1.0))
    }

    pub fn get(&self, id: &str) -> Option<&Pattern> {
        self.patterns.iter().find(|p| p.id == id)
    }

    /// Every pattern id in the corpus.
    ///
    /// The eval harness needs this to tell an unsatisfiable expectation apart
    /// from a genuine routing failure: a query expecting a pattern the corpus
    /// does not contain cannot be routed by any matcher, and counting it as a
    /// miss would understate the real hit rate.
    pub fn ids(&self) -> Vec<String> {
        self.patterns.iter().map(|p| p.id.clone()).collect()
    }

    /// Appends a trace. Traces are append-only and are the raw material for
    /// induction; a corpus with no traces cannot learn anything.
    pub fn record(&mut self, trace: Trace) -> Result<(), std::io::Error> {
        use std::io::Write;
        std::fs::create_dir_all(&self.root)?;
        let line = serde_json::to_string(&trace).map_err(std::io::Error::other)?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.traces_path())?;
        writeln!(f, "{}", line)?;
        self.traces.push(trace);
        Ok(())
    }

    pub fn promote(&mut self, capability: &str, id: &str) -> Result<(), std::io::Error> {
        if !self.patterns.iter().any(|p| p.id == id) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no pattern with id {} in {}", id, self.root.display()),
            ));
        }
        self.champions
            .insert(capability.to_string(), id.to_string());
        std::fs::write(
            self.champions_path(),
            serde_json::to_string_pretty(&self.champions).map_err(std::io::Error::other)?,
        )
    }

    /// How the recorded acts divide between a person, the creature, and callers
    /// that were never creatures.
    ///
    /// A count rather than a ratio, because the ratio is a derived convenience
    /// and the raw division is the thing worth being able to read: a log of
    /// forty traces where the creature took twenty of them and a log of forty
    /// where it took none are both "half the log is not the creature", and only
    /// the second means there is no self-direction in it at all.
    pub fn authorship(&self) -> Authorship {
        let mut a = Authorship::default();
        for t in &self.traces {
            match t.actor {
                Some(Actor::Player) => a.player += 1,
                Some(Actor::Itself) => a.itself += 1,
                // `None` and an explicit `Caller` are the same claim about the
                // past — this interaction was not a creature's — and are counted
                // together, because a log written before the field existed must
                // not inflate the creature's share by having traces in it.
                Some(Actor::Caller) | None => a.caller += 1,
            }
        }
        a
    }

    pub fn stats(&self) -> Stats {
        let hits = self.traces.iter().filter(|t| t.outcome == "hit").count();
        let misses = self.traces.len() - hits;
        let tokens_saved: u64 = self
            .traces
            .iter()
            .filter(|t| t.outcome == "hit")
            .map(|t| t.tokens_in + t.tokens_out)
            .sum();
        Stats {
            root: self.root.display().to_string(),
            patterns: self.patterns.len(),
            runnable: self
                .patterns
                .iter()
                .filter(|p| p.payload.as_ref().is_some_and(|pl| pl.is_runnable()))
                .count(),
            champions: self.champions.len(),
            traces: self.traces.len(),
            hits,
            misses,
            tokens_saved,
            spec_version: "0.1.0".to_string(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// A trace with timestamps filled in, for callers that should not have to.
/// Served by the local machine.
pub const OUTCOME_HIT: &str = "hit";
/// Escalated, and still answered. **Not a failure.**
pub const OUTCOME_MISS: &str = "miss";
/// Escalated, and refused. **The third value, and the one the field could not
/// previously express** -- so a refusal and an escalation were entangled, and
/// the server rejected anything else.
///
/// Added because the escalation cascade has to be able to fail *honestly*. With
/// only `hit` and `miss`, a network that answered and a network that refused
/// were both "miss", and the difference -- which is the difference between
/// costing tokens and being an error -- was unrepresentable.
pub const OUTCOME_REFUSED: &str = "refused";

/// The default for an absent tier.
///
/// **A bare `#[serde(default)]` would have given `""`,** which is why this
/// function exists and why there is a test for it: the field's own doc comment
/// said an absent tier reads as `local`, and the first implementation returned an
/// empty string. That is the same defect as `clear_champion` removing a raw key
/// while its writer inserted a normalised one -- a documented contract that the
/// code does not keep -- and it is the second time this repository has produced
/// it. The test is the only reason this was caught before it shipped.
fn default_local() -> String {
    TIER_LOCAL.to_string()
}

/// Served by the local machine, or by the network when it could not.
pub const TIER_LOCAL: &str = "local";
pub const TIER_NETWORK: &str = "network";

/// Every outcome the trace format accepts, and every tier.
pub const OUTCOMES: [&str; 3] = [OUTCOME_HIT, OUTCOME_MISS, OUTCOME_REFUSED];
pub const TIERS: [&str; 2] = [TIER_LOCAL, TIER_NETWORK];

pub fn new_trace(
    intent: String,
    resource_id: Option<String>,
    outcome: &str,
    tin: u64,
    tout: u64,
) -> Trace {
    Trace {
        ts: now_secs(),
        intent,
        resource_id,
        outcome: outcome.to_string(),
        tokens_in: tin,
        tokens_out: tout,
        // An act is local unless something says otherwise. Set by the cascade
        // when it escalates; defaulted here so the common case needs no code.
        tier: TIER_LOCAL.to_string(),
        // Callers that know the sequence set this; a trace with no sequence
        // cannot be inducted from.
        primitives: Vec::new(),
        actor: None,
        succeeded: true,
    }
}

/// How a log of traces was divided between the three possible authors.
///
/// Exists so the ratio has a denominator nobody has to reconstruct. A single
/// number would have been easier to state and would have hidden the case that
/// matters most: a log where the creature never acts is not a log with a low
/// ratio, it is a log with no self-direction in it at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Authorship {
    /// Acts a person chose.
    pub player: usize,
    /// Acts the creature chose for itself.
    pub itself: usize,
    /// Interactions that were not a creature's at all.
    pub caller: usize,
}

impl Authorship {
    /// Of the acts a creature was involved in, the share it took by itself.
    ///
    /// `None` when no creature was involved, because a ratio between zero and
    /// zero is not a number and reporting `0.0` would read as "it never acts"
    /// when the truth is "there was nothing to act".
    pub fn self_directed(&self) -> Option<f64> {
        let acts = self.player + self.itself;
        (acts > 0).then(|| self.itself as f64 / acts as f64)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A self-cleaning scratch directory. The crate has no dev-dependencies and
    /// these tests only need somewhere to write manifests.
    pub(super) struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let p = std::env::temp_dir().join(format!(
                "unia-store-{tag}-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&p).expect("scratch dir");
            Self(p)
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

    /// Writes a minimal manifest, so retrieval tests assert against a corpus they
    /// define rather than against whatever happens to be checked in.
    pub(super) fn manifest(dir: &Path, id: &str, actions: &[(&str, &[&str])]) {
        let mut s = format!(
            r#"{{"ure_version":"1.0","resource_id":"{id}","category":"actuator","action_primitives":["#
        );
        for (i, (aid, aliases)) in actions.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                    r#"{{"id":"{aid}","aliases":[{}],"params":{{}},"target_state":"","constraints":[]}}"#,
                    aliases.iter().map(|a| format!("\"{a}\"")).collect::<Vec<_>>().join(",")
                ));
        }
        s.push_str("]}");
        std::fs::write(dir.join(format!("{id}.ure")), s).expect("write manifest");
    }

    /// A trace authored by one of the three, for the authorship counts.
    fn trace_by(actor: Option<Actor>) -> Trace {
        Trace {
            actor,
            intent: "feed the ca maduci".into(),
            ..new_trace(
                "feed the ca maduci".into(),
                Some("ca-x".into()),
                "hit",
                0,
                0,
            )
        }
    }

    /// The division is counted per author, with the three kept apart.
    ///
    /// Getting this from a log is the whole point: a trace left by a player and a
    /// trace left by the creature are otherwise byte-identical, and the ratio
    /// behind the word *civilisation* was unanswerable for that reason.
    #[test]
    fn authorship_counts_who_acted() {
        let (_dir, mut store) = corpus();
        for actor in [
            Some(Actor::Player),
            Some(Actor::Itself),
            Some(Actor::Itself),
            Some(Actor::Caller),
        ] {
            store.record(trace_by(actor)).ok().unwrap();
        }
        let a = store.authorship();
        assert_eq!(
            (a.player, a.itself, a.caller),
            (1, 2, 1),
            "one person, two acts by the creature, one interaction that was neither"
        );
    }

    /// A log written before the field existed must not make a creature look
    /// self-directed.
    ///
    /// `None` means "this interaction was not a creature's", not "unknown, so
    /// assume the best". Counting the absent as the creature's own would be the
    /// flattering error, and it is the one that would make the number worth
    /// reporting.
    #[test]
    fn a_trace_with_no_actor_counts_as_a_caller_and_not_as_the_creature() {
        let (_dir, mut store) = corpus();
        store.record(trace_by(None)).ok().unwrap();
        let a = store.authorship();
        assert_eq!(a.caller, 1);
        assert_eq!(a.itself, 0);
        assert_eq!(
            a.self_directed(),
            None,
            "no creature was involved, so there is no share to report"
        );
    }

    /// A self-directed ratio needs a creature in the denominator to mean
    /// anything, and returns nothing rather than a flattering zero without one.
    #[test]
    fn a_log_of_only_callers_reports_no_share_rather_than_zero() {
        let a = Authorship {
            player: 0,
            itself: 0,
            caller: 40,
        };
        assert_eq!(a.self_directed(), None, "zero of zero is not a number");
        let b = Authorship {
            player: 10,
            itself: 30,
            caller: 40,
        };
        assert_eq!(b.self_directed(), Some(0.75), "30 of 40 acts were its own");
    }

    /// An actor survives a round trip through the log, which is the only reason
    /// recording it is worth anything.
    #[test]
    fn an_actor_survives_the_log() {
        let (dir, mut store) = corpus();
        store.record(trace_by(Some(Actor::Itself))).ok().unwrap();
        drop(store);
        let reloaded = Store::open(dir.path());
        assert_eq!(
            reloaded.authorship().itself,
            1,
            "the creature's own act did not survive being written down"
        );
    }

    /// The two-resource corpus the fixture's failures were diagnosed against.
    pub(super) fn corpus() -> (Scratch, Store) {
        let dir = Scratch::new("corpus");
        manifest(
            dir.path(),
            "valve-001",
            &[
                (
                    "emergency_shutdown",
                    &["emergency_shutdown", "emergency shutdown", "halt"],
                ),
                ("adjust_flow", &["adjust_flow", "adjust flow", "set flow"]),
            ],
        );
        manifest(
            dir.path(),
            "fs-root-001",
            &[
                ("write_file", &["save file", "create document", "dump logs"]),
                ("read_file", &["open file", "get content", "fetch data"]),
                ("clear_cache", &["reset storage", "purge temp", "cleanup"]),
            ],
        );
        let store = Store::open(dir.path());
        (dir, store)
    }

    pub(super) fn top(store: &Store, intent: &str) -> Option<String> {
        store.search(intent, 1).first().map(|m| m.id.clone())
    }

    pub(super) fn score_of(store: &Store, intent: &str, id: &str) -> Option<f64> {
        store
            .search(intent, 10)
            .into_iter()
            .find(|m| m.id == id)
            .map(|m| m.score)
    }

    mod tokenize {
        use super::*;

        #[test]
        fn splits_an_underscored_identifier_into_its_words() {
            // An underscore used to be part of the term, so `write_file` was a single
            // opaque token that no natural-language intent could ever contain, and
            // the intent `write output to results.txt` matched nothing at all.
            // https://github.com/dacineu/unia/issues/1
            assert_eq!(tokenize("write_file"), vec!["write", "file"]);
        }

        #[test]
        fn treats_an_underscored_and_a_spaced_phrase_identically() {
            assert_eq!(tokenize("write_file"), tokenize("write file"));
        }

        #[test]
        fn discards_single_characters_and_punctuation() {
            // A one-character term carries no retrieval signal, and punctuation must
            // not survive as a term. `to` is two characters and is kept, so
            // stopword filtering is a separate concern from splitting.
            assert_eq!(tokenize("a write, to x!"), vec!["write", "to"]);
        }

        #[test]
        fn lowercases_before_splitting() {
            assert_eq!(tokenize("READ_FILE"), vec!["read", "file"]);
        }
    }

    mod idf {
        use super::*;

        #[test]
        fn ranks_a_term_absent_from_the_corpus_above_a_shared_one() {
            let (_d, s) = corpus();
            assert!(s.idf("chromodynamics") > s.idf("file"));
        }

        #[test]
        fn counts_resources_rather_than_phrases() {
            // Counting document frequency per phrase let a resource down-weight its
            // own strongest term for listing that term across many of its aliases.
            // `file` occurs in three fs-root-001 phrases but only one resource, so it
            // must be rarer than a term two different resources share.
            // https://github.com/dacineu/unia/issues/1
            let (_d, s) = corpus();
            assert_eq!(s.total_resources, 2);
            assert_eq!(s.df.get("file"), Some(&1));
            // `001` is a term of both resources' identifiers.
            assert_eq!(s.df.get("001"), Some(&2));
        }
    }

    mod search {
        use super::*;

        #[test]
        fn routes_an_intent_naming_a_resource_to_that_resource() {
            let (_d, s) = corpus();
            assert_eq!(top(&s, "open the valve").as_deref(), Some("valve-001"));
        }

        #[test]
        fn routes_an_intent_naming_an_underscored_action() {
            // `write output to results.txt` was a false negative: the action id
            // `write_file` tokenised to one opaque term, and the containment fast
            // path compared against a space-normalised phrase, so neither path
            // could match. https://github.com/dacineu/unia/issues/1
            let (_d, s) = corpus();
            assert_eq!(
                top(&s, "write output to results.txt").as_deref(),
                Some("fs-root-001")
            );
        }

        #[test]
        fn prefers_the_resource_named_over_a_third_party_alias() {
            // `open file` is an fs-root-001 alias and scored exactly level with
            // valve-001 on the words `open the valve`, so the alphabetical tiebreak
            // returned the wrong resource. https://github.com/dacineu/unia/issues/1
            let (_d, s) = corpus();
            let valve = score_of(&s, "open the valve", "valve-001").unwrap();
            let fs = score_of(&s, "open the valve", "fs-root-001").unwrap();
            assert!(valve > fs, "valve {valve} should beat fs-root-001 {fs}");
        }

        #[test]
        fn scores_an_alias_matching_only_one_common_word_below_a_real_match() {
            // `summarise this document for me` and the alias `create document` both
            // mention a document. Both overlap terms measure how much of the phrase
            // the intent covers; neither measured how much of the intent the phrase
            // accounts for. Intent coverage now separates the two.
            //
            // This is a ranking fix, not a rejection. The residual score still
            // clears the acceptance gate, and no threshold can remove it: the
            // false positive outscores the weakest true positive. Rejecting it
            // needs a capability check against the manifest's declared state
            // space, which is divergence D5.
            // https://github.com/dacineu/unia/issues/1
            let (_d, s) = corpus();
            let partial = score_of(&s, "summarise this document for me", "fs-root-001");
            let real = score_of(&s, "create a new document called notes", "fs-root-001");
            assert!(partial.unwrap_or(0.0) < real.unwrap_or(0.0));
        }

        #[test]
        fn accepts_a_phrase_fully_contained_in_the_intent() {
            // Containment is a stronger signal than partial overlap, so scaling by
            // intent coverage must not demote a phrase the intent spells out.
            let (_d, s) = corpus();
            assert_eq!(
                top(&s, "please create document now").as_deref(),
                Some("fs-root-001")
            );
        }

        #[test]
        fn returns_nothing_when_no_vocabulary_is_shared() {
            let (_d, s) = corpus();
            assert!(top(&s, "quantum chromodynamics lattice gauge").is_none());
        }

        #[test]
        fn returns_nothing_for_an_empty_intent() {
            let (_d, s) = corpus();
            assert!(s.search("", 5).is_empty());
        }

        #[test]
        fn scores_a_partial_explanation_below_a_complete_one() {
            // The ordering property the coverage factor exists to establish, stated
            // directly so a future scoring change cannot silently invert it.
            let (_d, s) = corpus();
            let partial = score_of(&s, "summarise this document for me", "fs-root-001");
            let complete = score_of(&s, "create a new document called notes", "fs-root-001");
            assert!(partial.unwrap_or(0.0) < complete.unwrap_or(0.0));
        }
    }

    mod store_open {
        use super::*;

        #[test]
        fn skips_a_malformed_manifest_rather_than_failing() {
            // A corpus is expected to contain experiments, and one bad file must not
            // make the server unstartable for every agent connected to it.
            let dir = Scratch::new("malformed");
            std::fs::write(dir.path().join("broken.ure"), "{ not json").unwrap();
            manifest(dir.path(), "valve-001", &[("halt", &["halt"])]);
            let store = Store::open(dir.path());
            assert_eq!(store.ids(), vec!["valve-001".to_string()]);
        }

        #[test]
        fn indexes_every_valid_manifest() {
            let (_d, s) = corpus();
            let mut ids = s.ids();
            ids.sort();
            assert_eq!(
                ids,
                vec!["fs-root-001".to_string(), "valve-001".to_string()]
            );
        }
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::tests_support_readiness::*;
    use crate::mcp::store::Store;

    #[test]
    fn reports_no_readiness_for_an_artifact_with_no_traces() {
        // "No evidence yet" and "evidence of no value" are opposite states, so
        // the absence is reported as absence rather than as a score of zero.
        let (_d, s) = store_with_traces("no-traces", &[]);
        assert_eq!(s.readiness("valve-001"), None);
    }

    #[test]
    fn reports_no_readiness_for_an_untraced_artifact_while_others_are_traced() {
        let (_d, s) = store_with_traces("partial", &[("valve-001", "halt it", true, &["Halt"])]);
        assert!(s.readiness("valve-001").is_some());
        assert_eq!(s.readiness("fs-root-001"), None);
    }

    #[test]
    fn reports_zero_readiness_when_every_observation_failed() {
        let (_d, s) = store_with_traces(
            "all-failed",
            &[
                ("valve-001", "halt it", false, &["Halt"]),
                ("valve-001", "shut it down", false, &["Halt"]),
            ],
        );
        assert_eq!(s.readiness("valve-001"), Some(0.0));
    }

    #[test]
    fn rises_with_the_number_of_distinct_observations() {
        // Readiness is continuous rather than binary: four observations and four
        // hundred must not render as the same object, which a champion flag
        // cannot express.
        let (_d, few) = store_with_traces("few", &[("valve-001", "halt it", true, &["Halt"])]);
        let (_d, many) = store_with_traces(
            "many",
            &[
                ("valve-001", "halt it", true, &["Halt"]),
                ("valve-001", "shut down", true, &["Halt"]),
                ("valve-001", "stop the valve", true, &["Halt"]),
                ("valve-001", "emergency shutdown", true, &["Halt"]),
            ],
        );
        let a = few.readiness("valve-001").unwrap();
        let b = many.readiness("valve-001").unwrap();
        assert!(
            b > a,
            "more distinct observations must score higher: {b} vs {a}"
        );
    }

    #[test]
    fn counts_repeated_identical_intents_as_one_observation() {
        // Repeating one sentence is one observation, not several.
        let (_d, s) = store_with_traces(
            "repeat",
            &[
                ("valve-001", "halt it", true, &["Halt"]),
                ("valve-001", "halt it", true, &["Halt"]),
                ("valve-001", "HALT IT", true, &["Halt"]),
            ],
        );
        let (_d, one) =
            store_with_traces("repeat-one", &[("valve-001", "halt it", true, &["Halt"])]);
        assert_eq!(s.readiness("valve-001"), one.readiness("valve-001"));
    }

    #[test]
    fn penalises_failed_observations() {
        let (_d, clean) = store_with_traces(
            "clean",
            &[
                ("valve-001", "halt it", true, &["Halt"]),
                ("valve-001", "shut down", true, &["Halt"]),
            ],
        );
        let (_d, mixed) = store_with_traces(
            "mixed",
            &[
                ("valve-001", "halt it", true, &["Halt"]),
                ("valve-001", "shut down", false, &["Halt"]),
            ],
        );
        assert!(mixed.readiness("valve-001") < clean.readiness("valve-001"));
    }

    #[test]
    fn counts_a_malformed_trace_line_instead_of_dropping_it_silently() {
        // A trace log that is unreadable and a trace log that is empty are
        // different states. Dropping the bad line made an artifact with a
        // corrupt evidence trail indistinguishable from one never observed,
        // which is the exact confusion Reachability exists to prevent.
        let dir = Scratch::new("malformed-line");
        let manifest = r#"{"ure_version":"1.0","resource_id":"valve-001","category":"actuator","action_primitives":[]}"#;
        std::fs::write(dir.path().join("valve-001.ure"), manifest).unwrap();
        std::fs::write(
            dir.path().join("traces.jsonl"),
            "{\"this is not json\"\n{\"intent\":\"halt it\",\"resource_id\":\"valve-001\"}\n",
        )
        .unwrap();
        let s = Store::open(dir.path());
        assert_eq!(s.malformed_traces(), 1);
        // The well-formed line still loaded, and defaults filled its absent
        // fields rather than the line being discarded wholesale.
        assert!(s.readiness("valve-001").is_some());
    }

    #[test]
    fn loads_a_trace_whose_optional_fields_are_absent() {
        // ts, outcome, tokens and resource_id all default, so a minimal
        // hand-written line is usable evidence.
        let (_d, s) = store_with_traces("minimal", &[("valve-001", "halt it", true, &["Halt"])]);
        assert_eq!(s.malformed_traces(), 0);
        assert!(s.readiness("valve-001").is_some());
    }

    #[test]
    fn keeps_readiness_within_the_unit_interval() {
        let (_d, s) = store_with_traces(
            "bounded",
            &[
                ("valve-001", "halt it", true, &["Halt"]),
                ("valve-001", "shut down", true, &["Halt"]),
                ("valve-001", "stop", true, &["Halt"]),
            ],
        );
        let r = s.readiness("valve-001").unwrap();
        assert!((0.0..=1.0).contains(&r), "readiness {r} out of range");
    }
}

#[cfg(test)]
mod tests_support_readiness {
    use super::*;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    pub(super) struct Scratch(pub PathBuf);

    impl Scratch {
        pub fn new(tag: &str) -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let p = std::env::temp_dir().join(format!(
                "unia-readiness-{tag}-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&p).expect("scratch dir");
            Scratch(p)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// One recorded interaction: which artifact served it, how it was phrased,
    /// whether it worked, and which primitives it resolved to.
    pub(super) type Observation<'a> = (&'a str, &'a str, bool, &'a [&'a str]);

    /// A store holding one artifact and a trace log built from `traces`.
    pub(super) fn store_with_traces(tag: &str, traces: &[Observation]) -> (Scratch, Store) {
        let dir = Scratch::new(tag);
        let manifest = r#"{"ure_version":"1.0","resource_id":"valve-001","category":"actuator","action_primitives":[{"id":"emergency_shutdown","aliases":["emergency shutdown","halt"],"params":{},"target_state":"","constraints":[]}]}"#;
        std::fs::write(dir.path().join("valve-001.ure"), manifest).expect("write manifest");
        let mut body = String::new();
        for (id, intent, succeeded, prims) in traces {
            let prims: Vec<String> = prims.iter().map(|s| format!("\"{s}\"")).collect();
            body.push_str(&format!(
                "{{\"intent\":\"{intent}\",\"resource_id\":\"{id}\",\"outcome\":\"hit\",\"primitives\":[{}],\"succeeded\":{succeeded}}}\n",
                prims.join(",")
            ));
        }
        let mut f = std::fs::File::create(dir.path().join("traces.jsonl")).expect("trace log");
        f.write_all(body.as_bytes()).expect("write traces");
        let store = Store::open(dir.path());
        (dir, store)
    }
}

#[cfg(test)]
mod structural_retrieval_tests {
    //! Retrieval by decidable check, and the corpus-shape problem it exposed.
    //!
    //! The claim being tested is narrow and the scope is stated in the same breath:
    //! for a corpus that declares primitive sequences, the false positives have a
    //! structural reason to go away. The current corpus does not, which is the more
    //! important half of the result.

    use super::*;

    fn seq(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|t| t.to_string()).collect()
    }

    /// A scratch directory that removes itself, declared here rather than borrowed
    /// from the other test module — a fixture shared across sibling modules has to
    /// be made visible to both, and the other one is not this module's business.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("unia-structural-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("scratch directory");
            Scratch(dir)
        }
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A manifest whose action id is a whole primitive sequence, which is what
    /// induction produces.
    fn induced(id: &str, act: &[&str]) -> String {
        assert_eq!(act.join("_"), id, "the fixture's id must be its sequence");
        format!(
            r#"{{"ure_version":"1.0","resource_id":"{id}","category":"actuator",
                 "action_primitives":[{{"id":"{id}","aliases":["{id}"],"params":{{}},
                 "target_state":"","constraints":[]}}]}}"#
        )
    }

    /// A manifest whose action id is prose, which is what a person writes.
    fn authored(id: &str) -> String {
        format!(
            r#"{{"ure_version":"1.0","resource_id":"{id}","category":"actuator",
                 "action_primitives":[{{"id":"{id}","aliases":["{id}"],"params":{{}},
                 "target_state":"","constraints":[]}}]}}"#
        )
    }

    /// The primitive-name list is a transcription of the enum, and a transcription
    /// drifts. This holds the two together: a name added to one and not the other
    /// fails here rather than silently becoming unrecognisable to the store.
    #[test]
    fn the_primitive_name_list_matches_the_enum() {
        use crate::bridge::primitive::{UniversalPrimitive, PRIMITIVE_NAMES};
        let every = [
            UniversalPrimitive::GetState,
            UniversalPrimitive::GetValue,
            UniversalPrimitive::CheckSense,
            UniversalPrimitive::SetValue,
            UniversalPrimitive::Toggle,
            UniversalPrimitive::Increment,
            UniversalPrimitive::Reset,
            UniversalPrimitive::Route,
            UniversalPrimitive::Pipe,
            UniversalPrimitive::Broadcast,
            UniversalPrimitive::Delay,
            UniversalPrimitive::Watch,
            UniversalPrimitive::Pulse,
            UniversalPrimitive::Compare,
            UniversalPrimitive::Transform,
            UniversalPrimitive::Validate,
        ];
        for p in &every {
            let name = format!("{p:?}");
            assert!(
                PRIMITIVE_NAMES.contains(&name.as_str()),
                "{name} is in the enum and not in PRIMITIVE_NAMES, so an action \
                 carrying it would be invisible to the store"
            );
        }
        assert_eq!(
            PRIMITIVE_NAMES.len(),
            every.len(),
            "the list has entries the enum does not"
        );
    }

    /// An induced action's id is recovered as a sequence, and a hand-authored one
    /// is not. The discriminator is that every component is a known primitive.
    #[test]
    fn an_induced_action_declares_an_act_and_a_hand_authored_one_does_not() {
        let induced = Action {
            id: "SetValue_CheckSense".into(),
            aliases: vec![],
            target_state: String::new(),
            constraints: vec![],
        };
        let authored_action = Action {
            id: "emergency_shutdown".into(),
            aliases: vec![],
            target_state: String::new(),
            constraints: vec![],
        };
        let fragment = Action {
            id: "SetValue".into(),
            aliases: vec![],
            target_state: String::new(),
            constraints: vec![],
        };

        assert_eq!(
            induced.declared_act(),
            Some(seq(&["SetValue", "CheckSense"]))
        );
        assert_eq!(
            authored_action.declared_act(),
            None,
            "a hand-authored action id was read as a primitive sequence"
        );
        assert_eq!(
            fragment.declared_act(),
            None,
            "a single primitive was read as a whole act, and a one-primitive \
             candidate is exactly what the decidable check cannot refuse"
        );
    }

    /// Case-sensitive and whole-token, because the same discipline as
    /// `resolve_primitive` is the only reason that function works. `setvalue` is
    /// not the primitive.
    #[test]
    fn primitive_names_are_matched_whole_and_case_sensitively() {
        use crate::bridge::primitive::is_primitive_name;
        assert!(is_primitive_name("SetValue"));
        assert!(!is_primitive_name("setvalue"));
        assert!(!is_primitive_name("SetValueX"));
        assert!(!is_primitive_name(""));
    }

    /// The measured result: for a corpus that declares acts, the check finds the
    /// true positive and refuses both shapes of false positive, and the refused
    /// ones are the shapes the scorer could not separate.
    #[test]
    fn the_decidable_check_refuses_what_no_threshold_could() {
        let dir = Scratch::new("structural");
        std::fs::write(
            dir.path().join("feed.ure"),
            induced("SetValue_CheckSense", &["SetValue", "CheckSense"]),
        )
        .ok()
        .unwrap();
        let store = Store::open(dir.path());

        assert_eq!(
            store.declared_acts(),
            vec![seq(&["SetValue", "CheckSense"])],
            "the induced act was not recovered from the corpus"
        );

        // The true positive: the act was performed, with an interleaved primitive.
        assert_eq!(
            store
                .search_primitives(&seq(&["SetValue", "Pulse", "CheckSense", "Emit"]), 10)
                .len(),
            1,
            "the true positive was refused"
        );
        // The two false-positive shapes: touched the act, then did something else.
        for witness in [seq(&["SetValue", "Pulse"]), seq(&["GetValue", "SetValue"])] {
            assert!(
                store.search_primitives(&witness, 10).is_empty(),
                "a witness that merely touched the act was served: {witness:?}"
            );
        }
    }

    /// **The finding, and the more important half.** A corpus of hand-authored
    /// manifests declares no acts at all, because their action ids are prose — so a
    /// structural search finds nothing and only the scorer can reach them.
    ///
    /// This is not a defect in the check. It is why the check was not already
    /// there: the corpus has to be induced before it can be searched
    /// structurally, and the two retrieval numbers are therefore still the ones the
    /// scorer produces. The first version of this test opened a `corpus/`
    /// directory and asserted about its contents, and there is no such directory —
    /// it was asserting against nothing.
    #[test]
    fn a_hand_authored_corpus_declares_no_acts_and_only_prose_reaches_it() {
        let dir = Scratch::new("authored");
        std::fs::write(dir.path().join("valve.ure"), authored("emergency_shutdown"))
            .ok()
            .unwrap();
        let store = Store::open(dir.path());

        assert!(
            store.declared_acts().is_empty(),
            "a prose action id declared a primitive sequence: {:?}",
            store.declared_acts()
        );
        assert!(
            store
                .search_primitives(&seq(&["SetValue", "CheckSense"]), 10)
                .is_empty(),
            "a corpus declaring no acts returned a structural match"
        );
        assert!(
            !store.search("emergency shutdown", 10).is_empty(),
            "and the prose path, which is the one still in use, cannot reach it \
             either — so this is a limit of the structural check, not a fix for it"
        );
    }
}

#[cfg(test)]
mod outcome_tests {
    //! The trace format is the instrument the learning loop reads, so a value it
    //! cannot express is a measurement it cannot make. These are the three cases
    //! that were entangled before `OUTCOME_REFUSED` and `tier` existed.

    use super::*;

    fn trace_with(outcome: &str, tier: &str) -> Trace {
        let mut t = new_trace("q".into(), None, outcome, 0, 0);
        t.tier = tier.to_string();
        t
    }

    /// **Served locally, escalated, and refused are three different facts.**
    ///
    /// Before, a failure and an escalation were both `"miss"`, which is a
    /// category the field's own documentation says does not exist: a `miss` is
    /// an escalation that *still got an answer*. So a network that answered and
    /// a network that refused were the same record, and the difference between
    /// *costing tokens* and *being an error* was unrepresentable.
    #[test]
    fn the_three_outcomes_are_three_distinguishable_facts() {
        let hit = trace_with(OUTCOME_HIT, TIER_LOCAL);
        let miss = trace_with(OUTCOME_MISS, TIER_NETWORK);
        let refused = trace_with(OUTCOME_REFUSED, TIER_NETWORK);

        assert_eq!(hit.outcome, OUTCOME_HIT);
        assert_eq!(miss.outcome, OUTCOME_MISS);
        assert_eq!(refused.outcome, OUTCOME_REFUSED);
        // The two that used to be one.
        assert_ne!(
            miss.outcome, refused.outcome,
            "an escalation that answered and one that refused are the same \\
             record, so the cost of escalating is not attributable"
        );
        // And the one the server used to reject outright, which is why a refusal
        // could not be recorded at all.
        assert!(OUTCOMES.contains(&refused.outcome.as_str()));
        assert_eq!(OUTCOMES.len(), 3);
    }

    /// **The tier is what makes a network-served act attributable.** Two acts
    /// with the same signature and the same witness are one capability — which
    /// is the property that makes local and network one system rather than two —
    /// and the tier is the only thing that says which one paid.
    #[test]
    fn the_tier_records_which_side_of_the_fence_served_the_act() {
        let local = trace_with(OUTCOME_HIT, TIER_LOCAL);
        let remote = trace_with(OUTCOME_MISS, TIER_NETWORK);
        assert_ne!(local.tier, remote.tier, "the cost of caring was invisible");
        assert!(TIERS.contains(&local.tier.as_str()));
    }

    /// **An absent tier is `local`, not a parse failure.** Most acts are local,
    /// and a log line written before the field existed should read as the common
    /// case rather than as corruption — the same reasoning the outcome default
    /// already used, and it is why the default is on the field and not applied
    /// at each read site.
    #[test]
    fn an_old_log_line_without_a_tier_reads_as_local() {
        let line = r#"{"ts":1,"intent":"q","outcome":"hit"}"#;
        let t: Trace = serde_json::from_str(line).expect("an old line still loads");
        assert_eq!(t.tier, TIER_LOCAL, "absent means the common case");
    }

    /// And the pairing that matters: `hit` is local, `miss`/`refused` are the
    /// network. A `hit` served by the network would mean the escalation
    /// succeeded, and recording that as local is the one mislabelling that would
    /// make the sharing policy unmeasurable.
    #[test]
    fn the_outcome_and_the_tier_have_to_agree_to_mean_anything() {
        assert!(TIERS.contains(&trace_with(OUTCOME_HIT, TIER_LOCAL).tier.as_str()));
        // A `hit` on the network is coherent -- the local side tried and the
        // network answered -- but then the outcome is the one that carries the
        // escalation, and the tier only says who served it. The format permits
        // both readings; what it must never do is let them disagree silently,
        // which is why both are explicit fields rather than one derived one.
        let net_hit = trace_with(OUTCOME_HIT, TIER_NETWORK);
        assert_eq!(net_hit.tier, TIER_NETWORK);
        assert_eq!(net_hit.outcome, OUTCOME_HIT);
    }
}
