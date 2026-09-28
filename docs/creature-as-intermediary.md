# The creature as intermediary

A plan, from the current implementation to a creature that sits in the request
path and learns from what passes through it.

This document is a projection, not a description. Where a step is measured it
says so; where it is designed it says that; where I do not know whether it will
work it says that too. The [discussion
record](./DISCUSSION-evolution-and-identity.md) records what has already been
retracted, and this document is written in the expectation that some of it will
be retracted as well.

---

## 1. Where we actually are

Measured, on the current commit.

**The loop runs and closes.** `POST /api/care` tends the pet, records a trace,
re-runs induction over the trace log, and stores the result on the pet. Five
routes, 249 tests, the creature persists across restarts.

**The creature learns and then does nothing with it.** `Pet.learn` is called
after every interaction. `Pet.learned` is read in exactly two places: the
`/api/pet` response and the test assertions. **No interaction is ever routed
through a learned rule.** The creature is a display surface with a memory, not
an intermediary.

**There is no vocabulary at all.** `Care::parse` is a hand-rolled match on four
fixed strings. The client sends one of `"feed"`, `"play"`, `"clean"`,
`"sleep"` or nothing happens. Free text is accepted as a *trace field* for the
evidence gate, and for nothing else. A player cannot say anything the four
literals do not already say.

**There is no external model.** `ExternalApi::call` returns
`format!("External response to '{}' using unia logic.", prompt)`. It is a
string-formatting stub with a provider name on it. There is no HTTP client in
the crate and no OpenAI-compatible surface anywhere. `oc-shim` appears nowhere
in the repository.

**Induced rules are unaddressable.** `induce_all` produces candidates with a
DU-UUID and a signature, and nothing writes them to `patterns/`. A signature
like `SetValue_CheckSense` therefore matches nothing on the next pass, which is
the measured blocker: a creature can learn a primitive sequence and never be
able to call it.

**The bridge and the creature do not know about each other.**
`IntelligenceBridge::request` runs intercept → intervene → evolve against the
registry, the router and the transducer. The pet lives in a separate binary with
no path into that path and no path out of it.

## 2. The one finding that makes this buildable

The two remaining retrieval false positives have been described as unfixable
without a capability check, because they score 0.2197 and 0.2236 against a
weakest true positive of 0.2041. No threshold separates them. That is correct,
and it is also the whole design.

The query is `set the valve brightness to eighty`. It matched `valve-001`. And
`valve-001` declares:

```json
"state_space": {
  "flow_rate": { "type": "float", "range": [0.0, 1.0], "unit": "percentage" },
  "status":    { "type": "enum", "values": ["open", "closed", "fault"] }
}
```

**There is no `brightness`.** The match is provably wrong, and it is wrong in a
way that is a *fact about the artifact* rather than a judgement about a score.
The second false positive, `summarise this document for me` against a
filesystem resource, has the identical shape.

This matters more than a fix, because it gives the creature something to do that
is neither guessing nor refusing. **A contradiction between what was asked and
what the addressed artifact declares is a checkable fact.** It is the same kind
of fact as the convergence equality test — set equality rather than a threshold
— and that is the only reason this design is possible at all. A creature that
stops when the architecture's own declarations contradict the request, and stays
silent otherwise, needs no judgement of its own.

## 2a. Two corrections, after reading the primary sources

Added after [`where-reasoning-comes-from.md`](./where-reasoning-comes-from.md).
Both of these were wrong in this document, and one was wrong in the project's
favour.

**The teacher is not scaffolding. It is the only mechanism known to add
capability.** [RLVR gains are bounded by the base model][neurips]: six popular
algorithms perform similarly, none fully exploits what the base contains, and
distillation from a teacher is what genuinely expands reasoning. Most RLVR gain is
search compression — `pass@k` to `pass@1` — not capability. So Stage 3 is not an
accelerator to be swapped out once the internal matcher works. It is the supply
of new capability, and the internal matcher is what takes over afterwards.
"Independence" means the teacher internalised its sequences, not that the teacher
went away.

**D5 is not the safety argument's second pillar. It is the reward function.**
`constraints` is declared and never evaluated, which under a verifiable-reward
loop is a partly-random verifier — and [random rewards buy most of the
apparent gain on MATH-500][spurious]. Everything the creature learns is a
function of the verifier. D5 therefore moves ahead of all model work, and until
it is closed any learning loop is optimising an unchecked signal.

[neurips]: https://neurips.cc/virtual/2025/poster/119944
[spurious]: https://www.promptfoo.dev/blog/rlvr-explained

## 3. Target

```
        free text, any language
                  │
                  ▼
    ┌───────────────────────────────┐
    │  creature: intervene()        │
    │                               │
    │  1. internal vocabulary model │  surface → candidate primitive sequences
    │  2. external transductible    │  optional; may only *choose among*
    │     (OpenAI-compatible)       │  primitives the artifact declares
    │  3. capability check          │  every proposed term ∈ state_space?
    │  4. contradiction? → STOP     │
    │  5. learned rules first       │
    └───────────────────────────────┘
                  │  (or a question, and nothing else)
                  ▼
        architecture, unchanged from here
                  │
                  ▼
        trace: which of the three paths served this
                  │
                  ▼
        induction → learned rules → next time, path 5
```

The creature is **not** a service in front of the architecture. It is a
participant in the same language as every other artifact: the same primitives,
the same addresses, reachable through the same bridge, renderable in the same
four forms. Making `ca(R)maduci` a first-class `.ure` resource is the point at
which the game and the architecture stop being two subjects.

## 4. Four properties, each of them testable

A design that says the creature "does not interfere" is a promise. These are
checks.

**P1 — Non-interference.** With no learned rules, no external model, and no
contradiction, the creature's output is **byte-identical** to the architecture's
own resolution of the same request. A property test, not a code review.

**P2 — No invention.** The creature can only emit primitives that exist in the
declared vocabulary. A transduction producing an unknown primitive is an error,
never a guess. This is the D6 lesson generalised: `resolve_primitive` returned
`SetValue` for thirteen dead primitives and every unresolvable action, and the
cost was a broadcast dispatched as an assignment with nothing saying so.

**P3 — Contradiction is a fact.** Interception fires only on something checkable
— a term absent from a declared `state_space`, an unmet `constraints` entry, a
learned rule that contradicts an invariant — and never on a score. If a future
change makes interception fire on a threshold, P3 has been violated.

**P4 — Learning changes the path, observably.** A rule induced from N
observations is consulted *before* the models, and the trace records that it was.
Without this there is no escalation rate, and the paper's central figure stays
unmeasurable.

## 5. The "why", in gaming terms

The creature stops and asks. It does not error, and it does not refuse to serve.
The distinction is the whole design: a creature that guesses between two readings
is the D6 bug with better manners.

The shape of the question is fixed by what is actually known:

> *I was asked to set the brightness. This valve has flow and status, and no
> brightness. So either it is not the thing you meant, or I have the wrong idea
> of what it can do. Which one?*

Three properties, and each one is a requirement:

- It **names both branches**, so the player can answer with one word.
- It **states the contradiction as a fact** (`no brightness`), not as a
  confidence score, so the player can check it.
- It **does not execute the action meanwhile.** The pet does not get fed on a
  guess. This is the only place the creature withholds, and it withholds exactly
  where the architecture would have guessed.

The voice is the game's own: the motto is already a question, and the creature
asking one is the same behaviour at a different scale. It should be able to say
this in all four forms — prose for a person, pseudocode or data for a peer.

## 6. Stages, in dependency order

Each stage has an acceptance criterion that can fail.

### Stage −1 — Make the verifier real

Evaluate `constraints`. This is divergence D5 and it precedes everything,
including Stage 0, because a learning loop run against an unchecked verifier
optimises the wrong thing and reports progress while doing it.

*Acceptance:* a manifest declaring `status != 'fault'` is dispatched when the
condition is false and refused when it is true. The corpus's two constrained
actions are the fixtures.

### Stage 0 — A vocabulary, with no model in it

`Care::parse` accepts four literals. Replace it with surface → primitive
sequence over the terms that already exist: the store's IDF-weighted scorer for
candidate artifacts, and `tokenize_id` + `PRIMITIVE_TABLE` for the mapping.

*Acceptance:* a player can say "give it some kibble" and the creature acts, with
no model configured and no network.

**The corpus is this stage's training data, and it is not a separate task.** The
missing corpus — twelve artifacts with zero shared capabilities — blocks the
escalation rate, leaves the cold-start stage with nothing to imitate, and starves
convergence. [Karpathy's method for adding an ability is a task generator][164];
the analogue here is a **manifest generator** emitting artifacts with declared
state spaces and overlapping primitives. One generator unblocks three of the
project's open items, which is why it belongs at Stage 0 and not at the end.

[164]: https://github.com/karpathy/nanochat/discussions/164

*This is not the interesting part and it is first anyway*, because every later
stage needs something to transduce. A vocabulary model with nothing to say is not
a design, it is a table.

### Stage 1 — The contradiction gate

Check every term the intent proposes to set against the addressed artifact's
declared `state_space`. On a contradiction, return the question and execute
nothing.

*Acceptance:* the 20-case fixture reports **20/20 hit@1 with 0 false positives**,
and the two known contradictions each return a question naming the missing term.
The two false positives stop being a threshold problem and become the first
worked example of the creature asking why.

*Risk: low. The data is already there; it is being read but not checked.*

### Stage 2 — The creature becomes the intermediary

A `Mediator` in the request path. `IntelligenceBridge::request` is already
intercept → intervene → evolve and is the natural home. The pet becomes a
first-class `.ure` resource with capabilities `tend`, `neglect`, `sleep`,
`learn`, `explain`, `refuse`, addressed by content address like any other.

*Acceptance:* P1 holds as a property test. `GET /api/pet` becomes
`search("tend the ca maduci")` against a `.ure` manifest. The special-case binary
stops being special.

### Stage 3 — External transduction

One trait, two implementations, so the comparison is real rather than rhetorical:

- **internal** — the Stage 0 matcher. Default, offline, no authority.
- **external** — an OpenAI-compatible `POST /v1/chat/completions`, which is what
  makes `oc-shim`, a local server, or an agent reachable without a bespoke client.

The external model **transduces only**. It maps surface onto a primitive
sequence drawn from the artifact's declared actions. It cannot introduce a
primitive (P2), it cannot see or write architecture state, and its output is
validated against the declaration before anything acts on it. It is a supplier
with no authority, which is the only safe way to add one.

**This is the distillation stage and it is the only one that adds capability.**
Per [the NeurIPS analysis][neurips], RLVR is bounded by the base model and
distillation is what expands it. So the teacher's job is to produce sequences the
creature cannot currently produce, and the escalation rate measures how much of
the teacher's repertoire has moved across. A teacher that only ever confirms
what the creature already does is not a teacher; it is a slower path to the same
place.

*Acceptance:* both implementations satisfy P1–P3 identically; the external one is
additionally shown to be unable to emit an undeclared primitive under adversarial
prompting. Without a network, the internal one is used and nothing else changes.

*This is the least certain stage.* Constrained decoding to a closed grammar is
achievable today, but "the model chose a primitive that does not exist" is a
failure mode with a long history, and P2 is the only thing standing between it
and the architecture.

### Stage 4 — Evolution from the game

`src/session.rs` has been blocked on exactly this. Its own doc comment says a
lesson becomes a trace "by one function call" once the vocabulary is defined, and
it deliberately emits nothing until then. Stage 0 defines the vocabulary.

So: a contradiction, the player's answer, and the answer is a trace. The creature
is corrected by being asked, and the correction is evidence like any other. Then
the induced rules become addressable — written back to `patterns/`, or consulted
directly — so `SetValue_CheckSense` can be called on the next pass.

*Acceptance:* P4 holds. A creature that has been corrected N times routes the
N+1th request through a learned rule, the trace says so, and the escalation rate
is computable from the trace log rather than asserted.

### Stage 5 — Two creatures

Convergence over the existing relay, mediated by the creature. Handover: it
arrives with its rules *and* its refusals, so the receiving player can read what
the creature would not do and why. That is the unsettling part and it is the
point.

*Acceptance:* the motto's answer is produced by the system rather than asserted,
and `firstness()` returns `Neither` with the address both reached.

## 7. Where I am uncertain

Stated plainly, because §9 of the paper is load-bearing and this document should
be read the same way.

- **Stage 1 is the load-bearing one and it is small.** Everything after it is
  reachable or not depending on choices not yet made. If it works, the design is
  sound; if it does not, the rest is decoration.
- **Stage 3's containment is an assumption about external models, not a
  measurement.** P2 constrains what reaches the architecture. It does not
  constrain what the model was influenced by, and a transductible that has been
  persuaded to emit a *valid but wrong* primitive is not caught by any of P1–P4.
  That needs a capability check (Stage 1) to matter, which is why Stage 1 is not
  optional and not merely a retrieval improvement.
- **A contradiction gate that is too eager is worse than none.** It withholds
  service on a false reading, and the player experiences that as a broken game.
  The gate must fire on declared facts only; anything probabilistic defeats it.
- **Escalation rate will not be a large number.** Twelve artifacts with zero
  shared capabilities cannot produce one. Stages 0–4 make it *measurable*; they
  do not make it impressive, and the corpus is still the binding constraint.
- **Whether distillation transfers to artifacts rather than weights.** Every
  result cited here is about models. Moving a behaviour into a content-addressed
  artifact and then serving it without the teacher has no published analogue I
  could find. That gap is the research contribution and it is also the largest
  risk in the plan.
- **Whether a 16-primitive vocabulary is a feature or a limit depends on the
  axis, and I conflated them.** For the reliability of each step it is the
  enabling structure — small steps are dependable steps, which is the whole
  decomposition argument. For open-domain coverage it is a hard limit. The
  reasoning claim depends on the first; it does not survive the second.
- **The external model is not required for any of this.** Everything through
  Stage 4 works offline. Adding one is a question of manners and cost, not of
  capability, and the design should not come apart if the answer is never.

## 8. Decisions that are not mine

1. **May the creature withhold service?** This document says yes, on a declared
   contradiction only, and never on a score. The alternative — execute and *then*
   apologise — is more forgiving and makes the D6 class of bug unrecoverable,
   because the damage is already done.
2. **Is the creature one creature or the most evolved of many?** The design
   assumes a single addressable creature that others can converge with. If it
   should instead be a *role* that any sufficiently-evolved artifact can take,
   Stage 2 is smaller and Stage 5 is larger.
3. **What may an external model be told?** P2 constrains its output. Whether it
   may see the corpus, the address space, or the player's history is a privacy
   question the primitive protocol does not answer, and it is a question about
   the *user*, not about the architecture.
