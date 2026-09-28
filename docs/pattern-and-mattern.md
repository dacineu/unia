# Pattern and Mattern

The project's vocabulary, and where it came from and where it stops being true.

There are two ways to keep a thing: hold it, or change it. A system that only
holds is stable and converges. A system that only changes is never the same twice
and never accumulates. Both are failures. This project has one name for the
first and one for the second, and this document is the argument that they are two
*forces* and not two *systems*, which is the part that is easy to get wrong.

---

## 1. The terms

From `cognition-log.md` §2, which is the only place they are defined:

> **Synaptic Transducer**: The sensory organ. It captures "**Patterns**"
> (probabilistic observations) and crystallizes them into "**Matterns**"
> (deterministic .ure actuators).

So:

- A **Pattern** is a probabilistic observation. Unresolved, word-bearing,
  uncertain, cheap. It is a *question* about what a thing is.
- A **Mattern** is a deterministic content-addressed actuator. Resolved,
  word-free, verifiable, retained. It is an *answer*, and it has an address.
- **Matterning** is the process of one becoming the other, in both directions.
  Crystallisation is the name for the Pattern → Mattern direction; a
  correction, a deprecation, or a pruning is the other.

A Pattern is addressed by its words. A Mattern is addressed by its skeleton.
**That is the whole distinction, and it is the one the project has already
enforced in code:** `skeleton()` in `src/identifiers/mod.rs` strips
`resource_id`, `guidance`, `description` and `aliases` before hashing, and the
surface lives beside the identity in the `Lexicon`. Two manifests that say the
same thing in different words are one Mattern.

### A collision to fix first

`mcp::store::Pattern` is **not a Pattern.** It is `{id, category, guidance,
actions, payload, source}` — a loaded corpus manifest, which is a Mattern. The
word currently names both ends of the pipeline, in the same crate, and
`store.rs` uses it for the wrong one. Until that is renamed, the vocabulary cannot
carry weight, because the type that should be the *product* is called the same as
the *input*. It is 16 references across 6 files, 6 of them inside `store.rs`
itself. Tracked in `TODO.md`.

---

## 2. The myth, and how to keep it honest

The memorable form, and the one to tell people:

> **The Pattern is the father. The Mattern is the mother.**
> The Pattern is what has been settled: principles you can rely on without
> doubting, the genetics, the part that has survived being tried. It secures
> itself and it keeps the best of what it has seen.
> The Mattern is the mother: the part that is cheap to be wrong. It is the one
> that changes easily, explores, and judges what the Pattern is about to keep.
> Neither makes a creature. Both are needed, and they are always in tension.

The tension is not decoration. It is [parent-offspring conflict][trivers-conflict]
(Trivers 1974) modelled honestly: the generator and the generated do not want the
same thing, and the resolution is a boundary rather than a harmony. The Pattern
wants to keep what worked. The Mattern wants to try things that have not worked
because nothing has tried them yet. Neither can be allowed to win.

### What the metaphor is *not*

**It is not inheritance, and the biology is clear about that.** Both parents
contribute comparably to the genome. There is no "pattern gene" and no "mattern
gene". The one genuine exception is [genomic imprinting][imprinting] —
parent-of-origin gene expression, which is a small minority of the genome and
should not be generalised into an architecture.

**It is not two kinds of system.** This is the load-bearing correction. See §4.

**It is not a claim that one sex is better at one of these.** Nothing in this
project's design should ever be justified that way, and the evidence does not
support it.

What the metaphor *does* give you, correctly: **asymmetric cost produces
asymmetric strategy.** A Pattern is expensive to make and cheap to keep, so it
optimises for not being wrong again. A Mattern is cheap to make and cheap to
discard, so it optimises for finding out. That asymmetry is real, it is
mechanically necessary in any system that must both keep and change, and it does
not require the sexes to be different kinds of thing to arise. [Parental
investment theory][trivers-pi] (Trivers 1972) is the canonical account of
exactly that mechanism, and it is about cost, not sex.

---

## 3. The lineage the project reinvented

The Pattern/Mattern distinction is, in the evolutionary-computation literature,
**quality-diversity**, and the fit is close enough to be worth stating precisely
because this project arrived at it independently.

**[MAP-Elites][map-elites]** (Mouret & Clune 2015, *Illuminating search spaces by
mapping elites*) partitions behaviour space into niches and keeps **one elite per
cell**:

> the behavior space is divided in niches ("cells") and only one species can
> occupy each niche (the elite). New individuals (generated by genetic variation)
> compete in their niche with the existing elite/niche.

Their word for the result is **illumination**: rather than converging on one
winner, it *"illuminates the fitness potential of every region of a chosen feature
space."* **"Storing the best from the rest" is the algorithm's purpose**, not a
poetic gloss on it.

The quality half alone is not enough. [Lehman & Stanley][novelty] showed that on
maze navigation and biped walking, **searching for novelty while ignoring the
objective significantly outperforms objective-based search** — drawing the
conclusion that *"objective functions themselves may actively misdirect search
toward dead ends."* Quality-diversity then combines the two, filling every niche
with the best example of that behaviour rather than converging on one global
optimum. The argument is in *Why
Greatness Cannot Be Planned* (Stanley & Lehman 2015): the whole reason to keep an
archive is that the thing that will matter is not visible in the current
objective.

The structural correspondences, which is the interesting part:

| Quality-diversity | unia |
| --- | --- |
| genotype — structure, the thing that is inherited | the **skeleton**, hashed |
| phenotype / behaviour descriptor — where it lands in behaviour space | `Capability` and `Sphere` in `src/gather.rs` |
| the archive of elites | the **corpus** |
| one elite per niche, retained | the **champion**, `set_champion` / `get_champion` |
| novelty pressure to fill unlit cells | **Matterns**; the unevidenced candidate |
| illuminate the whole space | `gather::gather`, denominators |

unia has a behaviour-descriptor space, a champion-per-region rule, a corpus, and
an archive — and had not named the field it was working in. That is worth
recording as evidence the structure is sound. It is not evidence the naming was
needed.

---

## 4. What the research does and does not support

This section exists because the myth is memorable and a memorable claim is easy to
over-read. The evidence is narrower than the metaphor.

**Not supported: two different kinds of system.** The quantitative record is
consistent and it says the distributions overlap almost entirely.
[Hyde's Gender Similarities Hypothesis][hyde] pooled 46 meta-analyses and 128
effect sizes: 30% of differences are trivial (d ≤ 0.10) and 48% small.
[Zell, Krizan & Teeter][zell] expanded this to 106 meta-analyses and 386 effects
and found a mean absolute difference of **d = 0.21 (SD = 0.14)**, *"largely
constant across age, culture, and generations."* Hyde's own summary carries one
caveat that belongs here: *"gender differences can vary substantially in magnitude
at different ages and depend on the context in which measurement occurs."* The
claim is about typical magnitudes, not about their invariance.

**The apparent counterweight is a measurement artefact.** Larger numbers do exist —
d = 0.73 for masculine versus feminine traits, and multivariate effects around
D = 2.06 to 2.71 — but they appear when many small effects are *aggregated* into
broad scales, and the correlational structure of personality is equivalent across
the two groups (**CC = .99**). The magnitude is a property of the instrument. The
methodology is contested in both directions — [Zuriff][zuriff] argues the
similarities hypothesis is untestable as formulated, in a formal comment answered
in the same issue — so the honest summary is that
the question is unsettled at the margins and not in dispute where it matters here.

**The parenting literature does not split by parent either.**
[Baumrind's][baumrind] four styles — authoritative, authoritarian, permissive,
uninvolved — are a 2×2 of **responsiveness × demandingness**, and what predicts
outcomes is warmth and structure, not which parent. The framework is also largely
mid-century American and its cross-cultural validity is contested. What is
sometimes observed is a difference in *behaviour during interaction* — more
physical and rough-and-tumble play from fathers, more emotional support from
mothers — and the effect sizes are small relative to the variability *within* each
parent.

**What is supported, and is all this document actually needs:** asymmetric cost
produces asymmetric strategy; the two strategies are in tension; and the tension
is resolved by a boundary rather than by one side winning. That is a claim about
mechanisms, it is testable, and it does not require the two forces to be different
kinds of thing.

---

## 5. The one clinical finding that is also a unia bug

Autism is diagnosed at roughly **4:1 male:female**, rising to **10:1** where there
is no intellectual disability. A 2024 systematic review and meta-analysis
[concludes][autism] that a substantial part of that is **camouflaging**:
*"Females used more compensation and masking camouflage strategies than males…
The results support the argument of a bias in clinical procedures towards males."*
ADHD shows the same signature — a clinical ratio near 4:1 against a community ratio
near 2:1, [narrowing toward 1:1 by adulthood][adhd].

The sharpest result is [Milner et al. 2023][milner], who measured camouflaging by
*discrepancy* rather than self-report — a much better instrument, because it
compares self-reported traits against observed ones. Diagnosed females scored above
diagnosed males on every measure. **In the high-trait, undiagnosed group there was
no sex difference at all**, and *effective* camouflaging was **highest in the
undiagnosed**. The masking correlates with not being caught, not with being female.

**This is the retrieval false positive, exactly.** `set the valve brightness to
eighty` scores 0.2197 against `valve-001` and is caught by a surface that matches
while the structure does not — `valve-001` declares `flow_rate` and `status`, and
there is no `brightness`. The ratio of false positives to true positives is a
property of the *instrument*, in the same way a 4:1 diagnosis ratio is a property
of the diagnostic procedure. The artifact did not change. The matcher decided.

Two design consequences, and the second is the general one:

1. **You cannot fix camouflage by sampling more of the same thing.** You fix it by
   measuring the thing the surface was hiding. Which is why the contradiction gate
   is a capability check against `state_space` and not a better score, and why no
   threshold will ever separate 0.2197 from 0.2041.
2. **An instrument that under-reports is worse than one that over-reports.** A
   masked capability is invisible, and an invisible capability cannot be routed,
   verified, or learned from. This is the strongest available argument for
   declaring capabilities rather than inferring them, and it is the same argument
   as P2 in the intermediary plan: a primitive that cannot be named is a primitive
   that cannot be called.

---

## 6. Where the project is today against this

Measured, not intended.

| | Pattern side | Mattern side |
| --- | --- | --- |
| **Learn** | observed phrasings | grouped by `primitives.join("_")` — **already vocabulary-free** |
| **Act** | the request | the packet; `Form::Data` needs no human language |
| **Harvest** | documents, sessions, source text | declared structure — **does not exist** |
| **Reason** | an observation in words | a sequence resolving to an address — **blocked: no evaluated semantics** |
| **Solve** | a goal in words | goal → evaluable step — **blocked: same** |
| **Capabilit[y]** | — | declared but **not checkable**, D5 |
| **Teach** | a correction in the user's words | the trace — **blocked, self-aware, waiting on the vocabulary** |
| **Evolve** | — | Mattern → Mattern′ with the skeleton changed — **currently a no-op** |

Three findings worth restating, all measured in this repository:

**`prune_fat` is a Pattern-only policy, and it is wrong here.** It filters to the
champion alone, discarding everything else. `src/gc.rs` does the opposite and never
deletes anything — it changes a lifecycle field. Nothing in `src/` deletes a `.ure`
at all. So the codebase's actual behaviour is retention, which is the
Pattern-and-Mattern policy, while the function named "trim the fat" encodes
Pattern-only pruning — precisely the failure quality-diversity exists to prevent,
where optimising quality alone converges and the archive goes empty. It has **zero
callers**, so fixing it costs nothing. Recorded as divergence D10.

**Mutation renames without changing behaviour.** `MutationEngine::mutate` appends a
marker to `guidance` and adds a `provenance` block. `guidance` is in the skeleton
deny-list, so it contributes nothing to the address; `provenance` is not, so the
address moves. Actions and state space are byte-identical afterwards, and the
skeletons are equal once `provenance` is dropped. **The child is a different
Mattern of the same thing, achieved by recording that it was mutated.** A Mattern
that can change identity without changing behaviour is not a Mattern.

**The vocabulary-free path starts with vocabulary.** Harvest produces Patterns, and
nothing in the current design produces a Mattern without a Pattern first. If the
thesis is that a primitive sequence is the interface, then harvesting *structure*
directly — from something that was never prose — is the missing first step. It is
the one capability in the table above with no Mattern-side plan at all.

---

## 7. The test for whether this vocabulary is real

A distinction that changes no dependency is a distinction that costs a paragraph.
This repository has a history of that: the old README's "Punctuation Layer" named
a component that did nothing, and the predecessor project carried an entire
`amater` vocabulary that outlived its code.

**The test:** Pattern and Mattern must be distinguishable by something checkable.
They are:

- Pattern is **probabilistic**, Mattern is **deterministic**.
- Pattern is **word-bearing**, Mattern is **word-free**.
- Pattern is **unaddressed**, Mattern has a **content address**.

Three checkable differences, and the skeleton hash enforces the second and third in
code. `mcp::store::Pattern` violating it is the proof the distinction is real and
being broken, rather than one that does not exist.

If a future change makes a Pattern and a Mattern indistinguishable on all three,
the pair should collapse to one word.

---

[hyde]: https://www.apa.org/pubs/journals/releases/amp-606581.pdf
[zell]: https://pubmed.ncbi.nlm.nih.gov/25581005
[zuriff]: https://pubmed.ncbi.nlm.nih.gov/26436318
[baumrind]: https://pmc.ncbi.nlm.nih.gov/articles/PMC6323136/
[trivers-pi]: https://en.wikipedia.org/wiki/Parental_investment
[trivers-conflict]: https://en.wikipedia.org/wiki/Parent-offspring_conflict
[imprinting]: https://en.wikipedia.org/wiki/Genomic_imprinting
[map-elites]: https://arxiv.org/abs/1504.04909
[novelty]: https://pubmed.ncbi.nlm.nih.gov/20868264
[autism]: https://pubmed.ncbi.nlm.nih.gov/38285291
[adhd]: https://www.thelancet.com/journals/lanpsy/article/PIIS2215-0366(24)00010-5/fulltext
[milner]: https://pubmed.ncbi.nlm.nih.gov/36490366
