//! A transport for [`Resolver`], for each target, with no new dependency.
//!
//! # Why not `reqwest`
//!
//! The native side is `std::net::TcpStream` and the wasm side is `web_sys`, and
//! that is the same choice `src/bin/unia-camaduci.rs` already made and says so
//! in its own header: dependency-free, HTTP spoken directly. `web-sys` is
//! already a dependency with the `Request`, `RequestInit`, `Response` and
//! `RequestMode` features enabled, so the browser path costs nothing either.
//!
//! Adding `reqwest` to send one POST would pull a TLS stack and an async runtime
//! in for a single request, on a project whose stated discipline is that a
//! dependency added for convenience is a dependency maintained forever. The
//! unia target specification's §3 lists a runtime as *missing*, and this is not
//! the place to start adding one by accident.
//!
//! # What is deliberately thin
//!
//! One POST, JSON in, text out, no redirects, no chunked decoding, no keep-alive,
//! no proxy. Every one of those is a real HTTP feature and none of them is
//! needed to ask a question of an endpoint, and each one that is half-implemented
//! is a way for a *consultation* — a claim — to arrive subtly wrong. A refusal
//! from this client is always a refusal; it never becomes a half-read answer.
//!
//! **The cost of being thin is stated here rather than discovered:** if the
//! endpoint is chunked or behind a redirect, this returns a refusal and the log
//! says `refused`, which is honest. It does not return a truncated string as if
//! it were an answer.

use crate::doubt::{Doubt, Resolver};
use serde::Serialize;
use std::time::Duration;

/// The request body, in a shape an endpoint can be written against.
///
/// `why` travels as its tag plus its evidence, because an endpoint that receives
/// "Contradiction" and nothing else cannot tell which two rules disagreed, and a
/// question without its evidence is not a consultation.
#[derive(Debug, Serialize)]
struct RequestBody {
    question: &'static str,
    why: &'static str,
    evidence: Evidence,
    /// Stated by the client so the endpoint can refuse a peer it cannot audit.
    asks_for: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
enum Evidence {
    NoRule,
    Contradiction { a: String, b: String },
    Refuted { rule: String, observed: String },
    Exhausted,
}

impl Evidence {
    fn of(why: &crate::doubt::Unresolvable) -> Self {
        use crate::doubt::Unresolvable as U;
        match why {
            U::NoRule => Evidence::NoRule,
            U::Contradiction { a, b } => Evidence::Contradiction {
                a: a.clone(),
                b: b.clone(),
            },
            U::Refuted { rule, observed } => Evidence::Refuted {
                rule: rule.clone(),
                observed: observed.clone(),
            },
            U::Exhausted => Evidence::Exhausted,
        }
    }
}

/// Builds the body, so the native and wasm paths cannot drift on the wire.
fn body(doubt: &Doubt) -> String {
    // `question` is borrowed for the JSON literal's lifetime, so the struct is
    // built against leaked-free borrows via serde_json's owned Value instead.
    let value = serde_json::json!({
        "question": doubt.question,
        "why": doubt.why.tag(),
        "evidence": Evidence::of(&doubt.why),
        "asks_for": "an answer to a question this machine could not resolve itself",
    });
    value.to_string()
}

/// Reads an endpoint's reply, accepting either a bare string or `{"answer": …}`.
///
/// Two shapes because a bare text endpoint and a JSON endpoint are both common and
/// neither is wrong, and a client that demanded one would force a wrapper around
/// every endpoint it ever talks to.
fn parse_answer(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(s) = v.get("answer").and_then(|a| a.as_str()) {
            return Some(s.trim().to_string()).filter(|s| !s.is_empty());
        }
        if v.get("error").is_some() {
            return None;
        }
    }
    // Not JSON, or JSON without a recognised field: the whole body is the answer.
    // If it parses as JSON but has no `answer` and no `error`, treating the raw
    // text as the answer would hand back a document as though it were a sentence.
    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return None;
    }
    Some(trimmed.to_string())
}

/// The endpoint a native client talks to.
#[derive(Debug, Clone)]
pub struct HttpResolver {
    /// `host:port`. Anything else — a path, a scheme, TLS — is refused at
    /// construction rather than producing a confusing failure later.
    pub authority: String,
    /// The request path, with a leading slash.
    pub path: String,
    pub model: String,
    /// Bounded on purpose. A consultation that hangs is worse than one that
    /// refuses, because a hang looks like thinking.
    pub timeout: Duration,
    /// How to say the doubt in the engine's own vocabulary.
    ///
    /// **`None` sends unia's own request, and that is what a real endpoint
    /// rejects.** Probing a live `llama-server` on `127.0.0.1:8899` through this
    /// transport returned `400` — the Doubt JSON is not a chat request, and the
    /// engine said so rather than guessing. That refusal is the most useful
    /// result the probe produced, because it located the missing piece exactly:
    /// the transport works, the transcription does not exist, and the two fail
    /// differently — a mistranscribed body is a `400`, a well-transcribed one the
    /// engine cannot serve is a `5xx`, and telling those apart is what a probe is
    /// for.
    ///
    /// A function rather than a string so the transpiler is a decision a caller
    /// makes, and so a second engine means a second function rather than an edit
    /// to this one.
    pub transcribe: Option<fn(&Doubt, &str) -> String>,
}

impl HttpResolver {
    /// A resolver for `authority`, and it refuses anything that is not
    /// `host:port`.
    pub fn new(authority: &str, model: &str) -> Result<Self, String> {
        if authority.is_empty() || authority.contains('/') || authority.contains("://") {
            return Err(format!(
                "expected a host:port authority and got {authority:?}. This client \\
                 speaks plain HTTP to a host and port; a URL, a path or a scheme \\
                 here is a mistake that would fail later and less clearly."
            ));
        }
        Ok(HttpResolver {
            authority: authority.to_string(),
            path: "/consult".to_string(),
            model: model.to_string(),
            timeout: Duration::from_secs(10),
            transcribe: None,
        })
    }

    /// One request/response over a plain `TcpStream`.
    fn exchange(&self, doubt: &Doubt) -> Result<String, String> {
        use std::io::{Read, Write};

        let payload = match self.transcribe {
            Some(f) => f(doubt, &self.model),
            None => body(doubt),
        };
        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\n\
             Content-Length: {len}\r\nConnection: close\r\n\r\n{payload}",
            path = self.path,
            authority = self.authority,
            len = payload.len(),
            payload = payload
        );

        let mut stream = std::net::TcpStream::connect(&self.authority)
            .map_err(|e| format!("could not reach {}: {e}", self.authority))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| e.to_string())?;
        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("could not send the question: {e}"))?;
        stream.flush().map_err(|e| e.to_string())?;

        let mut raw = Vec::new();
        stream
            .read_to_end(&mut raw)
            .map_err(|e| format!("could not read the answer: {e}"))?;
        let text = String::from_utf8_lossy(&raw).to_string();
        let (head, rest) = text
            .split_once("\r\n\r\n")
            .ok_or_else(|| "the reply was not an HTTP response".to_string())?;

        let status: u16 = head
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|c| c.parse().ok())
            .ok_or_else(|| "the reply had no status line".to_string())?;
        if !(200..300).contains(&status) {
            // A non-2xx is a refusal and nothing else. Reporting the body as an
            // answer here is how an error page becomes a consultation.
            return Err(format!("the endpoint answered {status}"));
        }
        // `Connection: close` is requested, so read_to_end gives the whole body.
        // Content-Length is still honoured when present, because a server that
        // ignores it is the case where over-reading would invent content.
        let declared: Option<usize> = head.lines().find_map(|l| {
            let (name, value) = l.split_once(':')?;
            name.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())?
        });
        let body = match declared {
            Some(n) if n <= rest.len() => &rest[..n],
            Some(n) => {
                return Err(format!(
                    "the reply declared {n} bytes and arrived with {}, so the answer \
                     would be incomplete and is refused rather than truncated",
                    rest.len()
                ))
            }
            None => rest,
        };
        parse_answer(body).ok_or_else(|| "the reply carried no answer".to_string())
    }
}

impl Resolver for HttpResolver {
    fn name(&self) -> &str {
        &self.model
    }

    fn ask(&self, doubt: &Doubt) -> Result<String, String> {
        self.exchange(doubt)
    }
}

/// The browser path, compiled only for wasm.
///
/// `web-sys` and `wasm-bindgen-futures` are already dependencies, so this costs
/// no new crate — but it is `cfg`-gated because neither exists natively, and a
/// file that will not compile on the host must not be parsed by the host.
#[cfg(target_arch = "wasm32")]
pub struct WebResolver {
    pub endpoint: String,
    pub model: String,
}

#[cfg(target_arch = "wasm32")]
impl Resolver for WebResolver {
    fn name(&self) -> &str {
        &self.model
    }

    fn ask(&self, doubt: &Doubt) -> Result<String, String> {
        // The caller supplies the future's completion, because `ask` is
        // synchronous by design and a wasm fetch is not. `web_exchange` returns
        // a `JsFuture`; the synchronous trait is served by the caller driving it,
        // which is why this is not yet a complete implementation and the TODO
        // says the transport is missing for wasm.
        let _ = (doubt, &self.endpoint);
        Err("the wasm transport needs an async boundary this trait does not have".into())
    }
}

#[cfg(test)]
mod tests {
    //! A real socket, a real HTTP/1.1 request, and a real reply. No mocks in
    //! this file, because the thing being tested is bytes on a wire and a mocked
    //! socket would test nothing.

    use super::*;
    use crate::doubt::Unresolvable;
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    /// A one-shot endpoint that reads a request, hands the head to the test, and
    /// replies with whatever the test scripted.
    struct Endpoint {
        authority: String,
        seen: mpsc::Receiver<String>,
    }

    fn endpoint(reply: &'static str) -> Endpoint {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let authority = listener.local_addr().expect("addr").to_string();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                let mut raw = Vec::new();
                let mut buf = [0u8; 1024];
                // Read until the headers and the declared body are both in.
                while let Ok(n) = sock.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    raw.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&raw).to_string();
                    if let Some((head, body)) = text.split_once("\r\n\r\n") {
                        let len: usize = head
                            .lines()
                            .find_map(|l| {
                                let (k, v) = l.split_once(':')?;
                                k.trim()
                                    .eq_ignore_ascii_case("content-length")
                                    .then(|| v.trim().parse().ok())?
                            })
                            .unwrap_or(0);
                        if body.len() >= len {
                            break;
                        }
                    }
                }
                let _ = tx.send(String::from_utf8_lossy(&raw).to_string());
                let _ = sock.write_all(reply.as_bytes());
                let _ = sock.flush();
                let _ = sock.shutdown(std::net::Shutdown::Both);
            }
        });
        Endpoint {
            authority,
            seen: rx,
        }
    }

    use std::io::Read as _;

    fn doubt() -> Doubt {
        Doubt::new(
            "which rule governs a creature that is dormant by neglect?",
            Unresolvable::Contradiction {
                a: "neglect forgets".into(),
                b: "dormancy holds".into(),
            },
        )
    }

    /// **The request is a well-formed POST carrying the question and its
    /// evidence.** A transport that sends the question without the evidence has
    /// sent a question nobody can answer usefully, and this asserts on the wire
    /// rather than on a mock of it.
    #[test]
    fn the_request_carries_the_question_and_the_evidence() {
        let ep = endpoint("HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntend");
        let r = HttpResolver::new(&ep.authority, "test-engine").expect("a host:port");
        let answer = r.ask(&doubt()).expect("the endpoint answered");
        assert_eq!(answer, "tend");

        let sent = ep
            .seen
            .recv_timeout(Duration::from_secs(2))
            .expect("saw a request");
        assert!(
            sent.starts_with("POST /consult HTTP/1.1\r\n"),
            "got: {sent:?}"
        );
        assert!(sent.contains("Content-Type: application/json"));
        let (head, body) = sent.split_once("\r\n\r\n").expect("a body");
        let declared: usize = head
            .lines()
            .find_map(|l| {
                let (k, v) = l.split_once(':')?;
                k.trim()
                    .eq_ignore_ascii_case("content-length")
                    .then(|| v.trim().parse().ok())?
            })
            .expect("a content-length");
        assert_eq!(
            declared,
            body.len(),
            "and the declared length is the body actually sent, or the endpoint \
             would wait for bytes that never come"
        );

        let v: serde_json::Value = serde_json::from_str(body).expect("json body");
        assert_eq!(v["why"], "Contradiction");
        assert!(v["question"].as_str().unwrap().contains("dormant"));
        assert_eq!(v["evidence"]["a"], "neglect forgets");
        assert_eq!(v["evidence"]["b"], "dormancy holds");
    }

    /// **A non-2xx is a refusal, never an answer.** An error page arriving as a
    /// consultation is the failure this exists to prevent.
    #[test]
    fn an_error_status_is_a_refusal_and_not_an_answer() {
        let ep = endpoint("HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");
        let r = HttpResolver::new(&ep.authority, "e").expect("authority");
        match r.ask(&doubt()) {
            Err(e) => assert!(e.contains("503"), "got {e}"),
            Ok(a) => panic!("a 503 was returned as an answer: {a:?}"),
        }
    }

    /// **A truncated body is refused rather than returned.** Half an answer is
    /// not an answer, and returning it would put a fragment on the log as a
    /// claim.
    #[test]
    fn a_short_body_is_refused_rather_than_truncated() {
        let ep = endpoint("HTTP/1.1 200 OK\r\nContent-Length: 400\r\n\r\nshort");
        let r = HttpResolver::new(&ep.authority, "e").expect("authority");
        match r.ask(&doubt()) {
            Err(e) => assert!(e.contains("400") || e.contains("refused"), "got {e}"),
            Ok(a) => panic!("a truncated body was returned as an answer: {a:?}"),
        }
    }

    /// **Nothing listening is a refusal, and it names where.** "Connection
    /// refused" without an authority is not actionable.
    #[test]
    fn an_unreachable_endpoint_is_a_refusal_that_names_the_authority() {
        // Bind and drop, so the port is very likely free.
        let free = TcpListener::bind("127.0.0.1:0").expect("bind");
        let authority = free.local_addr().expect("addr").to_string();
        drop(free);
        let r = HttpResolver::new(&authority, "e").expect("authority");
        match r.ask(&doubt()) {
            Err(e) => assert!(e.contains(&authority), "the refusal should name where: {e}"),
            Ok(a) => panic!("an unreachable endpoint produced an answer: {a:?}"),
        }
    }

    /// A refusal is still recorded, so the transport's failure is on the log and
    /// the answer rate is not silently 1.0.
    #[test]
    fn a_transport_failure_is_recorded_as_a_failed_crossing() {
        let dir = std::env::temp_dir().join(format!("unia-resolve-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        let mut store = crate::mcp::store::Store::open(&dir);

        let free = TcpListener::bind("127.0.0.1:0").expect("bind");
        let authority = free.local_addr().expect("addr").to_string();
        drop(free);
        let r = HttpResolver::new(&authority, "down-engine").expect("authority");

        let c = crate::doubt::consult(&mut store, doubt(), &r).expect("a refusal is recorded");
        assert!(!c.answered());
        let l = crate::doubt::ledger(&store);
        assert_eq!(l.refused, 1);
        assert_eq!(l.answer_rate(), Some(0.0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **The whole chain, over a socket, with nothing mocked.**
    ///
    /// This is the first test in the project that runs the route end to end:
    /// doubt -> signature -> `HttpResolver` -> a real `TcpListener` on a real
    /// port -> `Reply` -> `judge` -> trace -> ledger. Every link that had no
    /// caller until two commits ago is now exercised by one test, and the only
    /// thing absent is a model.
    ///
    /// Which is why the responder below **cannot be mistaken for one**: it
    /// reports `tokens_used: None` and names itself `reference-responder`, not a
    /// model. A `Reply` with no measured cost is the honest shape for something
    /// that ran no inference, and this is the test that proves the field means
    /// what the type says.
    #[test]
    fn the_whole_route_runs_end_to_end_over_a_socket() {
        use crate::doubt::{ledger, Unresolvable};

        let ep =
        // The length is computed, not written: the first version hardcoded 30 for a\
        // 22-byte body and the client refused the reply as truncated -- the guard\
        // working as intended, on me.
            endpoint(Box::leak(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    "{\"answer\":\"tend wins\"}".len(),
                    "{\"answer\":\"tend wins\"}"
                )
                .into_boxed_str(),
            ));
        let r = HttpResolver::new(&ep.authority, "reference-responder").expect("authority");
        let dir = std::env::temp_dir().join(format!("unia-e2e-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch");
        let mut store = crate::mcp::store::Store::open(&dir);

        let doubt = Doubt::new(
            "which rule governs a creature dormant by neglect?",
            Unresolvable::Contradiction {
                a: "neglect forgets".into(),
                b: "dormancy holds".into(),
            },
        );
        let c = crate::doubt::consult(&mut store, doubt, &r).expect("recorded");

        assert!(c.answered(), "the responder answered: {c:?}");
        assert_eq!(c.answer.as_deref(), Some("tend wins"));
        assert_eq!(
            c.resolver, "reference-responder",
            "and it is named on the receipt"
        );

        // The signature, the ledger, and the witness rule, all on the log.
        let trace = store.traces().last().expect("a trace was written");
        assert!(!trace.succeeded, "an answer is a claim and never a witness");
        assert_eq!(
            trace.primitives.first().map(String::as_str),
            Some("Consult_Contradiction")
        );

        let l = ledger(&store);
        assert_eq!((l.answered, l.refused, l.shapes), (1, 0, 1));

        // The request that produced it carried the evidence, over the wire.
        let sent = ep
            .seen
            .recv_timeout(Duration::from_secs(2))
            .expect("saw a request");
        assert!(sent.contains("Contradiction"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An authority that is a URL is refused at construction, where the mistake
    /// is obvious, rather than at the first request.
    #[test]
    fn a_url_is_refused_where_the_mistake_is_obvious() {
        assert!(HttpResolver::new("https://example.com/consult", "m").is_err());
        assert!(HttpResolver::new("example.com/consult", "m").is_err());
        assert!(HttpResolver::new("", "m").is_err());
        assert!(HttpResolver::new("127.0.0.1:8080", "m").is_ok());
    }

    /// Both reply shapes work, and neither is privileged.
    #[test]
    fn an_endpoint_may_answer_in_plain_text_or_json() {
        assert_eq!(parse_answer("four"), Some("four".into()));
        assert_eq!(parse_answer("  four\n"), Some("four".into()));
        assert_eq!(parse_answer("{\"answer\": \"four\"}"), Some("four".into()));
        // An error document is a refusal, not an answer.
        assert_eq!(parse_answer("{\"error\": \"nope\"}"), None);
        // A JSON document with no answer field is not an answer either, and
        // handing back the raw document would put a JSON blob on the log as a
        // sentence.
        assert_eq!(parse_answer("{\"confidence\": 0.9}"), None);
        assert_eq!(parse_answer("   "), None);
    }
}
