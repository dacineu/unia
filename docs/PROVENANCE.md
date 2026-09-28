# Provenance and prior art

A dated record of what was published when, and what was specified by whom. This
file exists so that authorship and priority can be established from the record
rather than from assertion.

## Publication record

unia was first published publicly at <https://github.com/dacineu/unia> on
**2026-09-14**. The initial public commit is dated 2026-09-13.

| Date | Event |
| --- | --- |
| 2026-09-13 | First public commit. Primitive Bridge and Actuator Nucleus, with empirical benchmarks. |
| 2026-09-13 | UPA, UPA distributed dispatch, and Wasm-backed browser runtime. |
| 2026-09-13 | unia-omni: SLM intelligence, distributed fabric, formal verification. |
| 2026-09-13 | unia-OS: virtual kernel and shell for a browser-runnable system. |
| 2026-09-17 | Funding link added. Repository published. |
| 2026-09-27 | `unia` crate renamed from `askillify`; build repaired; `.ure` specification written down as version 0.1.0. |

The substantive design work was committed on a single day, 2026-09-13. The
commit history is short and dense rather than incremental, which reflects a
prototyping period rather than a development timeline. The dates above are the
authoritative record of first public disclosure.

## What was specified here, and when

| Element | First public | Note |
| --- | --- | --- |
| The `.ure` universal resource format | 2026-09-13 | Manifest schema and Primitive Bridge |
| The Primitive Bridge, separating intent from implementation | 2026-09-13 | `src/bridge/primitive.rs` |
| The Actuator Nucleus, and the economic gate on actuation | 2026-09-13 | `src/nucleus/mod.rs` |
| DU-UUID, content-derived resource identity | 2026-09-13 | `src/identifiers/mod.rs` |
| The `ure_RES_CAT_LOC_UNIQ` identifier scheme | 2026-09-14 | Documented only; not implemented |
| UPA, the United Processor Architecture | 2026-09-13 | `src/bridge/upa.rs` |
| unia-OS, the virtual kernel and VFS | 2026-09-13 | `src/os/` |
| The actuator evolution path, OS-dependent to actuator-driven | 2026-09-13 | `docs/arch-decoupling-strategy.md` |
| A written `.ure` specification | 2026-09-27 | [`docs/SPEC.md`](./SPEC.md), version 0.1.0 |

The whitepaper predates the repository. A rendered
`docs/whitepaper_unia.pdf` and its LaTeX source `whitepaper_unia.tex` were
committed on 2026-09-13.

## Licence position

- **Code:** MIT, see [`LICENSE`](../LICENSE).
- **Specification:** MIT, covering the text of
  [`docs/SPEC.md`](./SPEC.md) as a document.
- **Names and marks:** not licensed, see [`TRADEMARK.md`](../TRADEMARK.md).
- **Contributions:** each contributor retains copyright, and grants the
  DCO sign-off described in [`CONTRIBUTING.md`](../CONTRIBUTING.md).

## Attribution

unia is the work of a single author, published under a pseudonym. The
pseudonym appears in the commit history and in `LICENSE`; the MIT licence
requires that attribution be preserved, so the copyright line in `LICENSE` is
part of the licence grant and must be reproduced in any distribution.

If you fork this project, keep the `LICENSE` file and the copyright notice
intact. Attribution is the only condition MIT imposes, and it is the entire
economic return this work asks for.

## A request to downstream implementers

If you implement `.ure` independently, this project would rather be a useful
standard than a licensed dependency. No permission is needed provided you do not
use the marks. Tell us so it can be listed in the ecosystem section of the
README.
