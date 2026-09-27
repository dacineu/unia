//! `unia-mcp` — serves the `.ure` corpus to a coding agent over MCP stdio.
//!
//! Also has a plain CLI mode, which exists so the matcher and the store can be
//! exercised and verified without standing up an MCP client.

use rmcp::transport::stdio;
use rmcp::ServiceExt;
use std::path::PathBuf;
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
