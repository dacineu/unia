# LPMM training plan: from imported origins to self-harvest

The endpoint: **the Large Patterns Matterns Model harvests and trains itself,
with no language model in the loop, and consults a more specialised model only
when it holds a doubt it cannot resolve from its own evidence.**

This document is the plan. It states the endpoint precisely, names what stands
between the repository and it, and orders the work so that each phase can be
*exited by a measurement* rather than by a feature landing. A phase that exits
because a function was written is not a phase.

`unia` today: 482 tests, 0 shared capabilities across 12 hand-written patterns,
6 denominators in 36 generated artifacts that were **manufactured** to agree,
and a corpus that no caller has ever written to.

---

## 0. The one distinction this plan is built on

> **The origin can be imported. The certitude cannot.**

A `.ure` has two separable parts and conflating them is the failure the whole
architecture exists to prevent:

| | what it is | can it be imported? |
| --- | --- | --- |
| **the act** | a signature — the largest primitive sequence substitutable in two witnesses | yes, from CILI, BabelNet, BFO, or anyone |
| **the certitude** | `confidence` — earned by distinct confirmed phrasings, decayed by neglect, floored by dormancy | **no. It has no import format, because it should not have one.** |

So an imported corpus starts with acts and no certitudes, and by the LPMM's own
credit function it is **inert**: `Production::since` credits nothing for a
candidate no trace confirmed, and `clean` re-matters it on the first pass. That
is deliberate and it is the answer to the concern in the LLM-ontology literature
that unverified extraction yields "plausible yet incorrect facts"
([Echo-LLM](https://www.tib-op.org/ojs/index.php/ocp/article/view/3173)). The
architecture does not need a rule against hallucinated triples. **The credit
function already makes them worth nothing.** That is a stronger position than a
safety policy and the paper should say it in those words.

---

## 1. What stands between here and the endpoint

Five blockers, named, with the measurement that would clear each. All five are
the *same* missing thing seen from different directions — the trace log — and the
plan converges on it by phase 2.

1. **The primitive vocabulary cannot express a program.** Sixteen verbs, all state
   or flow: `GetState GetValue CheckSense SetValue Toggle Increment Reset Route
   Pipe Broadcast Delay Watch Pulse Compare Transform Validate`. No binding, no
   definition, no call, no allocation, no recursion, no emission. A `.ure` is
   therefore necessarily a finite sequence of hardware verbs — a manifest, never a
   program. **This caps what self-training can ever produce**, and it is a
   design decision, not a missing feature.
2. **The creature knows four acts.** `new_signatures` saturates at four, so the
   implied economy equilibrium sits *below* the peak and a creature that has
   learned everything it can still declines. Pinned as a test.
3. **No caller writes a trace.** `Actor::Caller` exists, `Store::record` exists,
   every field exists — and nothing constructs one. The corpus is empty because
   nothing is listening, not because nothing can speak.
4. **The escalation rate is unmeasured**, and it must be per *signature* rather
   than per address. The corpus evidence already says why: the generated corpus
   escalates in reach and in no capability at all.
5. **Three of the five `MetaActuatorType` variants are stubs** that return
   hardcoded JSON asserting work they did not do — including a `Mutator`
   reporting `"EVOLVED"` and a fabricated `"performance_gain": "+15%"`, and a
   `Synthesizer` returning `HYBRID_CREATED` with a fresh `Uuid` and no artifact.
   The self-training machinery is the part of the system that is most declared and
   least built, which is the opposite of what a training plan needs.

---

## 2. Phases

Each phase names its exit measurement. Phases 0–3 are in reach now; phase 4 is the
endpoint; phase 5 is the proof.

### Phase 0 — Make doubt a value the type can hold

`Verdict` is three-valued and cannot say *a constraint is violated and the
violation is tolerated*. `clean::clean` already does exactly that — it keeps a
capability whose evidence contradicts itself and discards the contradiction — and
does so without the type to declare it.

**Exit:** `Verdict::Both` exists, `clean` returns it as a typed reason, and a test
forces the case that needs it — a capability confirmed *and* contradicted at once.
*Open decision first: lattice or residuated?* The plan's own earlier claim that
Kleene's K3 has no greatest lower bound is **wrong** — K3's order is the chain
`0 < ½ < 1` and is a distributive lattice. The real discriminator is
**residuated** structure, since K3's implication is non-monotone. Belnap's four
values form a distributive lattice. Whichever is chosen changes which
three-valued logics are admissible, and it must be decided before an imported
concept layer is committed to the skeleton, because that is expensive to move.

### Phase 1 — Give the source a vocabulary

The runtime alphabet cannot hear a maintainer. There is no encoding of "replace
this span, given a failure of kind E0308" in sixteen state verbs, so the one event
that could teach the system to code is the one it cannot hear.

Two alphabets, deliberately apart: the sixteen for *what happened to the world*,
and a small source-edit set (`ReplaceSpan`, `InsertSpan`, `DeleteSpan`,
`RenameSymbol`, `AddTest`, `RunTests`) for *what was done to the code*. Mapping
one onto the other would be a lie dressed as uniformity.

**Exit:** every edit in one session is retrievable as a trace with a real
primitive sequence, and `authorship` counts them.

### Phase 2 — The caller writes the trace

The cheapest and largest single change. Wrap the edit; write the trace; let
induction run on it with no other change required.

**Exit, and this is the number the paper needs:** a corpus exists that no one
authored by hand, with a measured `authorship` ratio, and a measured escalation
rate per signature. **The escalation rate stops being unmeasured here and
nowhere earlier.**

### Phase 3 — Close the loop, with a verifier

Induction proposes; the test suite adjudicates. A produced edit is production only
if the suite still passes — which is `Trace.succeeded`, the field that already
exists and that is already the "chaotic event" trigger in the self-cleaning pass.

A sequence of edits that break tests is rumination, and the machinery for that is
built: credit only for what is new, dormancy forgets what was never confirmed,
`clean` re-matters what went stale.

**Exit:** an edit proposed by induction, verified by the suite, and promoted to a
mattern — with the promotion counted, and the failures counted separately.

### Phase 4 — Route doubt outward

When a `Neither` or `Both` arises and the declared acts cannot resolve it,
consult a more specialised model. Two constraints, both non-negotiable:

- the response is a **candidate**, not an install. It must earn confirmation from
  traces like anything else, so a specialised model's contribution is credited
  only by evidence.
- the query is recorded as `Actor::Caller` with its own scope, so the authorship
  ratio measures exactly how much of the corpus is self-harvested.

**Exit:** doubt is routed, answered, and the answer is *not* trusted until
confirmed — measured by the ratio of routed answers that survived to matterns.

### Phase 5 — Remove the model

The endpoint, and it must be **proved by measurement rather than declared by
switching a flag**: a corpus harvested with no external model, against one
harvested with, on the same escalation rate per signature.

**Exit:** the self-harvested corpus meets or beats the LLM-assisted one, and the
authorship ratio is above a stated threshold with no intervention. If it does
not, the honest result is that the endpoint is not reached, and that is worth
more than a green flag.

---

## 3. What is imported, and when

Deferred past phase 2 deliberately, because seeding before there are traces
produces a corpus that looks populated and is inert.

| resource | gives | licence/format | phase |
| --- | --- | --- | --- |
| [CILI](https://github.com/globalwordnet/cili) + [BabelNet](https://babelnet.org/about) (500 languages, incl. Romanian WordNet) | the concept layer, the stop-lists, English↔Romanian alignment as a **data join** | wordnet/tab formats | 3 |
| [BFO](http://purl.obolibrary.org/obo/bfo) / [DOLCE](http://www-igm.unice.fr/~donna/) | the upper ontology as a table | OWL/RDF | 3 |
| [Navigli & Paprika, IJCAI 2021](https://www.ijcai.org/proceedings/2021/0620.pdf) | the survey, for the related-work section | — | — |
| an LLM-drafted ontology | plausible triples with no provenance | — | **never as certitude** |

BabelNet is the fix for the `FUNCTION_WORDS` asymmetry recorded in the code as "a
known asymmetry rather than a decision": with a CILI-linked stop-list the
transducer stays language-independent and Romanian arrives as data, so the fork
disappears rather than being maintained.

---

## 4. Blocked on other people, not on effort

- **The 200-query fixture.** Must be authored by someone who has not read
  `src/bridge/primitive.rs` or `src/meet.rs`. Whoever writes it will write in the
  vocabulary the matcher handles, and the escalation rate becomes a measurement of
  that author's assumptions — the same failure the corpus generator committed once
  by manufacturing the convergence it then reported. This gates the paper's
  central number and nothing in this plan substitutes for it.
- **The lattice-or-residuated decision** (phase 0).
- **The act-set design** — whether a creature's capability becomes a *shape* (four
  slot bindings, per the `MicroNucleus` already in the tree) rather than a set of
  acts. This decides whether phase 5 is reachable.
- **Trademark and `euni.me`**, for anything public.

---

## 5. Two things this plan will not do

**It will not widen `UniversalPrimitive` to cover knowledge.** Seventeen hardware
verbs is not an ontology. Growing that list is how the "declared, never wired"
pattern gets its seventh instance, and four of its five existents were found in
this session.

**It will not use a converter between logics.** A conversion can be wrong
silently and still produce a well-formed result. The swapped X and Z conjugations
in the quantum attempt produced a *perfectly valid* stabilizer group describing
the wrong state, and no structural test caught it — only a second independent
implementation did. The same applies to base-8/16/32/64 and to 3- versus
4-state: **build both and assert they agree.** That is how the meet, the retrieval
check, and the corpus generator were each caught.
