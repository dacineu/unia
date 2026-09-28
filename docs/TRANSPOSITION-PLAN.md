# Transposing unia's own code, and the two things that must move first

**A proposal. It changes the target specification, so the parts that change what
the project *is* are marked as the author's and the rest is a plan.**

---

## 1. The diagnosis, sharpened

Two functions have now been confirmed not to lower, in two sessions, in two
modules:

```rust
// src/camaduci.rs — the most important economic function in the project
pub fn credit(&self) -> f64 {
    self.new_rules as f64 * 0.10 + self.new_signatures as f64 * 0.05
        + self.new_phrasings as f64 * 0.01 + self.consolidated as f64 * 0.02
}
```

```rust
// src/clean.rs — the smallest fully-understood function in the tree
pub fn rematter(rule: &LearnedRule) -> LearnedRule {
    // new address = DuUuid(signature, aliases)   ← a content hash
    // confidence  = n / (n + 1)                  ← arithmetic on a count
    // observations = count(distinct phrasings)   ← a set operation
}
```

**Neither lowers.** And the reason I had written down was incomplete. I said
"no numeric type, no locals, nothing returns." That is three symptoms of one
thing, and the one thing is this:

> ## unia can compute but cannot factor.

`Increment` + `Compare` + `Route` over a state field is a while-loop. The target is
Turing-complete. **Every function above is computable.** What is missing is not the
ability to compute — it is the ability to *name* a computation and hand it
arguments. A `Signature` is `primitives.join("_")`: a closed sequence, no
parameters, no return, no locals.

So a Rust function cannot be transposed because a mattern cannot *be* a function.
It can only be an inlined, unshared sequence — and inlining is the exact opposite
of what `LARGE-PATTERN-MODELS` claims. **The vocabulary blocks the title claim
directly: it can express the computation and it cannot express the sharing of it.**

That also disposes of a worry I have had. A `Signature` need not grow: the fix is
not more primitives, it is **definition**.

## 2. The error in my own target specification

`docs/unia-target-spec.md` §6 frames the fork as **ownership or nothing**:
preserve aliasing rules and Rust's guarantee survives, or it is a heap and the
reason to compile Rust to it is gone.

**That framing is incomplete, and I got it by only looking at Rust's aliasing
rules rather than at what a processor actually does.** There is a second fork, and
it is not about safety at all:

> **The sixteen verbs were drawn from *actuators*, not from a *processor*.** A
> valve's verbs are open, close, read, write. A CPU's verbs also include add,
> multiply, compare and load. `UniversalPrimitive` has `Compare` and
> `Increment` — arithmetic *by two* — and no `Add`, no `Multiply`, no way to
> combine two values.

A target without arithmetic cannot be a compiler target for Rust, and **not
because of ownership.** `f64` has no representation in a store of untyped strings,
so the port fails before the borrow checker is ever consulted. This is a bigger
obstacle than the one I documented, and it is independent of it.

**So the specification needs two forks, not one:**

| | fork A: memory | fork B: arithmetic |
| --- | --- | --- |
| **option 1** | scoped, aliasing-checked | a value kind, or an encoding for numbers |
| **option 2** | heap, no ownership promises | strings only, no numbers |
| **consequence of 2** | a VM | **not a language target at all** |

**Neither row 2 is negotiable if Rust is to compile.** This is the author's
decision and it is not a preference: it determines whether the project is a
compiler target, a virtual machine, or an actuator bus, and those are three
projects with one name.

## 3. What does lower, and how much — the measurement nobody has made

A mattern lowers when it is **a gate or an act**:

| | primitives used | lowers? |
| --- | --- | --- |
| a precondition and a dispatch | `CheckSense` → `Route` | ✅ |
| an actuation on a declared field | `SetValue`, `Toggle`, `Increment` | ✅ |
| a timed wait | `Delay` | ✅ |
| **arithmetic on a value** | none exists | ❌ |
| **a content hash** | none exists | ❌ |
| **a set operation** | none exists | ❌ |
| **a function with parameters** | none exists | ❌ |

So the tree divides three ways: **gates and acts, which lower; computations, which
are computable but unnameable; and the bindings between them, which have no
representation at all.**

**The number that has never been produced is the first row's share of the
21,607 lines.** It is either the paper's compression ratio or the reason there
isn't one, and it is a classifier, not a transposer:

```
fn lowers(body) -> bool
  // does it reduce to a bounded sequence of the sixteen verbs
  // with no arithmetic, no hash, no set operation,
  // no local outside the state store, and no return value?
```

`rematter` fails on `DuUuid` and on `n/(n+1)`. `credit` fails on all three. A
`Care` act passes. That is the classifier, and it is small.

## 4. The proposal, in two moves

### Move 1 — Measure (no design decision required, and it sizes everything after)

Build the classifier, run it over the tree, and report the fraction with a
per-module breakdown and a hand-checked sample.

**Exit:** a number, and a list of the *kinds* of thing that fail, ranked by how
many lines each kind costs. **If the fraction is high, the claim is close to
arithmetic and the honest next step is a definition primitive. If it is low, the
title claim is unreachable in this vocabulary and that is a publishable finding
rather than a failure.**

This move is deliberately first because it costs nothing, decides nothing, and
converts the largest unknown in the project into a number.

### Move 2 — Add definition, if Move 1 says it is worth it

Not more primitives. **A way to name a computation and give it arguments** —
`Bind`/`Unbind`, or an equivalent. The target specification already says this in
§2.3 and it remains the load-bearing sentence:

> Binding and definition are not features to add. Definition is what a function
> *is*.

The cost is known and stated: `skeleton()`'s treatment of a manifest as a flat
value map stops being adequate, and a definition needs a scope, which is fork A.

## 5. Involving more agents

Four tasks, and the same rule as `docs/DELEGATION-PLAN.md` applies: **the read-set
is the only lever that creates independence, and not one task is a judgement.**

| # | task | why delegable | verified by |
| --- | --- | --- | --- |
| **1** | Build the `lowers()` classifier | mechanical, well-specified, no ambiguity about the answer | me, against a hand count of 30 functions **chosen before I see the result** |
| **2** | Hand-label those same 30 | it is a judgement, so it is done **twice, independently**, by two agents with no shared context | the two label sets; disagreements are the interesting rows |
| **3** | Adversarial review of the classifier | every measurement needs someone paid to say it is measuring the wrong thing | I run the counter-example it produces, or produce one myself |
| **4** | Specify `Bind`/`Unbind` semantics | a design document, not code, and it is the input to the author's decision | me, against the three known-unrepresentable cases |

**Task 2 is the one worth the most and the one most likely to be skipped.** A
classifier validated only by its author is exactly the circularity that produced
the fabricated benchmark — a number compared against a baseline that was
`thread::sleep`. Two independent label sets, with disagreements surfaced rather
than reconciled, is the cheapest available guard against that.

**Task 1 and Task 2 must be run in that order, and the sample must be chosen
before the classifier runs.** Otherwise the labels drift toward whatever the
classifier happens to say, and agreement becomes meaningless.

## 6. What stays with the author

- **Both forks.** Arithmetic and memory. They are not effort.
- **Whether the claim is worth the vocabulary change.** Move 1 measures the prize;
  whether to pay for it is a judgement about the paper.
- **The in-flight hotswap policy** and the trademark, unchanged.

## 7. The honest summary

The transposition is not blocked on effort and **not** primarily on ownership, as
my own specification said. It is blocked on arithmetic, which is a bigger and
more mundane gap, and behind both of them on **definition** — the one thing a
function is and the target has no way to express.

The whole of Move 1 is a classifier over one tree. It decides nothing, costs
almost nothing, and turns the largest unknown in this project into a number.
**It should be the next thing, and the first thing it produces may be the reason
the transposer is never written.**
