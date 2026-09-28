# TODO

Open items, ordered by what unblocks the most. Status reflects the repository,
not intentions.

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
