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
  belongs here rather than at the end.
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
