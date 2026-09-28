# Quantum → binary: the exact reduction, and where it stopped

A design note written after attempting the implementation, because the attempt
failed in an instructive way and the failure answers the question better than a
working module would have.

**The code is not in the tree.** An implementation was written, it had eight
failing tests, and it was removed rather than committed. Nothing in this document
is asserted from a run that did not happen; the one arithmetic claim made below
was checked numerically and the check is reproduced inline.

## The question

Reduce quantum computation to binary, exactly — no approximation, so that a
classical machine reproduces a quantum result without error, with binary as the
backward-compatible base case.

## What is actually true, and it is more than expected

**The reduction is a theorem, not a technique.** The Gottesman–Knill
construction: any circuit built from Clifford gates is exactly classically
simulable, in polynomial time, from a binary description. That is the "no
erroring" part — there is no floating point in the state at all, so there is
nothing to round.

The vocabulary is small and it is a hardware interface, not a language:

```
H  S  X  Z  CNOT  CZ  SWAP
```

Every one of these maps a stabilizer group to itself by a permutation of bits.
That is the whole mechanism.

## Where the implementation failed, precisely

**A Pauli operator's `(x, z)` two-bit-per-qubit encoding loses the sign, and the
sign is not optional.**

A Pauli on *n* qubits is written as two bit vectors: `x[i]` for an X factor and
`z[i]` for a Z factor, with `(true, true)` denoting Y. That is a bijection onto
`{0,1}^(2n)`, and it is exact as a *group* — multiplication is XOR on both
vectors, so the whole of the Pauli algebra is GF(2) linear algebra.

The encoding is exact as a group and **incomplete as a description of a state**,
because the operators it names differ by scalar multiples that it cannot see.
Conjugating a generator by a gate multiplies it by −1, and −1 is not in the
encoding. Checked numerically rather than asserted:

```
XZX = [[-1, 0], [ 0, 1]]      i.e.  -Z, not  +Z
ZXZ = [[ 0, -1], [-1, 0]]      i.e.  -X, not  +X
```

So both conjugations flip a sign the representation does not carry.

**Why this is fatal rather than cosmetic.** A stabilizer state is determined by
the Pauli operators with eigenvalue **+1**. A *group* without signs does not
determine a state at all: `|0⟩` and `−|0⟩` have exactly the same stabilizers, and
so does every state that differs from it by a phase. Worse, the failure is
invisible to self-consistency — the tableau remains a perfectly valid commuting
set of Paulis under every gate, for the *wrong* state. Only a comparison against
an independent implementation catches it.

**The measurement bug is the same bug.** Deciding whether a measured qubit is 0 or
1 means deciding whether an operator has eigenvalue +1 or −1. With signs dropped,
that decision has no input, so measurement — the operation that makes the whole
scheme classically simulable — is exactly the operation the encoding cannot
express. The property that justifies the technique is the one the representation
loses.

## What a correct implementation requires

Sign tracking, and it is the whole of the remaining work:

1. Each generator becomes `(x, z, phase)` with a phase bit, or equivalently a
   ±1 alongside the operator.
2. Group multiplication must carry the phase product, including the i factors —
   `X·Z = −iY` is where the phases enter, and it is why `Y` as `(true, true)` is
   adequate for conjugation but not for anything reading an eigenvalue.
3. Each gate's conjugation must be stated with its sign, which is why the
   standard implementations store a tableau *with* sign bits rather than a set.
4. Then the reduction is exact and the guarantee is real: 2ⁿ amplitudes become
   n stabilizers of 3n bits.

None of that is exotic. It is the difference between a group and a group with
its character, and the character is what the state lives in.

## What is *not* claimed

- **Not for arbitrary circuits.** Outside the stabilizer fragment, exactness needs
  2ⁿ amplitudes *and* algebraic numbers rather than binary ones, because
  `H|0⟩ = (|0⟩+|1⟩)/√2` and 1/√2 is not a dyadic rational. The exact
  representation there is the ZX-calculus one, not a wider bit vector. So
  "quantum reduces to binary exactly" is true of the Clifford fragment and not of
  quantum computation in general.
- **Not a claim about the project's own vocabulary.** The 16 `UniversalPrimitive`s
  are primitives of *action*; nothing here maps them onto gates, and inventing that
  mapping would be exactly the kind of unfounded correspondence this project's
  standards exist to prevent.

## Why this was not committed

Eight tests failed, and every one of them was the agreement test between the
binary path and an independent explicit simulation. Committing a module whose
central claim is "this is exact" while its evidence says otherwise would be the
one failure mode this project is built to avoid: an assertion of precision with
nothing behind it. The finding is worth more than the code, so the code went and
the finding stayed.

The witness simulation is also worth keeping in mind as a design lesson. It had
its own bug — the X gate swapped each pair of amplitudes twice and so was a no-op
— and only the cross-check exposed it. Two implementations of the same function,
written from different representations, is what makes either of them trustworthy;
neither is trustworthy alone.
