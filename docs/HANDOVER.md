# Session handover

**Where unia is, and the plan to reach the vision.** Written at the end of a
session that took the test count from 482 to **558** and closed the loop that was
missing.

**How to read the failures in this document.** Roughly a third of what follows is
bugs I introduced and the tests caught. They are kept because they are the most
useful thing here — every one of them is a case where something plausible, closed,
and self-consistent was wrong, and the pattern across all of them is the finding.

---

## 1. The vision, in one place

A creature with an inheritance, whose capabilities are certitudes. Open to receive
and offer those capabilities to other nuclei and to systems outside it. Its
knowledge is transposed from the implementation into patterns, and those patterns
evolve: from nucleused primitives toward more complex shapes, by melting, by
recovering, and by reshaping, **keeping the shape through all three and staying
productive**. And it learns to lose on purpose — spending resources freed by
achieved certitudes to explore matterns it has not reached, so that a loss becomes
a reason rather than a loss.

The same machinery runs at several magnitudes at once. One predicate governs all
of them — `reachable_scopes(credit)` — so Circle↔Global is a *dimension* read at
several scales rather than a pair of modes. What makes it a factory is its own
factorisation, and the measure of that is a **ratio**: a transposed module must be
shorter than its source, and the ratio must be measured.

**The acceptance criterion is stated in advance so it cannot move afterwards**
(`docs/unia-target-spec.md` §5). A ratio of 1.0 is a failure that looks like a
success.

---

## 2. What is built and verified

| | |
|---|---|
| **second alphabet** for the code, separate from the sixteen runtime verbs | `edit.rs` — `SourceAct`, and a test asserting the alphabets never merge |
| **a caller-authored corpus** with measured authorship and escalation | 14 traces, 5 confirmed capabilities, ~0.36 capability/act |
| **a linker** that binds a *capability* to a resource that declares it, breakable while running | `link.rs` — typed refusals, `promote`, `Inflight` |
| **the quantics**, with the sign | `quantum/` — signed `Pauli`, 2ⁿ witness, a real Bell pair |
| **a model boundary that cannot lie** | `slm/` — `Measured` exists, only an engine fills it |
| **the route for doubt out**, end to end over a socket | `doubt.rs`, `resolve.rs` — no `reqwest` |
| **an intent mapper that cannot turn a shrug into an action** | `slm_mapper.rs` — three typed refusals, menu enforced |
| **the learning loop, closed** | `loop_train.rs` — propose → witness → promote, nobody typing |
| **the exploration reserve** | `explore.rs` — the fourth movement, so loss can be deliberate |

Commits this session: `0aac9bf`, `3418f62`, `baa643f`, `e41c550`, `2256dd3`,
`cdd2bc4`, `e90089d`, `574df3d`, `6805937`, `f7a02fd`, `9b74f79`, `eef300d`,
`10a32da`, `ec43d93`.

---

## 3. The failures, which are the information

**Every one of these is a case where something plausible and internally consistent
was wrong.** In three of them the structure *closed* — and closure is not
evidence.

1. **The escalation metric rated pure repetition a perfect escalator.**
   `capabilities / distinct_attempts` gives 1.0 for fifty-one identical edits and
   1.0 for twelve distinct ones, because the deduplicated denominator does not
   penalise repetition. The denominator is now every act.
2. **A fabricated benchmark.** `tests/benchmark_tests.rs` timed one dispatch
   against `thread::sleep(800ms) + thread::sleep(50ms)`, printed the quotient as
   an efficiency gain, and asserted `gain > 1.0`. The numerator was a duration the
   test slept through. `docs/PROVENANCE.md` cited it as "empirical benchmarks".
   Removed rather than repaired, and the retraction is in `PROVENANCE.md`.
3. **`Pauli::mul` used the symmetric anticommutation as the product sign.** Only
   one cross term belongs there. Every non-commuting product was wrong — **and
   associativity still held**, so the closure test passed throughout. *Closure is
   not evidence.*
4. **A conjugation with an invented second term** made `XXX` read as `-X`. Caught
   by the four-relation test.
5. **CNOT is not a Pauli**, and my comment claiming otherwise was the bug: it
   produced `-|11⟩` from `|10⟩`. The test failed on the sign.
6. **A Bell pair is perfectly correlated.** I asserted the half-agreement of
   |01⟩+|10⟩, the anti-correlated pair, and read 10000/10000 as a broken state.
   It was a correct state and a wrong expectation.
7. **`-0.0` formats as `-0.0000`**, so comparing an amplitude by rendering it
   fails on an identity that holds. Every identity now goes through a tolerance.
8. **`MockSlm` fabricated `tokens_used: 450` and `reasoning_steps: 3`** for a
   `format!` call, and a test asserted `reasoning_steps > 1` — a suite whose only
   content was confirming the fabrication was *good*. Fixed by making the claim
   unrepresentable: `Measured` exists and only an engine fills it.
9. **`SemanticSLM` returned any reply as the action id.** A decline, an invented
   `close_valve_v2`, and a paraphrase all "succeeded". `SlmOptions` was a struct no
   call site could fill, so `temperature` was a constant with extra steps. And two
   types named `SlmResponse` existed, neither matching the other.
10. **The mapper was case-sensitive while the bridge was explicitly
    case-insensitive** — the crate held two incompatible answers to "what is an
    action id", and the mapper's was the stricter one, so it refused correct
    intents. Fixed by normalising to the declared spelling: `EMERGENCY_SHUTDOWN` is
    not the model inventing an action, and reporting it as `OffMenu` sends a
    debugger looking for a model problem instead of a one-character fix.
11. **My fixtures used one intent throughout**, so nothing cleared
    `MIN_OBSERVATIONS`. Read as "the loop promotes nothing"; the truth was "the
    fixture never cleared the bar". A misdiagnosis I would have shipped.
12. **I hardcoded `Content-Length: 30` for a 22-byte body** and the truncation
    guard refused the reply — the defect it exists to prevent, on me.
13. **The exploration cost was a fraction of the reserve**, which shrinks as the
    reserve shrinks, so exploration approached the floor asymptotically and never
    terminated. *A budget whose unit of spending vanishes as the budget vanishes
    is not a budget.*
14. **Twice the code was right and I was wrong**: exploration ends *above* the
    floor, and a group of only-losses has no evidence rather than ratio 0.0.

**And one that is not my bug, which is the most important of all:**

15. **`clean()` has zero callers outside its own module** — with a *proved
    fixpoint* and a convergence test. Six generations, one address, zero survivors
    naming a culled one. All true, and never invoked. The pattern that produced
    the five stubs is the same one at a higher level: **every loop here is closed
    in measurement and open in behaviour.** Each component is tested against its
    own contract and nothing verifies that one's output reaches the next one's
    input. That is a missing *category* of test, not missing tests.

**Two structural consequences, both measured:**

- **`learn()` is called only from tests.** So in the running server quants are
  debited on every act and never credited: the economy is a **drift, not a loop**.
  That is the thesis stated as a runtime fact — an act spends, induction pays, and
  in the deployed path the paying never happens.
- **CORRECTION — the writer and the readers use different key spaces.** I asserted
  twice that "nothing reads a champion", from a `grep | head -8` that truncated
  the two production readers. They exist:
  `bridge::PrimitiveBridge::request` (line 55) and
  `fluid::factory::resolve_best_actuator` (line 120) both call `get_champion`.
  **So the reader is not missing. The key is.**
  `loop_train::run` writes the champion under `c.signature` — a primitive
  signature like `ReplaceSpan_src/a.rs_1_1` — while `bridge::request` reads under
  `&prompt` (free text) and `resolve_best_actuator` reads under `&capability`.
  **A signature is never a prompt, so the write and the read can never meet.** The
  promotion is recorded and the lookup is performed and nothing connects them, and
  the failure mode is silent: no champion found, fall through to the manifest, the
  system behaves exactly as it did before the loop existed.

---

## 4. The plan, in order

Each step names its **exit**, and an exit is a test or a measured number. A step
whose exit cannot be stated is not a step.

### Phase 1 — Agree the capability key space. **Done.**

The readers were never missing: `bridge::PrimitiveBridge::request` (line 55) and
`fluid::factory::resolve_best_actuator` (line 120) both called `get_champion`. The
*keys* disagreed — the loop wrote under a primitive signature, they read under free
text — and **the failure was silent**, so the loop could run perfectly and change
nothing.

`ActuatorRegistry::capability_key` now normalises every champion key through
`bridge::primitive::tokenize_id` — the normaliser that already exists, because
`resolve_primitive` is deliberately indifferent to the form of an id and a champion
table indifferent to nothing would contradict the resolver in the same crate.
`champion_keys()` is a diagnostic: it is the first thing to read when a promotion
appears to have had no effect.

**Exit, and it now exists:** `a_promoted_signature_is_reachable_by_the_forms_
readers_use` takes a signature the *writer* produces and asks for it the way a
*reader* would — lower-cased, underscored, upper-cased — and requires a hit each
time. Its negative twin, `a_signature_that_was_never_promoted_still_misses`,
exists because the positive alone would pass against a registry that answered
everything.

**Still open within Phase 1:** whether the *hot* path is covered.
`map_intent` is what `unia-camaduci` calls, and `PrimitiveBridge::request` is a
different entry point. The key space is now shared; whether a promoted mattern is
reachable through the path the running server actually takes is not yet tested.

**Unblocks, all of it at once:** the creature consulting its own record (it already
has `learned: Vec<LearnedRule>` and already has a test proving its address moves
when taught); the learning loop having a payoff; the LPMM serving a learned skill
when called as a peer; and one bridge per act gained from witnessed evidence,
which is *elaboration* — evolution without composition.

**And the hidden part is why it must be tested rather than inspected:** a key
mismatch produces *no error*. The lookup returns `None`, the code falls through to
the manifest, and the system behaves exactly as it did before the loop existed. So
this gap is invisible to every test that checks a component, which is §7's rule
again — and it is invisible to me until now, for the same reason.

### Phase 2 — The factory, and the ratio.

Transpose one module. `src/clean.rs` first: ~500 lines, small surface, already
pinned by tests. Report the matterns the semantics decomposes into, the
occurrences of each, and the **ratio**.

**Exit:** the ratio measured, whatever it is. If it is ~1.5, transcription is not
compression and that is a better finding than a corpus that reproduces Rust in a
worse format.

**Then:** make "the ratio went down" a `Production` event, which puts the
acceptance criterion *inside* the economy instead of beside it. A refactor that
makes one mattern serve eleven sites earns credit; one that transposes a module
into the same number of lines earns none.

### Phase 3 — The self-model.

Three operations exist: `skeleton()` (melting, lossy by design), `rematter`
(recovering, fixpoint proved), re-stamp in `induce` (reshaping). The genome exists
— `Pet::address()`, content-derived, so a rename leaves it alone. The self-model
exists — `learned`.

**Three falsifiable criteria, none written:**

1. **The round trip preserves the address.** `Declared::of(manifest)` before and
   after melt-and-react gives the same address. If melting a driver to a
   capability and reacting it back moves it, the melt lost structure and nothing
   reports it.
2. **Reacting is actuator-agnostic.** Two structurally different drivers declaring
   the same capability are interchangeable to a demand. `Linker::promote` performs
   the swap; nobody has asserted the *demand is indifferent* to it, which is what
   makes the swap a melt rather than a reconfiguration.
3. **Abstracting is free and acting is not.** `Declared::of` is a pure function and
   cannot cost anything; dispatch is an act and pays 0.15·(quants−floor). Already
   true by construction, untested — and the failure has a *predictable
   consequence*: if melting ever cost power, demand would bind to the nearest
   concrete actuator instead of the best one. A consequence makes it a prediction,
   and a prediction makes it a test.

**The honest cost, stated not hidden:** a self-model maintained by the mechanism
that cleans it is **convergent and lossy, not cumulative**. What the creature
re-matters, it forgets it derived. That is not a defect in either half — the
fixpoint is only reachable because invention history is cut — but the genome is a
*stable* identity, not a growing history.

### Phase 4 — The economics, in play.

`explore.rs` is a library function with no caller. Two things:

- **Wire it into actual play.** A declared reserve spent on acts with no prior
  evidence, so a trajectory exists where the creature explores, loses, and the loss
  is not a miss.
- **Make the novelty check read the log.** `may_explore` currently trusts the
  caller's `no_prior_evidence` flag, and a caller that lies gets a free act.

**Exit:** a run where exploration happened, the balance fell, and the exploratory
losses are absent from the failure ratio. **This is the flag that has been red
since phase 2** — harmony as reachable equilibrium — and it was red because a
system that cannot afford to lose can only reach a local maximum. The mechanism
was missing; the flag is now blocked on nothing but the wiring.

### Phase 5 — 2a: invite. *Probably an afternoon, not a fork.*

unia already has the machinery for executing a description: `map_intent` plus
`dispatch`, every day, successfully. It is a creature that reads descriptions and
acts. **It cannot be pointed at a description that arrived from elsewhere.** That
is a door with no handle, not a wall.

**Exit:** a route that receives a peer manifest, registers it, and dispatches. The
market half already works — `broadcast_actuator`, `published`,
`discover_resources`, credit-gated. Only the delivery half is missing.

### Phase 6 — The fork, then 2b: compose.

**This one is the author's, and it is not effort.** Either the target preserves
ownership — `unia` is a real compiler target and Rust's central guarantee survives
the port — or it is a heap with no aliasing promises, **and the one reason to
compile Rust to it is gone.** The tree is already the second and it was never
written down: `Arc<Mutex<HashMap<String, HashMap<String, String>>>>` is a heap
without ownership.

2b is "one trick calling another trick": a step genuinely invoking another act.
That is the closed `Signature` — `primitives.join("_")`, no call, no return, no
locals — so **there is no calling convention because there is no notion of a
callable thing.** `Production::credit()` is six lines and lowers to *nothing*.

**Closing Phases 1–5 is worth doing either way**, because a better actor is a
better actor. **The ceiling is the fork.**

---

## 5. Blocked on other people

- **The 200-query fixture.** Must be authored by someone who has **not** read
  `src/bridge/primitive.rs` or `src/meet.rs`. Whoever writes it writes in the
  matcher's vocabulary, so the escalation rate becomes a measurement of their
  assumptions. This gates the paper's central number and nothing here substitutes
  for it.
- **Trademark** for `ca(R)maduci` and the `(R)`, and `euni.me`, for anything public.
- **Lattice or residuated** (phase 0 of the training plan). Import cost is high
  enough that it must be decided before seeding.
- **Performance is one machine.** `docs/perf-unia-vs-lua.md` has no repetition
  protocol and no multi-machine figures. The `11µs` breakdown is a reading of the
  code, not a profile.

---

## 5a. What happened after this document was first written

Recorded here rather than in a new file, so there is one handover.

**Task 1 of the delegation plan ran: a 200-query fixture was authored by a fresh
model whose only brief was a path to `docs/FIXTURE-SPEC.md`.** One pass, no
iteration. Verified on receipt: 200 rows, buckets exactly 80/60/30/20/10, ids
contiguous, 200 unique intents, 30 refusals, 16 multi-accept rows. Structurally
sound.

**And it found two bugs in the specification, both mine.**

1. `docs/SPEC.md` was on the **allowed** list. Its §3.2–3.3 describe the matching
   algorithm in prose and §7 names `SemanticMapper::compute_score` — and my own
   worked example in the self-declaration section used it as an example of a clean
   read-set. **A spec that leaks through its own example is worse than no spec**,
   because following it is the correct behaviour and the result is still invalid.
2. The format had **no field naming what `accepts` is anchored to**, so the author
   reconstructed an eight-act menu from documentation. Now `menu_ref`, and the
   harness must assert the menu matches the manifest and refuse otherwise.

**Then the menu was checked against the tree, and four of the eight acts do not
exist.** `get_status`, `check_health`, `give_medicine`, `set_hunger` appear
nowhere in `src/` or `patterns/`, and `creature-001` is not a manifest — there
are exactly two, `filesystem.ure` and `smart_valve.ure`.

The real menu is four acts and they are `Care` enum variants at
`src/camaduci.rs:272`, **so they are not in any manifest.** That reframes the
finding:

> **A creature cannot declare its own capabilities to a peer, because its
> capabilities are not in the artefact it would declare them in.**

The market exists. `Handover`/`receive` exists. The capability set is a field. And
none of it can carry four acts. **The offering axis of the vision has no payload
for the creature it was designed around** — and a fixture is what made it visible,
because scoring intents requires declared acts and there was none.

**So Phase 1 is not the next thing.** Putting the creature's four acts into a
`.ure` is, because until that exists `menu::check` refuses — correctly — and
every fixture authored in the meantime is another draft. `src/menu.rs` holds the
menu as data with its provenance and the gate, and the gate's tests are mostly
refusals, because a gate that has never refused anything has not been tested.

**And a fourth false claim was found in the README on this commit:** it stated the
wasm32 target does not build and its CI job is red. It has built for many commits.
The same shape as everything above, in the public face.

## 6. Three things that look like progress and are not

Each has already been mistaken once here.

- **A corpus that grew once is not a trajectory.** Phase 2 grew one, by hand,
  through a fixture. One observation is not a trend.
- **More matterns is not more capability.** A new address for a known act is a
  *transfer*. Escalation is per **signature** so the generated corpus — which
  escalates in reach and in no capability at all — reports honestly.
- **A log that reproduces the source reproduces nothing about it.** `SourceAct` is
  span-level, so the corpus is a derivation. A derivation is not a model, and that
  is why the ratio is the acceptance criterion and not a nice-to-have.

---

## 6a. Delegation

`docs/DELEGATION-PLAN.md` — four tasks handed to another reasoning model: author
the fixture, propose the transposition counting method, adversarially review this
session's diff, and propose what `Trace::succeeded` should be witnessed by. Each
with its read-set, its exclusions, and what verifies it.

**The load-bearing idea: the read-set is the only lever that creates
independence.** A model's context is writable in a way a person's is not, so
delegation is the construction of a specific audited context — and a deliverable's
credibility comes from what the context was *excluded* from. Which is why the
briefs are **paths to a document and nothing else**: if I describe the matcher in
my own words, I have contaminated the model through the brief.

**A stronger model is the right call for the open-ended tasks and the wrong call
for anything with a verdict.** Every one of the fourteen failures in §3 was a
fluent, internally consistent, wrong answer, and three of them *closed as
algebra*. So all four tasks are authoring or method, and **not one is a
judgement** — a delegated judgement is a score nobody computed, which is the
`MockSlm` defect with a reasoning model attached and strictly harder to spot,
because the fabricated number would be plausible.

---

## 7. The standing rule, and the one that is missing

**The rule that has held:** assert what is true and say so. Magic thresholds,
fixtures that disable the feature under test, hash-order coincidences, and numbers
nobody measured are the recurring causes of the failures in §3.

**The rule the repository does not have, and §3.15 is why:** *a component tested
against its own contract passes while the loop it describes never turns.* Four
pieces of the learning loop each had thorough tests and one wire between them
did not exist. `clean()` had a proved fixpoint and no caller. So the next test
category to write is not "does this function work" but **"does this function's
output reach that one's input"** — and Phase 1's exit is the first of them.
