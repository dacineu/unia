# The transducer: unia as a contract, not a machine

**What the user proposed, and what it dissolves.** The sixteen verbs gain a
computational facility, the primitives are implemented on the host processor, and
`.ure` becomes an ISA-independent artifact with a pluggable backend — the shape an
LLM inference engine has: the model is architecture-independent, the kernels are
compiled per target.

**Status: a design that needs one author decision and then a small amount of work.
Most of the scaffold is already here.**

---

## 1. Why this dissolves fork A instead of trading against it

`docs/unia-target-spec.md` §6 asks whether the target preserves ownership or is a
heap with no aliasing promises. **Implemented on the host, the question disappears.**

The backend is Rust. The borrow checker enforces aliasing on the *backend*, on
every ISA, at compile time. unia then has no memory model of its own to specify —
it has a **contract**: a fixed set of operations, each of which some backend
implements. `ActuatorDriver` is already that seam:

```rust
pub trait ActuatorDriver {
    fn execute(&self, packet: &PrimitivePacket, state: &mut HashMap<…>) -> Result<String, String>;
    fn get_resource_id(&self) -> String;
}
```

**So the target specification changes from a memory model to a portability
contract.** That is not a compromise between the forks; it is the observation that
the second one is somebody else's problem once you are not writing the machine.

**Portability across x86, x86-64, RISC-V comes from `rustc` for free** — which is
exactly the inference-engine analogy. There is no unia-specific codegen, no
register allocation, no calling convention to specify. The sixteen verbs are the
ISA-independent layer; the backend is the compiled kernel.

## 2. Reconciling this with the project's own rule

`AGENTS`-level rule: **never extend `UniversalPrimitive` — seventeen hardware verbs
is not an ontology.** That rule is correct and it is not in tension with this, for
a reason worth stating precisely:

- **Ontology** is *naming concepts of the world* — feed, play, sleep, valve, hunger.
  Extending it with those is how the "declared, never wired" pattern gets a
  seventh instance, and four of its five existents were found that way.
- **Arithmetic** is a *computational facility*. It names nothing about the world.
  `Add` is not a concept; it is an operation the target performs.

**So arithmetic goes in a second verb set, not in the first** — the same
two-alphabet pattern already used for `SourceAct` versus the runtime primitives,
with a test asserting they never merge. A `.ure` declares which set a capability
needs, and a capability with no arithmetic requirement is unchanged.

This is not a workaround. The pet's four `Care` acts need no arithmetic, and
`menu::check` refuses until they are in a manifest — so the first thing to ship
under the new plan is a capability set that **does not depend on the new feature
at all**, which is the only responsible order.

## 3. What already exists, and it is most of it

| | |
| --- | --- |
| the instruction set | `UniversalPrimitive`, 16 verbs |
| **the backend seam** | `ActuatorDriver::execute` |
| the arithmetic, *declared* | `UpaOp::Suma { a, b }`, `UpaOp::Product { a, b }` |
| the artifact format | `UreResource`, 17 `.ure` files, content addressing |
| the metrology | signature, meet, `Production`, witness, escalation |
| routing by capability | `link::Linker`, `serves` over declared actions |
| hotswap | `Linker::promote`, `Inflight` |

**`Suma` and `Product` take `String` arguments**, which is the tell: they were
written before the target had arithmetic, so they were named as *data movement*.
Turning them into *computation* is most of the work, and it is small.

**Naming collision, flagged before it bites:** `src/transducer/mod.rs` is already
a *shadow-resource* transducer — `InteractionPair`, `shadow_resource`,
`transduce`, `verify_mirror`. It has nothing to do with compute. The compute
transducer needs a different name, and pretending one module can be both is how
this repository ended up with a `Ura` operation and a `Router` in the same tree.

## 4. The two real costs, which the proposal does not remove

### 4.1 A `.ure` is portable; a *compiled* one is not

If the artifact is a contract and the backend is per-ISA, then a `.ure` lowered
for RISC-V is a **different value** from the same `.ure` lowered for x86-64 — a
different address, by this project's own content-addressing rule.

**So portability lives at the source level, not the artifact level.** A manifest
travels; a kernel does not. That is the same distinction as "weights portable,
kernels compiled", and it is correct — but it must be said, because
`LARGE-PATTERN-MODELS` claims reusability of the *artifact*, and only the source
half of that claim survives a per-ISA backend.

### 4.2 "Mutable, indeterministic, determinable" is three properties and two conflict

- **Determinable** and **indeterministic** are in direct tension with
  `Trace::succeeded` being the only witness. A witness that is sometimes
  different is not a witness.
- **Mutable** `.ure` is fine *only* if mutation produces a new artifact, which is
  exactly what `rematter` already does: same signature, new address, evidence
  discarded. **A `.ure` that is mutated in place has no address**, and an artifact
  with no address cannot be deduplicated, cleaned, or escalated.

So the three adjectives resolve as: **mutable by replacement** (re-mattering, already
built), **indeterminate in its input** (the hardware may answer differently), and
**determinate in its verdict** (a witness is one bit, always). A backend that
returns different answers for the same input on two runs has a defect, and the
witness is how it gets caught.

## 5. The plan, in slices

### Slice 1 — the pet's acts, in a manifest, with no arithmetic

Four `action_primitives` on a `.ure` for the creature. **Needs nothing from this
plan**, and it is the prerequisite for the offering axis of the vision (§4.1 of
the handover: a creature cannot offer capabilities it does not declare).
`menu::check` refuses until this exists, and every fixture authored before it is
another draft.

**Exit:** `Menu::creature().declared_in_manifest == true`, and a `.ure` that
`du -Uuid`s to a stable address.

### Slice 2 — `Add` and `Multiply` in a second verb set

`Numeric { Add, Multiply, Subtract, Divide, Compare }`, with a value kind. **Not
an extension of `UniversalPrimitive`** — a second set, per §2, with the
non-merging test.

**Exit:** a `.ure` declares a capability needing arithmetic, and a backend
evaluates it. And `Production::credit` — six lines, twice-confirmed unlowerable —
**lowers.**

### Slice 3 — the backend

A host implementation of the numeric set through `ActuatorDriver`. `rustc` supplies
x86, x86-64, RISC-V and whatever else; there is no unia codegen and none is
wanted.

**Exit:** the same `.ure` evaluates identically on two architectures, which is the
portability claim stated as a test rather than a hope.

### Slice 4 — the expressibility measurement, now with a denominator

The classifier from `docs/TRANSPOSITION-PLAN.md` Move 1, re-run **after** Slices 2
and 3, so the fraction it reports is the fraction that lowers **with arithmetic**.
Reporting it before and after is the honest way to price the vocabulary change.

**Exit:** two numbers, and the ratio between them is what `Add` and `Multiply`
bought.

## 6. What stays with the author

- **Whether the target becomes a contract rather than a machine.** This rewrites
  the target specification's purpose, so it is a decision and not a task.
- **Whether the artifact-level reuse claim survives §4.1.** It may not, and the
  paper should know before Slice 3 rather than after.
- Fork A is dissolved by this plan. **Fork B is answered by it too** — arithmetic
  exists because the host provides it. What is left of §6 is documentation to
  delete.

## 7. The honest summary

The proposal is better than the plan I was making, and the reason is structural:
**it does not pick a side on the fork, it moves the fork to somebody else's
compiler.** What unia specifies is then a contract and a metrology, which is the
honest description of what it has been all along — the parts that survived every
correction this year were signatures, meets, witnesses and escalation, and none of
them is a memory model.

**The first slice needs nothing from any of it**, and it has been blocked on the
same thing for three sessions: put the pet's four acts in a manifest.
