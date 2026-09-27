//! `unia-mcp` — serves the `.ure` corpus to a coding agent over MCP stdio.
//!
//! Also has a plain CLI mode, which exists so the matcher and the store can be
//! exercised and verified without standing up an MCP client.

use rmcp::transport::stdio;
use rmcp::ServiceExt;
use std::path::PathBuf;
use unia::gather;
use unia::mcp::store::Store;
use unia::mcp::UniaServer;

/// Corpus location. `UNIA_STORE` lets one server be pointed at a different
/// pattern library per project without changing the client configuration.
fn resolve_root() -> PathBuf {
    std::env::var("UNIA_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn usage() -> &'static str {
    "unia-mcp — serve the .ure pattern corpus over MCP

USAGE
  unia-mcp                     Serve MCP over stdio (this is the normal mode)
  unia-mcp search <intent>     Print ranked pattern matches and exit
  unia-mcp stats               Print corpus statistics and exit
  unia-mcp get <id>            Print one manifest and exit

ENVIRONMENT
  topology     Print the measured artifact graph as JSON, for a visualiser.

  UNIA_STORE   Directory holding .ure manifests, plus the champions.json and
               traces.jsonl written beside them. Defaults to the working
               directory.
"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = resolve_root();
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(|s| s.as_str()) {
        Some("search") => {
            let intent = args.get(1).map(|s| s.as_str()).unwrap_or("");
            if intent.is_empty() {
                eprintln!("unia-mcp: search needs an intent argument");
                eprint!("{}", usage());
                std::process::exit(2);
            }
            let store = Store::open(&root);
            let matches = store.search(intent, 5);
            println!("{}", serde_json::to_string_pretty(&render(&matches))?);
        }
        Some("eval") => {
            let fixture_path = args
                .get(1)
                .map(|p| std::path::PathBuf::from(p))
                .unwrap_or_else(|| root.join("tests/fixtures/queries.json"));
            let fixture = match unia::mcp::eval::load(&fixture_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("unia-mcp: {e}");
                    std::process::exit(1);
                }
            };
            let store = Store::open(&root);
            let ids = store.ids();
            let report = unia::mcp::eval::run(&fixture, |q| store.search(q, 3), &ids);
            println!("{}", report.render());
            println!("{}", report.render_cases(&report.cases));
            println!("  corpus: {} patterns from {}", ids.len(), root.display());
        }
        Some("stats") => {
            let store = Store::open(&root);
            println!("{}", serde_json::to_string_pretty(&store.stats())?);
        }
        Some("topology") => {
            // The measured graph, for a visualiser. Every field here is read
            // from the corpus; nothing is synthesised, so an empty result is a
            // real measurement rather than a rendering failure.
            let store = Store::open(&root);
            println!("{}", serde_json::to_string_pretty(&topology(&store))?);
        }
        Some("get") => {
            let id = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let store = Store::open(&root);
            match store.get(id) {
                Some(p) => {
                    let runnable = p.payload.as_ref().is_some_and(|x| x.is_runnable());
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "id": p.id,
                            "category": p.category,
                            "guidance": p.guidance,
                            "actions": p.actions,
                            "payload": p.payload,
                            "saves_tokens": runnable,
                            "source": p.source.display().to_string(),
                        }))?
                    );
                }
                None => {
                    eprintln!("unia-mcp: no pattern with id {id} in {}", root.display());
                    std::process::exit(1);
                }
            }
        }
        Some("-h") | Some("--help") | Some("help") => print!("{}", usage()),
        None => {
            // Nothing on stdout in server mode: stdio is the transport.
            let service = UniaServer::new(Store::open(&root)).serve(stdio()).await?;
            service.waiting().await?;
        }
        Some(other) => {
            eprintln!("unia-mcp: unknown command {other}");
            eprint!("{}", usage());
            std::process::exit(2);
        }
    }

    Ok(())
}

/// Builds the measured graph: one node per artifact, one edge per tie a
/// gathering found.
///
/// Node fields are chosen so a renderer can encode evidence as geometry rather
/// than as a boolean. `spikes` is the primitive count and becomes the spike
/// count of the node, `readiness` becomes spike height, and `lifecycle` becomes
/// colour. An artifact with four confirming observations and one with four
/// hundred are then visibly different objects, which is the point of a
/// continuous readiness rather than a champion flag.
///
/// Empty edges are a legitimate result, not an error: the checked-in corpus
/// shares no primitive between its two curated artifacts, so the graph is
/// correctly edgeless.
fn topology(store: &Store) -> serde_json::Value {
    let mut nodes = Vec::new();
    let mut spheres = Vec::new();

    for id in store.ids() {
        let Some(p) = store.get(&id) else { continue };
        let actions: Vec<String> = p.actions.iter().map(|a| a.id.clone()).collect();
        let aliases: Vec<String> = p.actions.iter().flat_map(|a| a.aliases.clone()).collect();
        // No trace-log readiness exists yet, so this reports the absence rather
        // than a plausible-looking default. A visualiser must render "unknown"
        // differently from "no evidence".
        let readiness = store.readiness(&id);
        nodes.push(serde_json::json!({
            "id": p.id,
            "name": p.id,
            "category": p.category,
            "spikes": actions.len(),
            "actions": actions,
            "aliases": aliases,
            "readiness": readiness,
            "readiness_known": readiness.is_some(),
            "saves_tokens": p.payload.as_ref().is_some_and(|x| x.is_runnable()),
            "source": p.source.display().to_string(),
        }));
        spheres.push(gather::Sphere::new(
            p.id.clone(),
            actions,
            aliases,
            readiness.map_or(gather::Readiness::unproven(), gather::Readiness::new),
        ));
    }

    let (g, profile) = gather::gather_profiled(&spheres);
    let edges: Vec<serde_json::Value> = g
        .ties
        .iter()
        .map(|t| {
            serde_json::json!({
                "source": t.from,
                "target": t.to,
                "relation": t.relation,
                "readiness": t.readiness.score(),
            })
        })
        .collect();

    let denominators: Vec<serde_json::Value> = g
        .denominators
        .iter()
        .map(|d| {
            serde_json::json!({
                "capability": d.capability.0,
                "contributors": d.contributors,
                "aliases": d.aliases,
                "readiness": d.readiness.score(),
                "convergence": d.convergence,
            })
        })
        .collect();

    serde_json::json!({
        "nodes": nodes,
        "edges": edges,
        "denominators": denominators,
        "measured": true,
        "malformed_traces": store.malformed_traces(),
        "profile": {
            "participants": profile.participants,
            "comparisons": profile.comparisons,
            "capability_matches": profile.capability_matches,
            "lexical_matches": profile.lexical_matches,
            "compare_nanos": profile.compare_nanos,
            "assemble_nanos": profile.assemble_nanos,
        },
        "note": "Every field is read from the corpus. An empty edge list is a real measurement, not a rendering failure. readiness is null where no trace log exists, which is different from a readiness of zero.",
    })
}

/// Mirrors the server's `unia_search` response so CLI and MCP cannot drift.
fn render(matches: &[unia::mcp::store::Match]) -> serde_json::Value {
    let runnable = matches.iter().filter(|m| m.saves_tokens).count();
    serde_json::json!({
        "match_count": matches.len(),
        "runnable_count": runnable,
        "provider_call_avoided": runnable > 0,
        "note": if runnable == 0 {
            "No runnable payload. These patterns describe capability but cannot be \
             executed locally, so acting on one still costs a provider request."
        } else {
            "Runnable payloads present. Executing one of these costs no provider tokens."
        },
        "matches": matches,
    })
}
