# The unia target specification

A specification of the machine `unia` is, in the terms a compiler uses — because
the question "can Rust compile for this architecture" has a precondition that is
not a compiler, and this document is that precondition.

**Status: a specification with one unresolved decision.** Sections 1–5 are
determinate and checkable against the tree. Section 6 is a fork that changes what
the project is, and it is not this document's to close.

**What the compiler pipeline looks like, filled in honestly:**

| stage | Rust → x86-64 | Rust → unia |
| --- | --- | --- |
| ISA | x86-64 instructions | the 16 `UniversalPrimitive` verbs — **sufficient** (§1) |
| target spec / ABI | registers, calling convention, alignment, endianness | **this document; §6 unresolved** |
| IR | LLVM IR | none (§3) |
| type mapping | `u32` → a word | none — every value is a `String` (§2.1) |
| memory model | flat address space | untyped per-resource store, global, unscoped (§2.2) |
| control flow | jump, call, return | `Route`, `Pulse`; **no call, no return, no stack** (§2.3) |
| linker | symbols → addresses | none (§3) |
| runtime | `std`, allocator, collections | none (§3) |
| verification | test suite | **✅ 488 tests; wasm target builds** |
| history | release notes | **✅ `SourceAct` + `Caller` traces** |

Three of nine are done, and they are the three that make a port *checkable*
rather than hopeful.

---

## 1. The instruction set is not the constraint

The sixteen verbs are

```
GetState  GetValue  CheckSense  SetValue  Toggle  Increment  Reset
Route     Pipe      Broadcast   Delay     Watch   Pulse     Compare
Transform Validate
```

`Increment` + `Compare` + `Route` over a state field is a while-loop, so **the
target is Turing-complete**. 2ⁿ amplitudes are not needed to compute; only
speed.

This is the good news and it is worth stating plainly, because the intuition that
"sixteen verbs is far too few to compile a real language" is wrong. The binding
constraint was never the instruction count.

## 2. What a value, a variable and a call would have to be

### 2.1 A value

Today: a `String` in `state_store: Arc<Mutex<HashMap<String, HashMap<String, String>>>>`.
Untyped, un-namespaced beyond the resource, and with no numeric type at all.

**Required:** a closed set of value kinds, or an encoding for numbers in strings.
Everything below depends on this and it is not decided.

### 2.2 A variable

Today: a key in that store. **The store is global per resource, so a variable has
no scope.** Two nested calls cannot both hold a local named `t`.

This is not a missing feature — it is the absence of the thing Rust's borrow
checker exists to check. A target that gives ownership nothing to own cannot
inherit Rust's central guarantee. See §6.

### 2.3 A function — and why nothing is callable

`Signature` is `primitives.join("_")`. A signature is therefore a **closed**
sequence: no parameters, no return, no invocation.

**So the target has no calling convention, because it has no notion of a callable
thing.** This is the same finding as "no binding" arrived at from the compiler
side, and it is sharper: it is not that binding is a feature to add, it is that
*definition* is what a function is.

**Consequence:** a `Signature` can be a *trace* of something that happened or a
*recipe* for something to do, and those are different things. `Clean` treats it as
the former — evidence — which is consistent. But a recipe requires parameters,
and there is no such concept.

### 2.4 Worked example: the function that cannot lower

```rust
pub fn credit(&self) -> f64 {
    self.new_rules as f64 * 0.10
        + self.new_signatures as f64 * 0.05
        + self.new_phrasings as f64 * 0.01
        + self.consolidated as f64 * 0.02
}
```

Six lines, and the most important economic function in the project. Lowering it
requires:

1. **Arithmetic.** No numeric type. Every constant is a `String`; multiplication
   needs `Increment`, an overflow test via `Compare`, and a branch.
2. **Four locals.** Each is a global state key with no scope (§2.2).
3. **A call** for `self.new_rules` — a struct read, so at minimum field access;
   there is no `GetField` among the sixteen, only `GetValue` against a resource.
4. **A return.** Nothing returns.

So the six lines lower to **nothing**, for want of somewhere to put four
intermediates and no way to return them. Not "something inefficient" — nothing.

## 3. What has no specification at all

- **IR.** Nothing sits between the source language and the sixteen verbs. A
  target with no IR is not a compilation problem yet, it is a transcription one.
- **Linker.** How one mattern names another. Today a `Signature` names an
  *address*, computed as a content hash, and content hashes do not resolve to
  call sites.
- **Runtime.** No allocator, no collections, no string operations beyond
  `Transform`. `HashMap` — used pervasively, including in the store itself —
  has no representation.
- **Data layout.** `UreResource` is a Rust struct. Its field order is the target's
  data layout and is decided by the compiler rather than by a specification.

## 4. What is already done, and it is not nothing

- **Verification.** 488 tests, and `cargo check --lib --target
  wasm32-unknown-unknown` builds. The wasm job was red for as long as it existed
  and fixing it was the first time this codebase compiled against a target that was
  not its own. A port is only meaningful against a suite that runs on the target.
- **A history shaped like a port.** `SourceAct` and `Actor::Caller` traces: a
  staged port is a sequence of span edits, each followed by a witness, and the
  rate is measurable in capability per act. Two alphabets exist deliberately — the
  sixteen for what happened to the world, the source acts for what was done to the
  code — and a test asserts they never merge.
- **A decidable subset.** `meet`, `serves`, `Production` and the escalation rate
  are decisions, not scores. A port of them is checkable in a way a port of
  floating-point arithmetic is not.

## 5. The acceptance criterion, stated before the work

> **A transposed module must be shorter than its source, and the ratio measured.**

Transposing 4000 lines of Rust into 4000 lines of matterns produces a new file
format and a worse editor. The gain is compression **by reuse** — one mattern
occurring in many sites, so one edit changes many places. That is what
`LARGE-PATTERN-MODELS.md`'s title claims and it is a number nobody has measured.

**A ratio of 1.0 is a failure that looks like a success**, and the same trap the
escalation metric fell into this session when a deduplicated denominator rated
fifty-one identical edits as a perfect escalator.

First candidate: `src/clean.rs`. ~500 lines, small surface — `rematter`, `serves`,
`meet`, `confirming_traces`, `clean` — and already pinned by tests. Report
(1) matterns the semantics decomposes into, (2) occurrences of each in the
source, (3) the ratio.

## 6. The fork, which is not this document's to close

x86-64 was a specified, stable, decades-old contract. The unia target is not
specified at all — not as an oversight, simply never written. And the
specification has a fork that changes what the project *is*:

### (a) An ownership-preserving target

The memory model gives scoping and aliasing rules that Rust's ownership discipline
maps onto. Then `unia` is a genuine compiler target and Rust's central guarantee
survives the port. **Cost:** the store stops being untyped strings; something like
`Bind`/`Unbind` or a scoped-frame primitive enters the vocabulary, and
`skeleton()`'s treatment of a manifest as a flat value map stops being adequate.

### (b) A virtual machine with a heap

The store stays untyped strings with a heap and no ownership promises. Then
nothing is lost that was ever there — but **the one reason to compile Rust to it
is gone.** A port from a language with no aliasing promises would be better than
a port from Rust.

**The tree is currently (b), unstated.** `Arc<Mutex<HashMap<String,
HashMap<String, String>>>>` is a heap without ownership. The project has been
assuming fork (b) without ever writing it down, and every design consequence above
follows from that silent assumption.

**This is the decision, and it belongs to the author.** It determines whether
`unia` is a compiler target or a virtual machine. Those are different projects
with the same name, and the vocabulary does not distinguish them yet.
