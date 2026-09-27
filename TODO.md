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

- [ ] **Measure an escalation rate.** This is the number that supports or refutes
  C1 in `docs/LARGE-PATTERN-MODELS.md`. It requires a corpus larger than two
  artifacts and does not exist yet.
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

## ca™maduci — the digital pet

A runnable core exists (`cargo run --example camaduci`, 15 tests): the care loop,
the declared state space, sleep-gated stages, the trace log, and death as a
lifecycle transition. Not a game yet — no renderer, no input, no score. See
`docs/camaduci.md`. The pet is the smallest
thing that exercises the whole loop: it is an actuator whose vitals are a
declared state space, whose care operations are primitives, and whose neglect is
an escalation.

### Blocking decisions

- [ ] **Decide whether ca™maduci is a game or a test fixture.** It is currently
  both, and they pull opposite ways: a fixture must be deterministic and
  clock-injectable, a game must be fun. Building before this is settled produces
  something that is neither.
- [ ] **Decide `.ure` artifact or Rust struct for the pet's state space.** The
  `.ure` route keeps the state space declarative and lets the existing loader
  and matcher handle it, which is what makes the pet a demonstration rather than
  a toy. A struct is easier to test.
- [ ] **Run a trademark clearance search for the name**, then file before
  publishing `(R)`. `TRADEMARK.md` records that MIT protects no names, so the
  mark is only worth anything once registered. Use `™` until then: 15 U.S.C.
  §1125 makes false designation as registered a civil cause of action.

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

- [ ] **Resolve `implementation/`.** It is a newer, richer draft of `src/`
  containing `set_champion`/`get_champion`, a `register_ure_file` DU-UUID to
  database pipeline, Collapse dive-depth scoring and a test module, none of which
  exist in the compiled crate. Two divergent copies of the registry and
  orchestrator are in the repository.
- [ ] **Add the DuckDB retrieval layer to CI.** `ci.yml` tests Rust only, which
  is why the broken `term_idf` survived two commits. The corpus views load and
  are verified; the ranking path raises a type-coercion error and is not
  callable.
- [ ] **Close divergence D5** — constraints must gate dispatch. The safety
  argument in the paper currently rests on the evidence gate alone.
- [ ] **Generate `Lifecycle` from one source.** It is mirrored by hand between
  `src/gc.rs` and `database/duckdb/001_schema.sql`; drift would silently retire
  artifacts the index still serves.
- [ ] **Clear the 48 compiler warnings.** Then enable `-D warnings` in CI.
- [ ] **Reclaim disk.** The working filesystem is at 98% with roughly 11 GB free.
  Models under `~/.local/share/unia-models` and the `target/` directory are the
  largest reclaimable items.
