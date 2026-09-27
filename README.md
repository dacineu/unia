<div align="center">

# unia

**Universal Nucleus Interface Architecture**

An orchestration system that decouples an agent's *intent* from its
*implementation*.

[![CI](https://github.com/dacineu/unia/actions/workflows/ci.yml/badge.svg)](https://github.com/dacineu/unia/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/unia.svg)](https://crates.io/crates/unia)
[![docs.rs](https://img.shields.io/docsrs/unia.svg)](https://docs.rs/unia)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

[`.ure` specification](./docs/SPEC.md) ·
[Whitepaper (PDF)](./docs/whitepaper_unia.pdf) ·
[Commercial and support](./LICENSE-COMMERCIAL.md) ·
[Contributing](./CONTRIBUTING.md) ·
[Security](./SECURITY.md)

</div>

## The problem

An agent that can do one useful thing is usually hardcoded to the machine that
thing lives on. It shells out to `/usr/bin/grep`, it expects a CUDA device, it
assumes a POSIX filesystem. Moving it to a browser, a different GPU, or a
different OS means rewriting the integration, so the capability and the
environment stay welded together and neither can evolve.

## The idea

Separate *what* an agent wants done from *how* it gets done.

A **Primitive Bridge** maps natural-language intent onto a small, stable
vocabulary of universal primitives. An **Actuator Nucleus** then satisfies those
primitives against a real resource. Neither knows much about the other, and
neither knows about the operating system.

The middle layer is a **Punctuation Layer**: capabilities are declared as
`.ure` resources rather than as paths to binaries, so the same declaration can be
satisfied by a shell command, a Wasm module, or a GPU kernel.

```json
{
  "resource_id": "valve-001",
  "category": "actuator",
  "state_space": {
    "flow_rate": { "type": "float", "range": [0.0, 1.0], "unit": "percentage" }
  },
  "action_primitives": [
    {
      "id": "emergency_shutdown",
      "aliases": ["emergency_shutdown", "emergency shutdown"],
      "params": {},
      "target_state": "flow_rate = 0.0",
      "constraints": ["status != 'fault'"]
    }
  ]
}
```

```
"Emergency shutdown the valve"
        │
        ▼
  PrimitiveBridge ──► PrimitivePacket ──► ActuatorNucleus ──► driver
  (intent → primitive)                   (packet → hardware)
```

## Evolution path

```
OS-dependent  →  Primitive Bridge  →  Actuator-driven  →  Universal Nucleus
```

OS primitives are treated as bootstrap resources. Over time an optimised `.ure`
actuator replaces them, and the system stops depending on the host OS's tooling.

## What is implemented today

Twenty-two modules, in five layers. This is a working prototype, not a product,
and the [specification](./docs/SPEC.md) records where the code and the prose
still disagree.

| Layer | Modules | Role |
| --- | --- | --- |
| **Declaration** | `identifiers`, `registry` | DU-UUID content addressing, manifest resolution, champion selection |
| **Resolution** | `bridge`, `primitives`, `router`, `slm` | Intent to primitive mapping, semantic scoring, SLM routing |
| **Execution** | `nucleus`, `os`, `weights`, `profiler` | Packet dispatch, virtual kernel and VFS, hardware abstraction, LoRA memory |
| **Economics** | `wmis` | Token metering, permissions, resource quality |
| **Evolution** | `orchestrator`, `pipeline`, `harvester`, `learner`, `transducer`, `evolution`, `fluid`, `meta_actuators` | Harvest, learn, transduce, project, mutate |

Cross-cutting: `mcp` for Model Context Protocol connectors, `release_manager` for
champion promotion, `node` and `transducer` for the distributed fabric, and a
`wasm32` build via `wasm_core`.

## Quick start

```sh
git clone https://github.com/dacineu/unia
cd unia
cargo test
cargo run --example camaduci
```

Requires a stable Rust toolchain, 1.75 or later.

```sh
cargo check --all-targets     # library, tests and examples
cargo test                    # 112 tests
cargo run --example verify_fs_pipeline
cargo build --target wasm32-unknown-unknown
```

### Examples

| Example | Shows |
| --- | --- |
| `camaduci` | A digital pet whose vitals are a declared state space and whose care is recorded as traces. Start here. |
| `full_system_demo` | Provisioning, orchestration, SLM execution and learning, end to end |
| `verify_fs_pipeline` | Bridge to nucleus with economic accounting on each actuation |
| `formal_verification_demo` | Property-based verification of the actuation contract |
| `benchmark_unia` | Decoupled intent mapping against a coupled baseline |
| `universal_demo` | Several resource categories through one pipeline |

#### ca(R)maduci — the first example

```sh
cargo run --example camaduci
```

A creature whose vitals are a declared state space, whose care operations are
primitives, and whose neglect is the *absence* of a call. It is the smallest
thing that exercises the whole loop, and it is first because everything else in
this crate is harder to see working.

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

Both creatures received the same number of ticks. One reached adulthood and has
a trace log; the other stayed an egg, died, and was **quarantined rather than
deleted**, because death is a lifecycle transition and the traces that caused it
are still evidence.

Three properties carry over from the research, and they are the reason the
example is worth reading:

- **Age advances on a completed sleep cycle, not on a clock.** A pet that is never
  put to sleep never leaves the egg. Progress is gated on a finished interaction
  rather than on elapsed time.
- **Neglect is not a punishment.** It is what happens when no intent arrives and
  no primitive is dispatched. The game cannot levy it at will, which is what
  makes the obligation real.
- **The clock is injected.** `now` is a parameter, not a call to the system
  clock, so the whole example is deterministic and testable.

What it does **not** show: escalation rate at any interesting corpus size,
cross-model execution, or convergence between nodes. One pet is one artifact with
no peer to converge with. See [`docs/camaduci.md`](./docs/camaduci.md) for the
design and the prior art it draws on.

## Status

**Pre-1.0. Experimental.** Known limitations are listed as divergences D1–D7 in
the [specification](./docs/SPEC.md), including two identifier schemes that are
documented but not yet reconciled, and action constraints that are declared but
not yet evaluated. Read those before building on this.

## Documentation

| Document | Contents |
| --- | --- |
| [`docs/SPEC.md`](./docs/SPEC.md) | The normative `.ure` format, version 0.1.0 |
| [`docs/whitepaper_unia.pdf`](./docs/whitepaper_unia.pdf) | The full design argument |
| [`docs/arch_decoupling_strategy.md`](./docs/arch_decoupling_strategy.md) | Why the decoupling layer exists |
| [`amater-agent-manager-design.md`](./amater-agent-manager-design.md) | Agent manager design |
| [`docs/PROVENANCE.md`](./docs/PROVENANCE.md) | Publication dates, authorship, prior art |
| [`COGNITION_LOG.md`](./COGNITION_LOG.md) | Design rationale, including the Patten/Mattern distinction |

## Using unia

The reference implementation is **MIT licensed**. You may use it commercially,
modify it, and sell products built on it, with only the requirement of
preserving the copyright notice.

MIT does **not** cover the names. `unia`, `unia-OS`, `UPA`, `DU-UUID` and
`.ure` are trademarks and are not licensed. See
[`TRADEMARK.md`](./TRADEMARK.md).

If you want supported, warranted, or bespoke work, a negotiated licence where
MIT does not fit, or permission to use the marks, contact
**<dacineu@proton.me>**. The terms are in
[`LICENSE-COMMERCIAL.md`](./LICENSE-COMMERCIAL.md).

If you are implementing `.ure` independently: no licence is required, provided
you avoid the marks. Tell us and it will be listed here.

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

<p align="center">
  <a href="https://ko-fi.com/A7O02756VY">☕ Support via Ko-fi</a>
</p>

## Licence

MIT. See [`LICENSE`](./LICENSE).
