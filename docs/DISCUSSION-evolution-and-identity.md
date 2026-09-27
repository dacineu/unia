# Discussion record — evolution, identity, and the vocabulary boundary

> **A study record, not a design document.** §15 is the live to-do list; the rest
> is the evidence behind it and the corrections made along the way.

**Dates:** 2026-09-27 → 2026-09-28
**Purpose:** a study record. Claims here are separated into *measured*, *asserted in
code*, and *open*, because several things believed early in this discussion were
later found to be false and the corrections are the most useful part.

---

## 1. The corpus is nearly empty, and that shaped everything

Measured, repeatedly, against the checked-in corpus:

```
unia-mcp topology  →  37 artifacts, 0 edges, 0 denominators
gather()           →  0 denominators, 0 ties
```

Ten of the twelve indexed artifacts are quarantined harvester output declaring
**zero** `action_primitives`. The two curated artifacts (`valve-001`,
`fs-root-001`) expose 2 and 3 primitives and share none.

Consequence: every architectural claim about *scale* — convergence, mesh
behaviour, multi-node dynamics — is a claim about a system that currently has
almost nothing to converge. This is stated in the paper's §9, the README, the
game design doc, and the gathering module's own doc comment, because it is the
single most important caveat in the project.

---

## 2. Retrieval: three unrelated defects, one number

Measured before → after:

| | before | after |
|---|---|---|
| Hit@1 | 18/20 (90.0%) | **20/20 (100%)** |
| False negatives | 2 | **0** |
| False positives | 2 | 2 |
| Matcher suite | 0 tests | 16 |

1. **D4, underscore tokenisation.** `tokenize` kept `_` as a word character, so
   `write_file` became one opaque token no intent could contain. The containment
   fast path disagreed with the token path — it compared against a
   space-normalised phrase. Closed.
2. **IDF per phrase, not per resource.** A resource listing a term across many of
   its own aliases down-weighted its own strongest term. Both patterns tied at
   exactly 0.5402 and an alphabetical tiebreak returned the wrong one.
3. **No intent coverage.** Both overlap terms measured how much of the *phrase*
   an intent covered. Nothing measured how much of the *intent* a phrase
   explained, so `create document` rivalled a real match against `summarise this
   document for me`.

**The paper's §7.3 blamed IDF alone. That was wrong** — three distinct causes.

### The 2 false positives are not fixable by tuning

```
set the valve brightness to eighty   → 0.2197
summarise this document for me     → 0.2236
what is the current flow rate       → 0.2041   ← a true positive
```

The false positives score **above** the weakest true positive. No threshold
separates them. They need a capability check against declared state space
(**D5**). This is the recurring lesson: *at this precision, thresholds are not
the answer.*

---

## 3. Ancestry is a relation, not a chain — a correction

An earlier proposal was a hash chain: `link[n] = H(link[n-1] ‖ child ‖ parent)`.
**Withdrawn.** A chain asserts a tree with one parent, and this architecture
cannot support that: two nodes that induce the same capability independently reach
equal content by different routes. Forcing that into single-parent links encodes
a false claim while destroying the most informative event in the system.

What replaced it, in `src/gather.rs`:

- `Relation::{Convergent, DerivedFrom, Requires, Supersedes}`
- Convergence requires **exact** set equality of action structure — a fact, not a
  score, sidestepping the calibration problem above
- Empty capabilities are excluded (ten corpus artifacts declare none and
  trivially "converge" on the empty set — a bug the first measurement found)
- Convergence counts **distinct** content addresses; two of the quarantined
  files carry different bodies under one `resource_id`, a live content-addressing
  collision

---

## 4. Readiness is quantised, and why that matters

`Readiness::key()` → `u32`, quantum 1e-6. Ordering runs on the integer.

The distinction is narrower than it first appears: **plain `f64` arithmetic is
bit-identical per IEEE-754.** The non-determinism was confined to the logarithm
inside the readiness score. Quantising the ordering key removed the last
non-reproducible step.

Merging is a **maximum, not a sum** — a capability corroborated from two
directions has the same standing as one observed once.

`GatheringProfile` splits counts (exact, citable) from nanoseconds (readings of
one host, not citable). Measured: 37 artifacts, 630 comparisons, 1 capability
match, 4.1 µs compare. The pass is all-pairs, `n(n-1)/2`, asserted as a test at
n = 10, 100, 200.

---

## 5. ca(R)maduci: the creature

Pure logic in `src/camaduci.rs` (no clock, no I/O), a terminal example, a
dependency-free browser client, and an Obsidian pane beside the corpus view.

- **Age advances on a completed sleep cycle, not a clock** (from `tama96`).
  Never slept → never leaves the egg.
- **Neglect is the absence of a call**, not a punishment the game levies.
- **Death is a lifecycle transition** — quarantined, retained, traces intact.
- **A grace period**: nothing decays until the first action. Without it the pet
  was dead before anyone could read the URL.

### The four defects that stopped it evolving

Found and fixed:

1. **Induction output never returned to the pet.** It went to the motto. The pet
   had no field for what it learned.
2. **Spikes counted operations, not learning** — `history.len()`, maximum 4. The
   most expressive channel reported *being played with*, never *being taught*.
3. **`Pet::restore` existed and was never called.** Line 42 was `Pet::new()` on
   every boot. The creature could not remember.
4. **Two disconnected stores** — the trace log remembered, the creature didn't.

Now verified end to end: three differently-worded feeds → one rule
`SetValue_CheckSense`, confidence 0.523, three learned aliases — and it survives
`kill` + restart (`restored ca-001: care 3×feed`, rule intact).

---

## 6. The teacher path, and the sentence that was wrong

`src/session.rs` extracts **corrections** and **confirmations** from a session
with a model. The unit is the correction, not the conversation.

**I originally wrote: "a session teaches vocabulary, not capability."** That was
wrong in a specific way. It made vocabulary sound like a preliminary to
capability. In this architecture the surface form *is* the act — a rule that
recognises "pour some kibble" and one that recognises "toarnă porumb" are the
same rule, because identity is the action. **Learning a vocabulary is learning a
capability.** There is no gap.

### The counterexample, measured

```
Romanian session: corrections found = 0
English session:   corrections found = 2
```

Same corrections, same meaning. `session.rs` has hardcoded English prefixes
(`no,`, `instead,`, `don't`). **The correct fix is not to add Romanian; it is to
delete the list and let transduction do it.** A per-language constant is the wrong
shape.

Two markers were deliberately dropped rather than tuned: `"that is not"` and
`"not quite"` both read as corrections and both appear constantly in
*retrospective* remarks. Missing evidence is recoverable; invented evidence is not.

**It does not emit traces.** A trace's value is its `primitives` field, and the
vocabulary is undefined (**D6**). A trace carrying an invented sequence would be
indistinguishable downstream from a real one.

---

## 7. What actually drives the LPMM

The question that reframed everything. Traced through the code:

| Driver | Implementation | Input |
|---|---|---|
| Retrieval (tier 1) | `score_phrase` → IDF token overlap + coverage → `f64` | **words** |
| Semantic (tier 1.5) | `SemanticMapper::compute_score` → Jaccard on `HashSet<String>` | **words** |
| Identity | `DuUuid` → SHA-256 over canonical body | exact |
| Convergence | `Capability::same_as` → `BTreeSet` equality | exact |
| Evidence gate | distinct-phrasing **count** ≥ `MIN_OBSERVATIONS` | cardinality |
| Selection | `Readiness::key()` → `u32` max, convergence primary | exact |

**The LPMM is still driven by tokens.** Every live retrieval path is lexical.
`SemanticMapper` is *not* semantic — it is Jaccard on a string set, despite the
name. The paper's embedding tier (bge-small, measured 8.3–20.8 ms) is specified
and benchmarked but **not wired into `search`**.

An earlier proposal claimed to remove vocabulary from the wire. It did not remove
it from the driver. Three of four drivers are already language-independent; the
one users feel — retrieval — is not.

---

## 8. THE BLOCKER: identity includes surface form

Measured:

```
english                 = fab46dd6…
romanian                = ecf5773b…   ← the SAME capability
english + 1 new alias   = f9ff520f…   ← the SAME capability

romanian == english?           false
+1 alias == original english?  false
```

`DuUuid::generate` strips only `resource_id`; **every alias enters the hash.**

Therefore, today:

- Convergence is **impossible across languages** — two players in different
  languages never meet, because there is nothing to converge on.
- Convergence is **fragile within a language** — a rule, and that same rule after
  one player used a new phrase, are different artifacts.
- "Teach it Romanian as a correlation" **presupposes a stable interior to
  correlate to**, and the interior currently moves whenever the surface does.

### The fix

```
address = SHA256( action structure ‖ state space ‖ payload )
surface = aliases, guidance, description   →  not hashed
```

Then English and Romanian are the same address; learning a phrase leaves the
address unchanged and the phrase merges; two players in any language converge.

The cost: surface must be stored *beside* the address. A storage-layout change,
not a semantic one.

**Test that should exist and currently fail:** same skeleton, different aliases,
same address, converged in `gather`.

---

## 9. Primitive-level addressing does not exist

```
pour some kibble       -> NO MATCH
toarna porumb          -> NO MATCH
emergency_shutdown     -> valve-001 @ 1.000    ← canonical primitive
SetValue_CheckSense    -> NO MATCH             ← induced primitive sequence
valve-001              -> valve-001 @ 1.000    ← content address
```

Canonical primitives address perfectly. **Induced primitive sequences do not
match at all** — induction adds the signature as an alias to a `Candidate`
manifest that is never written back to the corpus.

**A creature can learn a primitive and then never call it.** This dead-ends the
vocabulary-free protocol at step one.

Cross-lingual aliases *do* work, incidentally — with no code change, because
`aliases` is a flat `BTreeSet<String>` with no language anywhere in it:

```
inchide supapa         -> valve-001 (0.850)
oprit supapa urgent    -> valve-001 (0.850)
regleaza debitul       -> valve-001 (0.850)
```

The data model already supports it. It only needs populating.

---

## 10. Hazards a multilingual corpus introduces

Not theoretical. A cross-lingual alias set with a **single antonym in it**:

```
inchide supapa         -> valve-001 (0.850)   ← "close" as a shutdown alias
open the valve         -> valve-001 (0.289)   ← the ANTONYM still matches
close the valve        -> valve-001 (0.289)
deschide supapa        -> -
```

And the Romanian example is already a false friend: **`foaie`** means sheet of
paper, leaf, *and* banknote. One word, several nuclei.

Correspondence cannot be a flat alias bag. **Antonyms are not synonyms.** Needed
as a *typed* relation — `gather::Relation` currently has `Convergent` and
`Requires` and **no `Corresponds`**:

| Relation | Retrieval effect |
|---|---|
| `Corresponds` — same nucleus, another language | alias |
| `Opposes` — antonym | **must not** be an alias; polarity check |
| `Regional` — same sense, one locale | alias, lower confidence |
| `Homonymous` — one word, several nuclei | ambiguous; must disambiguate |

---

## 10b. Convergence is not about location

Stated as a design goal and already true in the code. `src/gather.rs` contains
**no network concept at all** — no host, address, port, node identifier or
position. Convergence compares `BTreeSet` equality of `Capability` and nothing
else. So two creatures on opposite sides of the world with the same primitives
converge, and two neighbours on one LAN with different primitives do not.
Location is irrelevant to whether two things can meet; capability to converge is
the only thing that decides it.

What did not exist was the output side. A creature could be *found* but not
*answered to* in anything but prose. Four forms now render from one address:

| Form | Cost | Who can read it |
|---|---|---|
| `Prose` | highest | anything, including a human |
| `Pseudocode` | low | a peer sharing the primitive vocabulary |
| `Rust` | high | a machine that can compile it |
| `Data` | lowest | exactly one peer, with the decoder |

`Form` is ordered, so a caller picks the cheapest form its audience can parse.
Only `Prose` and `Rust` are self-describing; the other two are meaningless
without a shared reference frame, which is exactly why they are fast rather than
safe. Every form refers to the same address and the same primitives, asserted by
a test, so a conversation is repeatable whatever encoding it happens to use.

The data channel carries **no checksum**, and says so: a peer that cannot parse a
blob cannot distinguish corruption from a well-formed payload. `decode_data`
returns `None` for anything malformed rather than reconstructing a
plausible-looking answer from a wrong count.

## 11. LLM intercommunication: the factual position

**No LLM has a non-textual interface.** Every model, including audio-native ones,
is a next-token predictor over a text-derived BPE vocabulary. There is no "data
mode."

**Data-only on the wire is achievable today** via constrained decoding — mask the
sampler to a grammar or schema, so no prose crosses the boundary. For a boolean
(`0`/`1`) it is one forward pass with two logits masked and an argmax.

What is gained: the two endpoints never depend on a shared human language, and a
peer that cannot render English is not a second-class citizen. What is not gained:
the model still tokenizes internally, so a primitive sequence is itself tokenized.

### The key architectural distinction

Two failures are currently conflated as "no match":

- **Vocabulary miss** — an unknown word. Under the new framing this is a
  *transduction* failure, and the model is the right answer.
- **Capability miss** — an unresolvable primitive. No model can help.

**The escalation rate only ever measured the second. The first was invisible.**

---

## 12. P2P signalling

`unia-signal`, installed as a systemd **user** service. Relays `hello`/`offer`/
`answer`/`ice`; never sees a peer payload.

- `hello` is **not** relayed — announcing an arrival before it authenticates leaks
  presence to peers that have not admitted it
- A peer that changes room mid-session is refused
- Room token compared in full, constant-time; prefix-sharing rejected
- Without a token, binds **loopback only** and refuses anything else
- Hardened: `ProtectSystem=strict`, `PrivateTmp`, `NoNewPrivileges`,
  `MemoryDenyWriteExecute`, no write access

**The bug worth recording:** the first version created the broadcast channel
*inside the connection handler*, so every peer broadcast into a channel only it
received. It compiled, passed all 16 unit tests, and started cleanly under
systemd — and forwarded nothing. Only two real WebSocket clients caught it. The
code comment described shared behaviour the code did not have.

---

## 13. Environment and infrastructure facts

- `/tmp` had **796 MB free**; `target/` was 6.2 GB inside it. Moved to
  `~/.unia-target` → `/tmp` at 6.9 GB. **`CARGO_TARGET_DIR` is required for
  builds now.**
- Obsidian plugin `main.js` is at **99.6% of a 5 MB guard**, 21 KB headroom.
  `three.js` cannot be added. Any 3D must be hand-rolled.
- `borderWidth` in the Tailwind config is restricted to `0` and `DEFAULT`; use
  `tw-border-l-[2px]`.
- Classes carry a `tw-` prefix.
- `webrtc` costs **31–55 MB** of dependencies; `str0m` is the better fit.

---

## 14. Corrections made during this discussion

Recorded because the wrong turns are more instructive than the right ones.

| Claim made | Correction |
|---|---|
| A hash chain gives secured ancestry | Withdrawn. A chain asserts a tree; the data is a DAG. |
| Convergence would make a growing corpus better | It doesn't. The richer artifact out-retrieves the other, and the denominator is never served. |
| Removing vocabulary from the wire changes the model | It moves the boundary. Retrieval is still token-driven. |
| A session teaches vocabulary, not capability | Wrong. Vocabulary *is* capability here. |
| §7.3 diagnosed the retrieval failures | Three causes, not one. |
| `Pet::restore` recomputes stage | An existing test caught me dropping that line. |
| A `.tex` with 48,000× | Unmeasured, against a simulated baseline. Removed. |

---

## 15. Open questions, ranked

1. ~~**Canonical skeleton hashing**~~ **DONE.** Identity is now computed over the
   skeleton, so the same capability has one address in any language and learning a
   phrase leaves the address alone. Cost: a narrow and a broad rule exposing the
   same primitives are now one artifact.
2. **Primitive-level addressing.** Induced signatures must be queryable. Without
   it the protocol dead-ends. `SetValue_CheckSense` still matches nothing.
3. ~~**D6**~~ **CLOSED.** `resolve_primitive` returned `SetValue` for anything it
   did not recognise, so 13 of 16 primitives were declared and dead, and a request
   to broadcast was silently dispatched as a plain set. It now covers the whole
   vocabulary and **returns `Err`** for anything else. Two subtler bugs surfaced on
   the way: substring matching fired on ordinary words (`widget` contains `get`,
   `offset` contains `off`), so it matches whole tokens; and a naive tokeniser
   split `SET_VALUE` into eight letters. Residual: a manifest's action id is free
   text, so the table still guesses less badly rather than not at all. A grammar
   for action ids is the same specification work as the constraint grammar.
4. **D5** — the constraint grammar. Undefined; the safety argument rests on the
   evidence gate alone.
5. **Typed correspondence relations** — `Opposes`, `Homonymous`, `Regional`.
6. **A capability check** against declared state space, which is what the 2
   remaining false positives need.
7. **Separating vocabulary-miss from capability-miss** in the metric.
8. **The corpus.** Two curated artifacts. Everything above is untestable at scale
   until this grows.
9. **Trademark** — clearance search, then filing. `TRADEMARK.md` records
   `ca(R)maduci` as not cleared and not filed while the symbol is used
   project-wide.

---

## 16. What changed since this record was written

Three blockers closed, in the order the discussion reached them:

| Closed | What it was |
|---|---|
| Skeleton hashing | identity included the aliases, so the same capability in two languages was two artifacts and learning a phrase created a sibling |
| Lexicon + four output forms | a creature could be found but only answered in prose; now it renders as prose, pseudocode, Rust, or opaque data from one address |
| D6 | `resolve_primitive` silently returned `SetValue` for 13 of 16 primitives, and for every unresolvable action |

**Two defects found while closing D6** that are worth more than the fix:

- Substring matching fired on ordinary words. `widget` contains `get` and
  `offset` contains `off`, so a request to read a widget became a value read and a
  request to set an offset became a shutdown. Matching is whole-token now.
- The first tokeniser split `SET_VALUE` into eight letters, because it split on
  every capital rather than at case *boundaries*.

Neither was visible from reading the code. Both were visible from asserting
behaviour.
