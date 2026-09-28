# Delegation plan: work handed to another reasoning model

**This is executable now, not aspirational.** A subagent starts with a fresh
context, which is precisely the property the fixture needs and which I do not
have. So "involve another LLM" has a concrete mechanism here, and the interesting
question is not *whether* to delegate but **what a read-set buys and what a
stronger model costs.**

---

## 1. The rule that governs every task below

Established in `docs/FIXTURE-SPEC.md` and it generalises:

> **A model is a valid author and an invalid judge.**

The **read-set is the only lever that creates independence.** A model's context is
the only thing that determines what it knows, and unlike a person's, it is
*writable*. So delegation here is not "ask an assistant to help" — it is **the
construction of a specific, minimal, audited context**, and the deliverable's
credibility comes from what the context was *excluded* from, not from the quality
of the prose.

Consequence for the brief: **I must not write the prompt.** If I describe the
matcher in my own words to a fresh model, I have contaminated it through the
brief. The brief is a *path to a document* and nothing else.

## 2. What a stronger model changes

The user asked for a model *capable of reasoning*. That is the right instinct for
some of this and the wrong one for the rest, and the reason is the pattern this
repository keeps hitting:

> **Closure is not evidence. Plausibility is not correctness.**

A stronger model produces more fluent, more confident, more internally consistent
answers — and every one of the fourteen failures in `docs/HANDOVER.md` §3 was a
fluent, internally consistent, wrong answer. Three of them *closed as algebra*.
Capability therefore:

- **raises the value of delegation on open-ended method work** — where there is no
  right answer yet and coherence is the goal
- **lowers the safety of delegation on anything with a verdict** — because it will
  produce a plausible answer instead of admitting it cannot check one

So the plan below delegates **authoring and method** and keeps **every judgement**
on this side, where `cargo test` is the witness.

---

## 3. The four tasks

### Task 1 — Author the 200-query fixture

**This is the one only a third party can do, and the reason I cannot do it is the
rule, not modesty.** I have read all seven forbidden files this session. Any
fixture I wrote would measure my assumptions wearing a measurement's label.

| | |
| --- | --- |
| **brief** | the path to `docs/FIXTURE-SPEC.md`, and nothing else |
| **read-set** | `docs/FIXTURE-SPEC.md`, `docs/SPEC.md` only |
| **excluded** | `src/bridge/`, `src/meet.rs`, `src/induce/`, `src/bridge/semantic.rs`, `tests/bridge/`, `tests/induce/`, `patterns/` |
| **deliverable** | `tests/fixtures/intents-200.jsonl` + the self-declaration block |
| **verified by** | the harness, **once**, and the first-run score is recorded verbatim |
| **failure mode to name** | an author who iterates against the harness becomes a benchmarker. **One pass.** The spec says so and the file header must say so. |

**Exit:** 200 rows, buckets 80/60/30/20/10, `read` declared, a first-run score, and
the intents not revised afterwards.

### Task 2 — Propose the counting method for the transposition denominator

**The open problem.** The ratio is *stated* (`docs/unia-target-spec.md` §5) and not
computable: the numerator is 21,607 lines and countable, the denominator is
"distinct matterns" and has no procedure. Worse, the vocabulary cannot express the
shapes — `Signature` is a flat sequence, so two occurrences of one shape at two
sites are indistinguishable from two shapes.

| | |
| --- | --- |
| **brief** | the three shapes found in `rematter` and the counts (28 / 1 / 1), as **data**, not as code |
| **read-set** | `docs/HANDOVER.md` §3, and one named function — `clean::rematter`, 17 lines |
| **excluded** | the other 21,590 lines. **A method proposed by looking at everything is a shape list, not a method.** |
| **deliverable** | a *proposal*, in writing, for how to count distinct shapes — plus its own stated failure mode |
| **verified by** | the proposal is run on a module I choose, and compared against a hand count I do independently |
| **failure mode to name** | a method that "works" because it finds shapes I already knew about. **The independent hand count is the only check.** |

**Exit:** a method that, on a module I pick, produces a count I can check by hand
— and a number that is *not* flattering if the method is bad.

### Task 3 — Adversarial review of this session's work

**The strongest delegation available, and the one nobody has asked for.** I wrote
most of the code in this session and I have been wrong about it fourteen times, so
my review of my own work has a known defect.

| | |
| --- | --- |
| **brief** | "find the claim in this diff that is wrong" — **not** "review this code" |
| **read-set** | the diff, and the tests. **Not the session narrative** — it is persuasive and it is mine |
| **delivered as** | findings only, each with a file, a line, and a **failing scenario** |
| **verified by** | me, by *running* the scenario. Not by reading the finding |
| **failure mode to name** | agreeing with me because I framed it. A review that finds nothing is evidence it read the framing, not the code. |

**Exit:** every finding either reproduced or dismissed with a reason. **A review
returning zero findings has told us the brief was too polite and should be re-run
as "name the three most suspicious lines."**

### Task 4 — The witness reading, not the writing

`Trace::succeeded` is the project's only falsifiable check. A model can propose
what *should* be witnessed.

| | |
| --- | --- |
| **brief** | a scenario where `Trace::succeeded` is read as confirmation and should not be |
| **read-set** | `edit.rs`, `loop_train.rs`, `doubt.rs` — the three writers |
| **excluded** | the session log, which explains why it was built |
| **deliverable** | concrete scenarios, each naming a function and an input |
| **verified by** | each scenario becomes a test, and the test must pass on the fixed code and fail on the current code |

**Exit:** one test per scenario. This task is unusual in that **its output is code
I must then run**, so it converts a review into a regression suite.

---

## 4. Order, and why

**1 → 3 → 2 → 4.**

- **1 first** because it is the only task with an external blocker — the paper's
  central number — and it takes the longest to review honestly, since a fixture
  must be produced in one pass and cannot be iterated.
- **3 second** because it is the cheapest and it should run against the work from
  tasks 1–2 rather than alongside them. A review of a review is still cheaper
  than a wrong fixture.
- **2 third** because it needs the fixture's failure modes as input: a counting
  method that cannot distinguish a shape the matcher gets wrong from one it gets
  right is not measuring what we care about.
- **4 last** because it needs the diff from everything above.

## 5. What I will not delegate, and the reason

- **Any judgement.** Scoring the fixture, accepting a counting method, deciding
  whether a review finding is real. All of it stays here, against `cargo test`.
  The project rule is that the engine sits outside the loop.
- **Anything where I have already contaminated the context.** Task 2's brief is the
  riskiest: I found the shapes, so I will find them again in the model's answer,
  and I will be tempted to accept a method that agrees with me. **The independent
  hand count is the mitigation, and it has to be done before the proposal is read.**
- **A decision I own.** The §6 fork, the in-flight hotswap policy, the trademark.

## 6. The honest summary

Four tasks are delegable and all four are **authoring or method** work. Not one
is a judgement, because a judgement delegated to a model is a score nobody
computed — the `MockSlm` defect with a reasoning model attached, and strictly
harder to spot, because the fabricated number would be *plausible*.

The strongest thing a second model buys here is not throughput. It is **Task 1**,
which I am structurally disqualified from, and **Task 3**, which I am the worst
person to run on my own work.
