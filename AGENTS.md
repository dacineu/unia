# AGENTS.md — working on `unia`

Read this before changing anything. The crate is well tested and has no `unsafe`, so most
of what can go wrong here is not a compile error. It is a plausible, well-formatted change
that quietly makes a number wrong, and this project has spent a lot of text defending
against exactly that.

`TODO.md` is the source of truth for what is open. This file is about *how* to work here,
and about the few things an agent must not decide.

---

## The gate

`cargo test` is the judge. Do not reason about whether a change is correct when running it
answers the question. The full CI set, in the order it fails you fastest:

```bash
cargo check --all-targets --locked    # library + tests + examples; the gate that matters most
cargo test --locked                    # 594 tests, all green on main
cargo build --examples --locked        # the demonstration surface must build
cargo run --example full_system_demo   # end-to-end, and it is a real assertion
cargo run --example verify_fs_pipeline
rustup target add wasm32-unknown-unknown
cargo check --lib --target wasm32-unknown-unknown --locked   # library only, see below
```

Toolchain is **stable, unpinned** — there is no `rust-toolchain.toml`. Do not add one
without being asked, and do not bump dependency versions; CI resolves with `--locked`.

### The wasm job is library-only, and that is a claim about the crate

`mio` alone produces 48 errors on `wasm32-unknown-unknown`, and the `mcp-server` feature
cannot run there at all. Both are gated out in `Cargo.toml` under
`[target.'cfg(not(target_arch = "wasm"))'.dependencies]`. So the wasm job builds
`--lib` only, deliberately: a fix that makes the wasm build green by moving those
dependencies out of the cfg gate has broken the statement the job exists to make. If you
need a dependency on wasm, change the architecture, not the gate.

---

## The five steps, and which one you are allowed to touch

The translucency claim is that the whole chain is inspectable:

```
resolve_primitive  →  signature  →  dispatch  →  induce  →  skeleton
    decoding          narrowing     transduction   patterning   matterning
```

The language model is confined to **one** of these — a transducer from text to a primitive
sequence. It is never anything that determines identity. Any change that lets a model
choose identity, skip `signature`, or author its own fixture has broken the central claim
of the paper, not merely added a feature. Check which step you are in before you start.

## `Pattern` and `Mattern` are not interchangeable

This trips up every agent, because the codebase is internally inconsistent on purpose for
now. A **Pattern** is the *input* to transduction. A **Mattern** is the *product* — the
loaded corpus manifest, which by this project's definition in `docs/cognition-log.md` is a
Mattern. `mcp::store::Pattern` is the type that carries the name of its own input, which is
the bug: until it is renamed, the Pattern/Mattern vocabulary cannot carry weight.

The rename is scoped in `TODO.md` (hygiene): 16 references across 6 files, 6 of them inside
`store.rs` itself, one commit, no behaviour change. Do it as exactly that — a rename, not a
refactor, and not an invitation to fix things you notice nearby.

---

## What you must not decide

`TODO.md` groups its open items by *why* they are open, and says the groups "need different
things from you and confusing them is how a project spends a year on the wrong one." The
grouping is load-bearing.

### Group A is the human's. Do not start it.

The fork at `docs/unia-target-spec.md` §6, the in-flight-act-on-demotion semantics, the
fixture authorship question, and the name/trademark clearance. Each is a decision with two
projects behind it. If you find yourself writing code for one, you have misread it — the
decision comes first and the code follows it.

### The 200-query fixture must not be authored by an agent

Not "an agent should not judge it" — an agent must not *author* it either, and the reason
is in `TODO.md` item 3. A third party supplies the questions, `cargo test` supplies the
verdict, never the same party for both. A model's priors are the matcher's priors, so an
LLM-authored set is an **upper bound** on matcher quality where a human one is truer — the
gap between them is itself a finding. And the distribution is deliberately unequal
(80 plain / 60 paraphrase / 30 ambiguous / 20 out-of-scope / 10 adversarial) because the
last 30 are what separate a matcher from a lookup table, and what a model author will
under-produce. **One pass. The first-run score is the number** — an author who iterates
against the harness has become a benchmarker and destroyed the independence the fixture
exists for.

### Measurements are reported, not optimised

Group B is blocked on numbers nobody has taken. When you take one:

- **Report the number you got.** Never adjust a benchmark, a threshold, or a fixture to
  make a result read better. If the number is bad, the number is the finding.
- **Score per bucket, never as a mean.** High paraphrase beside low adversarial means
  confidently wrong, which is worse than slow, and a mean rewards exactly that.
- **Say what the measurement is not.** `map_intent` scores lexically, so it measures
  **routing, not comprehension**, and the paper has to say so. A number presented as more
  than it measures is the failure this project cares about most.
- Baseline for orientation: a whole act is ~14,300 ns, of which ~11,000 (77%) is
  `map_intent` and ~3,300 (23%) is dispatch. If your "optimisation" does not move that
  split, you have not optimised the thing that is slow.

---

## Style, from the existing code

- **No `unsafe`.** There are zero occurrences in `src/`. Do not introduce the first.
- Comments explain **why**, and cite a document when one exists (`docs/SPEC.md §6`,
  `docs/pattern-and-mattern.md §1`, the commit that decided it). The codebase is unusually good at
  this; a comment that narrates what the next line does is a regression. Several comments
  in `Cargo.toml` and `ci.yml` exist specifically to stop someone re-litigating a decision,
  so read those before changing those files.
- `TODO.md` is written in a particular voice — precise, quantified, and willing to say a
  previous claim was backwards. When you resolve an item, say how, with the number. The
  resolved `implementation/` entry is the model: it lists what was wrong *and* what was
  kept and why.
- `src/camaduci.rs` is 4,968 lines and `src/mcp/store.rs` is 1,977. Large files are load
  bearing here, not an accident, but do not grow one without a reason you would defend.

## Use the index

`.codegraph/` is self-ignoring, so it is **not** in git and a fresh Orca worktree
starts without it. Run `codegraph init` once per worktree (~seconds for this crate) before
relying on it. In the primary checkout it is present and current (97 files, 1,900 nodes).
Prefer it over grep:

```bash
codegraph query map_intent          # FTS over the symbol graph -> file:line
codegraph context map_intent        # the symbol's source, with call paths
codegraph files src                 # indexed files under a prefix
codegraph status                    # index health
```

CLI 1.2.0 has no `explore` subcommand — `context` is the one that returns source plus call
paths. If you are following an older note that says `codegraph explore`, it is wrong.

`camaduci.rs` alone is larger than most agents' grep budget. If you cannot find a symbol,
check `codegraph status` before assuming it is missing.

## Working in a worktree

Development happens in Orca-managed worktrees, one per task, branched from `main`. That
means:

- **Read `TODO.md` before writing a branch name.** The item's own number is the branch
  name (`todo-15-wasm-resolver`), so the diff and the item stay connected.
- One task per worktree. Do not batch two hygiene items into one branch "while I'm here" —
  they are independently reviewable and that is the point of splitting them.
- `main` is the integration branch. Do not push to it directly.
