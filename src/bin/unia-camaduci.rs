//! Serves ca(R)maduci in a browser and records every care action as a trace.
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

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use unia::camaduci::Understanding;
use unia::camaduci::{
    firstness, Care, Drive, Firstness, LearnedRule, Personality, Pet, Sensor, MOTTO,
};
use unia::mcp::store::{Actor, Store, Trace};

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
    let mut pet = load_pet(&store_dir, &pet_id);
    let mut now = now_secs();
    let mut started = false;

    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("ca(R)maduci on http://127.0.0.1:{port}");
    println!("  traces -> {store_dir}/traces.jsonl");
    println!("  one tick = {TICK_SECONDS}s of neglect, starting with your first action");
    println!("  open http://127.0.0.1:{port} and press Feed to begin");

    // The accept loop is blocking, which is what a `std::net::TcpListener` gives.
    // That is deliberate: the relay is a long-lived foreground service under
    // systemd, and a blocking loop with a thread per connection has nothing to
    // gain from a reactor here.
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        if let Err(e) = handle(
            &mut stream,
            &mut pet,
            &mut store,
            &mut now,
            &mut started,
            &store_dir,
        ) {
            eprintln!("unia-camaduci: {e}");
        }
    }
    Ok(())
}

/// Persists a creature and its education. Called after every interaction, so a
/// crash costs one turn rather than the whole session.
fn persist(store_dir: &str, pet: &mut Pet, store: &Store) {
    pet.learn(learned_rules(store));
    save_pet(store_dir, pet);
}

/// Reads what the creature has worked out from its own trace log.
fn learned_rules(store: &Store) -> Vec<LearnedRule> {
    let mut rules: Vec<LearnedRule> = match unia::induce::induce_all(store.traces()) {
        Ok(c) => c
            .into_iter()
            .map(|c| LearnedRule {
                address: c.du_uuid.to_string(),
                signature: c.signature,
                aliases: c.learned_aliases,
                confidence: c.confidence,
                observations: c.observations,
            })
            .collect(),
        Err(e) => {
            eprintln!("unia-camaduci: induction failed: {e}");
            Vec::new()
        }
    };
    // Most confident first, so the strongest rule is the one a renderer puts at
    // the front and a player reads as the creature's headline.
    rules.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rules
}

/// Where a creature's memory is kept, beside the traces it learned from.
fn pet_path(dir: &str, id: &str) -> String {
    format!("{dir}/pet-{id}.json")
}

/// Loads a creature's memory, or starts a new one.
///
/// A missing file is a new creature, which is the ordinary case. A file that
/// exists but cannot be read is *not* silently replaced: substituting a live egg
/// for a creature that died would erase the one part of it that cannot be
/// regenerated, and the player would never learn their pet had been swapped.
fn load_pet(dir: &str, id: &str) -> Pet {
    let path = pet_path(dir, id);
    match std::fs::read_to_string(&path) {
        Ok(text) => match Pet::from_json(id, &text) {
            Some(p) => {
                println!(
                    "  restored {id}: {} sleep cycles, care {}",
                    p.vitals.age_ticks,
                    p.summary()
                );
                p
            }
            None => {
                eprintln!("unia-camaduci: {path} is unreadable; refusing to overwrite it");
                eprintln!("unia-camaduci: move it aside to start a new creature");
                std::process::exit(1);
            }
        },
        Err(_) => {
            println!("  {id} is new");
            Pet::new(id)
        }
    }
}

/// Writes a creature's memory.
///
/// Best effort: a failure to save loses progress the player can see is still
/// there, and aborting the process over it would be worse than the loss.
fn save_pet(dir: &str, pet: &Pet) {
    if let Err(e) = std::fs::write(pet_path(dir, &pet.id), pet.to_json()) {
        eprintln!("unia-camaduci: could not save {}: {e}", pet.id);
    }
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
    started: &mut bool,
    store_dir: &str,
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
    //
    // Nothing decays until the first care action. A pet that began ageing the
    // moment the process started would already be dead by the time anyone had
    // read the URL, and "neglect" has to mean a player choosing not to care for
    // a creature they have not yet met. The clock starts when they arrive.
    if *started {
        let elapsed = now_secs().saturating_sub(*now);
        if elapsed >= TICK_SECONDS {
            for _ in 0..(elapsed / TICK_SECONDS) {
                pet.neglect();
            }
            *now += (elapsed / TICK_SECONDS) * TICK_SECONDS;
        }
    }

    let response = match (method.as_str(), path.as_str()) {
        ("GET", "/") => html_response(),
        ("GET", "/api/pet") => {
            // Re-derive on read so the creature always shows the most current
            // picture of what it has learned, without waiting for an action.
            pet.learn(learned_rules(store));
            json_response(200, &state(pet))
        }
        // What the creature wants for itself, and what it can perceive. The
        // policy and the sensor layer are the two things that are new, and a
        // player cannot see either without being told.
        // The creature acting on its own reading of itself. This is the whole of
        // what "hunger is not just a number" means operationally: the drive names
        // the act, the acts are the same acts a player can perform, and the
        // recording is an ordinary trace so the same induction that learns from a
        // player also learns from this. Nothing about the trace distinguishes the
        // two, which is the point — a creature feeding itself produces evidence
        // exactly as feeding it does.
        ("POST", "/api/tend") => {
            let Some((care, drawn_by)) = pet.want() else {
                return Ok(stream.write_all(
                    json_response(
                        409,
                        &json!({
                            "acted": false,
                            "why": "Nothing is pulling me right now. Feed me, play with me, \
                                    or put me to sleep, and I will want what I need.",
                        })
                        .to_string(),
                    )
                    .as_bytes(),
                )?);
            };
            // The creature's own door, not the player's. This endpoint is the
            // creature acting on its own reading, so it must go through the
            // affordability and power gate — a stuck creature cannot feed
            // itself, and letting it try and succeed would quietly delete the
            // distinction between being helpless and being unhelped.
            let Some(primitives) = pet.tend_as_self(care) else {
                // Three reasons to be unable to act, and a client that is told
                // "409" cannot tell them apart.
                let why = pet
                    .why_wont_act()
                    .unwrap_or_else(|| "I am gone.".to_string());
                return Ok(stream.write_all(
                    json_response(409, &json!({ "acted": false, "why": why }).to_string())
                        .as_bytes(),
                )?);
            };
            *started = true;
            *now = now_secs();
            // The creature chose this act, on its own reading. The trace says so,
            // so a log can later report how much of its own living it did.
            let t = trace_for(&pet.id, care, &primitives, *now, None, Actor::Itself);
            match store.record(t) {
                Ok(()) => {
                    pet.learn(learned_rules(store));
                    save_pet(store_dir, pet);
                    json_response(
                        200,
                        &json!({
                            "acted": true,
                            "care": care.label(),
                            "drawn_by": drawn_by,
                            "primitives": primitives,
                            // `state(pet)` is already serialised, so nesting it
                            // in `json!` would double-encode it and a client would
                            // find a string where an object belongs.
                            "state": serde_json::from_str(&state(pet))
                                .unwrap_or(serde_json::Value::Null),
                        })
                        .to_string(),
                    )
                }
                Err(e) => json_response(500, &format!(r#"{{"error":"{e}"}}"#)),
            }
        }
        // Changing a creature's disposition. It is a new creature, and the address
        // says so — which is the whole argument for keeping drives and sensors in
        // the skeleton rather than in a settings file.
        ("POST", "/api/disposition") => {
            let requested = string_field(&body, "disposition").unwrap_or_default();
            match disposition(&requested) {
                None => json_response(
                    400,
                    &json!({
                        "error": "unknown disposition",
                        "known": ["default", "solitary", "insatiable", "watchful"],
                    })
                    .to_string(),
                ),
                Some(personality) => {
                    // Only the disposition changes. The creature keeps its
                    // vitals, its stage, its history and everything it has
                    // learned — it becomes a *different* creature, not a fresh
                    // one, and the address is what says so.
                    let before = pet.address();
                    pet.personality = personality;
                    save_pet(store_dir, pet);
                    json_response(
                        200,
                        &json!({
                            "disposition": requested,
                            "address_before": before,
                            "address_after": pet.address(),
                            "same_creature": before == pet.address(),
                            "drives": pet.drives().iter()
                                .map(|d| json!({ "care": d.care.label(), "when": d.when }))
                                .collect::<Vec<Value>>(),
                        })
                        .to_string(),
                    )
                }
            }
        }
        ("GET", "/api/wants") => {
            let want = pet
                .want()
                .map(|(care, field)| json!({ "care": care.label(), "drawn_by": field }));
            let draws: Vec<Value> = pet
                .drives()
                .into_iter()
                .map(|d| json!({ "care": d.care.label(), "when": d.when }))
                .collect();
            let readings: Vec<Value> = pet
                .vitals
                .readings()
                .into_iter()
                .map(|(name, reading)| json!({ "sensor": name, "reads": reading }))
                .collect();
            json_response(
                200,
                &json!({
                    "wants": want,
                    "draws": draws,
                    "readings": readings,
                    "sensors": unia::camaduci::Vitals::sensors()
                        .into_iter()
                        .map(|s| json!({
                            "name": s.name, "when": s.when, "reads": s.reads,
                            "sense": s.sense.label(),
                        }))
                        .collect::<Vec<Value>>(),
                })
                .to_string(),
            )
        }
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
        // How much of the acting was the creature's own. The number behind the
        // word *civilisation*, and deliberately reported as three counts rather
        // than one ratio: a log where the creature took half its acts and a log
        // where it took none both read as "half", and only one of them means
        // there is nothing self-directed in it.
        ("GET", "/api/authorship") => {
            let a = store.authorship();
            json_response(
                200,
                &json!({
                    "player": a.player,
                    "itself": a.itself,
                    "caller": a.caller,
                    // Null rather than 0.0 when no creature was involved at all.
                    "self_directed": a.self_directed(),
                })
                .to_string(),
            )
        }
        ("POST", "/api/care") => match resolve_care(&body, pet) {
            Ok(care) => {
                let result = pet.tend(care);
                match result {
                    Some(primitives) => {
                        // The first arrival starts the clock.
                        *started = true;
                        *now = now_secs();
                        // The player's own wording is recorded when they supply
                        // it, because the evidence gate counts distinct
                        // phrasings rather than occurrences. A client that always
                        // sends "feed the ca maduci" can never clear the gate no
                        // matter how many times it feeds, which is the gate
                        // working as specified rather than a defect.
                        // A person chose this one.
                        let t = trace_for(
                            &pet.id,
                            care,
                            &primitives,
                            *now,
                            parse_intent(&body),
                            Actor::Player,
                        );
                        // A failed write must not be reported as a recorded
                        // interaction: the pet really was cared for, but the
                        // training signal did not get it, and saying otherwise
                        // would overstate the evidence.
                        match store.record(t) {
                            Ok(()) => {
                                pet.learn(learned_rules(store));
                                save_pet(store_dir, pet);
                                json_response(200, &state(pet))
                            }
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
            // The three outcomes are reported as themselves. A player who has
            // been refused is told what would have worked, and a player whose
            // wording simply is not one the creature has heard is not told the
            // same thing as a player who asked for something impossible.
            Err(understanding) => {
                // A contradiction carries its question with it, because the
                // question is the whole of the response. Returning the outcome
                // alone would make the player reconstruct what it meant, and a
                // refusal that does not say what would work is the same as no
                // answer.
                let mut body = serde_json::to_value(&understanding).unwrap_or_default();
                if let (Some(why), Some(obj)) = (understanding.why(), body.as_object_mut()) {
                    obj.insert("why".to_string(), serde_json::json!(why));
                }
                json_response(409, &body.to_string())
            }
        },
        _ => json_response(404, r#"{"error":"not found"}"#),
    };

    stream.write_all(response.as_bytes())?;
    stream.flush()
}

/// Works out which care operation a request is asking for.
///
/// The client's own `care` field wins when it is one of the four wire words,
/// because that is an explicit instruction. Otherwise the player's free text is
/// resolved by the creature itself, through the phrasings it has actually been
/// fed. That is the whole difference between a four-word protocol and a
/// vocabulary: the list of ways to say "feed" is not written down next to the
/// parser, it is what the creature learned by being fed.
fn resolve_care(body: &str, pet: &Pet) -> Result<Care, Understanding> {
    if let Some(care) = string_field(body, "care").and_then(|v| Care::parse(&v)) {
        return Ok(care);
    }
    let spoken = string_field(body, "intent")
        .or_else(|| string_field(body, "care"))
        .unwrap_or_default();
    pet.understand(&spoken)
        .acts()
        .ok_or_else(|| pet.understand(&spoken))
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
#[allow(clippy::too_many_arguments)]
fn trace_for(
    pet_id: &str,
    care: Care,
    primitives: &[String],
    now: u64,
    intent: Option<String>,
    actor: Actor,
) -> Trace {
    Trace {
        ts: now,
        intent: intent.unwrap_or_else(|| format!("{} the ca maduci", care.label())),
        resource_id: Some(pet_id.to_string()),
        outcome: "hit".to_string(),
        tokens_in: 0,
        tokens_out: 0,
        primitives: primitives.to_vec(),
        actor: Some(actor),
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
        r#"{{"id":{},"address":{},"stage":{},"age_ticks":{},"hunger":{:.3},"happiness":{:.3},"health":{:.3},"mood":{},"cuante":{:.3},"nuante":{:.2},"endurance":{:.1},"posture":{},"quarantined":{},"primitives":{},"learned":{},"summary":{}}}"#,
        json_str(&pet.id),
        // The creature's content address: what it *is*, with nothing it has
        // learned and nothing it has done. Two players holding this creature
        // agree on this, and it does not move when the player learns new words
        // for it.
        json_str(&pet.address()),
        json_str(pet.stage.label()),
        pet.vitals.age_ticks,
        pet.vitals.hunger,
        pet.vitals.happiness,
        pet.vitals.health,
        json_str(&pet.vitals.mood()),
        // The economy. `cuante` is the power it acts at and `nuante` the
        // resources it spends; `endurance` is the number that makes the two
        // comparable at all, being a duration rather than an amount and shorter
        // for a weaker creature.
        pet.vitals.economy.cuante,
        pet.vitals.economy.nuante,
        pet.vitals.economy.endurance(),
        json_str(pet.vitals.economy.posture()),
        pet.quarantined,
        // The spike count is what the creature has *learned*, not what it was
        // fed. Those used to be the same number, which meant the most expressive
        // channel on the creature saturated after four distinct actions.
        pet.learned.len(),
        learned_json(&pet.learned),
        json_str(&pet.summary()),
    )
}

/// Serialises the creature's learned rules, strongest first.
fn learned_json(rules: &[LearnedRule]) -> String {
    let body: Vec<String> = rules
        .iter()
        .map(|r| {
            let aliases: Vec<String> = r.aliases.iter().map(|a| json_str(a)).collect();
            format!(
                "{{\"address\":{},\"signature\":{},\"aliases\":[{}],\"confidence\":{:.4},\"observations\":{}}}",
                json_str(&r.address),
                json_str(&r.signature),
                aliases.join(","),
                r.confidence,
                r.observations
            )
        })
        .collect();
    format!("[{}]", body.join(","))
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

/// The dispositions the creature can be given, as declared data.
///
/// Written as personalities rather than as flags, because the point of the
/// exercise is that a disposition is *structure*: it changes the creature's
/// content address. Three of the four are deliberately unlike the shipped one so
/// that the change is visible in behaviour and not only in a hash.
fn disposition(name: &str) -> Option<Personality> {
    let mut p = Personality::default();
    match name {
        // Never wants feeding: the same creature with one appetite removed.
        "solitary" => {
            p.drives.retain(|d| d.care != Care::Feed);
        }
        // Notices nothing at all: a creature that cannot perceive its own state,
        // and therefore never knows it is hungry however much it wants feeding.
        // The one disposition where perception and appetite disagree, which is the
        // most interesting of the four to watch.
        "watchful" => {
            p.sensors.clear();
        }
        // Wants feeding far sooner, and says so far sooner.
        "insatiable" => {
            p.drives = vec![
                Drive {
                    care: Care::Feed,
                    when: "hunger > 0.05".to_string(),
                },
                Drive {
                    care: Care::Feed,
                    when: "hunger > 0.2".to_string(),
                },
                Drive {
                    care: Care::Sleep,
                    when: "age_ticks > 0".to_string(),
                },
            ];
            p.sensors = vec![Sensor {
                name: "hunger".to_string(),
                when: "hunger > 0.2".to_string(),
                reads: "ravenous".to_string(),
                sense: unia::camaduci::Sense::Derived,
            }];
        }
        "default" => {}
        _ => return None,
    }
    Some(p)
}
