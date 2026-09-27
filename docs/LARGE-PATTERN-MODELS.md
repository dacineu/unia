# Large Pattern Models: Replacing Token Consumption with Artifact Reuse

**Author:** iulian dacineu
**Assistance:** authored and researched with AI assistance; see §A.
**Status:** Position paper with working prototype. Pre-1.0.
**Artifact:** https://github.com/dacineu/unia
**Prototype measurements:** all figures below were produced on the hardware
described in §8. None are quoted from a vendor or a prior paper.

---

## Abstract

Language models are consumed per request. A coding agent that has solved a task
before pays for it again, because the solution it produced is stored as text —
in context, in a commit, in a skill file — and text must be re-read by a
transformer to become an action. This paper argues that the unit of reuse should
be an *executable artifact* rather than a *description*, and that the resulting
system should be optimised for a metric I call **escalation rate**: the fraction
of requests that cannot be served from a local artifact and must reach a
provider.

I formalise a resource model, the `.ure` format, in which a unit of capability
is a content-addressed declaration bound to a runnable body. I describe an
induction loop that converts recorded interactions into candidate artifacts
without gradient descent, and an evidence gate that makes premature promotion
structurally difficult. The prototype is `unia`, in Rust, with a Model Context
Protocol server surface.

My measurements characterise the retrieval tier and establish a first
retrieval-quality baseline: a 4.6 µs lexical matcher achieves 90% Hit@1 against
a hand-written fixture, with all observed failures attributable to manifest
authoring rather than to the need for semantic machinery. I am explicit that
this fixture is too small to support the thesis, and I state the experiment that
would falsify it.

---

## 1. The problem, stated as a cost

The dominant cost of a language-model-backed agent is not reasoning. It is
**re-deriving knowledge the system already possesses.**

Consider a coding agent asked to add a dependency and refresh a lockfile. The
second time it is asked, the answer is identical. The first answer may have
taken 2,000 input tokens and 600 output tokens. The second costs the same, because
the first answer was recorded as *text* — a diff, a file, a markdown skill — and
text is inert. It becomes an action only by passing through a transformer again.

Three mechanisms exist to mitigate this, and all three leave the architecture
unchanged:

| Mechanism | Model still runs? | Eliminates |
|---|---|---|
| Prefix / prompt caching | Yes | Input cost on a repeated prefix |
| Semantic caching | No, on near-match | The request, on semantic similarity |
| Retrieval-augmented generation | Yes | Nothing; it adds context |

Prefix caching reduces the price of a call. Semantic caching skips calls whose
*answers* are near-identical. RAG feeds retrieved context to a model that must
still generate. None of them changes what the system *is doing*: reasoning from
scratch, per request.

The observation motivating this paper is that the ceiling on all three is set by
the same thing. A cache can only return something that was previously computed
and stored. If the stored artifact is a *description* of an action, serving it
requires a model. If the stored artifact *is* the action, serving it does not.

### 1.1 The claim

> **C1.** For a workload with sufficient repetition, a system that stores
> executable artifacts rather than descriptions can drive escalation rate toward
> zero without gradient training, and per-served-request token cost toward zero,
> because the model is *out-routed* rather than *distilled*.

"C suffices" — no training — is a deliberate constraint, not a limitation. It
means the learned artifact is inspectable, auditable, and deletable. See §6.

### 1.2 What would falsify C1

C1 is not unfalsifiable, and the falsification condition should be stated before
the evidence. It fails if:

> **F1.** On a labelled set of real coding intents, a pattern corpus of
> realistic size (200–2,000 artifacts) fails to route a large majority of
> requests — that is, if a substantial residue of intents shares no vocabulary
> with any candidate artifact and resists paraphrase.

I do not yet have this experiment. §9 states what exists and what does not. The
prototype exists to make F1 answerable, not to claim C1 is established.

---

## 2. Prior work and my relation to it

This is not a new idea in the shape of the idea. It is a specific claim about
the *unit* of reuse, and the literature already contains most of the ingredients.

**Skill induction.** Recent work has established that agents which memorise
action sequences fail, and that inducing *rules* — explicit control flow,
preconditions, variable binding — generalises better (NSI, ICML 2026). SkillGen
synthesises a single auditable skill from trajectories. SkillRevise couples
execution evidence with general repair knowledge. My induction follows this
line: traces are grouped by the **primitive sequence**, and the observed
phrasings become aliases, so the induced unit is a rule and not a script.

**Executable accumulation.** AgentFactory accumulates executable subagents and
reuses them. This is the closest prior work to my claim, and I do not claim
novelty over it. My contributions are narrower and are stated as such in §10.

**Library learning and program synthesis.** DreamCoder-style systems induce
reusable libraries from traces. The relevant difference is the target: a
synthesised program versus a declaration of intent bound to a pre-existing
driver.

**Semantic caching.** Caches return prior *responses*. I am describing a system where
the cache entry is executable and the provider is not called. The
relationship is a strict generalisation: my tier 1.5 is a semantic cache, and
tiers 1–2 are the part that has no analogue.

**My specific position.** I combine three properties that I have not found
together elsewhere:

1. The reusable unit is **content-addressed**, so two independently induced
   rules that are textually identical collapse to one artifact without a
   comparison step, and renaming an artifact does not change its identity.
2. Learning is **zero-gradient**, producing inspectable artifacts rather than
   weight deltas.
3. The objective is **escalation rate**, not accuracy — because accuracy is the
   wrong metric when a correct answer still costs a request.

I do not claim the combination is unprecedented in principle. I claim it is
unusual, that it is cheaper, and that it is auditable in a way weight-based
learning is not.

---

## 3. The resource model

A unit of capability is a resource, declared independently of where it lives and
what it physically is. Following the formal specification in
`docs/ure-specification-formal.md`, a resource is a tuple

```
U = < I, S, A, C >
```

where **I** is identity, **S** a state space, **A** a set of action primitives,
and **C** a set of constraints. Identity is a content address: a DU-UUID derived
from the canonical serialisation of the manifest with `resource_id` removed,
hashed with SHA-256. Consequently, a resource cannot contain its own identity in
its identity computation, and two manifests that serialise identically *are* the
same resource.

A concrete manifest:

```json
{
  "ure_version": "1.0",
  "resource_id": "valve-001",
  "category": "actuator",
  "state_space": {
    "flow_rate": { "type": "float", "range": [0.0, 1.0], "unit": "percentage" },
    "status": { "type": "enum", "values": ["open", "closed", "fault"] }
  },
  "action_primitives": [
    {
      "id": "emergency_shutdown",
      "aliases": ["emergency_shutdown", "emergency shutdown", "halt"],
      "params": {},
      "target_state": "flow_rate = 0.0",
      "constraints": ["status != 'fault'"]
    }
  ]
}
```

### 3.1 The gap between the formalism and the code

The formal specification requires that constraints "must return `True` for an
action to be dispatched to the Nucleus." The implementation prints constraints
for operator visibility and does not evaluate them. This is recorded as
divergence D5 in `docs/SPEC.md` and is not yet closed.

I report this rather than hide it because it is a load-bearing discrepancy: §6's
safety argument assumes constraints gate dispatch, and today they do not. The
argument therefore currently rests on the evidence gate alone.

I also record two identity divergences: the identifier scheme documented in
prose is not the one implemented (D2), and a document category vocabulary and a
resource-class enumeration are both in use for the same dimension (D8). A
manifest is currently addressable only if its category is trusted, because the
loader falls back to the filename stem for non-UUID identifiers.

---

## 4. Architecture

```
intent
  │
  ├─ tier 1   exact / containment / IDF-weighted token overlap   no model
  │           4.6 µs at 12 artifacts; 0.5 µs per artifact, linear
  │
  ├─ tier 1.5 semantic re-rank of the residue                     optional
  │           bge-small-en-v1.5: 33.2M params, 22.93 MiB, 8–21 ms
  │
  ├─ tier 2   verified artifact executes natively                 0 tokens
  │
  └─ tier 3   escalate to a provider                              full cost
```

The ladder is the object of study. Its property is not accuracy but
**monotone improvement**: every promotion moves traffic from a paying tier to a
free one, and the model is retired by being out-routed rather than replaced.

Crucially, **tier 1 requires no model at all**, so the architecture is useful
before any inference engine exists, and degrades gracefully when WebGPU or a
local runtime is unavailable. This is why the prototype is a working system
before it is a learning system.

### 4.1 Retrieval

Tier 1 scores every candidate phrase in the corpus against the intent using
inverse-document-frequency-weighted token overlap, with a decisive fast path for
containment. Trigram similarity is deliberately not used for scoring: it ignores
term frequency, and for short technical intents — where one distinctive token
carries most of the signal — that is strictly worse.

Measured, release build, single machine (§8):

| Corpus | p50 | p99 |
|---|---|---|
| 12 | 4.6 µs | 6.3 µs |
| 1,000 | 405 µs | 644 µs |
| 5,000 | 2.17 ms | 3.85 ms |
| 20,000 | 9.96 ms | 11.2 ms |

The scaling is linear at approximately 0.5 µs per artifact, which is the
limitation of a scan rather than an inverted index. For comparison, SQLite FTS5
with a real inverted index returns in **48 µs at 50,000 artifacts** in an 18.6 MB
file. Retrieval, not induction, is the component that breaks first at scale.

### 4.2 Storage as a view, not a copy

The corpus is the authoritative artifact: a tree of `.ure` files under version
control. The analytical index is a **view over those files** rather than a copy
of them, so an edit in Git changes query results immediately and there is no
migration to write. Only mutable state — promotions, traces, observations — is a
table. This dissolves the consistency problem that an ingest pipeline
introduces, at the cost of requiring the corpus to be schema-consistent, which
it is not yet (§9).

---

## 5. Induction: learning without gradients

`src/induce` converts recorded interactions into candidate artifacts.

Traces are grouped by the **primitive sequence** the intent resolved to, not by
phrasing. Two different sentences that reduced to the same primitives are
evidence for one rule; the sentences themselves become the action's aliases. A
trace with no recorded sequence is skipped rather than guessed at.

For each group the module synthesises a candidate with:

- an action id equal to the primitive sequence, stable across phrasings by
  construction;
- aliases drawn from every distinct observed phrasing, plus both underscore and
  space forms of the sequence;
- constraints recording the observed failure rate, so a precondition is learned
  rather than re-paid for on every call;
- a `resource_id` stamped with the DU-UUID of the resulting body, so identical
  rules deduplicate by content address.

### 5.1 What induction is not

It is not fine-tuning, and no component of the system learns weights. Two
consequences follow, both intended:

- Training requires no accelerator. Induction is string and integer work over a
  trace log.
- The learned artifact is a readable file. A wrong artifact can be deleted, and
  its deletion is a commit.

### 5.2 On auto-generating a corpus

A natural suggestion is to synthesesise a large corpus up front. I consider
this a category error in the current system. A corpus of artifacts that were
never executed has no evidence of correctness behind it, and would become
champion-eligible on the strength of a generative model's plausibility. The
evidence gate in §6 is designed specifically to resist this, and generating
corpus entries in bulk would be an attack on it rather than a use of it.

---

## 6. Safety: the failure mode this design must resist

The characteristic risk of a system that promotes learned artifacts is that **a
wrong artifact becomes fast, silent and reused.** A slow wrong answer is
recoverable. A fast wrong answer that is served from a cache and re-derived from
a trace is not, because the trace then confirms it.

The prototype treats this as the design constraint rather than as a downstream
check:

- **Candidates are emitted as `candidate` and never as `champion`.** Promotion
  is a separate, deliberate step. A function capable of emitting a promotion
  would have no gate at all.
- **A minimum of distinct observed phrasings is required.** One observation is an
  anecdote, and the same sentence repeated ten times is one observation, not ten.
  Both cases are covered by tests.
- **A maximum failure ratio blocks promotion.** A sequence that usually fails is
  not a rule.
- **Failures become recorded preconditions** rather than being discarded.
- **Traces without a recorded primitive sequence are not inducted from.**

Confidence is multiplicative in breadth and reliability, and log-scaled in the
number of distinct phrasings so that it never saturates at the minimum. A
straightforward `observations / minimum, clamped to 1` term reads 1.0 the instant
the minimum is met, which would make the fourth distinct phrasing worth exactly
as much as the second; this was found by a test rather than by inspection.

### 6.1 A known weakness of the safety argument

The evidence gate raises the cost of promotion; it does not establish
correctness. §7 reports a false-positive rate of 2 in 8 negatives, and §9 notes
that constraints do not yet gate dispatch. The honest summary is that the system
currently prevents *thin* evidence from being promoted, and does not yet prevent
*wrong* evidence from being promoted.

---

## 7. Prototype evaluation

### 7.1 Setup

A labelled fixture of 30 hand-written queries, split 20 expected-hit and 8
expected-negative, with 2 further negatives probing capabilities the corpus does
not possess. The corpus is the two curated manifests (`valve-001`,
`fs-root-001`); the other ten are quarantined and not indexed.

### 7.2 Results, tier 1 only, no model

| Metric | Value |
|---|---|
| Hit@1 | 18/20 — **90.0%** |
| Hit@3 | 18/20 — 90.0% |
| False negatives | 2 |
| False positives | 2 |
| Unsatisfiable expectations | 0 |

The four failures, verbatim:

| Query | Expected | Got | Diagnosis |
|---|---|---|---|
| `open the valve` | valve-001 | fs-root-001 | `open` is an alias of `read_file` and outranks `valve` |
| `write output to results.txt` | fs-root-001 | — | containment needs literal `write file` |
| `set the valve brightness to eighty` | none | valve-001 | shares `valve`; no capability check |
| `summarise this document for me` | none | fs-root-001 | `document` is an alias of `write_file` |

**All four are manifest-authoring defects, not semantic failures.** Three follow
from aliases that are reasonable for their own action (`open file`, `get
content`, `create document`) and actively harmful as resource-level match keys.
The fourth requires either constraint evaluation or semantic scoring.

This is the most important result in the paper and it cuts against a learned
model: the residue is two queries, which is not a training set.

### 7.3 A scoring defect this exposed

`open the valve` ranking `fs-root-001` first is not noise. IDF is currently
computed per *phrase* across the corpus, so a resource that lists `valve` across
many aliases has its own strongest term down-weighted for appearing in its own
alias set. A single rare term elsewhere then wins. The corpus should be weighted
per resource, not per phrase. This is unfixed.

### 7.4 Cost comparison against a local language model

For the routing question specifically — *which artifact matches this intent?* —
measured on the same machine:

| System | Latency | Tokens | Corpus |
|---|---|---|---|
| Qwen3-1.7B Q4_K_M (128 prompt + 64 output) | ~2,860 ms | 192 | n/a |
| Tier 1 matcher, 12 artifacts | **4.6 µs** | 0 | 12 |
| Tier 1 matcher, 20,000 artifacts | **9.96 ms** | 0 | 20,000 |
| Tier 1.5, bge-small-en-v1.5 | 8.3–20.8 ms | 0 | any |
| SQLite FTS5 | 48 µs | 0 | 50,000 |

The language model is ~620,000× slower than tier 1 at the current corpus size.
That comparison is not close, and it is also not the interesting claim, because
tier 1 does strictly less work. The interesting comparison is on *cost per
served request*, where tier 1 and tier 2 are free and tier 3 is not.

---

## 8. Hardware

All prototype measurements were taken on a single machine:

- AMD Ryzen 7 5700U (Zen 2, 8 cores / 16 threads)
- AMD Radeon Graphics, Vulkan 1.4, RADV RENOIR — **Vega 8, 8 CUs, no matrix
  cores**, unified memory sharing 15 GiB system RAM
- Reported by the inference backend as `matrix cores: none`, `int dot: 0`,
  `bf16: 0`, `uma: 1`

This is a low-end mobile integrated GPU. It is reported in full because
throughput without hardware is not interpretable, and because the absence of
matrix cores is the reason decode throughput is roughly an order of magnitude
below published figures for the same model class on Apple Silicon or discrete
GPUs.

Embedding tier, bge-small-en-v1.5 Q4_K_M, 33.21M parameters, 22.93 MiB, all
layers offloaded to the iGPU: 967 t/s at 8 tokens, 3,032 t/s at 32, 6,158 t/s at
128, giving 8.3–20.8 ms per query.

Generation tier, Qwen3-1.7B Q4_K_M, 2.03B parameters: prompt processing 272.8
t/s and generation 26.8 t/s on the iGPU, against 155.1 and 22.8 t/s for CPU
execution. The iGPU is 1.76× on prompt processing and 1.17× on generation.

---

## 9. What is not demonstrated

I consider this section load-bearing rather than a disclaimer.

- **No escalation rate is measured.** The corpus holds two usable artifacts. A
  rate over two artifacts is not an escalation rate, and the number that would
  support or refute C1 does not exist yet.
- **The fixture is 30 queries authored by the system's author.** It is a smoke
  test. It found a real defect in an afternoon, which is its purpose, but it
  cannot estimate behaviour on real traffic. A 200-query fixture written by
  someone who did not build the matcher is the minimum credible next step.
- **No learned component exists.** The induction module is implemented and
  tested but not connected to a running trace log, so it has processed no real
  data.
- **Constraints do not gate dispatch** (D5), so §6's argument rests on the
  evidence gate alone.
- **The analytical index is non-functional.** `term_idf` in the DuckDB retrieval
  layer raises a type-coercion error; the corpus views load and are verified, the
  ranking path is not. Retrieval in the shipping server is a separate in-process
  matcher.
- **The corpus is not schema-consistent.** Twelve artifacts carry approximately a
  dozen distinct key sets, one retains a corrupted key from an earlier
  substitution defect, none carries a human-readable title, and two use
  human-assigned identifiers where the specification expects a UUID.
- **No performance comparison against a learned router** exists, because no
  learned router exists.
- **No safety evaluation beyond the fixture.** The 2-of-8 false-positive rate is
  the only adversarial datapoint I have, and it is small.

---

## 10. Contributions

Stated conservatively, relative to the prior work surveyed in §2.

1. **A formalisation of the reusable unit as an executable, content-addressed
   artifact**, with the property that identical induced rules deduplicate without
   comparison and renaming does not change identity.
2. **A zero-gradient induction mechanism** that derives both the rule and its
   alias set from recorded interactions, together with an evidence gate
   specified to resist promotion on thin evidence.
3. **Escalation rate as the primary objective**, with the argument that accuracy
   is the wrong metric when a correct answer still costs a request.
4. **A measured characterisation** of the lexical tier and of a semantic tier on
   low-end integrated hardware, including the observation that a scan is
   adequate to ~10⁴ artifacts and that an inverted index is required beyond it.
5. **A first retrieval-quality baseline** that locates the current failure modes
   in manifest authoring rather than in retrieval, and identifies a
   document-frequency weighting defect that is cheap to fix.

I do not claim: novelty over executable skill accumulation; a working learned
model; a measured escalation rate; or that the safety argument is complete.

---

## 11. Future work, in dependency order

1. **Fix the manifest defects** surfaced by §7.2 — scope aliases to actions, add
   the space forms, and correct document-frequency weighting to the resource
   level. Expect this to move the fixture to 100% and to establish how much of
   the remaining problem is authoring rather than retrieval.
2. **Close the induction loop**: write real traces, induce, and re-measure. This
   is the first end-to-end exercise of the learning loop and the first test of
   whether induced aliases are better than authored ones.
3. **Falsify or support C1** with a fixture of 200 or more intents authored
   independently, reporting escalation rate and false-positive rate.
4. **Close D5**, so constraints gate dispatch and the safety argument has its
   intended second pillar.
5. **Only then** evaluate a semantic tier, and only on the residue that survives
   steps 1–3. If the residue is small, the correct decision is to stop.
6. **Define what a correct pattern is.** Every step above assumes an oracle for
   correctness. Self-evaluation without a verifier is the unsolved part, and
   property-based testing against observed outcomes is the current approach.

---

## References

* NSI — *Lifting Traces to Logic: Programmatic Skill Induction with Neuro-Symbolic
  Learning for Long-Horizon Agentic Tasks*, ICML 2026.
* SkillGen — synthesis of auditable skills from trajectories via contrastive
  induction.
* SkillRevise — trace-conditioned skill revision coupling execution evidence with
  general repair knowledge.
* AgentFactory — self-evolving framework through executable subagent accumulation
  and reuse.
* SoK: Agentic Skills — Beyond Tool Use in LLM Agents, 2026.
* *Don't Break the Cache*, arXiv 2601.06007, 2026.
* BAAI — *bge-small-en-v1.5* model card.
* `unia` specification and divergences: `docs/SPEC.md`,
  `docs/ure-specification-formal.md`.
