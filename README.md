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

- **A creature can now learn a new power, up to a ceiling that has not moved.**
  The learning loop is closed — propose, witness, promote, with nobody typing
  (`src/loop_train.rs`) — and the promotion is findable, because the champion table
  and its readers now share one key space. **The ceiling is unchanged**: the
  sixteen verbs cannot express a program, `Signature` is a closed sequence, and
  the pet's four acts are `Care` enum variants rather than anything in a manifest.
  It can be taught to reach more of what it was built to reach, which is not the
  same as reaching something new.
- **A creature cannot offer its own capabilities to a peer**, because they are not
  in an artefact it could offer them in. The four acts are `Care` variants in
  `src/camaduci.rs:272`; no `.ure` declares them. The market
  (`broadcast_actuator`, `published`, `discover_resources`) and `Handover`/`receive`
  both exist and neither can carry them. Found by trying to score a fixture: four
  of the eight acts its author had assumed did not exist anywhere.
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
- **~~The wasm32 target does not build, and the CI job that checks it is red.~~ CORRECTED.
  It builds.** The claim was true when written and false now, which is worse than
  being wrong: it is the public face asserting a broken build over a green one.
  `tokio` is `cfg(not(target_arch = "wasm32"))`, the signalling relay is gated out
  of the wasm target, and `cargo check --lib --target wasm32-unknown-unknown`
  passes. All seven CI jobs are green.
- Ancestry was modelled as a hash chain. A chain asserts a tree; the data is a
  DAG, and the most informative event is independent convergence. Withdrawn.
- "Convergence makes a growing corpus better" was asserted before the
  denominator was ever served by retrieval. Disproved by the code: the
  denominator is computed and then ignored.

## State of the work, measured

Everything below is a number this repository produced. Nothing here is a
projection, and the items that are *not* done are listed as plainly as the ones
that are.

### Closed in the last stretch

| | measured |
| --- | --- |
| **The learning loop is closed** | propose → witness → promote, no human in it. `src/loop_train.rs` |
| **A promotion is findable** | the champion table and both its readers share one key space, via `tokenize_id` |
| **An act can be refused** | the intent mapper returns three typed refusals — `OffMenu`, `NotAnId`, `Transport` — and the menu is enforced |
| **The model boundary cannot lie** | `Measured` exists and only an engine fills it; a mock returns `None` |
| **A doubt can leave the machine** | end to end over a real socket, refusals recorded as first-class outcomes |
| **The quantics are real** | a signed Pauli algebra and a 2ⁿ witness vector, with a Bell pair reached and measured |
| **Loss can be deliberate** | an exploration reserve, so a system can afford to find out |
| **565 tests**, wasm target builds, all seven CI jobs green | |

### Not done, and not claimed

- **The escalation rate for the matcher is unmeasured.** A 200-query fixture was
  authored by a fresh model in one pass; it turned out to be scored against a menu
  that does not exist. See [`docs/FIXTURE-SPEC.md`](./docs/FIXTURE-SPEC.md).
- **The transposition ratio is unmeasured, and the thing that would make it
  computable does not exist.** The numerator is 21,607 lines; the denominator is
  "distinct matterns" and there is no procedure for counting it. The sixteen verbs
  cannot express the shapes — a `Signature` is a flat sequence, so two occurrences
  of one shape at two site are indistinguishable from two shapes.
- **No language model is wired.** There is no tokenizer, no model, and no HTTP
  client beyond the dependency-free consultation route.
- **Harmony as a reachable equilibrium is not demonstrated.** A system that cannot
  afford to lose can only reach a local maximum; the mechanism for affording it now
  exists and the trajectory does not.
- **The §6 fork is undecided** and it is the author's: whether this is a compiler
  target or a virtual machine changes what everything else is for.

### The failures are the interesting part

Roughly a third of [`docs/HANDOVER.md`](./docs/HANDOVER.md) §3 is bugs this
project's own author introduced and its tests caught — a fabricated benchmark, an
escalation metric that rated pure repetition a perfect escalator, a Pauli
multiplication that was wrong for every non-commuting pair **while still being
associative**, a mock that reported `tokens_used: 450` for a `format!` call, and a
fixture specification that leaked the scoring algorithm through its own worked
example.

The recurring lesson is one line, and it is why the checks are cheap:

> **Closure is not evidence. Plausibility is not correctness.**

A fourth instance was found in this file, on this commit: it claimed the wasm32
target did not build and that its CI job was red. It has built for many commits. A
stale claim in the public face is worse than a missing one.

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
