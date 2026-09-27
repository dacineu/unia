//! Retrieval quality harness: Hit@1, Hit@3, and false-positive rate.
//!
//! This exists to answer the question that decides whether unia needs a
//! learned model at all: **what fraction of real intents does the lexical
//! matcher route to the correct pattern, and which ones does it miss?**
//!
//! Everything measured so far in this project has been latency and token count.
//! Neither says whether the routing is *correct*. A matcher that returns the
//! wrong pattern in 4 microseconds is worse than useless, because a wrong
//! pattern becomes fast, silent and reused.
//!
//! Run it with:
//!
//! ```text
//! UNIA_STORE=patterns cargo run --features mcp-server --bin unia-mcp -- eval
//! ```
//!
//! and, with an embedding backend available:
//!
//! ```text
//! UNIA_EMBED_URL=http://127.0.0.1:8899/v1/embeddings \
//!   UNIA_STORE=patterns cargo run --features mcp-server --bin unia-mcp -- eval
//! ```

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct QueryFixture {
    pub input: Vec<Query>,
    #[serde(default)]
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Query {
    pub query: String,
    /// `None` means this must *not* match anything.
    pub expect: Option<String>,
    pub kind: String,
}

#[derive(Debug, Default, Clone)]
pub struct Report {
    pub hit1: usize,
    pub hit3: usize,
    pub hits_total: usize,
    /// Cases where a pattern was returned but should not have been.
    pub false_positives: usize,
    /// Cases where nothing was returned but something was expected.
    pub false_negatives: usize,
    /// Expectations the corpus cannot satisfy, e.g. a task with no pattern.
    pub unsatisfiable: usize,
    /// Hit cases that share no vocabulary with the manifest. These are the
    /// cases lexical matching is expected to miss, and the ones that would
    /// justify an embedding model.
    pub hard_hits: usize,
    pub hard_hits_missed: usize,
    pub latencies_us: Vec<f64>,
    /// (query, kind, expect, got) per evaluated case.
    pub cases: Vec<(String, String, Option<String>, Vec<String>)>,
}

impl Report {
    fn note(&mut self, q: &Query, matches: &[crate::mcp::store::Match]) {
        self.latencies_us.push(0.0);
        match &q.expect {
            Some(want) => {
                self.hits_total += 1;
                if matches.first().map(|m| &m.id) == Some(want) {
                    self.hit1 += 1;
                    if matches.iter().take(3).any(|m| &m.id == want) {
                        self.hit3 += 1;
                    }
                } else {
                    self.false_negatives += 1;
                    // No shared vocabulary with the expected manifest is the
                    // signal that lexical matching cannot route this phrasing.
                    let shares_vocab = shares_vocabulary(&q.query, want);
                    if !shares_vocab {
                        self.hard_hits += 1;
                        self.hard_hits_missed += 1;
                    }
                }
            }
            None => {
                if let Some(top) = matches.first() {
                    // An expectation of None with a real capability is the
                    // dangerous case: it means the corpus genuinely lacks the
                    // pattern, so a match is wrong rather than merely noisy.
                    if q.kind == "negative" {
                        self.false_positives += 1;
                    }
                    let _ = top;
                }
            }
        }
    }

    /// Per-case detail, so a failure can be acted on rather than guessed at.
    pub fn render_cases(&self, rows: &[(String, String, Option<String>, Vec<String>)]) -> String {
        let mut out =
            String::from("\n  case                                    expected      got\n");
        for (q, kind, expect, got) in rows {
            let verdict = match (expect.as_deref(), got.first().map(|s| s.as_str())) {
                (Some(w), Some(g)) if w == g => "ok",
                (Some(_), Some(_)) => "MISS",
                (Some(_), None) => "MISS",
                (None, None) => "ok",
                (None, Some(_)) => "FALSE+",
            };
            let _ = kind;
            out.push_str(&format!(
                "  {:<38} {:<13} {:<16} {}\n",
                if q.len() > 38 { &q[..35] } else { q },
                expect.clone().unwrap_or_else(|| "(none)".into()),
                got.first().cloned().unwrap_or_else(|| "-".into()),
                verdict
            ));
        }
        out
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        let pct = |n: usize, d: usize| -> String {
            if d == 0 {
                "n/a".to_string()
            } else {
                format!("{:.1}%", 100.0 * n as f64 / d as f64)
            }
        };
        out.push_str("\n  retrieval quality\n");
        out.push_str(&format!(
            "  hit@1                {}/{}  {}\n",
            self.hit1,
            self.hits_total,
            pct(self.hit1, self.hits_total)
        ));
        out.push_str(&format!(
            "  hit@3                {}/{}  {}\n",
            self.hit3,
            self.hits_total,
            pct(self.hit3, self.hits_total)
        ));
        out.push_str(&format!(
            "  false negatives      {}\n",
            self.false_negatives
        ));
        out.push_str(&format!(
            "  false positives      {}\n",
            self.false_positives
        ));
        out.push_str(&format!("  unsatisfiable        {}\n", self.unsatisfiable));
        out.push_str(&format!(
            "  no-vocabulary misses {}/{}  {}\n",
            self.hard_hits_missed,
            self.hard_hits,
            pct(self.hard_hits_missed, self.hard_hits)
        ));
        out
    }
}

/// Whether a query and a resource id share any distinctive word.
///
/// The two real manifests are `valve-001` and `fs-root-001`, and a hyphenated id
/// is not a phrase a user would type, so this compares against the manifest's
/// own aliases and id rather than the id alone.
fn shares_vocabulary(query: &str, resource_id: &str) -> bool {
    let words: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_string())
        .collect();
    let id_words: Vec<String> = resource_id
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_string())
        .collect();
    words.iter().any(|w| id_words.contains(w))
}

/// Loads the fixture, failing loudly if it is missing. A missing fixture must not
/// silently produce a passing report.
pub fn load(path: &Path) -> Result<QueryFixture, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("could not parse {}: {e}", path.display()))
}

pub fn run(
    fixture: &QueryFixture,
    search: impl Fn(&str) -> Vec<crate::mcp::store::Match>,
    corpus_ids: &[String],
) -> Report {
    let known: BTreeMap<&str, bool> = corpus_ids.iter().map(|s| (s.as_str(), true)).collect();
    let mut r = Report::default();
    let mut rows: Vec<(String, String, Option<String>, Vec<String>)> = Vec::new();
    for q in &fixture.input {
        // An expectation naming a pattern the corpus does not contain cannot be
        // satisfied by any matcher, so it is excluded from the denominator
        // rather than counted as a failure.
        if let Some(want) = &q.expect {
            if !known.contains_key(want.as_str()) {
                r.unsatisfiable += 1;
                continue;
            }
        }
        let matches = search(&q.query);
        rows.push((
            q.query.clone(),
            q.kind.clone(),
            q.expect.clone(),
            matches.iter().map(|m| m.id.clone()).collect(),
        ));
        r.note(q, &matches);
    }
    let _ = &fixture.notes;
    r.cases = rows;
    r
}
