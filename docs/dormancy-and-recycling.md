# Dormancy, self-cleaning, and recycling without a genealogy

A design note, not a specification. Written down before any of it is coded,
because two of the three parts contradict something the architecture currently
relies on and the contradiction is more interesting than either design.

Dormancy is done. It is in `src/camaduci.rs` and this note records only what
the other two parts would have to mean.

---

## 1. What dormancy settled, and what it left open

A creature that falls below `DORMANT_BELOW_HEALTH` is dormant: alive, holding
nothing it learned, power at the floor, resources untouched. It will not act for
itself. A person acting on it wakes it.

That was meant to be one state among three. It turned out to be the answer to a
question that had been open much longer, and the connection is the reason the
other two are tractable now:

| | before | after |
| --- | --- | --- |
| health reaches zero | ~10 ticks | ~13 |
| `stuck` — resources, no power | ~16 | 12, the tick before death |
| `empty` — no resources | ~24 | still ~24 |
| anything at all recoverable | never | one tick, while alive |

The economy is now legible before death. That is what the rest of this note
assumes.

---

## 2. The two parts, stated plainly

**Self-cleaning.** The creation cleans itself. Three named triggers:

- **unknown situations** — an intent that resolved to no act the creature knows
- **chaotic events** — an act that failed, or a precondition that could not be met
- **readiness patterning with matterning** — a rule whose readiness has gone, and
  which is therefore re-matterned at a fresh address rather than deleted

**Recycling without a genealogy.** Other creatures take what they need from a
culled one. What nobody takes vanishes. And there is deliberately *no
traceability in the genealogy*: the new artifact is not a child of the old.

---

## 3. The contradiction, and the resolution I think is right

`Denominator` has a `contributors: Vec<String>`. That **is** a genealogy. Two
artifacts that agree on a denominator are recorded as having been found by named
parties, and the project can answer "who found this capability". "No traceability
in genealogy" says that question must not be answerable.

Left alone these are irreconcilable. But the project already has a distinction
that resolves them, and it is the one from §2 of the translucency chain:

> **Patterning is a meet. Acting is a monoid.**

- The **meet** is the capability: the largest primitive sequence substitutable in
  two witnesses. It is commutative, associative, idempotent, and a partial order
  has no maximum — only maximal elements.
- The **monoid** is the word: `feed; sleep` ≠ `sleep; feed`. It is the *history*.

So recycling can keep the meet and drop the word. A capability survives; the
chain of who found it does not. Concretely:

- A re-mattered artifact gets a **new address** — its content changed, so it must.
- Its `signature` — the meet — is **unchanged**. Same capability, different
  artifact.
- `contributors` becomes a **census of the current generation**, not a lineage.
  It answers "who has reached this lately", never "who found it first".

### Why this is not a loss but a measurement

The project has never run the convergence test it most wants: *creatures that
never met, agreeing anyway.* Twelve hand-written patterns share **zero**
capabilities. Thirty-six generated artifacts share six denominators they were
**manufactured** to share, which is a uniformity and not a convergence.

Recycling without a genealogy is what makes that test possible. A denominator
reached by six creatures across six generations — none of which can trace the
others, by design — is a genuine convergence. The moment `contributors` stops
being a lineage, its cardinality starts meaning something.

So the paper's central measurement, escalation rate, has to be stated per
**signature** and not per **address**. That is a real change and it is a
correction in the right direction: escalation in capability is what the claim is
about, and reach was always the easier number to mistake it for. The corpus
measurement already showed the distinction — 36 artifacts escalating in reach and
in no capability at all.

**What this costs, stated plainly.** A store can no longer answer "is this
artifact a descendant of that one", and the escalation rate can no longer be
computed by counting reuses of an address. Both are recoverable — address reuse
is a set membership test — but neither is free.

---

## 3a. What was measured once it was built

Both parts are in `src/clean.rs`, and the convergence claim is no longer an
argument. Six generations, each authoring its own mattern at an address of its own
choosing, teaching it a vocabulary nothing else had, and then being culled with
nothing but failed traces left behind:

| | |
| --- | --- |
| generations | 6 |
| capabilities afterwards | **1** |
| distinct addresses afterwards | **1** |
| surviving artifacts naming a culled one | **0** |
| re-mattering an already-clean corpus | **changes nothing** |

That last row was a bug and then a property. The cleaner first reported a
re-matter whenever anything was stale, so `changed()` came to mean "something was
stale" rather than "the corpus differs", and a second pass over a clean corpus
claimed to have cleaned all eleven of its matterns. Being stale and being
*replaced* are different facts: a mattern with no evidence may already be
canonical, and the fix is that a replacement is recorded only when the result
differs. What is left is a genuine fixed point, which is what makes the cleaner
safe to run repeatedly.

The address is also a stronger thing than this note claimed. It was written as a
content address of "the signature and the phrasings still standing", and a test
appeared to confirm that. It did not: `DuUuid::generate` runs its input through
`skeleton`, which drops `aliases` as surface, so three vocabularies produced one
identical address and the test was asserting a coincidence. The architecture is
right and the note was wrong — identity is the act, the phrasings are the surface,
and two creatures that learned different words for the same thing *are* the same
artifact. That is also what makes the run above converge at all.

---

## 4. The horizons, and what "trim the fat" would mean there

Three horizons, already half-separated by the economy:

| horizon | quantity | direction | trimmed by |
| --- | --- | --- | --- |
| short | `nuants` | spent | neglect, absolutely |
| middle | `quants` | production only | dormancy, to the floor |
| long | signatures, address | neither spent nor sustained | self-cleaning |

The long horizon is the one that does not exist yet, and it is the one the
creation cleans. If `nuants` and `quants` are the two economies a creature lives
in, then the *matterns* are the third, and self-cleaning is the thing that keeps
the third one from becoming an archive.

The obvious form of the trim, and it is a guess:

- a rule nobody has confirmed within some window is **re-mattered**, not deleted
  — a new address, the same signature, so the capability is intact and the
  evidence is not
- an artifact that no denominator reaches is dropped
- a denominator with no artifact behind it disappears

Re-mattering rather than deleting is the load-bearing choice. Deleting a rule
deletes a capability and the creature forgets; re-mattering it leaves the
capability and throws away the evidence. That is dormancy applied to an artifact
instead of to a creature, and it is why the two belong in the same note.

---

## 5. Open, and needing a decision

1. **What is the trigger, and who runs it?** Self-cleaning could be a background
   process, a per-request step, or something the player pays for. Each is a
   different game.
2. **How long is "nobody has confirmed it"?** A window in ticks is a tuning
   number pretending to be a principle. Maybe it should be relative to the
   creature's own history — a rule not confirmed since it learned its last act
   is stale — which is scale-free but circular.
3. **Is "chaotic event" a failed precondition, a failed act, or a trace with
   `succeeded: false`?** The last already exists in the schema and would be the
   honest answer, because it is the only one that is recorded.
4. **What is "readiness" here?** `Denominator` has a `Readiness`, and
   `Sensor::Inferred` has **no inhabitant** — nothing dispatches a sequence to
   evaluate a reading. Readiness patterning is that gap, and closing it may be
   most of the work.
5. **Does a re-mattered artifact count as escalation?** If not, the escalation
   rate is honest by construction and needs no caveat. If yes, then reaching a
   denominator again is progress, and the metric has to say so.

Items 3 and 4 are the ones that block. Item 3 is answerable from what is already
recorded; item 4 is the empty square in the architecture that "readiness
patterning" is pointing at.
