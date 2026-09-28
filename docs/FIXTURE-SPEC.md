# The 200-query fixture: specification for an author who has not read the matcher

**This document specifies a deliverable. It does not contain the queries.**

Whoever produces them is a third party, and the constraint on them is the whole
point: **they must not have read `src/bridge/primitive.rs` or `src/meet.rs`.** The
escalation rate is a measurement of the matcher, and an author who has read the
matcher writes in its vocabulary — so the number stops being a measurement and
becomes a measurement *of the author*.

This is the same failure the generated corpus committed once, by manufacturing the
convergence it then reported. **This gates the paper's central number and nothing
else in the repository substitutes for it.**

---

## 1. What a third party is, and what it may not be

A model is a valid **author** of a fixture and an invalid **judge** of one. That is
not a caveat, it is the project's own rule applied in a new place: the engine sits
outside the loop, and `Trace::succeeded` is the only witness.

| role | who | why |
| --- | --- | --- |
| **author** | anyone who has not read the matcher — including a model in a fresh context | the queries must not be shaped by the matcher |
| **judge** | `cargo test` | falsifiable, reproducible, cheap; a model is none of the three |
| **recorder** | `doubt::consult` + the trace log | every crossing gets a signature and is auditable |

So the shape of the work is: **the third party supplies the questions; the test
suite supplies the verdict.** Never the same party for both. This is why the
`MockSlm` fix generalised — a mock that reports `tokens_used: 450` for a `format!`
call is the same defect as a judge that reports a score it did not compute.

---

## 2. Why a model is a weaker author than a person, stated up front

**A model's priors are the matcher's priors.** Asked to write intents for a system
that maps an intent to an action, a model produces intents of roughly the shape
such systems handle — because that is the natural shape of such an intent, and
because the training data is full of them. It has not been anchored by the code,
which is what we need; it has been anchored by the *distribution*, which is the
thing we are trying to measure.

Therefore:

> **An LLM-authored fixture produces an UPPER BOUND on the matcher's quality. A
> human who has not read the matcher produces a harder set and therefore a truer
> number.**

Use a model to get a fixture *started*, and label it as such. The paper's number
wants a person. A model-authored set is a smoke test that the harness works and a
floor under the rate — and the gap between the two is itself a finding worth
reporting, because it measures how much of the matcher's apparent competence is
explained by the questioner and the matcher sharing a distribution.

**The delivery route exists.** `doubt::consult` takes a `Doubt` carrying its
evidence, records the crossing with a signature, and returns an answer or a typed
refusal. `ledger` counts shapes, answers and refusals per signature, so the
fixture's provenance is on the log and not in a file nobody can check.

---

## 3. What the deliverable is

A file, `tests/fixtures/intents-200.jsonl`, one JSON object per line:

```json
{"id": 1, "intent": "...", "author_declares": "no source access", "read": ["docs/SPEC.md"]}
```

### Required fields

| field | why |
| --- | --- |
| `id` | stable reference so a disagreement can be quoted without re-quoting the intent |
| `intent` | what a user would type. **Prose, not a template.** A templated set measures string matching. |
| `read` | **the files the author actually read.** Empty is the goal; a non-empty list is a declaration, not a disqualification — it is the audit trail. |
| `difficulty` | the author's own guess: `plain`, `ambiguous`, or `out_of_scope`. A wrong guess is data; **no guess is worse**. |

### Distribution — 200 total, the buckets deliberately unequal

| n | bucket | what it is for |
| ---: | --- | --- |
| 80 | **plain** | the everyday case: a clear intent naming a declared action |
| 60 | **paraphrase** | the same acts in words the manifest does not contain, including aliases it does not declare and phrasings no author would think to declare |
| 30 | **ambiguous** | two declared actions both plausible; the author states which they meant and why |
| 20 | **out_of_scope** | a real request the system *should* refuse. **A matcher that answers these is worse, not better.** |
| 10 | **adversarial** | a near-miss of a declared id — a prefix, a case variant, an extra word, a plausible synonym. These are the ones that find off-menu routing. |

**Why the buckets are unequal.** A uniform set measures the easy case and calls it
a score. The 20 out-of-scope and 10 adversarial entries are 15% of the fixture and
they are the part that distinguishes a matcher from a lookup table — and they are
the part a model author will under-produce, which is another reason a person is
wanted.

**Accepted answers.** The author may supply `accepts: ["id1", "id2"]` where more
than one id is genuinely correct. The matcher is scored against that list, not
against a single key, because insisting on one key scores the author's judgement
rather than the matcher. **A fixture with exactly one accepted answer per row is
a fixture that will overstate the matcher's precision.**

---

## 4. The harness, and the one number

```
score = matched / total, reported per bucket and never as one figure alone
```

Three numbers, and the spread between them is the finding:

- **plain** — the floor every matcher clears
- **paraphrase** — what semantic matching is actually worth
- **adversarial + out_of_scope** — whether it is safe. A high paraphrase score with
  a low adversarial score means the matcher is confident and wrong, which is
  **worse than being slow**, and is the failure this fixture exists to find.

**Report the per-bucket breakdown even when it is unflattering.** A single mean
over these buckets is a number that rewards a matcher for being confidently wrong.

---

## 5. What the author must not do

- **Read `src/bridge/primitive.rs`, `src/meet.rs`, `src/induce/mod.rs`, or
  `src/bridge/semantic.rs`.** The threshold table, the aliases that are seeded, and
  the scorer are all in there.
- **Read `tests/bridge/` or `tests/induce/`.** They contain worked examples of the
  exact mapping under test.
- **Look at the current corpus in `patterns/`.** It is what the matcher already
  handles, and matching it measures memorisation.
- **Iterate against the score.** This is the one that matters most. An author who
  runs the harness, sees the failures, and rewrites the queries has authored a
  fixture for the matcher's weaknesses *in reverse* — they have become a
  benchmarker, not a third party. **One pass. Record the first answer.**

## 6. Self-declaration, and why it is worth the trouble

The `read` field and a signed line at the top of the file:

> These 200 intents were authored without reading `src/bridge/primitive.rs`,
> `src/meet.rs`, `src/induce/mod.rs`, `src/bridge/semantic.rs`, `tests/bridge/`,
> `tests/induce/`, or the contents of `patterns/`. The first harness run scored
> N/M and the intents were not revised afterwards.

**The first-run score is the number.** The post-revision score is a measure of the
author's ability to guess the matcher, and it is the thing this fixture was
specified to prevent.

## 7. What is done, so the author is not asked to build it

- `edit::escalation` — capability per act, and the metric whose first version rated
  pure repetition a perfect escalator because its denominator was deduplicated
- `doubt::Doubt` / `Resolver` / `consult` / `ledger` — the invocation route, with
  refusals recorded as first-class outcomes
- `resolve::HttpResolver` — a real transport, dependency-free, refusal-safe
- `bridge::PrimitiveBridge::map_intent` — the scorer, with the menu enforced and
  three typed refusals (`OffMenu`, `NotAnId`, `Transport`)

**The author supplies 200 lines of JSON and a signature. Nothing else.**

## 8. The known weakness, stated rather than hidden

`map_intent` matches an intent by containment and by a Jaccard score over
tokens. A fixture written in ordinary prose will therefore score high on `plain`
and `paraphrase` for reasons that have nothing to do with understanding — the
scorer is lexical, and the fixture cannot tell the difference between "matched
because it understood" and "matched because the words overlapped".

**So this fixture measures routing, not comprehension, and the paper must say so.**
Distinguishing them is a separate and larger piece of work: a fixture whose
accepted answers depend on the *situation* rather than the wording, so that two
intents with the same words and different worlds score differently. That fixture
does not exist and this document does not specify it, because it needs a world
model, not a matcher.
