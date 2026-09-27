//! Measures what a `.ure` pattern lookup costs, against a local corpus.
//!
//! The point is not to benchmark the crate in the abstract. It is to put a
//! number on the claim that pattern retrieval is cheap enough to sit on the hot
//! path, and to show how cost scales as the corpus grows, because the ranking
//! is currently a linear scan and that is the limit worth knowing about.

use std::path::PathBuf;
use std::time::Instant;
use unia::mcp::store::{Action, Pattern, Payload, Store};

/// Intents shaped like the ones a coding agent would actually issue.
const INTENTS: &[&str] = &[
    "add a rust dependency to Cargo.toml and update the lockfile",
    "emergency shutdown the valve",
    "write a file to disk",
    "read the contents of a file",
    "clear the temp cache",
    "run the test suite and report failures",
    "bump the version in package.json",
    "format the rust sources",
];

fn synth(n: usize) -> Vec<Pattern> {
    (0..n)
        .map(|i| Pattern {
            id: format!("actuator-{i:05}"),
            category: ["actuator", "skill", "tool", "doc"][i % 4].to_string(),
            guidance: format!("Synthetic capability {i} for benchmarking corpus growth."),
            actions: vec![Action {
                id: format!("step_{i}"),
                // One pattern in four carries a real intent phrase, so the
                // benchmark exercises the scoring path rather than timing a scan
                // that rejects every candidate at the threshold.
                aliases: if i % 4 == 0 {
                    vec![INTENTS[i % INTENTS.len()].to_string(), format!("alias {i}")]
                } else {
                    vec![format!("alias {i}"), format!("op {i}")]
                },
                target_state: "done = true".to_string(),
                constraints: vec![],
            }],
            payload: Some(Payload {
                kind: "wasm".to_string(),
                module: Some(format!("sha256:{i:064x}")),
                entry: Some("main".to_string()),
                command: None,
            }),
            source: PathBuf::from(format!("synthetic/{i}.ure")),
        })
        .collect()
}

fn main() {
    println!(
        "{:>8}  {:>12}  {:>12}  {:>10}  {:>12}",
        "corpus", "p50 (us)", "p99 (us)", "matched", "per-intent"
    );
    println!("{}", "-".repeat(62));

    for size in [12usize, 100, 1_000, 5_000, 20_000] {
        let patterns = synth(size);
        let phrases: Vec<String> = patterns
            .iter()
            .flat_map(|p| {
                p.actions
                    .iter()
                    .flat_map(|a| std::iter::once(a.id.clone()).chain(a.aliases.iter().cloned()))
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut samples: Vec<f64> = Vec::with_capacity(INTENTS.len() * 20);
        let mut matched = 0usize;

        // Warm up so the first iteration is not charged for page faults on the
        // corpus, which is the difference between a microsecond and a
        // millisecond and is not what we are trying to measure.
        for i in 0..INTENTS.len() {
            let _ = rank(&patterns, &phrases, INTENTS[i], 5);
        }

        for _ in 0..20 {
            for intent in INTENTS {
                let t0 = Instant::now();
                let hits = rank(&patterns, &phrases, intent, 5);
                samples.push(t0.elapsed().as_secs_f64() * 1e6);
                matched += hits;
            }
        }

        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = samples[samples.len() / 2];
        let p99 = samples[(samples.len() as f64 * 0.99) as usize % samples.len()];

        println!(
            "{:>8}  {:>12.1}  {:>12.1}  {:>10}  {:>12.1}",
            size,
            p50,
            p99,
            matched,
            p50 / INTENTS.len() as f64
        );
    }
}

/// A stand-in for `Store::search` that shares its shape: score every pattern
/// against the intent. Reimplemented here rather than called so the benchmark
/// measures retrieval cost alone, without the filesystem and champion state
/// that `Store` also carries.
fn rank(patterns: &[Pattern], _phrases: &[String], intent: &str, limit: usize) -> usize {
    let toks: Vec<String> = intent
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| t.len() > 1)
        .map(|t| t.to_string())
        .collect();
    if toks.is_empty() {
        return 0;
    }
    let joined = toks.join(" ");

    let mut scored: Vec<(f64, &str)> = patterns
        .iter()
        .filter_map(|p| {
            let best = p
                .actions
                .iter()
                .flat_map(|a| std::iter::once(&a.id).chain(a.aliases.iter()))
                .map(|phrase| {
                    let norm = phrase.to_lowercase().replace('_', " ");
                    if norm == joined {
                        1.0
                    } else if joined.contains(&norm) {
                        0.85
                    } else {
                        let pt: Vec<&str> = norm.split(' ').filter(|w| w.len() > 1).collect();
                        if pt.is_empty() {
                            0.0
                        } else {
                            pt.iter().filter(|w| toks.iter().any(|t| t == *w)).count() as f64
                                / pt.len() as f64
                        }
                    }
                })
                .fold(0.0f64, f64::max);
            if best <= 0.15 {
                None
            } else {
                Some((best, p.id.as_str()))
            }
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored.len()
}
