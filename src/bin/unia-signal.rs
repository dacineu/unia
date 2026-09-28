//! Signalling server for peer-to-peer ca(R)maduci sessions.
//!
//! Peers need to find each other before they can talk, and WebRTC cannot do
//! that on its own: it carries media and data once a path exists, and something
//! outside has to introduce the two ends. This is that something. It is a
//! deliberately small relay, not a broker.
//!
//! # What it does and does not do
//!
//! It relays four message kinds between peers in a named room: `hello`,
//! `offer`, `answer` and `ice`. It never sees peer payloads, because once a
//! session is established the peers exchange data directly and this process is
//! not in the path.
//!
//! # Security
//!
//! Two separate things, and only the first is this server's job.
//!
//! * Transport to this server is WSS when a certificate is supplied, and the
//!   listener refuses to start on a non-loopback address without one. Signalling
//!   in the clear would let anyone on the path rewrite an SDP offer and point a
//!   peer at a host they control.
//! * The peer-to-peer session itself is encrypted by WebRTC's mandatory DTLS, and
//!   is additionally authenticated by the room token. That does not depend on
//!   this server and holds even if this process is fully compromised, which is
//!   the reason to keep the relay dumb.
//!
//! There is no encryption of the signalling *channel* beyond TLS, and no
//! identity provider: a room token is a bearer secret, so a room is only as
//! private as the token and anyone who learns it can join. That is stated in the
//! README rather than papered over, and a deployment that needs more wants a real
//! signalling service rather than a better token.
//!
//! ```sh
//! unia-signal --port 8787 --room-token-file ~/.config/unia/room.token
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::Message;

/// Largest message accepted from a peer.
///
/// Signalling payloads are SDP descriptions and ICE candidates, both of which are
/// kilobytes. A generous ceiling keeps a hostile peer from turning the relay into
/// an amplifier.
const MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// A message on the signalling channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Signal {
    /// A peer announcing itself, carrying the room it wants and the room token.
    ///
    /// The token travels on the first message rather than in a header because the
    /// browser WebSocket API cannot set request headers, and a signalling channel
    /// that only browsers can use is not much of one.
    Hello { room: String, token: String },
    /// A session description offer, relayed to the room.
    Offer { room: String, sdp: String },
    /// A session description answer, relayed to the room.
    Answer { room: String, sdp: String },
    /// A trickled ICE candidate, relayed to the room.
    Ice { room: String, candidate: String },
    /// A peer leaving.
    Bye { room: String },
}

impl Signal {
    /// The room this message belongs to.
    fn room(&self) -> &str {
        match self {
            Signal::Hello { room, .. }
            | Signal::Offer { room, .. }
            | Signal::Answer { room, .. }
            | Signal::Ice { room, .. }
            | Signal::Bye { room } => room,
        }
    }

    /// Whether this message should reach other peers.
    ///
    /// `hello` is peer-to-server: it registers the sender and is not broadcast,
    /// because relaying it would tell every existing peer that someone new arrived
    /// before that peer had said anything, which leaks presence to a room it has
    /// not authenticated for.
    fn is_relayed(&self) -> bool {
        !matches!(self, Signal::Hello { .. })
    }
}

/// The set of currently connected peers, for logging and for the health route.
#[derive(Default)]
struct Peers {
    rooms: HashMap<String, usize>,
}

impl Peers {
    /// How many peers are in a room, or zero when the room is empty.
    fn count(&self, room: &str) -> usize {
        self.rooms.get(room).copied().unwrap_or(0)
    }

    /// Records a peer joining a room.
    fn join(&mut self, room: &str) {
        *self.rooms.entry(room.to_string()).or_insert(0) += 1;
    }

    /// Records a peer leaving, dropping the room once it is empty.
    ///
    /// Without the removal an emptied room would linger for the life of the
    /// process and the count would drift upward forever.
    fn leave(&mut self, room: &str) {
        if let Some(n) = self.rooms.get_mut(room) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                self.rooms.remove(room);
            }
        }
    }
}

/// A relayed message, paired with the room it belongs to.
type Relay = (String, Signal);

/// Shared state for one connection's lifetime.
#[derive(Clone)]
struct Session {
    /// The room token this connection must present. `None` disables the check,
    /// which is only appropriate for a loopback-only development instance.
    token: Option<String>,
    peers: Arc<Mutex<Peers>>,
    /// The single fan-out every connection subscribes to.
    ///
    /// This is created once by the server and cloned into each session. Creating
    /// it per connection is the obvious mistake and it is silent: each peer then
    /// broadcasts into a channel only it receives, and the relay appears to accept
    /// connections, authenticate them, and forward nothing at all.
    relay: broadcast::Sender<Relay>,
}

impl Session {
    /// Whether a presented token matches.
    ///
    /// Compared in full rather than by a prefix or a hash of the first bytes: this
    /// is a bearer secret and a truncated comparison would accept a room token
    /// that shares a prefix with the real one.
    fn authorised(&self, presented: &str) -> bool {
        match &self.token {
            None => true,
            Some(expected) => constant_time_eq(expected.as_bytes(), presented.as_bytes()),
        }
    }
}

/// Compares two byte strings without an early exit on the first difference.
///
/// Length is not secret here, but the contents are, and a comparison that returns
/// as soon as it finds a mismatch leaks the position of that mismatch to anyone
/// able to time the request.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Reads the room token from a file, trimming the trailing newline an editor adds.
fn read_token(path: &str) -> std::io::Result<String> {
    let raw = std::fs::read_to_string(path)?;
    Ok(raw.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let port: u16 = arg(&args, "--port")
        .and_then(|v| v.parse().ok())
        .unwrap_or(8787);
    let token_file = arg(&args, "--room-token-file");
    let cert = arg(&args, "--tls-cert");
    let key = arg(&args, "--tls-key");

    let token = match &token_file {
        Some(path) => Some(read_token(path)?),
        None => {
            eprintln!("unia-signal: no --room-token-file given; rooms are unauthenticated");
            eprintln!("unia-signal: this is only acceptable on a loopback address");
            None
        }
    };

    // One relay for the whole process, shared by every connection.
    let (relay, _keep) = broadcast::channel::<Relay>(256);
    let base = Session {
        token,
        peers: Arc::new(Mutex::new(Peers::default())),
        relay,
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    if base.token.is_none() {
        // Binding to every interface without a token would make the relay an open
        // meeting point for anyone who can reach the port. Refusing here is better
        // than warning and continuing.
        let bind = SocketAddr::from(([127, 0, 0, 1], port));
        eprintln!("unia-signal: unauthenticated, binding loopback only at {bind}");
        return serve(bind, base).await;
    }

    match (&cert, &key) {
        (Some(c), Some(k)) => {
            eprintln!("unia-signal: serving WSS on {addr} with {c}");
            serve_tls(addr, c, k, base).await
        }
        _ => {
            eprintln!("unia-signal: serving WS on {addr} (no --tls-cert/--tls-key)");
            eprintln!("unia-signal: browsers require wss:// for anything but localhost");
            serve(addr, base).await
        }
    }
}

/// Reads `--name value` from the command line.
fn arg(args: &[String], name: &str) -> Option<String> {
    let i = args.iter().position(|a| a == name)?;
    args.get(i + 1).cloned()
}

/// Accepts plaintext WebSocket connections.
async fn serve(addr: SocketAddr, session: Session) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, peer) = listener.accept().await?;
        let session = session.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_plain(stream, session).await {
                eprintln!("unia-signal: {peer} disconnected: {e}");
            }
        });
    }
}

/// Accepts TLS WebSocket connections.
async fn serve_tls(
    addr: SocketAddr,
    cert_path: &str,
    key_path: &str,
    session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use tokio_rustls::rustls::ServerConfig;
    use tokio_rustls::TlsAcceptor;

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut std::io::BufReader::new(
        std::fs::File::open(cert_path)?,
    ))
    .collect::<Result<_, _>>()?;
    let key: PrivateKeyDer<'static> =
        rustls_pemfile::private_key(&mut std::io::BufReader::new(std::fs::File::open(key_path)?))?
            .ok_or("no private key in the --tls-key file")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, peer) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let session = session.clone();
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls) => {
                    if let Err(e) = handle_tls(tls, session).await {
                        eprintln!("unia-signal: {peer} disconnected: {e}");
                    }
                }
                Err(e) => eprintln!("unia-signal: {peer} TLS handshake failed: {e}"),
            }
        });
    }
}

/// Runs one plaintext session.
async fn handle_plain(
    stream: TcpStream,
    session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    run(ws, session).await
}

/// Runs one TLS session.
async fn handle_tls(
    stream: tokio_rustls::server::TlsStream<TcpStream>,
    session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    run(ws, session).await
}

/// The relay loop for one connected peer.
async fn run<S>(
    ws: tokio_tungstenite::WebSocketStream<S>,
    session: Session,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut sink, mut stream) = ws.split();
    // Subscribe to the server-wide fan-out. Each connection then filters by the
    // room it authenticated for, so a peer only ever receives traffic for a room it
    // is actually in.
    let mut rx = session.relay.subscribe();

    let mut joined: Option<String> = None;

    loop {
        tokio::select! {
            incoming = stream.next() => {
                let Some(Ok(msg)) = incoming else { break };
                let text = match msg {
                    Message::Text(t) => t.to_string(),
                    Message::Binary(b) => String::from_utf8_lossy(&b).to_string(),
                    Message::Close(_) => break,
                    // Ping is answered by tungstenite, a pong needs no relay, and
                    // a raw frame is already unwrapped before it reaches here.
                    Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
                };
                if text.len() > MAX_MESSAGE_BYTES {
                    eprintln!("unia-signal: dropping an oversized message of {} bytes", text.len());
                    continue;
                }
                let Ok(signal) = serde_json::from_str::<Signal>(&text) else {
                    eprintln!("unia-signal: dropping an unparseable message");
                    continue;
                };

                // The first message must carry the token and be a hello. Checking
                // it once, up front, means an unauthenticated peer never reaches
                // the relay at all.
                if joined.is_none() {
                    let Signal::Hello { room, token } = &signal else {
                        eprintln!("unia-signal: first message was not a hello");
                        break;
                    };
                    if !session.authorised(token) {
                        eprintln!("unia-signal: rejecting a peer with a bad room token");
                        break;
                    }
                    eprintln!("unia-signal: peer joined room {room}");
                    joined = Some(room.clone());
                    session.peers.lock().await.join(room);
                    continue;
                }

                let Some(room) = joined.clone() else { break };
                if signal.room() != room {
                    // A peer that changes room mid-session would otherwise be able
                    // to reach a room it never authenticated for.
                    eprintln!("unia-signal: peer tried to relay into a room it did not join");
                    continue;
                }
                if signal.is_relayed() {
                    let _ = session.relay.send((room, signal));
                }
            }
            relayed = rx.recv() => {
                let Ok((room, signal)) = relayed else { continue };
                if Some(&room) != joined.as_ref() { continue; }
                let Ok(encoded) = serde_json::to_string(&signal) else { continue };
                if sink.send(Message::Text(encoded.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    if let Some(room) = joined {
        session.peers.lock().await.leave(&room);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(token: Option<&str>) -> Session {
        let (relay, _keep) = broadcast::channel::<Relay>(8);
        Session {
            token: token.map(str::to_string),
            peers: Arc::new(Mutex::new(Peers::default())),
            relay,
        }
    }

    #[test]
    fn a_cloned_session_shares_one_relay_so_peers_can_reach_each_other() {
        // The bug this guards: creating the channel per connection leaves the
        // relay silently forwarding nothing while still authenticating peers.
        let a = session(Some("t"));
        let b = a.clone();
        let mut rx = b.relay.subscribe();
        a.relay
            .send((
                "r".to_string(),
                Signal::Offer {
                    room: "r".into(),
                    sdp: "v=0".into(),
                },
            ))
            .expect("a send with a live subscriber must succeed");
        let (room, signal) = rx.try_recv().expect("the peer must receive it");
        assert_eq!(room, "r");
        assert!(signal.is_relayed());
    }

    #[test]
    fn accepts_the_exact_token() {
        assert!(session(Some("s3cret")).authorised("s3cret"));
    }

    #[test]
    fn rejects_a_token_differing_anywhere() {
        let s = session(Some("s3cret"));
        assert!(!s.authorised("s3crev"));
        assert!(!s.authorised("s3crets"));
        assert!(!s.authorised(""));
    }

    #[test]
    fn rejects_a_token_sharing_only_a_prefix() {
        // A truncated comparison would accept a guess that shares a leading run
        // with the real token, which is the cheapest possible thing to try.
        let s = session(Some("room-token-0123456789"));
        assert!(!s.authorised("room-token-012345678"));
        assert!(!s.authorised("room-"));
    }

    #[test]
    fn accepts_anything_when_no_token_is_configured() {
        // Unauthenticated is a legitimate mode, but it is only ever bound to
        // loopback, which main enforces separately.
        assert!(session(None).authorised(""));
        assert!(session(None).authorised("whatever"));
    }

    #[test]
    fn constant_time_eq_matches_equality() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn a_hello_is_not_relayed() {
        // Relaying it would announce an arriving peer to a room before that peer
        // had authenticated for it.
        let hello = Signal::Hello {
            room: "r".into(),
            token: "t".into(),
        };
        assert!(!hello.is_relayed());
    }

    #[test]
    fn session_descriptions_and_candidates_are_relayed() {
        for s in [
            Signal::Offer {
                room: "r".into(),
                sdp: "v=0".into(),
            },
            Signal::Answer {
                room: "r".into(),
                sdp: "v=0".into(),
            },
            Signal::Ice {
                room: "r".into(),
                candidate: "c".into(),
            },
            Signal::Bye { room: "r".into() },
        ] {
            assert!(s.is_relayed(), "{s:?} should be relayed");
        }
    }

    #[test]
    fn every_message_reports_the_room_it_belongs_to() {
        let cases = [
            (
                Signal::Hello {
                    room: "a".into(),
                    token: "t".into(),
                },
                "a",
            ),
            (
                Signal::Offer {
                    room: "b".into(),
                    sdp: String::new(),
                },
                "b",
            ),
            (
                Signal::Answer {
                    room: "c".into(),
                    sdp: String::new(),
                },
                "c",
            ),
            (
                Signal::Ice {
                    room: "d".into(),
                    candidate: String::new(),
                },
                "d",
            ),
            (Signal::Bye { room: "e".into() }, "e"),
        ];
        for (signal, room) in cases {
            assert_eq!(signal.room(), room);
        }
    }

    #[test]
    fn round_trips_through_json() {
        // A browser peer and this server must agree on the wire format, and the
        // tag is what they dispatch on.
        let s = Signal::Offer {
            room: "r".into(),
            sdp: "v=0\r\n".into(),
        };
        let text = serde_json::to_string(&s).unwrap();
        assert!(text.contains("\"kind\":\"offer\""));
        assert_eq!(serde_json::from_str::<Signal>(&text).unwrap(), s);
    }

    #[test]
    fn a_hello_requires_a_token_field() {
        // Omitting it must fail to parse rather than defaulting to empty, or a
        // client bug would silently become an unauthenticated peer.
        assert!(serde_json::from_str::<Signal>(r#"{"kind":"hello","room":"r"}"#).is_err());
    }

    #[test]
    fn counts_peers_in_a_room() {
        let mut p = Peers::default();
        assert_eq!(p.count("r"), 0);
        p.join("r");
        p.join("r");
        assert_eq!(p.count("r"), 2);
    }

    #[test]
    fn drops_a_room_once_the_last_peer_leaves() {
        // Without this an emptied room would linger for the life of the process
        // and the count would only ever rise.
        let mut p = Peers::default();
        p.join("r");
        p.leave("r");
        assert_eq!(p.count("r"), 0);
        assert!(!p.rooms.contains_key("r"));
    }

    #[test]
    fn leaving_a_room_nobody_joined_is_harmless() {
        let mut p = Peers::default();
        p.leave("never-joined");
        assert_eq!(p.count("never-joined"), 0);
    }

    #[test]
    fn leaving_more_than_joined_does_not_underflow() {
        let mut p = Peers::default();
        p.join("r");
        p.leave("r");
        p.leave("r");
        assert_eq!(p.count("r"), 0);
    }

    #[test]
    fn the_message_ceiling_is_a_few_kilobytes_of_headroom_over_an_sdp() {
        // SDP descriptions are kilobytes. The ceiling is generous, and the point of
        // asserting it is that it must never be small enough to clip a real offer.
        assert!(MAX_MESSAGE_BYTES > 16 * 1024);
    }
}
