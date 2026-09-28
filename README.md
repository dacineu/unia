<div align="center">

# unia

**Identity is what a thing does, not how it is worded.**

[![CI](https://github.com/dacineu/unia/actions/workflows/ci.yml/badge.svg)](https://github.com/dacineu/unia/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/unia.svg)](https://crates.io/crates/unia)
[![docs.rs](https://img.shields.io/docsrs/unia.svg)](https://docs.rs/unia)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![DCO](https://img.shields.io/badge/DCO-required-blue.svg)](./DCO)

[`docs/SPEC.md`](./docs/SPEC.md) ·
[the paper](./docs/LARGE-PATTERN-MODELS.md) ·
[the game](./docs/camaduci-game.md) ·
[commercial and support](./LICENSE-COMMERCIAL.md) ·
[contributing](./CONTRIBUTING.md) ·
[security](./SECURITY.md)

</div>

---

## The one idea

An agent that can do one useful thing is usually welded to the machine that
thing lives on. It shells out to `/usr/bin/grep`, expects a CUDA device, assumes
a POSIX filesystem. Move it to a browser or another OS and you rewrite the
integration, so the capability and the environment can never evolve apart.

unia separates **what** is wanted from **how** it is served. A capability is
declared as a `.ure` resource. A **Primitive Bridge** maps intent onto a small
stable vocabulary of primitives. An **Actuator Nucleus** satisfies them against
a real resource, which may be a shell command, a Wasm module, or a GPU kernel.

```json
{
  "resource_id": "valve-001",
  "category": "actuator",
  "state_space": { "flow_rate": { "type": "float", "range": [0.0, 1.0] } },
  "action_primitives": [
    {
      "id": "emergency_shutdown",
      "aliases": ["emergency_shutdown", "opriți supapa"],
      "params": {},
      "target_state": "flow_rate = 0.0",
      "constraints": ["status != 'fault'"]
    }
  ]
}
```

The claim that matters is narrower than "orchestration system", and it is the
part the rest of this repository is trying to earn:

> **Prose must never determine identity.** The same capability, declared once and
> worded differently, is one artifact with one address — not a sibling that
> happens to look similar.

That single rule is what lets two people who share no language converge, and it
is enforced by hashing a *skeleton* of the manifest with the surface fields
(`resource_id`, `guidance`, `description`, `aliases`) removed. Measured before
the change: the same valve in English and in Romanian were two different
artifacts. Measured after: one address.

The surface lives beside the identity, in a `Lexicon` — address → language →
phrasings — so a rule learned in one language extends the rule you already have
instead of quietly creating a near-duplicate.

## The research question

The paper is [`docs/LARGE-PATTERN-MODELS.md`](./docs/LARGE-PATTERN-MODELS.md).
In one paragraph:

Software reuses *artifacts*. Machine learning reuses *weights*. We are
interested in a third unit: a **large pattern** — an executable artifact that can
be recognised in a language you do not speak, addressed by what it does, and
handed to another party without either of you agreeing on a shared human
language. The reuse rate we optimise for is **escalation rate**: how much
capability a corpus gains per artifact added, rather than how much a single
artifact can do.

**The model is a transduction layer, not the interface.** The interface is the
sequence of primitives. A language model helps *call* an act; it is never what
identifies it. Today it can be removed entirely, and the vocabulary-free channel
still works — that is the design target, and it is not yet the default path.

The paper has a section called **"What is not demonstrated"** and it is
load-bearing. Read it before quoting anything from this project.

## ca(R)maduci

The smallest thing that exercises the whole loop is a pet.

It has a hunger, a happiness and a health. It never asks for them. It gets
worse quietly, at a rate you will not notice until you have already lost the
thread of it. You feed it, play with it, clean it, and eventually put it to
sleep — **and sleep is the only thing that ages it.** Feed a creature for a week
and it is the same egg.

```
tended:                             neglected:
  stage       adult                   stage       egg
  age ticks   6                      age ticks   0
  vitals      hunger 0.05            vitals      hunger 1.00
               happiness 1.00                     happiness 0.00
               health 1.00                        health 0.00
  mood        content                 mood        gone
  lifecycle   served                 lifecycle   quarantined
```

Both received the same number of ticks. One has a trace log; the other died and
was **quarantined rather than deleted**, because death is a lifecycle transition
and the traces that caused it are still evidence.

There is no antagonist. **There is apathy.** A creature nobody fed does not
announce itself: no alert, no badge, one notch of drift at a time. That is
deliberate, and it is the property the whole system is built to detect — a
service that is quietly getting worse is indistinguishable from one that is
idle.

The game is named for its own question:

> *Let's find together the answer to the ever question: what came first, the egg
> or the chicken?*

One creature cannot answer it, because within one creature the egg obviously
comes first. The interesting answer needs two. If two players tend their
creatures with the same words, the creatures learn the *same act* — and since
identity **is** the act, they are not two creatures that happened to agree. They
are one act arrived at twice. When they meet, the system does not say who came
first. It says **neither**, and hands over the address both reached independently.
When two creatures never match, it says **undetermined**. It will not invent an
ancestor to give you a satisfying answer.

There is no host, address, port or position anywhere in the convergence rule.
Two creatures on opposite sides of the world with the same capabilities converge
exactly as two on one machine do. Two neighbours with different capabilities do
not. **Proximity is not intimacy.**

The full design, including the revisions this project forced on it, is in
[`docs/camaduci-game.md`](./docs/camaduci-game.md). The mechanics are in
[`docs/camaduci.md`](./docs/camaduci.md), the peer relay in
[`docs/signalling.md`](./docs/signalling.md).

```sh
cargo run --example camaduci                       # terminal, deterministic
cargo run --bin unia-camaduci -- --port 7731       # playable in a browser
```

## What is measured, and what is not

This is the section to read before forming an opinion.

Reproduce any of it with `cargo run --features mcp-server --bin unia-mcp -- eval`
and `... -- topology`.

| Claim | Measurement |
| --- | --- |
| Retrieval hit@1 on the 20-case fixture | **20/20 (100%)** |
| False negatives | **0** |
| False positives | **2** |
| Primitive vocabulary reachable by the resolver | **16 of 16**, and it returns an error otherwise |
| Tests | **382** |
| Corpus, hand-written | **12 patterns** on a clean tree, 0 edges, **0 denominators** |
| Corpus, generated | **36 artifacts** → **6 denominators**, each reached by 6 artifacts |
| Phrasings per denominator | **6 to 24** — the reach one identity acquired |

Two of those rows are the interesting ones.

**The two false positives score 0.2197 and 0.2236 against a weakest true
positive of 0.2041.** No threshold separates them. They need a capability check
against the declared state space, not a better score — which is why the honest
answer here is "not fixed" and not "tune the constant".

**The hand-written corpus has 12 artifacts and zero shared capabilities.** So
there is no escalation rate to report from it, and this repository does not
report one from it. What *is* measurable is the machinery, on a generated corpus
that shares primitives on purpose: 36 artifacts over 6 profiles converge onto 6
denominators, each reached independently by 6 artifacts carrying between 6 and 24
distinct phrasings. That is the shape of the claim — *one act, many ways to ask,
independently arrived at* — measured on a fiction. `cargo run --bin unia-corpus`
reproduces it, and every generated manifest says in its own `guidance` field that
it is synthetic. The gather
machinery converges on common denominators; on this corpus it has 12
participants and finds nothing, which is a real measurement and is printed as
one. Everything downstream of that — the research claim, the paper's central
figure — is waiting on a corpus with shared primitives in it. Building one is
the next piece of work, not a footnote.

**The corpus is the same size for you as it is here.** That was not true until
this restructure. Synthesis used to write manifests to the process working
directory, so running the tests added to the corpus: 63 locally against 12 in a
fresh clone, on the same commit, with the retrieval numbers unchanged at 20/20.
The two sides of resolution agreed only because both happened to be the working
directory. Synthesis output now has one home, `.unia/out`, and both the
transducer and the registry default to it. **If you measure 14 rather than 12,
you have run the tests, and the two extra are synthesized mirrors.** See
[`docs/SPEC.md`](./docs/SPEC.md) §6.

**What this project does not have yet**, stated plainly:

- **A creature cannot learn a new power.** It can be kept, it persists across
  restarts, and it induces its own routines. It cannot be taught to reach
  anything it was not built to reach — not a file, not a socket, not another
  creature. The limbs are missing, and this blocks meeting, handover, and the
  whole primitive protocol.
- **Nothing is translated.** A phrasing exists because somebody supplied it. No
  code infers that one word is another, on purpose: an inferred synonym that
  turns out to be an antonym is worse than a missing one. Measured: `inchide
  supapa` scores 0.850 while `open the valve` scores 0.289 against the same
  artifact, and a corpus with one antonym in an alias set is currently
  undetectable. It needs a typed `Opposes` relation, not a flat bag of words.
- **Induced sequences are unaddressable.** A creature can learn
  `SetValue_CheckSense` and never be able to call it, because candidates are
  never written back to `patterns/`.
- **Constraints are declared and not evaluated.** `constraints` is an array of
  free strings. This is the second pillar of the safety argument and it is
  currently a comment.
- **The wasm32 target does not build, and the CI job that checks it is red.** The
  crate depends on `tokio` with `features = ["full"]`, which pulls `mio`, and
  `mio` does not support `wasm32-unknown-unknown`. `src/wasm_core.rs` is behind
  `#[cfg(target_arch = "wasm32")]` and has therefore never been compiled. Fixing
  it means gating `tokio` and the signalling relay's `tokio-tungstenite`,
  `tokio-rustls` and `rustls-pemfile` out of the wasm target, so the library
  would build and the two network binaries would not. The wasm build was removed
  from the quick start rather than left as a command that fails.

## What is implemented

Twenty-eight modules in five layers. This is a working prototype, not a product,
and [`docs/SPEC.md`](./docs/SPEC.md) records every place the code and the prose
still disagree.

| Layer | Modules | Role |
| --- | --- | --- |
| **Declaration** | `identifiers`, `registry`, `lexicon` | skeleton hashing and content addressing, manifest resolution, surface forms |
| **Resolution** | `bridge`, `primitives`, `router`, `slm`, `session` | intent to primitive mapping, semantic scoring, SLM routing, session extraction |
| **Execution** | `nucleus`, `os`, `weights`, `profiler` | packet dispatch, virtual kernel and VFS, hardware abstraction, LoRA memory |
| **Economics** | `wmis` | token metering, permissions, resource quality |
| **Evolution** | `orchestrator`, `pipeline`, `harvester`, `learner`, `transducer`, `evolution`, `fluid`, `meta_actuators`, `induce`, `gather`, `gc` | harvest, induce, converge, transduce, retire |

Cross-cutting: `camaduci` (the creature), `mcp` (Model Context Protocol
connectors, off by default), `node` and `transducer` (the distributed fabric),
and a `wasm32` build via `wasm_core`.

Four binaries: `unia-corpus` (writes a synthetic corpus and reports its convergence),
`unia-mcp` (the corpus over MCP), `unia-camaduci` (the playable
server, no dependencies), `unia-signal` (the peer-to-peer relay, running as a
systemd user service).

## Quick start

```sh
git clone https://github.com/dacineu/unia
cd unia
cargo test                                          # 382 tests
cargo run --example camaduci                        # start here
```

Requires a stable Rust toolchain, 1.75 or later.

```sh
cargo check --all-targets

# the corpus, over MCP
cargo run --features mcp-server --bin unia-mcp -- search "purge the temp storage"
cargo run --features mcp-server --bin unia-mcp -- eval
cargo run --features mcp-server --bin unia-mcp -- topology

# the creature, playable
cargo run --bin unia-camaduci -- --port 7731
cargo run --bin unia-signal -- --port 8787
cargo run --bin unia-corpus -- --out /tmp/corpus   # a synthetic corpus, and what it converges on
```

### Examples

| Example | Shows |
| --- | --- |
| `camaduci` | A creature whose vitals are a declared state space and whose care is recorded as traces. Start here. |
| `full_system_demo` | Provisioning, orchestration, SLM execution and learning, end to end |
| `verify_fs_pipeline` | Bridge to nucleus with economic accounting on each actuation |
| `formal_verification_demo` | Property-based verification of the actuation contract |
| `benchmark_unia` | Decoupled intent mapping against a coupled baseline |
| `universal_demo` | Several resource categories through one pipeline |

## Status

**Pre-1.0. Experimental.** Known limitations are tracked as divergences in
[`docs/SPEC.md`](./docs/SPEC.md) and, in prose, in
[`docs/DISCUSSION-evolution-and-identity.md`](./docs/DISCUSSION-evolution-and-identity.md)
— which also records the claims that were **withdrawn**, and why, including three
that were wrong in this project's own favour. Two of the more expensive lessons:

- Ancestry was modelled as a hash chain. A chain asserts a tree; the data is a
  DAG, and the most informative event is independent convergence. Withdrawn.
- "Convergence makes a growing corpus better" was asserted before the
  denominator was ever served by retrieval. Disproved by the code: the
  denominator is computed and then ignored.

## Documentation

| Document | Contents |
| --- | --- |
| [`docs/SPEC.md`](./docs/SPEC.md) | The normative `.ure` format, version 0.1.0, and its open divergences |
| [`docs/LARGE-PATTERN-MODELS.md`](./docs/LARGE-PATTERN-MODELS.md) | The paper, including §9 *What is not demonstrated* |
| [`docs/DISCUSSION-evolution-and-identity.md`](./docs/DISCUSSION-evolution-and-identity.md) | The study record: what was tried, what was measured, what was retracted |
| [`docs/camaduci-game.md`](./docs/camaduci-game.md) | The game, and the revisions the research forced on it |
| [`docs/creature-as-intermediary.md`](./docs/creature-as-intermediary.md) | The plan for making the creature the intermediary in the request path |
| [`docs/where-reasoning-comes-from.md`](./docs/where-reasoning-comes-from.md) | What the primary sources say reasoning requires, and where this project was wrong |
| [`docs/pattern-and-mattern.md`](./docs/pattern-and-mattern.md) | The vocabulary: what a Pattern and a Mattern are, the quality-diversity lineage, and where the terms collide |
| [`docs/camaduci.md`](./docs/camaduci.md) | Creature mechanics and the prior art they draw on |
| [`docs/signalling.md`](./docs/signalling.md) | The peer-to-peer relay |
| [`docs/whitepaper/whitepaper_unia.tex`](./docs/whitepaper/whitepaper_unia.tex) | LaTeX source of the whitepaper (compiles to 8pp) |
| [`docs/arch-decoupling-strategy.md`](./docs/arch-decoupling-strategy.md) | Why the decoupling layer exists |
| [`docs/PROVENANCE.md`](./docs/PROVENANCE.md) | Publication dates, authorship, prior art |
| [`docs/cognition-log.md`](./docs/cognition-log.md) | Design rationale, including the Patten/Mattern distinction |
| [`docs/history/`](./docs/history/README.md) | Working papers from the predecessor `amater` project, where `.ure` came from |
| [`TODO.md`](./TODO.md) | The live, ranked list of what is not done |

## Using unia

The reference implementation is **MIT licensed**. You may use it commercially,
modify it, and sell products built on it, with the only requirement of preserving
the copyright notice.

MIT does **not** cover the names. `unia`, `unia-OS`, `UPA`, `DU-UUID`, `.ure`
and `ca(R)maduci` are trademarks and are not licensed. See
[`TRADEMARK.md`](./TRADEMARK.md), which records their actual status rather than
an aspiration — nothing here has been filed or cleared yet.

If you want supported, warranted, or bespoke work, a negotiated licence where
MIT does not fit, or permission to use the marks, contact
**<dacineu@proton.me>**. Terms are in
[`LICENSE-COMMERCIAL.md`](./LICENSE-COMMERCIAL.md).

If you are implementing `.ure` independently: **no licence is required**, so
long as you avoid the marks. Tell us and it will be listed here.

## Contributing

Contributions are welcome, and the project is deliberately easy to start in.
Read [`CONTRIBUTING.md`](./CONTRIBUTING.md) first, in particular the
[Developer Certificate of Origin](./DCO) sign-off, which is what keeps the
option of relicensing or selling the project open later. Participation is
governed by the [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).

## Author and attribution

unia is the work of a single author, published under the pseudonym
**iulian dacineu**. The pseudonym appears in the commit history and in
[`LICENSE`](./LICENSE); the MIT licence requires that attribution be preserved.

The pseudonym is deliberate. Please use it in correspondence, and treat
unsolicited approaches involving a legal identity as a security matter under
[`SECURITY.md`](./SECURITY.md).

## Licence

MIT. See [`LICENSE`](./LICENSE).
