# History

Working papers, kept rather than deleted, and not part of the specification.

`amater` was the predecessor project. The `.ure` format, the resource model and
the intent-resolution idea all come from it, so these documents are the record of
where the current design came from. They are **not** the current design: the
normative format is [`../SPEC.md`](../SPEC.md) and the current argument is
[`../LARGE-PATTERN-MODELS.md`](../LARGE-PATTERN-MODELS.md). Where the two
disagree, the parent documents win.

| Document | What it is |
| --- | --- |
| [`amater-ure-design.md`](./amater-ure-design.md) | The original `.ure` design. The longest and the most complete of the set. |
| [`amater-ure-analysis.md`](./amater-ure-analysis.md) | Analysis of the format's weaknesses, written before it was adopted. |
| [`amater-ure-proposals.md`](./amater-ure-proposals.md) | Proposed changes to the format, most of which were never taken up. |
| [`amater-agent-manager-design.md`](./amater-agent-manager-design.md) | The agent manager that became the orchestrator. |
| [`amater-rust-sqlite-design.md`](./amater-rust-sqlite-design.md) | A Rust and SQLite design, superseded before it was built. The corpus later moved to DuckDB. |
| [`brainstorming-session-actuators.md`](./brainstorming-session-actuators.md) | Early notes on driving actuators from a recorded session. |
| [`implementation-plan.md`](./implementation-plan.md) | The implementation plan written for the `amater` era. |

## Why these are kept

Three reasons, in order of how much they matter.

**Attribution.** The format is the asset, and a format with a traceable
provenance is a much stronger claim than one that appears from nowhere.
[`../PROVENANCE.md`](../PROVENANCE.md) carries the formal record; this directory
is the working papers behind it.

**Reversibility.** Several current decisions were made against these documents
and it is worth being able to check whether the reasoning still holds. The DuckDB
decision in particular was made by rejecting the SQLite design in
`amater-rust-sqlite-design.md`.

**The failures are the useful part.** `amater-ure-analysis.md` is a list of
things wrong with a design that was then adopted anyway, and reading it now is
the fastest way to see which of those complaints this repository has actually
answered and which are still open.

## What is *not* here

`implementation/` at the repository root is a divergent draft of six `src/`
modules, not a historical document, so it is not filed here. Its status is
recorded in [`../../TODO.md`](../../TODO.md).
