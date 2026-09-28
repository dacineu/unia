# TODO

Open items, ordered by what unblocks the most. Status reflects the repository,
not intentions.

---

## What is not solved

The list below is the one to read first. It is not the whole TODO — that is 700
lines organised by subsystem, and a subsystem list cannot tell you which open
items would falsify the paper and which are chores. This one is ordered by that,
and every entry names the measurement or the decision that would close it.

Grouped by *why* it is open, because the four groups need different things from
you and confusing them is how a project spends a year on the wrong one.

### A. Undecidable by me. Yours to answer.

1. **The fork at `docs/unia-target-spec.md` §6.** Either the target preserves
   ownership and `unia` is a genuine compiler target with Rust's central
   guarantee intact, or it is a virtual machine with a heap and **the one reason
   to compile Rust to it is gone.** The tree is already the second and it was
   never written down: `Arc<Mutex<HashMap<String, HashMap<String, String>>>>` is
   a heap without ownership. Decides what this project is; two projects with the
   same name.
2. **What happens to an act in flight when its resource is demoted.** *Complete*
   (default; the old resource must stay alive until the count is zero), *refuse*,
   or *reissue* (needs a replayable act, and per §2.3 a closed `Signature` is not
   one). "Unreachable contract exit choosable by all parties." `Linker::promote`
   reports the count rather than choosing, deliberately.
3. **Who authors the 200-query fixture.** Someone who has not read
   `src/bridge/primitive.rs` or `src/meet.rs`. Whoever writes it writes in the
   matcher's vocabulary, so the escalation rate becomes a measurement of their
   assumptions. **This gates the paper's central number.**
4. **The name.** Trademark clearance for `ca(R)maduci` and the `(R)`, and
   `euni.me` ownership, before anything public.

### B. Blocked on a measurement nobody has made

5. **Where the 11µs of an act goes.** Measured: whole act ~14,300 ns, of which
   ~11,000 (77%) is `map_intent` and ~3,300 (23%) is dispatch. The breakdown
   *inside* that 11µs is a reading of the code, not a profile, and my readings
   have been wrong about a dozen times this session. A profiler, one afternoon.
   **This is the cheapest high-value thing on the page.**
6. **The transposition ratio.** A transposed module must be **shorter than its
   source** and the ratio measured, stated in advance so it cannot move. First
   candidate `src/clean.rs`. Nobody has measured it. Blocked on (1), not effort.
7. **An escalation rate on a corpus nobody hand-authored.** Demonstrated once,
   on a real corpus, through a test fixture. Not a trajectory, so
   "harmony as reachable equilibrium" stays red and correctly so.
8. **Convergence on two competing candidates.** Re-mattering is proved to reach a
   fixpoint; convergence *between* two is not built.
9. **Whether induced matterns can be assembled into a working system.** The
   corpus exists. Nothing composes it. The gap between a corpus and a system is
   unmeasured and may be impassable.
10. **Reproducible performance figures.** `docs/perf-unia-vs-lua.md` reports one
    machine. No multi-machine, no `criterion`, no repetition protocol.

### C. Contradictions in the tree, right now

11. **~~`Superposition` and `Entangle` were names, not operations.~~ Cleared.**
   Both fell through to `Pulse` with `arguments: HashMap::new()` and a comment
   saying the arguments ought to be extracted from the op. They were not, so the
   operand was dropped and a request to superpose two qubits was an
   unparameterised pulse computing nothing — the only two names in the vocabulary
   asserting something the code did not do. `src/quantum/mod.rs` now builds the
   operand into `arguments`. **The lesson is the one the signless reduction taught
   twice, and this is the third instance:** a plausible abstraction, closed under
   composition, describing the wrong thing.
12. **`MockSlm` reports `tokens_used: 450` and `reasoning_steps: 3` for a
   `format!` call.** A mock that fabricates the measurements a real inference
   engine would take, in fields named after those measurements — the same defect
   as the three `MetaActuatorType` stubs, and worse, because the demos run on it.
   Either the fields go or the mock stops claiming them.
13. **~~`MockSlm` fabricated measurements.~~ Cleared.** It reported
    `tokens_used: 450` and `reasoning_steps: 3` for a `format!` call, and a test
    asserted `reasoning_steps > 1` — a suite whose only content was confirming
    the fabrication was good. **The fix is not honest numbers, because there are
    none: there is no engine.** `Measured` now exists and only a real engine can
    populate it, so a mock returns `None` and the claim is *unrepresentable*
    rather than merely false. A budget check against a mock disappeared instead
    of passing, which is correct: it had been comparing two fabricated
    quantities.
14. **~~The route for doubt out did not exist.~~ Built; native transport done,
    wasm stubbed honestly.** `src/doubt.rs` has the three things a bare HTTP call
    lacks: doubt is a value carrying *why* it was unanswerable; a consultation is
    a trace with a signature, so the engine is a capability subject to `clean` and
    induction and **never the witness** (`succeeded` is `false` whatever came
    back); and a refusal is a recorded failed trace, not an absence, so a system
    cannot learn to work without asking. `src/resolve.rs` adds the native
    transport — **no `reqwest`**, because `unia-camaduci` already established the
    house style of speaking HTTP over `std::net::TcpStream` and adding a TLS
    stack and an async runtime for one POST is a runtime, which target-spec §3
    lists as *missing* and this is not the place to add one by accident.
15. **The wasm resolver is a stub that says so, and that is the honest state.**
    `Resolver::ask` is synchronous and `web_sys::fetch` is not, so `WebResolver`
    returns `Err("the wasm transport needs an async boundary this trait does not
    have")` rather than pretending. **The fix is a decision, not an effort: either
    `Resolver::ask` becomes async, which propagates through `doubt::consult` and
    every caller, or the browser path gets a blocking shim, which a single-threaded
    wasm event loop cannot honestly provide.** The trait being synchronous is the
    actual obstacle and it was chosen to keep the native path simple.
16. **~~`SemanticSLM` could not tell an answer from a shrug.~~ Closed, and the
    seam moved.** It returned `response.trim()`, so *any* reply became the action
    id: a decline, an invented `close_valve_v2`, or a paraphrase all "succeeded",
    and `available_actions` was used only to write the prompt. The injected
    `F: Fn(String, String) -> Result<String, String>` also meant `SlmOptions` was
    a struct no call site could fill — `temperature` and `num_predict` were
    declared and never sent — and that a model's decline and a dead network were
    the *same* `Err`. Now: an answer must be exactly one of the declared ids or
    it is `Mapped::Refused` with the model's own words kept (`OffMenu` for an
    invented action, `NotAnId` for prose, `Transport` for a dead network — three
    different events); the fetch receives a whole `SlmRequest` and returns a whole
    `Reply`; and the duplicate `SlmResponse` that lived beside the real one in
    `src/slm/` is gone.
17. **The route runs end to end; there is still no model.** One test now drives
    doubt → signature → `HttpResolver` → a real `TcpListener` on a real port →
    `Reply` → `judge` → trace → ledger, with nothing mocked. Every link that had
    no caller three commits ago is exercised by it. The only absent piece is a
    model, and a responder cannot substitute for one: it reports
    `tokens_used: None` and names itself `reference-responder`, which is the
    point — **a `Reply` with no measured cost is the honest shape for something
    that ran no inference**, and the test is what proves the field means what the
    type says. What is still missing: Closed the
    half that was a real inconsistency: the mapper's menu match was
    case-*sensitive* while `resolve_primitive` and `map_intent` are explicitly
    indifferent ("the format does not constrain the casing of an action id"), so
    the crate held two answers to "what is an action id" and the mapper's was the
    stricter one — refusing correct intents. Matching is now case-insensitive and
    **normalises to the declared spelling**, because reporting `EMERGENCY_SHUTDOWN`
    as `OffMenu` would say the model invented an action and send a debugger
    looking for a model problem instead of a one-character fix. The other half
    stands: The transport
    can now reach one; no `SemanticSLM` is constructed in `src/`, `tests/` or
    `examples/`. A `Resolver` is a transport, not a peer — naming a model honestly
    means a peer can tell which engine answered, and `Reply::tokens_used` plus
    `slm::Measured` are the fields that would carry its own report.
18. **`Resolver::ask` is synchronous and `web_sys::fetch` is not.** The wasm
    transport returns an error naming that rather than pretending. Either `ask`
    becomes async — which propagates through `doubt::consult` and every caller —
    or the browser gets a blocking shim a single-threaded event loop cannot
    honestly provide. **The synchronous trait is the obstacle, and I chose it to
    keep the native path simple.**
15. **A shadow is fan-out
14. **A shadow is fan-out and hotswap is a promotion, and both are implemented.**
    `UpaDispatcher::shadow_routes` is commented "for hotswapping" but `route()`
    delivers to primary **and** shadow — for a counter, the act happens twice.
    `Linker::promote` promotes instead. The old behaviour is not removed. The two
    disagree and only one is right.
12. **The generated corpus is denormalised ~4.6×.** Rewriting it is 20× and
    re-resolving is 21× against a 9.6% quality ceiling. Nobody has decided
    whether `resolve` should, and "never use a logic/radix converter" says it
    should not.
13. **`DU-UUID` is content-derived and labelled v4.**
14. **The AES-GCM fixed nonce** is a confidentiality limitation, documented
    nowhere in the security material.
15. **A manifest may name an action `SetValue`.** Nothing stops it; the two
    vocabularies share one string space and the linker's only defence is looking
    in the right field. I wrote a test for this and deleted it — it asserted
    `true || true`. The real check is a name-space assertion over every manifest
    in `tests/fixtures/`.

### D. Impossible with the current vocabulary, not merely unfinished

16. **The sixteen verbs cannot express a program.** No binding, no definition, no
    call, no allocation, no recursion, no emission. `Signature` is
    `primitives.join("_")` — a **closed** sequence, so there is no calling
    convention because there is no notion of a callable thing. `credit()` is six
    lines and lowers to *nothing*. **This caps what self-training can ever
    produce and it is a design decision, not a missing feature.**
17. **A second alphabet exists but is span-level.** `SourceAct` is
    `ReplaceSpan`/`InsertSpan`/`DeleteSpan`/`AddTest`/`RunTests`. A log that
    reproduces the source reproduces nothing *about* the source, so it is a
    derivation, not a model.
18. **The creature knows four acts.** `new_signatures` saturates at four, so the
    economy equilibrium sits below the peak.
19. **Three of five `MetaActuatorType` variants report work not done** — a
    `Mutator` returning `"EVOLVED"` and a fabricated `"+15%"`, a `Synthesizer`
    returning `HYBRID_CREATED` with a fresh `Uuid` and no artifact, a `Distiller`
    returning `"10:1"`. The self-training machinery is the most declared and least
    built part of the system.
20. **`Action.items` 1/1** unimplemented. **`examples/camaduci.rs`**
    reimplements the pet rather than using `unia::camaduci`. **Lattice vs
    residuated** (phase 0) undecided, and importing it is expensive enough that
    it must be decided before seeding.
21. **Immutability: unanswered, and the intuition leads somewhere else.**
    The user-visible proposal was "JSON immutable, and unia mutates immutate
    itself to adapt". `rematter(&LearnedRule) -> LearnedRule` **already is** that:
    self-modification by value replacement, same signature, new address, evidence
    discarded, fixpoint proved, and a holder of the old rule is not corrupted.
    `skeleton()` + `DuUuid` is already the immutable content-addressed identity.
    So the *architecture* is ready.
    But **immutability would not make this faster.** The 77% is
    `to_lowercase()` per call, per action, and two `HashSet<String>` per
    candidate — allocation and re-tokenisation, not aliasing. Marking the
    manifest `&self` changes none of it, and full immutability *without*
    structural sharing is more allocation. There is **no persistent data
    structure in the dependencies** (`im`, `rpds`, `arc-swap`: none), so the
    sharing machinery the idea depends on is not here.
    What the intuition actually licenses is a **memoisation keyed on content
    address** — `(resource_id, intent) -> PrimitivePacket`, invalidated when the
    resource's address changes — and the addressing machinery for that already
    exists. Blocked on (5).

---

## The exploration reserve — the fourth movement

The economy had exactly two movements: `apply_economy` **debits**
`0.15·(quants−floor)` on an act, and `learn` **credits** `Production` for
confirmed novelty. So a **deliberate loss** — an act run expecting to fail, to
gather evidence about a shape not yet known — had nowhere to be paid from. It was
a debit with no reachable credit, the rational strategy was never to try one, and
the economy forbade exploration for the worst available reason: it was
indistinguishable from waste.

**And it was worse than a gap, because exploration and induction were in direct
conflict.** `is_evidentiary` requires `failure_ratio ≤ MAX_FAILURE_RATIO`, and
failure is the only signal induction has. Every exploratory loss raised the
failure ratio of the very group it was exploring, and could disqualify it from
ever becoming a mattern. *The system was being asked to pay for losses that the
thing learning from those losses used as evidence against them.*

**Built in `src/explore.rs`.** `EXPLORATION_SHARE = 0.15` of the headroom is the
reserve; `EXPLORATION_COST = 0.02` flat is one experiment; an act is exploration
only when its signature has **no prior evidence**; and the credit is the zero
`Production`, whatever the witness said. Exploration is affordable and unbounded
exploration is not.

**`induce::group_traces` no longer counts an exploratory loss as a miss** —
marked by an `explore` intent prefix, so the declaration is visible in the log
rather than hidden in a field the persisted format does not have.

**Three things this is not, each a test:**

- **Not a discount.** A cheaper act is still an act; the problem was never the
  price, it was that no debit can be earned back.
- **Not an amnesty.** Exempt from the *count* is not counted as a *success*: a
  group that only ever lost is still not a mattern. Reaching repeatedly proves
  nothing; reaching repeatedly **and winning at least once** does.
- **Not free of the floor.** Exploration stops when the reserve can no longer hold
  one experiment, which is *above* the floor — so a creature that explored itself
  down could still act. The first version charged a fraction of the reserve, which
  shrinks as the reserve shrinks, so exploration approached the floor without
  reaching it and the first test of it looped until its own guard fired. **A budget
  whose unit of spending vanishes as the budget vanishes is not a budget.**

**Open, and stated rather than papered over:** the economy cannot check novelty
without reading the trace log, so `may_explore` trusts the caller's `no_prior_
evidence` flag. A caller that lies gets a free act. The log is the witness, which
is why the prefix is visible there rather than only in a boolean.

## To become a model

A model is a representation plus a loop that improves the representation from
evidence. unia has the representation. This is the loop, and what stands in it.

**It does not train, and the word is the wrong one.** Training implies gradients
and a loss. unia has neither, and its lack of them is the design rather than the
gap: a gradient update needs a loss function, and unia has something better —
`cargo test` is falsifiable, reproducible and cheap, and a model is none of the
three. A proposed mattern is judged by whether the suite still passes, which is
`Trace::succeeded`, the field already doing chaotic-event duty in the
self-cleaning pass. `Production` already prices the result: a *new* rule is worth
0.10, a repeat phrasing 0.01, a consolidation 0.02, so repetition is not learning
and the currency says so.

### The one link that is missing, and it is wiring

| piece of the loop | state |
| --- | --- |
| traces written with an author | ✅ `Actor::Caller`, `SourceAct` |
| induction proposes matterns | ✅ `src/induce`, tested |
| a witness that can falsify | ✅ `Trace::succeeded` — the suite |
| self-cleaning, dormancy, re-mattering | ✅ fixpoint proved |
| **propose → verify → promote, automatically** | ❌ **not wired** |

`registry::set_champion` existed and nothing called it. So induction proposed, the
witness was capable of judging, and the promotion was a human typing. **This was
the whole of "training unia": a wiring job, not a research one, and it needed no
new vocabulary.** It is phase 3 of `docs/LPMM-TRAINING-PLAN.md`.

**Closed in `src/loop_train.rs`.** `run(store, registry)` does propose → witness →
promote with no human in it, and `ActuatorRegistry::clear_champion` was added so a
champion a later run refutes is *removed* — a registry that can install a winner
and never remove one is a write-once cache with extra steps, and `clear_champion`
had no caller for exactly as long as promotion was manual.

Three things it refuses to do silently, each a test:
- **It does not promote on confidence.** `Candidate::confidence` measures how
  stable the *description* is, not whether the *rule* is right, so promoting on it
  would promote a rule described many ways by many failed acts.
- **It does not promote what it has not seen witnessed.** An unwitnessed candidate
  is `Awaiting`, and the count distinguishes that from "nothing was proposed".
- **A failure poisons.** One green run then one red is not confirmed; the fold is
  strict, because the looser reading lets an early success outvote a later
  refutation.

**Two findings from writing the fixtures, both worth more than the code:**

- **Promotion needs breadth *and* a witness.** `induce::MIN_OBSERVATIONS` is 2
  distinct *phrasings*, so one phrasing repeated five times with five green runs
  proposes **nothing** — the witness is not a substitute for breadth. The first
  version of the promotion test used one intent throughout, read as "the loop
  promotes nothing", and was really "the fixture never cleared the bar". The
  induction bar is inherited, not weakened, and a test now pins both halves.
- **`promotion_rate` and `witness_rate` are different numbers and a test asserts
  they stay different.** A green witness, a re-confirmed champion, no new
  capability: witness rate 1.0, promotion rate 0.0. Reading the first as the
  second is how a system that learned nothing reports perfect health.

**Exit, and it is the flag that matters:** an edit proposed by induction, verified
by the suite, and promoted to a mattern — with the promotion counted and the
failures counted separately. That moves *grow certitudes* from "demonstrated
once, by hand, through a fixture" to a trajectory, which is what
**harmony as reachable equilibrium** has been waiting on since phase 2 and is
still correctly red.

### The ceiling that wiring does not lift

**unia can learn to be better at acting, and cannot learn to express a program.**
Even a fully closed loop produces only sequences of sixteen hardware verbs:
`Signature` is `primitives.join("_")`, closed, with no call, return or locals, so
there is no calling convention because there is no notion of a callable thing.
`Production::credit()` — six lines, the most important economic function in the
project — lowers to *nothing*. These are two different models and only one of
them is this project.

**Which is why item A1 and this are the same gap.** The §6 fork decides it:
ownership-preserving target → a real compiler target, and the program-shaped gap
becomes closable; heap with no aliasing promises → a virtual machine, and the one
reason to compile Rust to it is gone. **Closing the loop is worth doing either
way, because a better actor is a better actor. But the ceiling is the fork, and
the fork is the author's.**

### What "becoming a model" is not

Three claims that look like progress and are not, each already found once:

- **A corpus that grew once is not a trajectory.** Phase 2 grew a corpus, by
  hand, through a fixture. One observation is not a trend, and reporting it as
  growth is the same defect as the fabricated benchmark.
- **More matterns is not more capability.** A new address for a known act is a
  *transfer*. Escalation is counted per **signature** precisely so the generated
  corpus, which escalates in reach and in no capability at all, reports honestly.
- **A log that reproduces the source reproduces nothing about it.** `SourceAct`
  is span-level, so the corpus is a derivation. A derivation is not a model, and
  this is why the transposition *ratio* is the acceptance criterion and not a
  nice-to-have.


---

## Provenance and attribution

- [ ] **Complete the `euni.me` ownership record.**
  Settled so far: RDAP shows the current registration began 2024-01-11, GoDaddy /
  Identity Digital, registrant org `EUNI`, country TW, `fn` empty, all four
  status flags `clientProhibited`. Independently, whoisfreaks shows a prior
  registration created 2020-08-25 at Namecheap, updated 2023-08-25, registrant
  `Withheld for Privacy ehf` (Iceland). The user's own DomainTools session
  reports 37 historical records, oldest more than 7 years, at least 26
  significant changes, and 89% of records publishing ownership data.
  Outstanding: identify the **earliest record with a published registrant name**
  and determine whether that registrant is the author. Requires the free
  DomainTools "PREVIEW REPORT" PDF, or the author's own Namecheap/GoDaddy
  account history, which is authoritative and needs neither.
  **Blockers:** the 26 significant changes are unclassified; if they are
  registrant transfers rather than DNS churn, the continuous-ownership reading
  does not hold and the paper must not claim it.
  **Why it is tracked rather than written:** a domain claim is falsifiable in one
  click, and the public record currently names a privacy proxy. Until the PDF is
  read, no ownership claim belongs in the paper.

- [x] **Decide the paper's author line.** Resolved: the paper carries the
  pseudonym `iulian dacineu` and does not name the author. The 2020 Devpost
  submission publishes the civil name in public (`devpost.com/dacineu`,
  Romania), so the name is reachable by anyone who goes looking for it, but it
  is not asserted in the paper. The rendered capture is held at `docs/proof/` and
  is gitignored, because a PDF containing the civil name inside this public
  repository would be a stronger and more convenient exposure than the page that
  has been public since 2020.
  **Residual, not a defect:** `TRADEMARK.md` and `LICENSE-COMMERCIAL.md` rest on
  the pseudonym being the operative identity, and that remains true — the
  pseudonym is what the licences, the repository and the paper all use.

- [x] **Add a provenance section to the paper** citing the Devpost record, which
  is dated, attributed, and third-party-hosted. Done: §A now records the 24 April
  2020 EUvsVirus submission, cites the canonical URL, and notes that a capture is
  held locally. It does not depend on the domain question above and did not wait
  for it.

---

## Retrieval and the corpus

- [ ] **Fix divergence D4** — `map_intent` normalises underscores to spaces for
  the action `id` but not for aliases, so an intent phrased with underscores only
  matches through an alias. Four of the four failures in the first retrieval
  baseline trace to manifest authoring rather than to retrieval, and this is one
  of them.
- [ ] **Scope aliases to actions, not resources.** `open file`, `get content` and
  `create document` are correct for their own actions and harmful as
  resource-level match keys. This caused the two false positives.
- [ ] **Fix IDF weighting.** It is computed per phrase across the corpus, so a
  resource listing `valve` across many aliases has its own strongest term
  down-weighted for appearing in its own alias set. This is why `open the valve`
  ranks `fs-root-001` first. Weight per resource instead.
- [ ] **Write `patterns/_schema/manifest.schema.json`** and validate every
  manifest. Twelve artifacts currently carry roughly a dozen distinct key sets,
  one retains a corrupted key (`la_piece_de_resistance`) from an earlier
  substitution defect, none carries a `title`, and two use human-assigned
  identifiers where the specification expects a UUID.
- [ ] **Delete or retain `patterns/_quarantine/`.** Ten manifests that are
  harvester output, not curated knowledge. Retained so the decision is
  auditable; they are not indexed and not served.

## Induction and the learning loop

- [ ] **Close the induction loop.** `src/induce` is implemented and tested but
  nothing writes traces, `primitives` is empty, and no candidate has been
  induced from real data. `unia_record` exists and no agent calls it.
- [ ] **Fix the error semantics in the trace record.** `outcome = "miss"` means
  escalation, not failure; failure is `Trace::succeeded`. Any synthetic
  dataset generated under the old reading is mislabelled.
- [ ] **Wire the promotion step.** `registry::set_champion` exists and nothing
  calls it. The evidence gate is specified and untested against a real candidate.

## The missing experiment

- [ ] **Measure declared completeness.** How much of what the system is asked to
  do is covered by a declaration it can check. This is the quantity RLVR is
  bounded by, it is measurable today, and it is a better diagnostic than
  escalation rate because it reports what the system *cannot* learn rather than
  what it currently does not.
- [ ] **Measure an escalation rate.** This is the number that supports or refutes
  C1 in `docs/LARGE-PATTERN-MODELS.md`. It requires a corpus larger than two
  artifacts and does not exist yet. **Stage 4 of the plan above is what makes it
  computable** — not because it enlarges the corpus, but because a trace that
  records *which path served the call* is the input the number is computed from.
  Until then the figure is asserted rather than measured, which is the state the
  paper's §9 is written about.
- [ ] **Build a 200-query labelled fixture authored by someone who did not build
  the matcher.** The current 30-query fixture was written by the author of the
  matcher and is a smoke test. It found a real defect, which is its purpose, but
  it cannot estimate real-traffic behaviour.
- [ ] **Decide whether a semantic tier is warranted,** and only on the residue
  that survives the alias and IDF fixes. If the residue is small the correct
  decision is to stop. Do not add a model before measuring.

## Cross-model interoperability

- [ ] **Define the constraint grammar.** `constraints` is an array of free
  strings such as `status != 'fault'`. Two implementations will parse it
  differently and neither currently evaluates it. This is the single blocker
  that prevents cross-model execution outright; the others are degraded mode.
- [ ] **Define a capability declaration.** A peer cannot tell whether the
  receiver implements all 20 `UniversalPrimitive` variants.
- [ ] **Define schema negotiation.** `ure_version` is present and nothing reads
  it, so two divergent vocabularies merge and fail at runtime rather than at
  load time.
- [ ] **Correct the DU-UUID version label.** Content-derived but labelled v4,
  which RFC 4122 defines as random, so deduplication tooling will mishandle it.
  Should be v5, or a custom v8.
- [ ] **Specify canonical serialisation and test address stability.** The
  canonicalisation comment in `src/identifiers` is wrong: without the
  `preserve_order` feature, `serde_json::Map` is BTreeMap-backed and already
  deterministic. If anyone enables that feature, every content address in
  existence changes silently.
- [ ] **Document the AES-GCM fixed nonce as a confidentiality limitation.**
  Already in `SECURITY.md`; it must also appear wherever identifiers are
  exchanged.

## The creature as intermediary

The plan and its projection are in
[`docs/creature-as-intermediary.md`](./docs/creature-as-intermediary.md). This
section is the ordered live list; the document is the argument. The stages are in
dependency order and each has an acceptance criterion that can fail.

**Measured today, and the reason the stages are in this order.** The loop closes
— `POST /api/care` tends the pet, records a trace, re-induces, and stores the
result — but `Pet.learned` is read in exactly two places, the `/api/pet`
response and the test assertions. **No interaction is routed through a learned
rule.** `Care::parse` accepts four string literals and nothing else, so there is
no vocabulary to speak. `ExternalApi::call` returns a formatted echo and there is
no HTTP client in the crate. Induced candidates are never written to `patterns/`,
so a learned signature matches nothing on the next pass.

- [ ] **Stage −1 — make the verifier real.** Evaluate `constraints` (D5, above).
  Precedes every stage including Stage 0, because a learning loop against an
  unchecked verifier optimises the wrong thing and reports progress doing it.
- [ ] **Stage 0 — a vocabulary, with no model in it.** Replace `Care::parse`'s
  four literals with surface → primitive sequence over what already exists: the
  store's IDF-weighted scorer for candidates, `tokenize_id` + `PRIMITIVE_TABLE`
  for the mapping. *Acceptance:* "give it some kibble" acts, with no model
  configured and no network. **The corpus is this stage's training data, not a
  separate task**: Karpathy's method for adding an ability is a task generator
  (nanochat discussion #164), and the analogue is a *manifest generator* emitting
  artifacts with declared state spaces and overlapping primitives. It unblocks the
  cold start, the escalation measurement and convergence at once, which is why it
  belongs here rather than at the end. **Done** — `src/corpus.rs` and
  `cargo run --bin unia-corpus`. 36 artifacts over 6 profiles, converging on 6
  denominators with 6 to 24 phrasings each. The generator is organised by
  *profile* rather than by overlapping kind because `gather` converges on exact
  equality of the action set: two artifacts sharing two of three actions do not
  interpenetrate, and a corpus built on overlap would have found nothing.
- [ ] **Give mutation a behavioural effect or remove it from the address.**
  `MutationEngine::mutate` appends a marker to `guidance` and adds a `provenance`
  block, which changes the content address while leaving actions and state space
  identical. That is a training signal rewarding an identity change with no
  behavioural change — the spurious-reward failure in miniature. *Acceptance:*
  either a mutation alters an action or a state space, or the child keeps the
  parent's address.
- [ ] **Stage 1 — the contradiction gate.** Check every term the intent proposes
  to set against the addressed artifact's declared `state_space`; on a
  contradiction, ask and execute nothing. *Acceptance:* the 20-case fixture
  reports **20/20 hit@1 with 0 false positives**, and both known
  contradictions return a question naming the missing term. This is the smallest
  stage and the load-bearing one — the two false positives are not a threshold
  problem, they are provable contradictions (`brightness` is not in
  `valve-001`'s `state_space`), and a checkable fact is the only kind of thing
  the creature may act on.
- [ ] **Stage 2 — the creature becomes the intermediary.** A `Mediator` in
  `IntelligenceBridge::request`, which is already intercept → intervene → evolve.
  The pet becomes a first-class `.ure` resource with capabilities `tend`,
  `neglect`, `sleep`, `learn`, `explain`, `refuse`, addressed by content address
  like any other. *Acceptance:* non-interference holds as a property test, and
  `GET /api/pet` becomes a `search` against a manifest.
- [ ] **Stage 3 — external transduction.** One trait, two implementations: the
  Stage 0 matcher (default, offline, no authority) and an OpenAI-compatible
  `POST /v1/chat/completions`, which is what makes `oc-shim`, a local server or
  an agent reachable without a bespoke client. The external model transduces
  only: it may choose among primitives the artifact declares and nothing else.
  *Acceptance:* both satisfy the four properties identically, and the external one
  cannot emit an undeclared primitive under adversarial prompting.
- [ ] **Stage 4 — evolution from the game.** A contradiction, the player's
  answer, and the answer is a trace. This is what `src/session.rs` has been
  blocked on: its own doc comment says a lesson becomes a trace by one function
  call once the vocabulary is defined, and Stage 0 defines it. Then make induced
  rules addressable. *Acceptance:* a creature corrected N times routes the N+1th
  request through a learned rule, the trace records it, and the escalation rate
  is computable rather than asserted.
- [ ] **Stage 5 — two creatures.** Convergence over the existing relay, mediated
  by the creature. Handover carries its refusals as well as its rules.
  *Acceptance:* the motto's answer is produced by the system, not asserted.

### Properties the stages must not lose

Four checks, because "the creature does not interfere" is otherwise a promise
rather than a design. **P1** with no learned rules, no external model and no
contradiction, the creature's output is byte-identical to the architecture's own.
**P2** it can only emit primitives that exist; an unknown primitive is an error,
never a guess. **P3** interception fires only on a checkable fact — a term absent
from a declared `state_space`, an unmet constraint — and never on a score.
**P4** a learned rule is consulted before the models and the trace records it, or
there is no escalation rate to measure.

## ca(R)maduci — the digital pet

A runnable core exists (`cargo run --example camaduci`, 15 tests): the care loop,
the declared state space, sleep-gated stages, the trace log, and death as a
lifecycle transition. Not a game yet — no renderer, no input, no score. See
`docs/camaduci.md`. The pet is the smallest
thing that exercises the whole loop: it is an actuator whose vitals are a
declared state space, whose care operations are primitives, and whose neglect is
an escalation.

### Blocking decisions

- [x] **Decide whether ca(R)maduci is a game or a test fixture.** Both, and not
  by compromise: `src/camaduci.rs` stays a deterministic, clock-injectable core
  with no I/O, and the game is a view over it. The renderer is the part allowed
  to be fun; the core is not allowed to become untestable to get there.
- [x] **Decide `.ure` artifact or Rust struct for the pet's state space.**
  `.ure`, and it is Stage 2 of the plan above rather than a free-standing
  question. The state space stays declarative so the existing loader, matcher and
  contradiction gate handle it, which is what makes the pet a demonstration
  rather than a toy. A struct is easier to test and is what `Pet` is today;
  Stage 2 is where it stops being.
- [ ] **Run a trademark clearance search for the name, then file.** The `(R)` is
  already used project-wide on the understanding that it marks an intended mark;
  `TRADEMARK.md` records the status as not cleared and not filed. MIT protects no
  names, so the mark is worth nothing until registered, and 15 U.S.C. §1125 makes
  false designation as registered a civil cause of action. Filing is what closes
  the gap between the symbol and the status.

### Implementation, once the above is settled

- [ ] **Read TamaFi before designing anything.** ESP32-S3, MIT, ~400 stars, and
  the closest existing implementation: WiFi-driven mood, `BABY → TEEN → ADULT →
  ELDER` on wall-clock age. Also `tama96`, which is Rust and already speaks MCP.
- [ ] **Declare the state space as four typed variables** — hunger, happiness,
  health, all `0.0..1.0`, plus a monotonic `age_ticks` — so the constraint
  machinery has something to check.
- [ ] **Gate stage advancement on a completed sleep cycle**, not on elapsed time.
  A pet that never sleeps never advances. Borrowed from tama96, and it maps onto
  unia: progress gated on a completed interaction rather than on a clock.
- [ ] **Record care as traces and induce the later form from them**, rather than
  picking a branch of a hand-written evolution matrix. This is the part that
  makes it a demonstration of the paper's claim rather than a game.
- [ ] **Make death a lifecycle transition, not a deletion.** The artifact moves to
  `Quarantined` in `crate::gc` terms and is retained, so a pet that died stays
  inspectable and the traces that killed it stay as evidence.
- [ ] **Decide the hardware target.** The research points at ESP32-S3 (TamaFi's
  platform) and the M5Stack ecosystem. `wasm32-unknown-unknown` already builds in
  this repo, so a browser or micro-frontend target is available today; bare metal
  is not. Nothing here has been run on hardware.
- [ ] **Decide the network shape.** Whether two pets ever meet is still open, and
  the answer determines whether this is one crate or two plus a transport.
- [ ] **Give it its own repository.** It is a game and will want its own assets
  and release cycle; unia is a research prototype. It can depend on unia as a
  library.
- [ ] **Decide whether two pets ever meet.** Digimon's linked interaction and the
  mesh architecture both point at it, and it is the most interesting version and
  the most scope. Nothing in the single-pet design requires it.

---

## Codebase hygiene

- [ ] **Rename `mcp::store::Pattern` to `Mattern`, and audit the word.** It is a
  loaded corpus manifest, which by this project's own definition in
  `cognition-log.md` is a *Mattern* — the type that is the **product** of
  transduction currently carries the name of its **input**. Until it is renamed,
  the Pattern/Mattern vocabulary cannot carry weight. Mechanical: 16 references across
  6 files, 6 of them inside `store.rs` itself, one commit, no behaviour change. See
  [`docs/pattern-and-mattern.md`](./docs/pattern-and-mattern.md) §1.
- [ ] **Decide the retention policy, and make the two agree** (divergence D10).
  `prune_fat` keeps the champion and discards everything else; `gc.rs` changes a
  lifecycle field and never deletes; nothing in `src/` removes a `.ure` at all.
  The shipped behaviour is retention, so `prune_fat` is the one that is wrong. It
  has zero callers, so this is free to fix. The decision to make explicitly: does
  a low-evidence artifact get quarantined like a dead one, or promoted, or
  dropped? Retention-with-a-lifecycle-field is what the code does and is the
  Pattern-and-Mattern policy; champion-only is Pattern-only and converges.


- [x] **Resolve `implementation/`.** It was recorded here as "a newer, richer
  draft of `src/` containing `set_champion`/`get_champion`, a `register_ure_file`
  DU-UUID to database pipeline, Collapse dive-depth scoring and a test module,
  none of which exist in the compiled crate". **Every clause of that was
  backwards**, and the directory is now deleted:
  - It is **earlier**, not newer. Its `DuUuid::generate` hashes the manifest
    directly; `src/` hashes the skeleton. It has no `skeleton()` at all.
  - `set_champion`, `get_champion`, `with_base_dir` and `find_matching_actuators`
    are in `src/registry/mod.rs` and **not** in the draft.
  - `register_ure_file` is in both.
  - Its public API is a strict **subset** of `src/`'s in all five modules.
  - Its Collapse tests asserted on `ACTUATOR_MODE: FAST`; the code now emits
    `[MODE:FAST]`, which is why they could never have run as written.

  The one thing it had that `src/` lacked was **test coverage**: the draft had
  test modules for `registry` and `orchestrator`, and `src/` had none for either.
  Those seven tests are now in `src/`, rewritten to the current API and the
  current prompt format. Nothing else was worth keeping, which is why the
  directory is gone rather than filed under `docs/history/`.
- [ ] **Make `examples/camaduci.rs` use `unia::camaduci` instead of
  reimplementing it.** Found during the restructure, not yet fixed. The example
  defines its own `Pet`, `Vitals`, `Stage` and `Care` — the same four type names
  as `src/camaduci.rs` — and 39 of its own functions, importing only
  `mcp::store::{Store, Trace}` from the library. So there are two creatures, and
  the one the README sends a reader to first is the one that is not the shipped
  module. The dead code is already showing in it: a `Clean` lifecycle variant
  that is never constructed and an unused `now`. Fixing this means reducing the
  example to a driver, which also removes the last `Clean` variant question,
  because `src/gc.rs` owns lifecycle.
- [ ] **Add the DuckDB retrieval layer to CI.** `ci.yml` tests Rust only, which
  is why the broken `term_idf` survived two commits. The corpus views load and
  are verified; the ranking path raises a type-coercion error and is not
  callable.
- [ ] **Close divergence D5 — constraints must gate dispatch.** Promoted to
  first priority by [`docs/where-reasoning-comes-from.md`](./docs/where-reasoning-comes-from.md).
  It was recorded as "the second pillar of the safety argument"; under a
  verifiable-reward loop it is **the reward function**. A verifier that does not
  check the declared preconditions is partly a random verifier, and random rewards
  buy most of the apparent gain on MATH-500 (21.4% vs 29.1% from ground truth).
  Everything the creature learns is a function of this, so no learning loop should
  run before it is closed. *Acceptance:* a manifest declaring `status != 'fault'`
  dispatches when the condition holds and is refused when it does not.
- [ ] **Generate `Lifecycle` from one source.** It is mirrored by hand between
  `src/gc.rs` and `database/duckdb/001_schema.sql`; drift would silently retire
  artifacts the index still serves.
- [ ] **Clear the 48 compiler warnings.** Then enable `-D warnings` in CI.
- [ ] **Reclaim disk.** The working filesystem is at 98% with roughly 11 GB free.
  Models under `~/.local/share/unia-models` and the `target/` directory are the
  largest reclaimable items.

---

## Translucency: the five steps and what each still needs

The claim is that the whole chain is inspectable — `resolve_primitive`
(decoding), `signature` (narrowing), `dispatch` (transduction), `induce`
(patterning), `skeleton` (matterning) — and that the language model is
confined to one of them, as a transducer from text to a primitive sequence,
never as anything that determines identity.

- [x] **The economy pays for production, not for repetition.** Done. An act
  spends and debits; the only path to power is `Pet::learn`. The first
  version credited the power on every act, so twelve identical feeds took a
  creature from 0.500 to 0.694 with nothing learned; the same twelve now end
  at 0.157. Four countable integers, of which the fourth — consolidation — had
  to be added because there are only four built-in acts and novelty saturates.
- [x] **The liveness hole the correction opened.** Closed by two entry points
  rather than by weakening the economy: `tend` is the player's and has no
  power gate, `tend_as_self` is the creature's and does.
- [x] **The authorship ratio, so "civilisation" has a number.** Done. `Trace` now
  carries an `Actor` — `Player`, `Itself`, or `Caller` — and `Store::authorship`
  counts all three, with a `self_directed()` share that returns `None` rather
  than a flattering `0.0` when no creature was involved. `GET /api/authorship`.
  Measured in play: the creature took four of six acts against itself,
  `self_directed` 0.667. A missing actor counts as a caller, not as the
  creature — the flattering error is the one that would make the number worth
  reporting.
- [x] **Feeding brings nuants.** Done, and the game is passable again. The
  direction of the stock depends on who acted: a person feeding a creature hands
  it something to act with, a creature feeding itself uses what it had. Only
  `Feed` does this — if every act were free to the player there would be no
  economy, if none were the player could rescue nothing. A person feeding also
  does not debit the power, because the debit is for a creature repeating itself
  and someone holding a bowl is not that.
  Verified in play, and the economy became visible doing it: feeding carried the
  resources 12 → 13 → 14, and the power went 0.500 → 0.585 because two new
  phrasings induced a rule and `learn` paid for it. Bring food, teach a new
  word, production, power rises. That is the loop.
- [x] **Dormancy: a state the creature can be in and come back from.** Done,
  and it resolved the neglect ordering rather than being a separate feature. A
  creature that falls below `DORMANT_BELOW_HEALTH` is dormant: alive, holding
  nothing it learned, with the power at the floor and the resources untouched.
  It will not act for itself and says so. A person acting on it wakes it, and
  the wake is an *event* — the credit still comes only from `learn`. Dormancy is
  the same tick as `stuck`, so the economic failure is now observable on a
  living, reachable creature instead of only on one that no longer exists.
  Measured in play: taught two words, neglected for a minute, caught alive at
  health 0.100 with `posture: stuck` and `learned: 0`; it refused to act for
  itself; a person taught it a third word and it woke at quants 0.262 with its
  address unchanged.
  Four bugs surfaced on the way, all of them by playing rather than reading:
  - `health` never reached zero — f64 residue left it at ~1e-16, which is
    `> 0.0` and `< 0.1` at once, so a neglected creature went dormant and became
    **unkillable**. The unit test missed it because its health trajectory
    happened to land on 0.0 exactly.
  - The threshold was 0.1, one decay step above zero, so a creature *stepped
    over* the band and went from healthy to dead without ever being dormant. A
    threshold is only reachable if it is wider than the step that crosses it.
  - `learn` woke the creature on any non-empty rule set, and induction
    re-derives a creature's own rules from its own trace log — so it woke itself
    from evidence written before it slept.
  - `neglect` had its own weaker copy of the transition: it cleared the rules
    and set the flag but neither counted the loss nor floored the power. Every
    test passed because they all called `go_dormant` directly. One transition,
    one place.
- [x] **Self-cleaning, and recycling with no genealogy.** Done. A mattern nobody
  has confirmed is re-mattered rather than deleted: the same signature at a new
  address, with the confidence and observation count discarded. Deleting would
  delete the capability and the creature would forget; re-mattering keeps the
  capability and throws the evidence away. The new address is a content address of
  the signature alone — *not* a hash of the old one — because chaining them would
  be a genealogy wearing a disguise.
  All three triggers are read out of the trace log, because a trigger nothing
  records cannot fire honestly: unknown situations are successful traces no
  mattern covers, chaotic events are `succeeded: false`, and readiness patterning
  is a mattern with no confirming trace.
  **And it made the convergence test runnable.** Six generations, six
  author-chosen addresses, six private vocabularies, six cullings: 1 capability,
  1 address, 0 survivors naming a culled one. The project had never been able to
  produce a convergence at all — twelve hand-written patterns share zero
  capabilities, and thirty-six generated artifacts share six denominators they
  were manufactured to share. See `docs/dormancy-and-recycling.md`.
- [x] **`Sense::Inferred` has an inhabitant.** It was a variant nothing was ever
  of: distinguishable, documented, unreachable, and the test that noticed read as
  a claim about the sensors rather than as an admission. A `Sensor` now carries
  the sequence it dispatches, and an inferred one cannot be evaluated without a
  `Dispatch` — with none, its `reading` is absent from the state, its predicate
  is unevaluable, and it is not reporting. The same answer a malformed sensor
  gets, with no special case in the evaluator. The inhabitant is a readiness
  sensor: whether the evidence behind this creature's own matterns still stands is
  a question about the trace log, so reading it is an act against that log, and
  the long horizon becomes askable as something a creature notices about itself.
- [ ] **The escalation rate, now that it can be measured.** The measurement has to
  be per *signature*, not per address — which is the direction the corpus evidence
  was already pointing, since the generated corpus escalates in reach and in no
  capability at all. Still needs the 200-query fixture authored by someone who did
  not build the matcher.
- [ ] **`nuants` never regenerate, so an empty creature is dead for good.**
  Found by playing rather than by reading: `apply_economy` and
  `decay_economy` were the only writers outside tests and both only
  subtracted, so three teaching phrasings against a creature at `nuants: 0.00`
  were all refused with nothing able to raise it. Reached in about eight acts.
  **Resolved by the item above** — feeding now brings resources, so the player's
  door actually opens. The dead end was real and is closed.
- [ ] **The creature cannot learn a power it did not ship with.** Four built-in
  `Care` acts, so `new_signatures` saturates at four. This is now *blocking*
  rather than merely noted, because the implied economy equilibrium (about
  0.25, from a 0.03 credit against a 0.15 debit) sits below the peak the
  creature reaches at act four — so a creature that has learned everything it
  can learn ends up weaker than one still learning, and the long game is
  decay. Pinned as a test so it cannot be rediscovered in play. Also blocks
  meeting, handover, and the primitive protocol.
- [x] **The meet, concretely.** Done, in `src/meet.rs`. The order is the
  subsequence relation — gaps allowed, order not — chosen because a prefix order
  is too strict (a trace can interleave a primitive the rule does not name) and a
  set order is too loose (it forgets order, which is what makes a sequence a word
  in a monoid rather than a bag). `meet` is the longest common subsequence with
  the tie broken toward the lexicographically least, which is what makes it a
  function of its two arguments and lets the four lattice laws be tested at all;
  without that, `AB`/`BA` would have no unique answer and commutativity would fail
  for a reason that has nothing to do with lattices. Reflexive, antisymmetric,
  transitive, and the meet is a real greatest lower bound — the first
  implementation returned the empty sequence for `meet(a, a)`, which satisfies
  every lattice law and is not a meet.
- [x] **The decidable retrieval check, and the reason it was not already there.**
  `Store::search_primitives` uses `meet::serves` with no threshold and no ranking.
  Measured on a corpus of induced manifests: it finds the true positive (an act
  performed with a primitive interleaved) and refuses both shapes of false
  positive — the witnesses that merely *touched* the act and did something else
  after. Those are the shapes the scorer could not separate at 0.2236 and 0.2197
  against 0.2041.
  Getting there exposed something bigger. **The store had no primitive sequences
  at all.** `Action` carries an id, aliases, a target state and constraints; the
  sequence a pattern performs lives nowhere in a manifest, so the decidable check
  could not be applied to the corpus even in principle. It turns out the sequence
  *is* recoverable, because induction sets an action's id to the signature it
  induced — so `SetValue_CheckSense` is a declared act and `emergency_shutdown` is
  prose, decidably. That is a convention rather than a schema and it is fragile in
  one way worth naming: a hand-authored action legitimately named after a
  primitive sequence would be read as one.
- [ ] **So the retrieval number has not changed, and cannot yet.** Every
  checked-in manifest is hand-authored, so the corpus declares no acts, a
  structural search finds nothing, and the 20/20-with-2-false-positives figure is
  still the scorer's. Two things have to happen and neither is a code change: the
  corpus has to be *induced* so it has acts to search, and `search` needs a
  `primitives` field on `Action` so the sequence stops depending on an id
  convention. A persisted-schema change, and not something to slip in beside a
  measurement.
- [x] **The commons says which promises are funded.** Done, in
  `src/primitives/mod.rs`, which had **no tests at all** and seeded exactly one
  nucleus, so fifteen of the sixteen primitives could not be called and nothing said
  so. Every primitive now has a binding where a command genuinely exists and a
  recorded `unfunded_reason` where none does. Measured: **10 of 16 on Linux, 8 of 16
  on macOS, 6 of 16 on Windows.** Six are unfunded everywhere and the reason is
  the same in each: they are not operations on a machine. `Toggle` is inversion,
  `Increment` is arithmetic, `Pipe` is shell syntax, `Broadcast` is networking,
  `Pulse` is a scheduled signal, and `Validate` would need a dependency rather than
  a standard tool. Binding those to plausible strings would issue bonds that default
  on first use, which is worse than a visible gap.
  Two bugs the new tests caught. A `GenericLinux` binding was a fallback for *every*
  platform, so `cat` resolved on Windows — a bond quietly holding on the wrong
  machine, which is the failure the catalogue exists to prevent. The fallback is now
  restricted to the Linux family by explicit list, so a new `OsVariant` defaults to
  *not* inheriting a Linux promise. And an unfunded primitive returned `"No mapping
  for OS"`, which reads as a lookup miss; it now says it is known and unfunded,
  which is the distinction a caller actually needs between *never heard of this* and
  *knows this and cannot call it*.
- [x] **The handover — the one relation that keeps a genealogy.** Done, in
  `clean::{Handover, Provenance, receive}`. A creature can now be given an artifact
  by another, and records who gave it. The three relations are now separate
  mechanisms rather than three phrasings of one: handover preserves provenance,
  convergence severs it, matching is provisional and scored.
  The gift is **re-mattered on arrival**, which two tests forced. Passed through
  verbatim, the recipient arrived holding the *sender's* confidence — so sharing
  transferred demonstrated power rather than vocabulary — and the artifact's address
  was the sender's rather than the capability's, which would have made the address
  depend on how a capability was arrived at and contradicted the meet. So a gift
  arrives as a fresh claim, exactly as a re-mattered artifact does: the giver
  conveys the act and the words for it, the power is not transferable because the
  holder has demonstrated nothing, and the address stays the capability's whichever
  way the capability arrived.
  A lineage survives dormancy, because what was given is a historical fact and does
  not become untrue when the creature can no longer demonstrate it.
- [ ] **The bridge this declares is still not the one that runs.** Two types share
  the name: `bridge::primitive::PrimitiveBridge` is a `.ure` resource loader and is
  what `wasm_core`, `os::kernel` and `orchestrator::meta` all use; `primitives::
  PrimitiveBridge` is the OS mapper and **nothing references it**. So execution goes
  through drivers, not commands, and the catalogue above is a truthful statement
  about the vocabulary rather than a description of the running system. Wiring it in
  would be speculative, so it is not wired. The real binding question is therefore
  *which primitives have a registered driver*, and that is not measured yet.
- [x] **The shape experiment.** Ran it: one `SynthesisRequest` from a creature's
  own state, one `synthesize_nucleus` call, and an answer. **A shape is real and a
  request synthesises one** — four slot bindings from a request, and an unfulfilled
  slot refuses the *whole* shape rather than returning a partial one, which is right.
  But **the request does not decide the shape.** Three defects, all pinned as tests
  in `fluid::factory::shape_tests` so fixing one is a deliberate visible diff:
  - **The market was unreachable, and is now not.** `discover_resources` read
    `if !query.tags.is_empty() { return false; }` with a comment saying the real
    check was unwritten — which made the *unfinished* branch the *rejecting* one.
    Both production callers (`FluidFactory` and `OrchestratorMeta`) always send a
    tag, so both always got zero results, and the only test that passed was the one
    that sent no tags. Measured before: 3 offerings on the mesh, 3 found untagged,
    **0** found tagged. Fixed and regression-tested.
  - **A champion short-circuits the request entirely.** `resolve_best_actuator`
    checks `get_champion` first and returns, so scope, quality floor and required
    capabilities are compiled into a query that is never built. A champion is
    *imposed* — one actuator declared the answer for a capability — and it is the
    only path where choice is exercised at all.
  - **Narrowing the scope widens the choice.** `allowed_scopes` is
    `[preferred_scope, Global]`, so a `Circle` request sees `Circle ∪ Global` and a
    `Global` request sees only `Global`. Measured: with `reasoning` offered at
    `Circle` only, a **`Global` request is refused outright** and a `Circle` request
    is served. Backwards from every word involved, and it compounds with the quality
    floor — a `Global` request at `min_qor 0.5` refuses a qor-0.9 offering it would
    otherwise have taken, because that offering is at `Circle`.
  - **Quality is a gate, never a preference.** Nothing ranks what passes the floor:
    `discover_resources` returns a `Vec` in `HashMap` order and the caller takes the
    first. Measured: a qor-0.2 offering beat a qor-0.9 one. The winner is stable
    within one market and differs between two, because `HashMap` seeds its hasher
    per instance — which is worse than nondeterminism, because it looks principled.
- [ ] **So "a different shape" is mechanical but not yet chosen.** The four-act
  blocker stands, and the route to it is now identified rather than guessed: a
  creature's capability would be a *shape* — four slot bindings — rather than a set
  of acts, and the first thing to build is a ranking for the market, because without
  one the market is either imposed (champion) or arbitrary (hash order) and neither
  is the "choosable not imposable" the design is for.
- [ ] **LPMM training plan.** Written: `docs/LPMM-TRAINING-PLAN.md`. Six phases,
  each exiting on a *measurement* rather than a feature landing. Phase 0 is
  `Verdict::Both`; phase 1 is a source-edit alphabet, deliberately separate from
  the sixteen runtime verbs; **phase 2 — a caller writing a trace — is where the
  escalation rate stops being unmeasured and nowhere earlier**; phase 3 closes the
  loop with the test suite as verifier via `Trace.succeeded`; phase 4 routes doubt
  outward with the answer treated as a *candidate* that must earn confirmation;
  phase 5 removes the model and **proves it by comparison against the
  LLM-assisted corpus**, not by flipping a flag.
  The plan rests on one distinction: **the origin can be imported, the certitude
  cannot.** An imported corpus is inert under the credit function — `Production`
  credits nothing no trace confirmed, and `clean` re-matters it on the first pass
  — which is the answer to the LLM-ontology literature's concern about
  hallucinated triples. The architecture needs no policy against them; the credit
  function already prices them at zero.
- [ ] **Three of the five `MetaActuatorType` variants are stubs** that report work
  they did not do. `Mutator` returns `{"status": "EVOLVED", "performance_gain":
  "+15%"}` while mutating nothing; `Synthesizer` returns `HYBRID_CREATED` with a
  fresh `Uuid` and no artifact; `Distiller` claims a `10:1` compression it did not
  perform. `actuate` ignores both its input and its context. The self-training
  machinery is the most declared and least built part of the system, which is the
  opposite of what a training plan needs. This corrects what I told the user an
  earlier turn, when I described `MetaActuatorType` as "the reason the long
  horizon is askable" without opening the file.
- [x] **The unia target specification.** Written: `docs/unia-target-spec.md`. The
  question "can Rust compile for this architecture" has a precondition that is not
  a compiler, and this is it. Of nine pipeline stages, **three are done** — the ISA
  is sufficient (the sixteen verbs are Turing-complete, so the instruction count
  was never the constraint), verification exists at 488 tests plus a building wasm
  target, and the history is already shaped like a port via `SourceAct` and
  `Caller` traces. **Six are missing**: the ABI, an IR, a type mapping, a memory
  model, a linker, a runtime. The load-bearing finding is §2.3: `Signature` is
  `primitives.join("_")`, a **closed** sequence, so a signature has no parameters,
  no return and no invocation — **there is no calling convention because there is
  no notion of a callable thing.** Binding and definition are not features to add;
  definition is what a function *is*. Worked through on `Production::credit()`:
  six lines, and they lower to nothing at all, for want of four scoped locals and
  a way to return. The acceptance criterion is stated in advance — **a transposed
  module must be shorter than its source, and the ratio measured** — because a
  ratio of 1.0 is a failure that looks like a success, which is the same trap the
  escalation metric fell into when a deduplicated denominator rated fifty-one
  identical edits as a perfect escalator.
- [ ] **The fork at §6, and it is the author's to close.** (a) An
  ownership-preserving target, and `unia` is a real compiler target with Rust's
  central guarantee intact. Or (b) a virtual machine with a heap and no ownership
  promises, and **the one reason to compile Rust to it is gone.**
  **The tree is already (b) and it was never written down**:
  `Arc<Mutex<HashMap<String, HashMap<String, String>>>>` is a heap without
  ownership, and every consequence in the spec follows from that silent
  assumption. It decides whether this is a compiler target or a virtual
  machine — different projects with the same name, which the vocabulary does not
  yet distinguish.
- [ ] **Transpose one module and measure the ratio.** `src/clean.rs` first:
  ~500 lines, small surface (`rematter`, `serves`, `meet`, `confirming_traces`,
  `clean`), already pinned by tests. Report the matterns the semantics decomposes
  into, the occurrences of each, and the ratio. Blocked on the fork above, not on
  effort.
- [x] **The fake benchmark, retracted.** `tests/benchmark_tests.rs` timed one
  dispatch against `thread::sleep(800ms) + thread::sleep(50ms)` and printed the
  quotient as an "Efficiency Gain", asserting `gain > 1.0`. The numerator was a
  duration the test slept through, so it could not fail, and
  `docs/PROVENANCE.md` cited it as "empirical benchmarks" for the first public
  commit. **Removed rather than repaired**: repairing it means asserting that one
  unia act and one LLM inference are comparable quantities, which is a claim
  about value and not a timing. Replaced with two tests that check what was always
  worth checking — a permitted act is admitted, executed and paid for, and an act
  whose precondition cannot be evaluated is refused. Real measurement in
  `examples/dispatch_bench.rs`; the retraction is in `docs/PROVENANCE.md` and the
  numbers in `docs/perf-unia-vs-lua.md`.
- [ ] **The 77% is `map_intent`, not interpretation.** Measured: a whole act is
  ~14,300 ns, of which ~11,000 (77%) is turning a natural-language intent into an
  action key and ~3,300 (23%) is gate + route + construct + execute. The hot path
  re-allocates on every call — `to_lowercase()` per call, per action, and
  `SemanticMapper::compute_score` builds two `HashSet<String>` and intersects and
  unions them per candidate. None of it is inherent to interpreting sixteen verbs.
  **This is the measurement to make next**: whether an induced matcher
  (`src/lexicon.rs`, `src/induce/`) removes the per-call tokenisation.
- [ ] **"unia vs Lua" has no program-shaped meaning yet.** ~69,600 acts/s against
  Lua 5.4.9's 3.5M combined table-write-and-format ops/s is a 51× figure, and
  quoting it as "51× slower than Lua" is a category error: an act resolves an
  intent, checks preconditions, charges an economy, routes and returns a receipt,
  and the list is the product. Making the comparison real needs three things, none
  of which hold: an agreed unit of work (a specification, like the fork at
  `docs/unia-target-spec.md` §6), a fixture authored by someone who has not read
  the matcher, and the 16 verbs able to express a program — per §2.3, a
  `Signature` is a closed sequence with no call, no return and no locals, so there
  is no program to run and no loop to time.
- [x] **A linker that binds capabilities, and can be broken while running.**
  `src/link.rs`. The tree had a hand-written routing table — `UpaDispatcher::
  set_primary(slot, resource_id)` — and a human filled it in; nothing discovered
  anything, and nothing checked that the named resource could do what the slot
  needed. That is a table, not a linker. The difference is one line: **a routing
  table binds a name to a resource, a linker binds a capability to a resource
  that declares it, and the binding can be broken and re-made without stopping
  the machine.** Both halves matter; the second was missing entirely.
  A capability set is a resource's `action_primitives` and nothing else, so the
  thing that declares a capability and the thing checked against it are the same
  artifact and cannot drift apart. Widening it was declined: `UniversalPrimitive`
  is 17 hardware verbs and an ontology is not that, and a second vocabulary
  beside it is a means of confusing the two. Resolution is a meet over declared
  acts, so `serves` and the lattice laws are the whole of the test. Primary is
  chosen by sorted id, so the binding is a function of the declarations and not
  of hash order — tested with opposite arrival orders.
- [ ] **What happens to an act in flight when its resource is demoted.** The
  linker counts them (`Inflight`) and **reports the count rather than deciding**.
  Three policies: *complete* (finishes on the old resource, so it must stay
  alive until the count is zero — this is the default), *refuse* (abandoned,
  caller learns and may retry), *reissue* (restarted on the new one, which needs
  the act to be replayable, and per target-spec §2.3 a closed `Signature` is
  not). **It belongs to the author, and "unreachable contract exit choosable by
  all parties" is the vocabulary for it.** Reporting rather than choosing is
  deliberate: the old `resolve_primitive` returned `SetValue` for every
  unrecognised action, so a typo and an unresolvable request took the same path
  and nothing said so. A linker that unlinks a busy resource without saying what
  happens to the act on it is that mistake in a different hat.
- [ ] **A shadow is declared to be for hotswapping and is actually fan-out.**
  `UpaDispatcher` has `shadow_routes` commented "for hotswapping" and `route()`
  returns primary *and* shadow, so a packet is delivered to both. For a counter
  that is the act happening twice. A shadow should be **promoted**, not written
  to. `Linker::promote` does that and the old behaviour is not yet removed from
  the dispatcher, so the two disagree and only one of them is right.
- [ ] **A manifest may name an action `SetValue`.** Nothing stops a
  `action_primitives` entry being spelled exactly like a `UniversalPrimitive`
  variant. The two live in different fields so nothing is confused at the type
  level, and `PRIMITIVE_NAMES` is not a capability vocabulary — but the names
  overlap in the same string space, and the linker's only defence today is that
  it looks in the right field. I wrote a test for this and deleted it: it
  asserted `true || true`. The real check is a name-space assertion over every
  manifest in `tests/fixtures/`, which has not been written.
- [ ] **The 200-query fixture.** Unchanged and still the blocker for the paper.
  It has to be authored by someone who has not read the matcher, because whoever
  writes the queries will write them in the vocabulary the matcher was built to
  handle and the escalation rate will be a measurement of my assumptions. That is
  the same failure as the corpus generator manufacturing the convergence it then
  reported.
- [ ] **Is there a civilisation at all?** The instrument above plus shared
  denominators. The honest starting position: 12 hand-written patterns with
  **0** shared capabilities — twelve solitaries, one author, one intent — and
  36 generated artifacts agreeing on 6 denominators they were *manufactured*
  to agree on. Neither is a convergence. The test that would find one is
  creatures that never met agreeing anyway, and it has never been run.
- [ ] **Three horizons as three types.** Not a gap; a roadmap item, and the
  correction above already separates two of the three by accident. `nuante` is
  the short horizon and is spent; `cuante` is the middle and is now
  production-only; the signatures and the content address are the long one and
  are neither spent nor sustained. Making that explicit removes the tuning war
  between credit and debit, both of which are currently rates against different
  distances on the same axis.
