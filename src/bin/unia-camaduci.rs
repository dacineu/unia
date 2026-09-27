//! Serves ca™maduci in a browser and records every care action as a trace.
//!
//! The point of this binary is the trace log, not the game. A player's clicks
//! become intents, each intent becomes a primitive sequence, and each sequence
//! becomes a line in `traces.jsonl` — which is exactly the input
//! `unia::induce` needs and cannot currently get, because nothing calls
//! `unia_record`. A neglected pet writes nothing at all, so the same corpus
//! records both the care given and the care withheld.
//!
//! Deliberately dependency-free: it binds a `std::net::TcpListener` and speaks
//! enough HTTP to serve one page and four routes. It is a local development
//! tool bound to loopback, not a server.
//!
//! ```sh
//! cargo run --bin unia-camaduci            # http://127.0.0.1:7731
//! cargo run --bin unia-camaduci -- --port 9000 --store ./patterns
//! ```

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use unia::camaduci::{Care, Firstness, Pet, firstness, MOTTO};
use unia::mcp::store::{Store, Trace};

/// Seconds of real time per in-game tick.
///
/// The pet's decay is defined per tick, not per second, so this only decides how
/// fast a neglected pet becomes visibly unhappy. It is a wall-clock convenience
/// and has no bearing on the traces.
const TICK_SECONDS: u64 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let port = arg_value(&args, "--port")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(7731);
    let store_dir = arg_value(&args, "--store").unwrap_or_else(|| ".".to_string());
    let pet_id = arg_value(&args, "--pet").unwrap_or_else(|| "ca-001".to_string());

    let mut store = Store::open(&store_dir);
    let mut pet = Pet::new(pet_id);
    let mut now = now_secs();

    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("ca™maduci on http://127.0.0.1:{port}");
    println!("  traces -> {store_dir}/traces.jsonl");
    println!("  one tick = {TICK_SECONDS}s of neglect; Ctrl-C to stop");

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        if let Err(e) = handle(&mut stream, &mut pet, &mut store, &mut now) {
            eprintln!("unia-camaduci: {e}");
        }
    }
    Ok(())
}

/// Reads `--name value` from the command line.
fn arg_value(args: &[String], name: &str) -> Option<String> {
    let i = args.iter().position(|a| a == name)?;
    args.get(i + 1).cloned()
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Routes one request and writes one response.
fn handle(
    stream: &mut TcpStream,
    pet: &mut Pet,
    store: &mut Store,
    now: &mut u64,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        if header.trim().is_empty() {
            break;
        }
        if let Some(v) = header.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    let body = String::from_utf8_lossy(&body).to_string();

    // Time passes between requests, so a player who walks away watches the pet
    // worsen without anything having to poll.
    let elapsed = now_secs().saturating_sub(*now);
    if elapsed >= TICK_SECONDS {
        for _ in 0..(elapsed / TICK_SECONDS) {
            pet.neglect();
        }
        *now += (elapsed / TICK_SECONDS) * TICK_SECONDS;
    }

    let response = match (method.as_str(), path.as_str()) {
        ("GET", "/") => html_response(),
        ("GET", "/api/pet") => json_response(200, &state(pet)),
        ("GET", "/api/motto") => {
            // The question is answered from what the log actually contains, not
            // from a stored verdict, so it changes as more players contribute.
            let induced: Vec<String> = unia::induce::induce_all(store.traces())
                .unwrap_or_default()
                .into_iter()
                .map(|c| c.du_uuid.to_string())
                .collect();
            let mut by_player: BTreeMap<String, Vec<String>> = BTreeMap::new();
            by_player.insert(pet.id.clone(), induced);
            json_response(200, &motto_state(&firstness(&by_player)))
        }
        ("GET", "/api/traces") => {
            let n = store.stats().traces;
            json_response(200, &format!(r#"{{"traces":{n}}}"#))
        }
        ("POST", "/api/care") => match parse_care(&body) {
            Some(care) => {
                let result = pet.tend(care);
                match result {
                    Some(primitives) => {
                        *now = now_secs();
                        // The player's own wording is recorded when they supply
                        // it, because the evidence gate counts distinct
                        // phrasings rather than occurrences. A client that always
                        // sends "feed the ca maduci" can never clear the gate no
                        // matter how many times it feeds, which is the gate
                        // working as specified rather than a defect.
                        let t = trace_for(&pet.id, care, &primitives, *now, parse_intent(&body));
                        // A failed write must not be reported as a recorded
                        // interaction: the pet really was cared for, but the
                        // training signal did not get it, and saying otherwise
                        // would overstate the evidence.
                        match store.record(t) {
                            Ok(()) => json_response(200, &state(pet)),
                            Err(e) => json_response(
                                500,
                                &format!(r#"{{"error":"could not record trace: {e}"}}"#),
                            ),
                        }
                    }
                    None => json_response(
                        409,
                        &format!(r#"{{"error":"the pet is gone","state":{}}}"#, state(pet)),
                    ),
                }
            }
            None => json_response(400, r#"{"error":"unknown care operation"}"#),
        },
        _ => json_response(404, r#"{"error":"not found"}"#),
    };

    stream.write_all(response.as_bytes())?;
    stream.flush()
}

/// Extracts the care operation from a JSON body without pulling in a parser.
///
/// The body is produced by this project's own client and is one of four known
/// strings, so a hand-rolled match is proportionate; anything that does not
/// parse is rejected rather than guessed at.
fn parse_care(body: &str) -> Option<Care> {
    string_field(body, "care").and_then(|v| Care::parse(&v))
}

/// Reads a string field out of the request body.
///
/// Hand-rolled rather than pulled from a JSON parser: the body comes from this
/// project's own client, and a dependency for four routes would cost more than
/// the twenty lines it saves. Anything unparseable yields `None`, which the
/// caller treats as "not supplied" rather than guessing.
fn string_field(body: &str, key: &str) -> Option<String> {
    let quoted = format!("\"{key}\"");
    let start = body.find(&quoted)? + quoted.len();
    let rest = &body[start..];
    let open = rest.find('"')?;
    let after = &rest[open + 1..];
    let close = after.find('"')?;
    let raw = &after[..close];
    if raw.is_empty() {
        None
    } else {
        // Minimal unescaping, enough for the two escapes a browser form emits.
        Some(raw.replace("\\\"", "\"").replace("\\\\", "\\"))
    }
}

/// The wording the player used, if they supplied one.
fn parse_intent(body: &str) -> Option<String> {
    string_field(body, "intent")
}

/// Builds the trace for one care action.
///
/// `tokens_in` is zero because tiers 1 and 2 cost none, and that is the number
/// an escalation rate would be computed from. `outcome` is `hit` because a
/// pattern served the call.
fn trace_for(
    pet_id: &str,
    care: Care,
    primitives: &[String],
    now: u64,
    intent: Option<String>,
) -> Trace {
    Trace {
        ts: now,
        intent: intent.unwrap_or_else(|| format!("{} the ca maduci", care.label())),
        resource_id: Some(pet_id.to_string()),
        outcome: "hit".to_string(),
        tokens_in: 0,
        tokens_out: 0,
        primitives: primitives.to_vec(),
        succeeded: true,
    }
}

/// Serialises the pet for the client.
///
/// Readiness is reported as null rather than zero, matching the corpus
/// convention: no trace-derived score exists for a live pet, and reporting zero
/// would say the pet had been shown to be worthless.
fn state(pet: &Pet) -> String {
    format!(
        r#"{{"id":{},"stage":{},"age_ticks":{},"hunger":{:.3},"happiness":{:.3},"health":{:.3},"mood":{},"quarantined":{},"primitives":{},"summary":{}}}"#,
        json_str(&pet.id),
        json_str(pet.stage.label()),
        pet.vitals.age_ticks,
        pet.vitals.hunger,
        pet.vitals.happiness,
        pet.vitals.health,
        json_str(pet.vitals.mood()),
        pet.quarantined,
        pet.history.len(),
        json_str(&pet.summary()),
    )
}

/// Escapes a string for a JSON document.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Renders the motto and what the log currently says about it.
fn motto_state(f: &Firstness) -> String {
    let answer = match f {
        Firstness::SingleLineage => {
            "One lineage, so the egg comes first. That is true and uninteresting: \
             every pet starts as an egg. A second player is what makes it a question."
                .to_string()
        }
        Firstness::Converged { shared, lineages } => format!(
            "Neither. {lineages} lineages reached the same content address, so the \
             chicken and the egg are one artifact. The question dissolves rather \
             than being decided. Address {shared}."
        ),
        Firstness::Undetermined => {
            "Undetermined. No two lineages have reached the same address yet, and a \
             shared primitive is not a shared ancestor. The log does not contain the \
             answer, so it is not guessed at."
                .to_string()
        }
    };
    format!(
        "{{\"motto\":{},\"answer\":{}}}",
        json_str(MOTTO),
        json_str(&answer)
    )
}

fn json_response(status: u16, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        _ => "Internal Server Error",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
}

fn html_response() -> String {
    let body = CLIENT_HTML;
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
}

/// The browser client, inlined so the binary is the whole deliverable.
const CLIENT_HTML: &str = include_str!("../../web/camaduci.html");
