# Peer-to-peer signalling for ca(R)maduci

`unia-signal` is the rendezvous point two ca(R)maduci peers use to find each
other. It is a deliberately dumb relay, and keeping it dumb is the point.

## What it does

Four message kinds, relayed between peers in a named room:

| Kind | Direction | Carries |
| --- | --- | --- |
| `hello` | peer → server | room and room token; registers the peer, never relayed |
| `offer` | peer → peer | an SDP offer |
| `answer` | peer → peer | an SDP answer |
| `ice` | peer → peer | a trickled candidate |
| `bye` | peer → peer | departure |

`hello` is deliberately not relayed. Announcing an arriving peer to a room before
it has authenticated would leak presence to peers that have not admitted it.

## What it does not do

**It never sees a peer payload.** Once a session is established the peers exchange
data directly and this process is not in the path. That is the reason to keep it
dumb: an operator who compromises the relay learns who met whom, and nothing
about what they said.

## Security

Two separate properties, and only the first is this server's job.

**Signalling transport.** WSS when `--tls-cert` and `--tls-key` are supplied.
Without a room token the listener refuses to bind anything but loopback, because
an open relay on every interface is a meeting point for anyone who can reach the
port. Signalling in the clear would let anyone on the path rewrite an offer and
point a peer at a host they control.

**Peer-to-peer session.** Encrypted by WebRTC's mandatory DTLS, and
authenticated by the room token. Neither depends on this server, and both hold
even if this process is fully compromised.

**The known weakness, stated rather than hidden.** A room token is a bearer
secret: a room is only as private as the token, and anyone who learns it can join.
There is no identity provider, no revocation, and no rotation without restarting
the service. A deployment that needs more wants a real signalling service rather
than a better token.

The token is compared in full, in constant time, and a token sharing only a prefix
with the real one is rejected. There is a test for exactly that.

## Running it

Installed as a systemd **user** service, so it starts with the session rather than
at boot and needs no root.

```sh
systemctl --user status  unia-signal
systemctl --user restart unia-signal
systemctl --user stop     unia-signal
journalctl --user -u unia-signal -f
```

Unit file: `deploy/unia-signal.service`. It is hardened — `ProtectSystem=strict`,
`PrivateTmp`, `NoNewPrivileges`, `MemoryDenyWriteExecute`, and no write access at
all, because a forwarder needs none.

Generate a token once:

```sh
mkdir -p ~/.config/unia
head -c 32 /dev/urandom | base64 | tr -d '\n=' > ~/.config/unia/room.token
chmod 600 ~/.config/unia/room.token
```

## The bug worth recording

The first working version created the broadcast channel **inside the connection
handler**. Every peer therefore broadcast into a channel only it received, and the
relay accepted connections, authenticated them, joined rooms, tracked counts, and
forwarded nothing at all.

It compiled, passed every unit test, and started cleanly under systemd. It was
only caught by connecting two real WebSocket clients and sending an offer. The
comment above the code described shared behaviour the code did not have, which is
the usual way this class of bug arrives.

There is now a test asserting that a cloned session shares one relay, and the
end-to-end check asserts three things: a wrong token is rejected, an offer reaches
the other peer, and a message naming a room the sender did not join is refused.

## Not built yet

**No WebRTC.** This is signalling only. The peer connection itself needs a
WebRTC stack, and that is a separate decision:

| Crate | Shape | Dependency cost |
| --- | --- | --- |
| `str0m` | Sans-I/O, no shared state, from LiveKit | moderate |
| `webrtc` | async, batteries-included, 1M downloads/month | **~31–55 MB** |

`str0m` is the better fit for this project — it is lower level, has no shared
state, and does not force the whole `unia` process into an async WebRTC runtime.
`webrtc` is the pragmatic choice if SFU-style infrastructure is wanted later.

**Two peers still cannot play.** Convergence in the motto — two lineages reaching
one content address — needs a shared trace store, not a real-time mesh, and the
current server already demonstrates it. WebRTC is for pets *meeting*, which is a
larger question and a separate one.
