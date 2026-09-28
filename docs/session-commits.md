# Commit record — the session, verbatim from git

Machine-extracted, not transcribed. Every paragraph below is the commit body
exactly as committed, `5e54ed5` (the economy correction) to `HEAD`.
It cannot have drifted, because it was not retyped.

Extracted with `git log --reverse --format='## %h %s%n%n%b' 5e54ed5^..HEAD`.

This is the *work* record. The dialogue that produced it — user
messages and my prose replies — is in `console-transcript.md`, which is
transcribed from context and is therefore not machine-exact.

Commits: 17. Repository total: 79.

---

## 5e54ed5 The economy pays for production, not for repetition

`apply_economy` credited the power on every act, so feeding a creature the
same sentence twelve times took it from 0.500 to 0.694 with nothing learned.
A currency that pays for repetition pays for rumination. The credit is now
separated from the cost: an act spends and debits, and the only path to power
runs through `Pet::learn`, which is handed an induction result and can see
what is in it that was not there before. Measured on the same twelve acts:
0.500 to 0.157.

Production is four countable integers -- new rules, new signatures, new
phrasings, and rules already known that became more reliable. The fourth
exists because the first two are finite: there are four built-in acts, so
`new_signatures` saturates after four, and a creature that produced on twelve
of twelve acts ended *weaker* than one that produced on five. Consolidation
is `LearnedRule::confidence`, and it is not inflation because
`observations` counts distinct phrasings -- the same sentence arriving again
raises nothing.

Splitting the credit exposed a liveness hole. A stuck creature cannot act, so
it cannot tend, so it cannot produce, so it is never credited, so it stays
stuck for good. Closed by two entry points rather than by weakening the
economy: `tend` is the player's and has no power gate, `tend_as_self` is the
creature's and does. Being helped and helping yourself are different events,
and a stuck creature is lifted by being taught rather than by being fed --
which is a better rule than the one it replaced, where a free act of sleep
was supposed to do it.

Six tests rewritten, four of them because the behaviour they described is
gone. Two were wrong about the world rather than the code and now say so:
that a repeater ends up `empty` rather than `stuck`, because the resources
go first; and that production and repetition diverge over twenty acts
(1.20 to 2.47) while a creature that has learned every act it can still
declines from its peak, because the implied equilibrium sits below it. That
last one is blocked on the act set, not on the economy, and is pinned as a
known limit rather than balanced around.

392 tests pass.

---
## cdfea15 Record the translucency board in TODO.md


---
## f55d533 Give a trace an author, so "civilisation" has a number

`Pet::last_actor` recorded who performed each act and `Trace` had no field to
receive it, so a player's trace and a creature's were byte-identical and
nothing could count the difference. `Actor` is now `Player`, `Itself`, or
`Caller` -- the third being a real case rather than a missing value, since a
pattern serving a caller is not a creature's act. `Store::authorship` returns
all three counts and `self_directed()` returns `None` rather than `0.0` when no
creature was involved, because a log where the creature took half its acts and
a log where it took none both read as "half" under a bare ratio, and only one
of them means there is nothing self-directed in the log. A trace with no actor
counts as a caller: the absent is read as "not a creature's" rather than
"unknown, so assume the best", which is the flattering error.

`GET /api/authorship`. Measured in play: the creature fed and played itself
four times against two of a person's, `self_directed` 0.667.

Playing it also turned up the next defect, which is in TODO.md rather than
fixed here: `nuante` is only ever decremented outside tests, so an empty
creature is dead for good, and splitting the two doors removed the power gate
without removing the affordability gate, so the player's door looks open and
is not. Reached in about eight acts.

396 tests pass.

---
## 8b74b87 Rename the economy to English plurals: quants and nuants

`cuante` and `nuante` were Romanian, and the paper is in English. The
semantics are unchanged and the crossing is preserved: a *quant* is a power the
creature holds and never spends, and a *nuant* is a stock it spends. Plural
forms as field names and JSON keys, so the vocabulary and the wire format say
the same thing.

The docs section on the economy was also describing the old behaviour --
"Actions spend nuante and raise cuante" -- which has been wrong since the
credit moved to `learn`. Rewritten, with the measured before and after
(0.500 to 0.694, now 0.500 to 0.157) and both open limits recorded.

396 tests pass.

---
## 570da5a A person's feeding brings nuants; a creature's own spending them is all

`nuants` were only ever decremented, so a creature that spent them all was
unrecoverable -- and a person could not even feed it, because splitting the
two doors removed the *power* gate from the player's door and left the
*affordability* gate on both. Playing found it: three teaching phrasings
against a creature at zero were all refused, with nothing able to raise it.

The direction of the stock now depends on who acted. A person feeding a
creature hands it something to act with; a creature feeding itself uses what
it already had. Only `Feed` does this, because if every act were free to the
player there would be no economy, and if none were the player could rescue
nothing. A person feeding also does not debit the power: the debit is for a
creature repeating itself, and someone holding a bowl is not that.

Seven tests changed, all of them by conflating the two doors -- the
production tests were using the player's door to model the creature repeating
itself, which stopped meaning anything once the doors diverged. Three new
tests hold the difference as a property, so a future change breaks a test
rather than quietly inverting the economy.

Verified in play, and the economy became visible doing it. Feeding carried
the resources 12 -> 13 -> 14, and the power went 0.500 -> 0.585 because two
new phrasings induced a rule and `learn` paid for it. Bring food, teach a new
word, production, power rises.

399 tests pass.

---
## 9cb8846 Record the resolved resource dead end and the neglect ordering that remains


---
## 3fbde7b Dormancy: the failure that has an exit

Until now every failure a creature could reach was terminal. A dead one cannot
be brought back and a quarantined one is refused care, so a session that went
wrong could not be recovered inside the session. Dormancy is a third state:
alive, holding nothing it learned, power at the floor, resources untouched. It
will not act for itself and says why; a person acting on it wakes it.

It also resolved a tuning question that had been open since the floor work.
Neglect used to kill a creature about ten ticks in, before either economic
failure could be seen -- health hit zero around ten, the power floored around
sixteen, the resources ran out around twenty-four -- so the economy was legible
only on a creature that no longer existed. Dormancy lands the tick before
death, which makes it the same tick as `stuck`, so the failure that spending
cannot fix is now observable for one tick on something alive and reachable.

The wake is an event and not a credit. A dormant creature is not credited and
does not accumulate; the credit still comes only from `learn`, and only for
what was genuinely new. Forgetting does not move the content address: learning a
language must not move it, and forgetting is learning's inverse, so a creature
and the one it used to be are the same creature. What is lost is evidence, not
identity.

Four bugs, all found by playing and none by reading:

`health` never reached zero. Subtracting 0.10 from 0.1 in binary floating point
leaves ~1e-16, which is greater than zero and below the dormancy threshold at
once -- so a neglected creature went dormant and then could not be killed. The
unit test missed it because the health trajectory it happened to take landed on
0.0 exactly.

The threshold was one decay step above zero, so a creature stepped *over* the
band and went from healthy to dead without ever being dormant. A threshold is
only reachable if it is wider than the step that crosses it.

`learn` woke the creature on any non-empty rule set. Induction re-derives a
creature's rules from its own trace log, so it woke itself from evidence it had
written before it slept. A dormant creature has to be woken from outside or it
was never asleep.

`neglect` had its own weaker copy of the transition -- it cleared the rules and
set the flag but neither counted the loss nor floored the power, so a creature
that fell asleep by neglect kept its forget-count at zero and was credited for
its own last rules a moment later. Every test passed because they all called
`go_dormant` directly, which is the only way it ever happens in the game.

Verified end to end in play: taught two words, neglected for a minute, caught
alive at health 0.100 with posture stuck and nothing learned; it refused to act
for itself; a person taught it a third word and it woke at quants 0.262 with
its address unchanged.

410 tests pass.

---
## 6a68f22 Write the dormancy and recycling note before coding either

Self-cleaning and genealogy-free recycling both contradict something the
architecture currently relies on, and the contradiction is worth writing down
before deciding it rather than after.

The contradiction: `Denominator.contributors` is a genealogy, and "no
traceability in genealogy" says the question must not be answerable. The
resolution is the distinction the project already has -- patterning is a meet
and acting is a monoid -- so recycling can keep the capability and drop the
history. A re-mattered artifact gets a new address and the same signature, and
`contributors` becomes a census of the current generation rather than a
lineage.

That is not a loss but a measurement. The convergence test the project has
never run -- creatures that never met agreeing anyway -- becomes possible the
moment contributors stops being a lineage. Twelve hand-written patterns share
zero capabilities and thirty-six generated artifacts share six denominators they
were manufactured to share; neither is a convergence. The cost is that
escalation rate must be stated per signature rather than per address, which is
the direction the corpus measurement already pointed.

Five open questions recorded. Two of them block: whether "chaotic event" means
a failed precondition or the `succeeded: false` the schema already records, and
what "readiness" means given that `Sensor::Inferred` has no inhabitant at all.

---
## 3a3e7e8 Self-cleaning, and the sensor that makes the long horizon askable

Two halves that were blocking each other, done together.

`clean` is the long-horizon trim. A creature lives in two economies — `nuants`
spent, `quants` sustained — and both are reached by decay, so nothing had ever
cleaned the third: the matterns themselves. A mattern nobody has confirmed any
more is not a capability the project has lost, it is evidence it has not earned.
So it is re-mattered rather than deleted: the same signature, the same
phrasings, at a new address, with the confidence and observation count
discarded. Deleting would delete the capability and the creature would forget —
dormancy applied to an artifact instead of to a creature. Re-mattering keeps the
capability and throws the evidence away, and that is the whole difference.

No genealogy, as specified. The new address is a content address of the
surviving material and is deliberately *not* a hash of the old one, because
chaining them would be a genealogy with extra steps. `Cleaning::remattered` is a
report returned to a caller and stored nowhere, and a test pins the property that
actually matters: two rules with one signature and different histories re-matter
to the *same* address, so the address is a function of the capability rather than
of the lineage.

All three triggers are read out of the trace log, because a trigger nothing
records cannot fire honestly. Unknown situations are successful traces no mattern
covers. Chaotic events are `succeeded: false` — the only failure the schema
already records, which is why it is the answer and a failed precondition is not,
since a precondition that could not be met leaves no record of its own.
Readiness patterning is a mattern with no confirming trace, which for a
`Readiness` that is a maximum over evidence means the evidence stopped counting
the moment it stopped arriving.

`Sense::Inferred` now has an inhabitant. It was a variant nothing was ever of:
distinguishable, documented, and unreachable, and the test that noticed said so
while reading as a claim about the sensors. A `Sensor` now carries the sequence
it dispatches, and evaluating an inferred one needs a `Dispatch`. The reading
lands in the evaluated state under a named key, so its `when` is ordinary
constraint grammar — and a sensor with no dispatcher simply has no reading, its
predicate is unevaluable, and it is not reporting. The same answer a malformed
sensor gets, arrived at with no special case in the evaluator.

The inhabitant is a readiness sensor: whether the evidence behind this creature's
own matterns still stands is not a number the creature holds, it is a question
about the trace log, so reading it is an act against that log. That makes the
long horizon askable as something a creature notices about itself rather than
something an operator computes.

Two tests whose premise was "inferred has no inhabitant" were rewritten rather
than satisfied, and one of them was asserting that every sensor was `Derived` —
which was true only because nothing was ever inferred. The readiness dispatcher
also ignored the subject of the sequence and answered for any creature asked
about, found by a test written against the wrong expectation.

423 tests pass.

---
## 5da22b5 Measure the convergence, and correct two claims the code did not support

The design note said recycling without a genealogy would make the convergence
test possible. That is now a measurement rather than an argument: six
generations, each authoring a mattern at an address of its own choosing, taught
a private vocabulary, then culled leaving only failed traces. Afterwards: one
capability, one address, and no surviving artifact naming any culled address.

The project had never been able to produce a convergence at all. Twelve
hand-written patterns share zero capabilities -- one author, one intent, twelve
solitaries. Thirty-six generated artifacts share six denominators they were
manufactured to share, which is a uniformity. In both cases the arrival was
arranged, and neither is evidence that independent creatures find the same
capability. Nothing here has to be arranged, which is the whole point.

Two claims the code did not actually support, both corrected rather than worked
around.

The re-mattered address was documented as a content address of "the signature
and the phrasings still standing", and a test appeared to confirm it. It did
not: `DuUuid::generate` runs its input through `skeleton`, which drops `aliases`
as surface, so three different vocabularies produced one identical address and the
test was asserting a coincidence. The architecture was right and the note was
wrong. Identity is the act; the phrasings are the surface; two creatures that
learned different words for the same thing are the same artifact. That is also
what makes the six generations converge, so the stronger property is the one worth
having. The test now asserts the address is the capability and nothing else, and
that a different act gives a different address so it is not simply constant.

`changed()` meant "something was stale" rather than "the corpus differs", so a
second pass over an already-clean corpus claimed to have cleaned all eleven of
its matterns. A re-matter that yields the identical artifact is not a re-matter;
it is the artifact already being canonical. Replacements are now recorded only
when the result differs, which leaves a real fixed point and makes the cleaner
safe to run repeatedly.

426 tests pass.

---
## 4d83057 The meet, and a measurement that says it is not the whole fix

Everything in this project asserts that patterning is a meet and acting is a
monoid, and until now that lived only in prose. `src/meet.rs` is the meet.

The order is the subsequence relation: gaps allowed, order not. A prefix order is
too strict, because a trace can interleave a primitive the rule does not name and
still be the same act. A set order is too loose, and destroys the distinction the
architecture turns on -- a set forgets which order the acts happened in, and order
is what makes a sequence a word in a monoid rather than a bag, so `feed; sleep` and
`sleep; feed` would be one act. The two orders are kept in one module, with a test
that the bag order is strictly coarser, so the difference cannot be lost as a
comment.

`meet` is the longest common subsequence with the tie broken toward the
lexicographically least. That tie-break is load-bearing: a pair of sequences can
have several longest common subsequences, and picking arbitrarily makes the answer
depend on argument order, so commutativity and associativity would fail for reasons
unrelated to lattices. With it, the four laws are testable and hold, and folding
the meet over six witnesses gives the same answer as over two -- which is what
makes a denominator over six participants well defined.

The first implementation returned the empty sequence for `meet(a, a)`. That
satisfies reflexivity, commutativity, associativity and idempotence, which is
worth recording as a warning rather than an anecdote: a function that always
returns the bottom element is a semilattice homomorphism and a useless meet. The
reconstruction took the smallest token appearing in both suffixes and hoped; it now
considers only tokens that begin a maximal common subsequence from there.
`is_subsequence` also used `Iterator::any`, which consumes its iterator and leaves
the answer resting on where the standard library happens to stop, so it is an
explicit index walk now.

**And measuring the retrieval claim refuted half of it.** The note said the meet
explains the 2 false positives, and it does not, on its own. Against a whole
two-primitive act `instantiates` is exact: both false positives refused, an
interleaved true positive accepted. But a *one*-primitive candidate is
instantiated by every witness containing that primitive, because the subsequence
relation is monotone -- the smaller the candidate, the more witnesses sit above
it. And the false positives score 0.2236 and 0.2197 against a weakest true
positive of 0.2041, so they are precisely the thin-coverage candidates where the
cheap check has nothing to say.

`serves` is the fix: the same check plus the clause that a candidate must be a
*whole declared act*, so a fragment of one is served by nothing. Measured against
the same three witnesses it refuses all three for the fragment, and behaves
correctly for the act. The pipeline still calls the scorer, so the 2 false
positives are still there, and TODO.md says the swap is not done.

The no-genealogy property is also a type now: `Witness` carries only its
primitives, no predecessor and no contributor, and a test pins that its
serialised form contains neither.

442 tests pass.

---
## 37114d5 The decidable retrieval check, and why the corpus cannot yet be searched by it

`Store::search_primitives` uses `meet::serves`: no threshold, no ranking, a
pattern matches when one of its declared acts is an act the intent performed, in
order, whole. Measured on a corpus of induced manifests it finds the true
positive -- an act performed with a primitive interleaved -- and refuses both
shapes of false positive, the witnesses that merely touched the act and did
something else after. Those are the shapes that scored 0.2236 and 0.2197 against
a weakest true positive of 0.2041.

Getting there exposed something larger. **The store had no primitive sequences at
all.** `Action` carries an id, aliases, a target state and constraints, and the
sequence a pattern performs lives nowhere in a manifest -- so the decidable check
could not be applied to the corpus even in principle, and the false positives had
no structural reason to go away. Retrieval scores prose because prose is all the
store has.

The sequence turns out to be recoverable, because induction sets an action's id to
the signature it induced. `SetValue_CheckSense` is a declared act and
`emergency_shutdown` is prose, and the difference is decidable: an id made
entirely of known primitive names is a sequence. `PRIMITIVE_NAMES` is a
transcription of `UniversalPrimitive` with a test holding the two together,
because a transcription drifts and a name added to one and not the other would
become invisibly unrecognisable to the store.

A single primitive is deliberately *not* a declared act. A one-primitive id is a
fragment, and a one-primitive candidate is exactly what `serves` cannot refuse --
recovering fragments here would put them straight back into the place that breaks
the check.

**And the retrieval number has not changed.** Every checked-in manifest is
hand-authored, so the corpus declares no acts, structural search finds nothing,
and 20/20-with-2-false-positives is still the scorer's figure. The corpus has to
be induced before it can be searched structurally, and `Action` wants a
`primitives` field so the sequence stops depending on an id convention -- a
persisted-schema change, and not something to slip in beside a measurement. Both
are in TODO.md.

One test here opened a `corpus/` directory and asserted about its contents. There
is no such directory; it was asserting against nothing, and it now builds its own.

453 tests pass.

---
## 15460a3 The commons says which of its sixteen promises are funded

A primitive is a bond: an issued promise about the future, backed by a driver,
worth only what backs it. `src/primitives/mod.rs` had no tests at all and seeded
exactly one nucleus, so fifteen of the sixteen could not be called and nothing
said so -- a promise that defaults on first use, discovered only by trying.

Every primitive now has a binding where a command genuinely exists and a recorded
`unfunded_reason` where none does. Measured: 10 of 16 on Linux, 8 of 16 on macOS,
6 of 16 on Windows. Six are unfunded on every platform and the reason is the same
in each -- they are not operations on a machine. `Toggle` is inversion,
`Increment` is arithmetic, `Pipe` is shell syntax, `Broadcast` is networking,
`Pulse` is a scheduled signal, and `Validate` would need a dependency rather than a
standard tool. Binding those to plausible-looking strings would issue bonds that
default on first use, which is worse than a visible gap. A test asserts the exact
unfunded list rather than a count, so funding one of them is a deliberate act with
a visible diff.

An actuator can still fund an unfunded primitive, and the vocabulary stays at
sixteen while the binding changes: the promises are stable and the bodies are
revisable, which is the whole design of the bridge.

Two bugs the new tests caught, both about promises being per-machine. A
`GenericLinux` binding was a fallback for *every* platform, so `cat` resolved on
Windows; the fallback is now restricted to the Linux family by explicit list, so a
new `OsVariant` defaults to not inheriting a Linux promise until someone says it
does. And an unfunded primitive returned "No mapping for OS", which reads as a
lookup miss -- it now says it is known and unfunded, which is the distinction a
caller needs between never having heard of a primitive and knowing one it cannot
call.

The catalogue holds all sixteen against the enum, with a test keeping the two
together: a name in one and not the other would become invisibly uncallable.

**This is not the bridge that runs.** `bridge::primitive::PrimitiveBridge` is a
`.ure` resource loader and is what `wasm_core`, `os::kernel` and
`orchestrator::meta` use; `primitives::PrimitiveBridge` is the OS mapper and
nothing references it. Execution goes through drivers, so the catalogue is a
truthful statement about the vocabulary rather than a description of the running
system, and it is not wired in because doing so would be speculative. The real
question is which primitives have a registered driver, and that is in TODO.md
unmeasured.

460 tests pass.

---
## 462b33a The handover: the one relation that keeps a genealogy

Convergence severs provenance on purpose, so that six creatures arriving at the
same capability cannot be traced to each other -- that is the convergence the
project wanted to measure. A handover is the opposite gesture: I am giving you
*this* artifact, and the fact that it came from me is the point of the act. Both
are real, neither subsumes the other, and running them together in the prose is
why the two numbers have both been awkward.

They are three named mechanisms now, not three phrasings of one: handover
preserves provenance, convergence severs it, matching is provisional and scored.
Keeping both of the first two is what makes the third measurable -- an escalation
rate per address counts a handover as escalation, which is a transfer and not a
capability, while per signature it counts a convergence. The difference cannot be
told apart without the record, which is the argument for recording the handover
rather than treating it as redundant with cleaning.

**A gift arrives as a fresh claim.** `receive` re-matters what it is given, the
same operation the cleaner performs, and two tests forced it. Passed through
verbatim, the recipient arrived holding the *sender's* confidence, so sharing
transferred demonstrated power rather than vocabulary -- and the artifact's address
was the sender's rather than the capability's, which would have made the address
depend on how a capability was arrived at and contradicted the meet. So the giver
conveys the act and the words for it and nothing else: the power is not
transferable, because the holder has demonstrated nothing. That is the "knowledge
brings power but also responsibilities" asymmetry with the bookkeeping made
explicit. The knowledge travels and the proof does not.

A lineage is therefore a record of a relationship still in force, not of a claim
once made: dormancy forgets the rules and keeps the lineage, because what was given
to a creature does not become untrue when the creature can no longer show it. And a
repeated gift is two events, since an act that leaves no trace is
indistinguishable from one that never happened.

466 tests pass.

---
## 79a4760 Run the shape experiment: a request does not decide the shape

One `SynthesisRequest` built from a creature's own state, one call to
`synthesize_nucleus`, and an answer to whether a creature can get a different
shape. **A shape is real and a request synthesises one**: four slot bindings, and
an unfulfilled slot refuses the *whole* shape rather than returning a partial one,
which is the right answer and is now pinned. But the request does not decide it,
and three defects are measured.

**The market was unreachable.** `discover_resources` read
`if !query.tags.is_empty() { return false; }` with a comment saying the real check
had not been written -- which made the *unfinished* branch the *rejecting* one. Both
production callers always send a tag, so both always received zero results, and the
only test that passed was the one that happened to send none. Measured before the
fix: three offerings on the mesh, three found untagged, **zero** found tagged. Fixed,
with a regression test, and a tag now matches what an offering actually declares:
several tags must all match one offering, and an undeclared capability is not a
near miss.

**A champion short-circuits the request entirely.** `resolve_best_actuator` checks
`get_champion` first and returns, so the scope, the quality floor and the required
capabilities are compiled into a query that is never built. A champion is
*imposed* -- one actuator declared the answer for a capability -- and it is the only
path on which choice is exercised at all. This is the exact place the "choosable, not
imposable" requirement is currently lost, and it is one line.

**Narrowing the scope widens the choice.** `allowed_scopes` is built as
`[preferred_scope, Global]`, so a `Circle` request sees `Circle ∪ Global` while a
`Global` request sees only `Global`. Measured: with `reasoning` offered at `Circle`
only, a **`Global` request is refused outright** and a `Circle` request is served.
It compounds with the quality floor -- a `Global` request at `min_qor 0.5` refuses a
qor-0.9 offering it would otherwise have taken, because that offering is at `Circle`.

**Quality is a gate and never a preference.** Nothing ranks what passes the floor:
`discover_resources` returns a `Vec` in `HashMap` order and the caller takes the
first. Measured: a qor-0.2 offering beat a qor-0.9 one. The winner is stable within
one market and differs between two, because `HashMap` seeds its hasher per instance
-- which is worse than nondeterminism, because it looks principled.

Three of the tests I wrote while measuring were themselves wrong, and the failures
were the information. One asserted the mesh held two tagged `reasoning` offerings
when only one was in `Global` scope -- the scope filter was working and my
arithmetic was not. One asserted `preferred_scope` was inert, on the strength of a
probe whose ids I had truncated to eight characters, so two different shapes printed
identically; the behaviour is sharper than inert and the assertion was checking the
probe instead of the code. And one asserted a specific offering won, which passed in
isolation and failed once unrelated resources were on the mesh -- the winner depends
on what else is in the market, which is a stronger statement of arbitrariness than
any fixed answer.

475 tests pass.

---
## fbe35d4 Reach is earned: credit gates the scope, and any party can withdraw

Two of the three things the contract vocabulary needs, both measured missing.

**Reach is earned rather than widened.** `allowed_scopes` was built as
`[preferred_scope, Global]`, so a `Circle` request saw `Circle ∪ Global` and a
`Global` request saw only `Global` — asking for less got you more. Measured: with
`reasoning` offered at `Circle` only, a `Global` request was refused outright, and
a `Global` request at `min_qor 0.5` refused a qor-0.9 offering it would otherwise
have taken because that offering sat in its own circle. It compounded two gates into
a trap, and both were backwards from the words involved.

`SynthesisRequest` now carries `credit`, and `reachable_scopes` is a function of
demonstrated credit alone: below `CREDIT_FLOOR` a requester is confined to its own
circle however global the question is, and a scope it asks for but cannot reach is
refused *for that reason* and the refusal names the credit and the floor. The
property that matters is the round trip — at full credit, out to `Global` and back
to `Circle` returns the same shape, because the credit held throughout. Below the
floor the crossing is not merely refused: asking again does not recover it, because
nothing in the system re-earns credit on a creature's behalf.

This is the creature economy's own asymmetry applied to reach: power that is
demonstrated rather than owned, so it can be spent by neglect and re-earned only by
production. A scope is a promise about what a creature can still do, and a promise
nobody has demonstrated is not one.

**Any party can withdraw.** `WmisResource` has carried an `owner` field since the
type was written and nothing ever read it — the party that could have withdrawn an
offering was recorded and never asked, and no party could, because there was no way
to. `withdraw` is unilateral and unauthorised, which is what "exitable from all
contractual parties" means: no permission to obtain and no counterparty to ask. It
returns whether anything was there, because an event that leaves no trace is
indistinguishable from one that never happened, and one party's withdrawal leaves
every other offering published.

`SharingScope` is now `Copy` and `Eq`. It is a fieldless enum passed alongside
`min_qor` into every query, and cloning it at each of those sites had obscured the
one place that mattered.

Three of the tests I wrote while measuring were wrong, and again the failures were
the information. One asserted the market held two tagged `reasoning` offerings when
only one was in `Global` scope — the scope filter was working and my arithmetic was
not. One asserted `preferred_scope` was inert, on the strength of a probe whose ids
I had truncated to eight characters, so two different shapes printed identically;
the behaviour is sharper than inert and the assertion was checking the probe instead
of the code. And one asserted that a *qor-0.2* offering beat a *qor-0.9* one as
evidence that quality was ignored — it was hash order, and once the scope gate was
fixed the good offering won instead, which is the same arbitrary outcome wearing a
different id. The defect is not that the worse one wins; it is that there is no way
to know in advance which one wins, which is now what the test says.

482 tests pass.

---
## 24b0561 Record the session: the economy, dormancy, the meet, and the market

A complete record of this working session, written to preserve the *reasoning*
rather than the conclusions, including every place the reasoning was wrong --
because that is the part worth keeping and it is the part a paper cannot carry.

Eleven sections, in the order the work happened. The economy's correction with
both measurements; the vocabulary crossing and why it survives in English; feeding
bringing nuants and the loop completing in play for the first time; dormancy with
its four bugs; self-cleaning, the genealogy-free convergence, and the handover;
the meet as a function; the market and the shape experiment; the commons and which
of its sixteen promises are funded; the fact that the store had no primitive
sequences at all.

The recurring finding is recorded where it belongs rather than as an aside: roughly
a dozen of the assertions written during the session were wrong, and in every case
the truth turned out to be more interesting than the hypothesis. A magic threshold
where a shape was wanted. A fixture that pinned every rule's confidence, so the
sustaining term could never fire. A probe with truncated ids standing in for a
measurement. A hash-order coincidence asserted as a ranking defect. Each is written
up with what the failure actually revealed, because a log that only records
successes is a log that cannot be checked.

Numbers are quoted from measurements rather than paraphrased, and the ones that
govern a claim were re-verified against the tree before committing -- 10/16 funded
on Linux, 8/16 on macOS, 6/16 on Windows; the six unfunded primitives and their
reasons; `CREDIT_FLOOR` and `DORMANT_BELOW_HEALTH` both at 0.2; the four credit
weights. 482 tests pass, from 382 at the start.

The three things that could not honestly be done are named as such: the 200-query
fixture needs an author who has not read the matcher, the act set is still closed
at four built-in acts so novelty saturates and the economy's equilibrium sits below
its peak, and the wasm CI job is still red on `tokio full` pulling in `mio`.

---
