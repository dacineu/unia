# Session record: the economy, dormancy, the meet, and the market

A complete record of one working session on `unia`, written to preserve the
*reasoning* rather than the conclusions — including every place the reasoning was
wrong, because that is the part worth keeping. 482 tests pass at the end of it,
from 382 at the start; 14 commits, `5e54ed5` to `fbe35d4`; nothing fabricated, and
the three things that could not honestly be done are named as such at the end.

Every number in here was measured in the tree, and the measurements are quoted
rather than paraphrased so a later reader can tell which claims were checked.

Written in the first person, as the paper is. This is a log, not a paper: it
contains the dead ends, the failed hypotheses, and the measurements that
contradicted my own claims.

---

## 0. Where the session started

The project had a working game (`ca(R)maduci`), an economy with two quantities, a
creature that could feed itself, and a retrieval baseline that scored 20/20 true
positives with 2 false positives and **no threshold that separated them**. A paper
was in preparation. §9 "What is not demonstrated" was load-bearing.

The central metric — **escalation rate** — was still unmeasured, because the corpus
generator produced *reach* escalations (aliases) and no capability escalations at
all. The generator manufactured the convergence it then reported, which was noticed
only because the result looked too clean.

The session was spent on five threads, in this order: the economy, the vocabulary,
dormancy, the meet, and the market.

---

## 1. The economy paid for repetition

### 1.1 The prompt

The user pointed at a specific word: **cuante** was "a 0.0..=1.0 rate that decays
unless fed", and then said, in effect, that my version of it was *useless*:

> useless cycles consumed, repetition without production — inflation with
> recession in economic terms or infatuation, jealousy, envy, triumph etc. in
> psychological terms

That is a diagnosis of a specific defect, and it was correct. Repetition was
producing the power. A currency that pays for repetition pays for rumination.

### 1.2 Measured before anything was changed

Twelve identical feeds. The **same sentence** every time. Zero rules learned, zero
new aliases.

```
act#   power   resources  rules  new-alias
   1    0.520        11.00      0           0
   4    0.575         8.00      0           0
   8    0.639         4.00      0           0
  12    0.694         0.00      0           0
```

**Power rose 0.500 → 0.694, a 39% gain, for nothing.** And `LearnedRule.confidence`
never even appeared in the run — my earlier claim that "you already built the
primitive" was aspirational, not implemented. That was worth recording: the
primitive was named in conversation and absent from the code.

### 1.3 The correction

**An act spends. Induction pays.**

- `Care::apply_economy` no longer credits the power. It spends `nuants` and
  *debits* the power.
- The only path to power is `Pet::learn`, which is handed an induction result and
  can see what is in it that was not there before.

Same twelve acts, after: **0.500 → 0.157**.

The structure of the credit, `Production`, is four countable integers:

| counter | worth | what it is |
| --- | --- | --- |
| `new_rules` | 0.10 | a new act, induced, at an address nobody had |
| `new_signatures` | 0.05 | a new capability, regardless of who found it |
| `new_phrasings` | 0.01 | a new way of asking for a known act |
| `consolidated` | 0.02 | a rule already known that became *more reliable* |

A new act is worth far more than a new sentence for a known act, and that is a
claim about what matters: nobody gained anything because the same sentence arrived
in a different language, and a great deal because a new act became possible.

### 1.4 Two counters that are deliberately different

`new_rules` and `new_signatures` sound redundant. They are not, and the reason is
the project's own corpus measurement: the generator escalates in **reach** (new
addresses for the same act) and in **no capability at all**. Paying for both at one
rate would be paying for the corpus being uniform and calling it an escalation.

So a new address carrying an already-known signature is new reach, and is paid as
neither a rule nor a signature — only as the phrasing it brought.

### 1.5 The fourth counter, and why it had to exist

The first three saturated. There are four built-in `Care` acts, so `new_signatures`
stops at four, after which the only credit is a new phrasing worth 0.01 against a
0.15 debit. Measured:

```
act#   producer  repeater  ratio
   1      0.529      0.440  1.204
   4      0.577      0.309  1.867
   8      0.403      0.209  1.928
  12      0.236      0.157  1.512
```

A creature that produced on **twelve of twelve** acts ended weaker than one that
produced on five. A currency with no sustaining term is a tax, and the tax falls
hardest on whoever plays longest.

`consolidated` is `LearnedRule.confidence` — which is exactly the primitive the
user had named at the start and I had dismissed as aspirational. **It is not
inflation**, and the reason is a property of the type rather than a rule anyone
remembered to write: `observations` counts *distinct* phrasings, so the same
sentence arriving again raises nothing. Consolidation needs new evidence, and it
saturates.

### 1.6 Four tests in a row were wrong, and the failures were the information

This is the part worth keeping. Each of these was a test I wrote, asserted, and then
had to rewrite because the assertion was wrong rather than the code:

1. **`at_equal_acts_a_producer_is_much_stronger_than_a_repeater`** asserted a power
   above 0.5. An arbitrary number I had not measured. Failed at 0.417.
2. Rewritten to assert a *ratio* above 2.0 — the same mistake comparatively. Failed
   at 1.50, because the test helper pinned every rule's confidence at 0.8, so
   consolidation could never fire and the producer was paid novelty alone. **A
   fixture that quietly makes the economy it is testing unrepresentative.**
3. Rewritten to assert the ratio *widens* over 20 acts. Failed at 1.99 at ten acts
   — not because the economy was wrong but because ten acts is not where the number
   I wanted lives. The ratio is 1.20 at one act and 2.47 at twenty.
4. The final version asserts the *shape*: production beats repetition at **every
   act**, and the margin grows monotonically. No magic threshold.

And the honest limit, pinned as a test rather than balanced around: the producer
**peaks at 0.577 on act four** and declines to 0.323 by act twelve, because the
implied equilibrium from a 0.03 credit against a 0.15 debit sits *below* the peak.
Any creature that has learned everything it can currently learn ends up weaker than
one still learning. That is blocked on the act set, not on the economy.

### 1.7 The liveness hole the correction opened

Separating credit from cost exposed a liveness bug that reading had not:

> a stuck creature cannot act → cannot tend → cannot produce → is never credited →
> stays stuck for good

Closed by **two entry points** rather than by weakening the economy:

- `Pet::tend` is the **player's** door, and has no power or affordability check.
- `Pet::tend_as_self` is the **creature's** door, and requires usable power.

Being helped and helping yourself are different events, and the asymmetry between
them is most of what the game is about. A stuck creature is lifted by being
**taught**, not by being fed — which replaced an older test asserting that a free
act of sleep was the way out.

---

## 2. Vocabulary: quants, nuants, and the terms that were crossed

The user asked for the English plurals. `cuante` and `nuante` were Romanian; the
paper is in English. 133 references across 3 files, renamed to **`quants`** and
**`nuants`**, field names and JSON keys alike.

The semantics are unchanged and the crossing survives:

> In quantum computing a quantum is a unit of *consumed* computing power. Here
> **a quant is productivity** — a power the creature has and sustains and never
> spends — and because that word is then taken, the thing it actually spends needs
> a name of its own: **nuants**, which in plain terms is resources.

| | kind | failure | means |
| --- | --- | --- | --- |
| **quants** | a power, sustained | `stuck` | it has nuants and no power |
| **nuants** | a stock, spent | `empty` | it has power and nothing to act with |

The docs section describing the economy said "Actions spend nuante and raise
cuante", which had been wrong since the credit moved to `learn`. Rewritten, with
the measured before and after (0.500 → 0.694, now 0.500 → 0.157).

---

## 3. Feeding brings nuants

Found by **playing**, not by reading. Three concerns, in the same commit.

### 3.1 The dead end

`nuants` were only ever decremented. `apply_economy` and `decay_economy` are the
only writers outside tests and both only subtract. So a creature that spent them
all was unrecoverable — and splitting `tend` from `tend_as_self` had removed the
*power* gate from the player's door while leaving the *affordability* gate on both.
Three teaching phrasings against a creature at `nuants: 0.00` were all refused, and
nothing could raise it. Reached in about eight acts.

### 3.2 The rule

The direction of the stock depends on **who acted**:

- a person feeding a creature **brings** nuants — hands it something to act with;
- a creature feeding itself **spends** what it already had.

Only `Feed` does this. If every act were free to the player there would be no
economy; if none were the player could rescue nothing. A person feeding also does
**not** debit the power: the debit is for a creature repeating itself, and someone
holding a bowl is not that.

### 3.3 The loop, working, for the first time

Verified in play, and the whole economy became visible doing it:

```
feeding "pour some kibble"  → nuants 12 → 13, quants 0.500
feeding "serve the food"    → nuants 13 → 14, quants 0.585
```

The power rose because the second phrasing induced a rule and `learn` paid for it.
**Bring food, teach a new word, production, power rises.** That loop had never once
completed in play before this change.

Seven tests changed, all of them by conflating the two doors — the production tests
were using the *player's* door to model the creature repeating itself, which
stopped meaning anything once the doors diverged.

---

## 4. Dormancy: the failure that has an exit

The user introduced it: *"the creature has a state of dormant and gets lost also as
knowledge."*

### 4.1 What it is

`DORMANT_BELOW_HEALTH`. Dormant means: alive, holding nothing it learned, power at
the floor, resources untouched. It will not act for itself. A person acting on it
wakes it.

Three states now exist where every failure used to be terminal:

| | before | after |
| --- | --- | --- |
| `stuck` — resources, no power | ~16 ticks, **after death** | 12 ticks, the tick before death |
| `empty` — no resources | ~24 | still ~24 |
| anything recoverable | never | one tick, while alive |

That resolves a question open since the floor work: the economy used to be legible
only on a creature that no longer existed. It is now legible for one tick on
something **alive and reachable**.

### 4.2 Four bugs, all found by playing, none by reading

**1. `health` never reached zero.** Subtracting 0.10 from 0.1 in binary floating
point leaves ~1.1e-16, which is simultaneously `> 0.0` and `< 0.1`. So a neglected
creature went dormant and then **could not be killed** — `alive()` stayed true,
`neglect` returned before quarantining, and it sat at zero health forever. The unit
test missed it because the health trajectory it happened to take landed on 0.0
exactly. Fixed by snapping below 1e-9, and pinned by a test that starts from
**twenty different health values**.

**2. The threshold was unreachable by construction.** At 0.1 — one decay step above
zero — a creature steps *over* the band and goes from healthy to dead without ever
being dormant. **A threshold is only reachable if the band it names is wider than
the step that crosses it.** Moved to 0.2.

The comment on that constant had claimed the ninth tick, having read the 0.10 and
ignored the strain gate (`if strain > 0.7`). Reading a decay rate as a schedule is
the mistake the model invites.

**3. `learn` woke the creature on any non-empty rule set.** Induction re-derives a
creature's rules from its own trace log, so it woke itself on the very next request
from evidence written *before it slept*. A dormant creature has to be woken from
outside, or it was never asleep. The wake moved to `tend`, and only a person
performs an act that counts.

**4. `neglect` had its own weaker copy of the transition.** It cleared the rules and
set the flag but neither counted the loss nor floored the power — so a creature
that fell asleep by neglect kept its forget-count at zero and was credited for its
own last rules a moment later. **Every test passed because they all called
`go_dormant` directly**, which is the only way it ever happens in the game. One
transition, one place.

A fifth, found immediately after delegating: `go_dormant` also emptied `nuants`,
which meant every neglected creature ended `empty` and `stuck` became unreachable by
neglect **for the second time in this file**. A creature that has forgotten things is
not starving.

### 4.3 Verified in play, end to end

```
taught two words                    learned=1  quants=0.585
neglected for a minute              quants→0.184  nuants 14→8.5
t+59.5s  h=0.100  posture=stuck  quants=0.100  learned=0
  it refuses to act for itself      "Nothing is pulling me right now"
  a person teaches it "futter es"   dormant=False  learned=1  quants=0.262
  address unchanged                 1cc37775-4901-43a8-8230-e4d3518a9046
```

**The address does not move.** Learning a language must not move it, and forgetting
is learning's inverse, so a creature and the one it used to be are the same
creature. What is lost is evidence, not identity.

### 4.4 Two tests whose premise was "inferred has no inhabitant"

See §7.3 — the same shape of error, on a different subsystem.

---

## 5. Self-cleaning, and the three relations

### 5.1 The long horizon

A creature lives in two economies: `nuants` spent (short), `quants` sustained
(middle). Both are reached by decay. Nothing had ever cleaned the third — the
matterns themselves — so a corpus only ever grew.

A mattern nobody has confirmed is **not a capability the project has lost; it is
evidence it has not earned.** So it is re-mattered, not deleted: same signature,
same phrasings, new address, confidence and observation count discarded. Deleting
would delete the capability and the creature would forget — dormancy applied to an
artifact instead of to a creature.

### 5.2 All three triggers are read out of the trace log

Because a trigger nothing records cannot fire honestly:

- **unknown situations** — a successful trace no mattern covers;
- **chaotic events** — `succeeded: false`, the only failure the schema already
  records, which is why it is the answer and a failed precondition is not, since a
  precondition that could not be met leaves no record of its own;
- **readiness patterning** — a mattern with no confirming trace. For a `Readiness`
  that is a *maximum* over evidence, evidence that has stopped arriving stops
  counting immediately.

### 5.3 No genealogy, and what it made possible

The re-mattered address is a content address of the **signature alone**. Not a hash
of the old address: chaining them would be a genealogy wearing a disguise.

That was documented as hashing "the signature and the phrasings still standing",
and a test appeared to confirm it. **It did not.** `DuUuid::generate` runs its input
through `skeleton`, which drops `aliases` as surface — three vocabularies produced
one identical address, and the test was asserting a coincidence. The architecture was
right and the note was wrong: identity is the act, the phrasings are the surface, and
**two creatures that learned different words for the same thing are the same
artifact.** That is also what makes the generations below converge.

**And it made the convergence test runnable.** The project had never produced one:
12 hand-written patterns share **zero** capabilities (one author, one intent,
twelve solitaries), and 36 generated artifacts share 6 denominators they were
*manufactured* to share. In both cases the arrival was arranged.

```
six generations, each authoring a mattern at an address of its own choosing,
taught a private vocabulary, then culled leaving only failed traces:

  capabilities afterwards          1
  distinct addresses afterwards    1
  survivors naming a culled one    0
  cleaning an already-clean corpus  changes nothing
```

The last row was a bug and then a property. `changed()` came to mean "something was
stale" rather than "the corpus differs", so a second pass over a clean corpus
claimed to have cleaned all eleven of its matterns. Being stale and being *replaced*
are different facts — a mattern with no evidence may already be canonical. What is
left is a genuine fixed point.

### 5.4 The handover: the one relation that keeps a genealogy

| relation | what it is | provenance |
| --- | --- | --- |
| **handover** | I give you *this* artifact | **preserved** — that is the act |
| **convergence** | we arrived at the same capability independently | **severed**, by design |
| **matching** | I think that artifact might fit this call | provisional, and scored |

Keeping both of the first two is what makes the third measurable: an escalation rate
per address counts a handover as escalation, which is a *transfer* and not a
capability; per signature it counts a convergence. The two cannot be told apart
without the record — which is the argument for recording the handover rather than
treating it as redundant with cleaning.

**A gift arrives as a fresh claim.** `receive` re-matters what it is given, the same
operation the cleaner performs, and two tests forced it:

> Passed through verbatim, the recipient arrived holding the *sender's* confidence
> — so sharing transferred demonstrated power rather than vocabulary — and the
> artifact's address was the sender's rather than the capability's, which would
> have made the address depend on how a capability was arrived at and contradicted
> the meet.

So the giver conveys the act and the words for it, and nothing else. **The knowledge
travels and the proof does not.** That is the "knowledge brings power but also
responsibilities" asymmetry with the bookkeeping made explicit.

A lineage survives dormancy, because what was given is a historical fact and does
not become untrue when the creature can no longer show it. A repeated gift is two
events, since an act that leaves no trace is indistinguishable from one that never
happened.

---

## 6. The meet

Everything in the project asserts that patterning is a **meet** and acting is a
**monoid**, and until now that lived only in prose.

### 6.1 Why the subsequence order

| order | verdict |
| --- | --- |
| prefix | too strict — a trace can interleave a primitive the rule does not name and still be the same act |
| **subsequence** | **chosen**: gaps allowed, order not |
| set / bag | too loose — forgets order, and order is what makes a sequence a word in a monoid rather than a bag |

`feed; sleep` and `sleep; feed` expose the same two primitives and are different
acts. Both orders are kept in one module, with a test that the bag order is strictly
*coarser*, so the difference cannot be lost as a comment.

### 6.2 The tie-break is load-bearing

A pair of sequences can have several longest common subsequences. `AB` and `BA`
share `A` and `B`, each alone. Picking arbitrarily makes the answer depend on
argument order and commutativity fails for reasons unrelated to lattices. So the tie
breaks toward the **lexicographically least** longest common subsequence, which makes
the result a function of its two arguments and nothing else.

The first implementation **returned the empty sequence for `meet(a, a)`.** That
satisfies reflexivity, commutativity, associativity *and* idempotence — a function
that always returns the bottom element is a semilattice homomorphism and a useless
meet. Recorded as a warning rather than an anecdote. The reconstruction took the
smallest token appearing in both suffixes and hoped; it now considers only tokens
that begin a *maximal* common subsequence from there.

`is_subsequence` also used `Iterator::any`, which consumes its iterator and leaves
the answer resting on where the standard library happens to stop. An explicit index
walk now.

### 6.3 The retrieval claim, refuted half

The note said the meet explains the 2 false positives. Against a whole two-primitive
act `instantiates` is exact — both refused, an interleaved true positive accepted.
**But a one-primitive candidate is instantiated by every witness containing that
primitive**, because the relation is monotone: the smaller the candidate, the more
witnesses sit above it. And the false positives score 0.2236 and 0.2197 against a
0.2041 weakest true positive — precisely the thin-coverage candidates where the
cheap check has nothing to say.

`serves` is the fix: the same check plus the clause that a candidate must be a
**whole declared act**, so a fragment of one is served by nothing.

---

## 7. The market, and the shape experiment

The user's framing: capabilities dynamic, offers and requests dynamic at all
spheres, nucleuses evolving into actuators, "a creature can get a different shape"
— in IT terms, *improve the hardware by software driving*. Self-motivation (the
system's own patterns and matterns) against crowd-motivation.

### 7.1 The vocabulary was already in the tree

| term | what exists |
| --- | --- |
| capabilities dynamic | `ActuatorRegistry`, `set_champion` per capability |
| offers/requests at all spheres | `SynthesisRequest` ↔ registry; `SharingScope` **Circle → User → Team → Region → Country → Continent → Global** |
| nucleuses evolve into actuators | `FluidFactory::synthesize_nucleus` → `MicroNucleus` |
| **a different shape** | `MicroNucleus { Logic, Presenter, Auditor, Optimizer }` — the slot composition *is* the shape |
| self-motivation | `MetaActuatorType { Analyst, Synthesizer, Distiller, Mutator, Connector }` |

Four of those five were already used widely. `MicroNucleus` appeared in one example
and one test. `FluidFactory` and `SynthesisRequest` appeared **nowhere outside their
own module** — the request type was never constructed by anything.

So the blocker was not "the creature knows four acts". It was that `camaduci` and
the market were two systems that had never been connected, and I had been describing
the blocker as if the market did not exist.

This is now the **third** instance of the same defect — declared, demonstrated once,
never wired (the primitive bridge, `MicroNucleus`, `FluidFactory`). Three
subsystems, one pattern, and it belongs in §9 of the paper: **the project is much
better at declaring mechanisms than at connecting them.**

### 7.2 The experiment

One `SynthesisRequest` from a creature's own state, one call to
`synthesize_nucleus`.

**A shape is real and a request synthesises one** — four slot bindings, and an
unfulfilled slot refuses the **whole** shape rather than returning a partial one,
which is right and is pinned. But the request does not decide it.

**The market was unreachable.** `discover_resources` read:

```rust
// Tags check (simplified)
if !query.tags.is_empty() {
    // In a real system, we'd check tags here
    return false;          // the *unfinished* branch was the *rejecting* one
}
```

Both production callers — `FluidFactory` and `OrchestratorMeta` — **always** send a
tag, so both always got zero results, and the only test that passed was the one that
sent none. Measured before the fix: three offerings on the mesh, three found
untagged, **zero** found tagged. Fixed, regression-tested.

**A champion short-circuits the request entirely.** `resolve_best_actuator` checks
`get_champion` first and returns, so scope, quality floor and required capabilities
are compiled into a query that is never built. **This is the exact line where
"choosable, not imposable" is currently lost** — a champion *is* imposition, one
actuator declared the answer for a capability, and it is the only path where choice
is exercised at all.

**Narrowing the scope widened the choice.** `allowed_scopes` was
`[preferred_scope, Global]`, so a `Circle` request saw `Circle ∪ Global` and a
`Global` request saw only `Global`. Measured: with `reasoning` offered at `Circle`
only, a **`Global` request was refused outright** and a `Circle` request was served.
It compounded with the quality floor: a `Global` request at `min_qor 0.5` refused a
qor-0.9 offering it would otherwise have taken, because that offering sat in its own
circle.

**Quality was a gate and never a preference.** Nothing ranks what passes the floor.
The winner is stable within one market and *differs between two*, because `HashMap`
seeds its hasher per instance — which is worse than nondeterminism, because it looks
principled.

### 7.3 Two more of my own tests, wrong again

- One asserted the mesh held two tagged `reasoning` offerings when only one was in
  `Global` scope. The scope filter was working; my arithmetic was not.
- One asserted `preferred_scope` was **inert**, on the strength of a probe whose ids
  I had truncated to eight characters, so two *different* shapes printed
  identically. I was checking the probe instead of the code. The behaviour is
  sharper than inert: narrowing the scope widens the choice.
- One asserted a *qor-0.2* offering beat a *qor-0.9* one as evidence that quality was
  ignored. It was hash order. Once the scope gate was fixed the good offering won
  instead — the same arbitrary outcome wearing a different id. **The defect is not
  that the worse one wins; it is that there is no way to know in advance which one
  wins.** That is now what the test says.

### 7.4 Reach is earned, and any party can withdraw

The user's clarification: `circle → global` and `global → circle` is like melting
and reacting; there are opened ways to choose back and forth that, **if credit is
known, do not lose either direction or shape**; and in commercial terms these are
conditions and terms of contract — possibilities, readiness, preparedness, working
in contract, **exiting contract periodically and/or definitely, choosable definitive
from all contractual parties**.

Three things were entirely absent: no entry condition, no exit, and no withdrawal —
while `WmisResource.owner` sat there unread, recording the party that could have
withdrawn.

**Reach is earned.** `SynthesisRequest` now carries `credit`, and `reachable_scopes`
is a function of demonstrated credit alone. Below `CREDIT_FLOOR` a requester is
confined to its own circle however global the question is, and a scope it asks for
but cannot reach is refused *for that reason*, naming the credit and the floor.

The property that matters is the **round trip**: at full credit, out to `Global` and
back to `Circle` returns the same shape, because the credit held throughout. Below
the floor the crossing is not merely refused — asking again does not recover it,
because nothing in the system re-earns credit on a creature's behalf.

This is the creature economy's own asymmetry applied to reach: **power that is
demonstrated rather than owned**, so it can be spent by neglect and re-earned only by
production. A scope is a promise about what a creature can still do, and a promise
nobody has demonstrated is not one.

**Any party can withdraw.** `withdraw` is unilateral and unauthorised, which is what
"exitable from all contractual parties" means: no permission to obtain and no
counterparty to ask. It returns whether anything was there, and one party's
withdrawal leaves every other offering published.

The scope-and-quality trap is gone, because it *was* the inversion: a requester with
credit can now see its own circle, so a floor of 0.5 is met by the qor-0.9 offering
rather than refused because of where it sits.

---

## 8. The commons, and which promises are funded

A primitive is a bond: an issued promise about the future, backed by a driver, worth
only what backs it. `PrimitiveNucleus` records `os_mapping`,
`is_replaced_by_actuator` and `replacement_uuid` — so the same promise can be
re-grounded onto different hardware, and when superseded it resolves to
`actuate:{uuid}`. **The vocabulary is stable and the binding is not.**

`src/primitives/mod.rs` had **no tests at all** and seeded exactly one nucleus. Every
primitive now has a binding where a command genuinely exists and a recorded
`unfunded_reason` where none does:

| platform | funded |
| --- | --- |
| Linux (Debian / Arch / Fedora) | **10 / 16** |
| macOS | **8 / 16** |
| Windows | **6 / 16** |

Six are unfunded everywhere, and the reason is the same in each: **they are not
operations on a machine.** `Toggle` is inversion, `Increment` is arithmetic, `Pipe` is
shell syntax, `Broadcast` is networking, `Pulse` is a scheduled signal, `Validate`
would need a dependency rather than a standard tool. Binding those to
plausible-looking strings would issue bonds that default on first use, which is worse
than a visible gap.

Two bugs the new tests caught, both about promises being per-machine:

- a `GenericLinux` binding was a fallback for **every** platform, so `cat` resolved on
  Windows — a bond quietly holding on the wrong machine. The fallback is now
  restricted to the Linux family by explicit list, so a new `OsVariant` defaults to
  *not* inheriting a Linux promise.
- an unfunded primitive returned `"No mapping for OS"`, which reads as a lookup miss.
  It now says it is **known and unfunded**, which is the distinction a caller needs
  between never having heard of a primitive and knowing one it cannot call.

**This is not the bridge that runs.** `bridge::primitive::PrimitiveBridge` is a
`.ure` resource loader and is what `wasm_core`, `os::kernel` and
`orchestrator::meta` use; `primitives::PrimitiveBridge` is the OS mapper and nothing
references it. Execution goes through drivers, so the catalogue is a truthful
statement about the vocabulary rather than a description of the running system, and
it is not wired in because doing so would be speculative.

---

## 9. The store had no primitive sequences

`Store::search` retrieves by **prose**, and the honest reason is that it has nothing
else. `Action` carries an id, aliases, a target state and constraints — the sequence
a pattern performs lives nowhere in a manifest, so the decidable check could not be
applied to the corpus **even in principle**.

The sequence turns out to be recoverable, because induction sets an action's id to
the signature it induced: `SetValue_CheckSense` is a declared act and
`emergency_shutdown` is prose, and the difference is decidable — an id made entirely
of known primitive names is a sequence. `PRIMITIVE_NAMES` is a transcription of
`UniversalPrimitive` with a test holding the two together, because transcriptions
drift.

A single primitive is deliberately **not** a declared act: a one-primitive id is a
fragment, and a one-primitive candidate is exactly what `serves` cannot refuse.

**And the retrieval number has not changed.** Every checked-in manifest is
hand-authored, so the corpus declares no acts, structural search finds nothing, and
20/20-with-2-false-positives is still the scorer's figure. The corpus has to be
*induced* before it can be searched structurally, and `Action` wants a `primitives`
field so the sequence stops depending on an id convention — a persisted-schema
change, not something to slip in beside a measurement.

---

## 10. The state at the end

**482 tests pass**, from 382 at the start. 14 commits, tree clean, everything
pushed.

### Done

- The economy pays for production, not repetition. An act spends and debits; the
  only path to power is `learn`. Measured 0.500 → 0.157 on twelve identical acts
  where the first version gave 0.500 → 0.694.
- `quants` / `nuants`, English plurals, crossing preserved.
- Feeding brings nuants. The loop completes in play for the first time.
- Dormancy: the failure with an exit, and the neglect ordering resolved.
- Self-cleaning, with all three triggers read from the trace log.
- Recycling without a genealogy — **and the convergence measured**, for the first
  time in the project's history.
- The handover, the third relation, and "a gift arrives as a fresh claim".
- The meet, as a function, with the four lattice laws tested.
- The decidable retrieval check, and the reason it was not already there.
- The commons says which of its sixteen promises are funded.
- Reach is earned; any party can withdraw.

### Not done, and why

**The 200-query fixture.** I built the matcher. Whoever writes the queries will write
them in the vocabulary the matcher handles, and the escalation rate would be a
measurement of my own assumptions. That is the same failure as the corpus generator
manufacturing the convergence it then reported — and I only caught that one because
I was suspicious of it.

What can be done without contaminating it: the fixture **specification** — query
distribution, vocabulary constraints, what counts as an acceptable answer, and the
rule that the author must not have read `src/bridge/primitive.rs` or `src/meet.rs`.
And the user's own reframe dissolves the problem rather than answering it: a fixture
in this architecture is a population of `SynthesisRequest`s, and if the requester is
the market and the fixture is the **response log**, the measurement is of the system
under demands it did not design.

Which also makes the escalation rate **three** rates rather than one: capability,
sustainability (does the QoR hold after fulfilment?), and **openness** (did the
sphere widen, or did fulfilment close it?). A system can pass the first and fail the
second absolutely.

**The act set.** Four built-in `Care` acts, so novelty saturates and the implied
equilibrium sits below the peak — a creature that has learned everything it can
still declines. Pinned as a test. The route is now identified rather than guessed: a
creature's capability would be a *shape* — four slot bindings — rather than a set of
acts, and the first thing to build is a **ranking for the market**, because without
one it is either imposed (champion) or arbitrary (hash order), and neither is
choosable.

**`wasm` CI is red** — `tokio full` pulls in `mio`, which does not support
`wasm32-unknown-unknown`. Pre-existing and documented; 6 of 7 other jobs pass. Not
fixed in this session.

---

## 11. What the session actually taught

Four things, and none of them were the features.

**1. Playing finds bugs reading does not.** Four bugs in dormancy, one in the market
tag filter, one dead end in the economy. Every one of them was a code path that
looked correct and could not happen from a test. The `health` residue in particular:
a unit test driven from a full `1.0` passes, because that trajectory lands on `0.0`
exactly, and a creature driven from `0.7` is unkillable. **A test that only ever
starts from one place is a test of that place.**

**2. The mistakes worth keeping were in my own tests, not the code.** Roughly a dozen
assertions were wrong — a magic threshold, a fixture that pinned confidence so the
feature could not fire, a probe with truncated ids standing in for a measurement, a
hash-order coincidence asserted as a finding. Each time, the fix was to assert *what
is true and say so*, and each time the truth was more interesting than the
hypothesis. Three of them changed what a test was *for*.

**3. A vocabulary you do not own is not a design.** The `quants`/`nuants` crossing,
the sphere gradient, the credit-gated crossing, the contractual exit — each arrived as
a correction to a mechanism I had already built and mis-measured. The economy did not
become correct by being tuned; it became correct by being *read* as a contract.

**4. Declaring a mechanism and connecting it are different skills, and the project
has only one of them.** The primitive bridge, `MicroNucleus`, `FluidFactory`: three
subsystems, all sound, none connected. That is not three coincidences and it is the
most useful thing in this record.
