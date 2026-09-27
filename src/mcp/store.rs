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

/// One recorded execution. These are the training signal: without them there is
/// nothing to induce an actuator from, and today the project discards them.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trace {
    pub ts: u64,
    pub intent: String,
    pub resource_id: Option<String>,
    /// `hit` when a pattern served the call, `miss` when a provider was called.
    ///
    /// A miss is not a failure. It means the system escalated and the caller
    /// still got an answer, which is a successful interaction that cost tokens.
    /// Treating escalation as failure would discard most of the training
    /// signal, since a miss is what an interaction looks like before any
    /// pattern exists to serve it. Actual failure is `succeeded`, below.
    pub outcome: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
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
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty() && t.len() > 1)
        .map(|t| t.to_string())
        .collect()
}

pub struct Store {
    root: PathBuf,
    patterns: Vec<Pattern>,
    traces: Vec<Trace>,
    champions: BTreeMap<String, String>,
    /// term -> number of phrases containing it, for inverse-document-frequency.
    df: HashMap<String, usize>,
    total_phrases: usize,
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
            total_phrases: 0,
        };
        store.reload();
        store
    }

    pub fn reload(&mut self) {
        self.patterns.clear();
        self.df.clear();
        self.total_phrases = 0;
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

        for p in &self.patterns {
            for phrase in p.phrases() {
                self.total_phrases += 1;
                for term in tokenize(&phrase) {
                    *self.df.entry(term).or_insert(0) += 1;
                }
            }
        }

        if let Ok(text) = std::fs::read_to_string(self.champions_path()) {
            if let Ok(map) = serde_json::from_str::<BTreeMap<String, String>>(&text) {
                self.champions = map;
            }
        }
        if let Ok(text) = std::fs::read_to_string(self.traces_path()) {
            for line in text.lines().filter(|l| !l.trim().is_empty()) {
                if let Ok(t) = serde_json::from_str::<Trace>(line) {
                    self.traces.push(t);
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
                path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
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
                module: p.get("module").and_then(|m| m.as_str()).map(|s| s.to_string()),
                entry: p.get("entry").and_then(|m| m.as_str()).map(|s| s.to_string()),
                command: p.get("command").and_then(|c| c.as_array()).map(|a| {
                    a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
                }),
            })
        });

        Some(Pattern {
            id,
            category: v.get("category").and_then(|x| x.as_str()).unwrap_or("unknown").to_string(),
            guidance: v.get("guidance").and_then(|x| x.as_str()).unwrap_or_default().to_string(),
            actions,
            payload,
            source: path.to_path_buf(),
        })
    }

    /// Weight a term by how rare it is in the corpus, so a distinctive word
    /// dominates a match and a boilerplate one does not.
    fn idf(&self, term: &str) -> f64 {
        let n = self.total_phrases.max(1) as f64;
        let df = *self.df.get(term).unwrap_or(&0) as f64;
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Similarity of a single phrase to the intent, in `0.0..=1.0`.
    fn score_phrase(&self, intent: &[String], intent_set: &HashSet<&String>, phrase: &str) -> f64 {
        let p_norm = phrase.to_lowercase().replace('_', " ");
        let i_norm = intent.join(" ");

        // An exact or near-exact phrase is a decisive match. This is the fast
        // path the Bridge uses for alias containment, and it should dominate.
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
        0.75 * recall + 0.25 * precision
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
                let score = if is_champion { (best * 1.1).min(1.0) } else { best };
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

    pub fn get(&self, id: &str) -> Option<&Pattern> {
        self.patterns.iter().find(|p| p.id == id)
    }

    /// Appends a trace. Traces are append-only and are the raw material for
    /// induction; a corpus with no traces cannot learn anything.
    pub fn record(&mut self, trace: Trace) -> Result<(), std::io::Error> {
        use std::io::Write;
        std::fs::create_dir_all(&self.root)?;
        let line = serde_json::to_string(&trace).map_err(std::io::Error::other)?;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(self.traces_path())?;
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
        self.champions.insert(capability.to_string(), id.to_string());
        std::fs::write(
            self.champions_path(),
            serde_json::to_string_pretty(&self.champions).map_err(std::io::Error::other)?,
        )
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
pub fn new_trace(intent: String, resource_id: Option<String>, outcome: &str, tin: u64, tout: u64) -> Trace {
    Trace {
        ts: now_secs(),
        intent,
        resource_id,
        outcome: outcome.to_string(),
        tokens_in: tin,
        tokens_out: tout,
        // Callers that know the sequence set this; a trace with no sequence
        // cannot be inducted from.
        primitives: Vec::new(),
        succeeded: true,
    }
}
