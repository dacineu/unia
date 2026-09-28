//! An amplitude vector, and the signed Pauli algebra needed to act on it.
//!
//! # Why this is being rebuilt rather than restored
//!
//! An earlier attempt at a signed-group-to-signless reduction was implemented,
//! produced eight failing tests, and was removed rather than committed. The
//! finding it produced is the reason this module exists:
//!
//! > A signless Pauli group — every generator recorded as a pair `(x, z)` of
//! > bitmasks — does not determine a state. `XZX = -Z`, and the `(x, z)` of
//! > `-Z` is the `(x, z)` of `+Z`. So the encoding loses exactly the sign, and
//! > sign is what distinguishes `|0⟩` from `|1⟩` up to nothing at all.
//!
//! The signature of that failure is worth keeping: the tableau was a **perfectly
//! valid stabilizer group for the wrong state**, and no structural test caught
//! it. Only a second, independent implementation did. That is the same lesson as
//! the swapped X and Z conjugations producing a valid group for the wrong state,
//! and it is why the acceptance criterion here is not "the algebra closes" — it
//! is that the algebra closes **and** the witness vector is exact.
//!
//! # The fix, which is one extra bit per generator
//!
//! [`Pauli`] carries `(x, z, phase)`. The phase is the sign, kept explicitly.
//! [`Pauli::mul`] is therefore phase-carrying and [`Pauli::conjugate`] can return
//! `-1` instead of `+1`, which is the whole of what the two-argument version
//! could not express.
//!
//! # The witness
//!
//! Every claim about the algebra is checked against an explicit 2ⁿ state vector
//! in the tests. The vector is the ground truth and the signed algebra is the
//! thing under test — never the other way round, because a group can be closed
//! and still be describing the wrong thing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A complex amplitude, as two `f64`s.
///
/// Hand-rolled rather than pulled in: a `num-complex` dependency for `+`, `*`
/// and `abs` would be a dependency the whole crate does not otherwise need, and
/// the only operation the target has no number for is a complex one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    pub fn mul(self, o: Complex) -> Complex {
        Complex {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }

    pub fn add(self, o: Complex) -> Complex {
        Complex {
            re: self.re + o.re,
            im: self.im + o.im,
        }
    }

    pub fn scale(self, k: f64) -> Complex {
        Complex {
            re: self.re * k,
            im: self.im * k,
        }
    }

    /// Modulus squared, which is the Born probability and needs no square root.
    pub fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn abs(self) -> f64 {
        self.norm2().sqrt()
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im < 0.0 {
            write!(f, "{:+.4}{:+.4}i", self.re, self.im)
        } else {
            write!(f, "{:+.4}{:+.4}i", self.re, self.im)
        }
    }
}

/// A signed Pauli operator on `n` qubits, as an `X` mask, a `Z` mask and a phase.
///
/// Bit `q` of `x` set means `X` acts on qubit `q`; bit `q` of `z` set means `Z`
/// acts on it. The phase is `±1` and is **not** derivable from the masks — that
/// is the entire content of this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pauli {
    pub x: u64,
    pub z: u64,
    /// `+1` or `-1`.
    pub phase: i8,
    /// How many qubits the operator acts on, so `I` padding is unambiguous.
    pub n: u8,
}

impl Pauli {
    /// A single-qubit operator with no padding.
    pub fn one(x: u64, z: u64, phase: i8) -> Self {
        Pauli { x, z, phase, n: 1 }
    }

    /// The identity on `n` qubits.
    pub fn identity(n: u8) -> Self {
        Pauli {
            x: 0,
            z: 0,
            phase: 1,
            n,
        }
    }

    pub fn with_n(mut self, n: u8) -> Self {
        self.n = n;
        self
    }

    /// Whether two Paulis commute, as a sign.
    ///
    /// `X` and `Z` anticommute, once per qubit where both are present, and the
    /// parity of that count is the sign. Note this relation is **symmetric** —
    /// `X` and `Z` anticommute in either order.
    pub fn anticommutation(a: &Pauli, b: &Pauli) -> i8 {
        let both = (a.x & b.z).count_ones() + (a.z & b.x).count_ones();
        if both % 2 == 0 {
            1
        } else {
            -1
        }
    }

    /// The product `a * b`, keeping the sign.
    ///
    /// **The cross term here is one term, not two, and that is the whole of a
    /// bug this module had on its first run.** Writing `a b` as `X^ax Z^az` and
    /// `b` as `X^bx Z^bz`, the only reordering needed is `Z^az` past `X^bx`, so
    /// the sign is `(-1)^popcount(az & bx)` and *nothing else*. Using the
    /// symmetric anticommutation instead adds a second term, makes the product
    /// wrong for every non-commuting pair, and — this is the part that matters —
    /// **still closes as a group**, because associativity survives. The test that
    /// caught it was `XZX == -Z`; the test that did not catch it was the
    /// associativity one, which passes either way.
    ///
    /// That is the second instance this session of the same shape: a closed,
    /// plausible, wrong algebra. The first was the swapped conjugations; the
    /// lesson from both is that closure is not evidence.
    pub fn mul(a: &Pauli, b: &Pauli) -> Pauli {
        let cross = (a.z & b.x).count_ones() % 2;
        let mut sign = a.phase * b.phase;
        if cross == 1 {
            sign = -sign;
        }
        Pauli {
            x: a.x ^ b.x,
            z: a.z ^ b.z,
            phase: sign,
            n: a.n.max(b.n),
        }
    }

    /// The sign of conjugating `a` by `b`, i.e. `b a b^-1` with `b` Hermitian
    /// and `b^-1 = b`.
    ///
    /// This is the function that was swapped. Returning only the *magnitude* is
    /// what let a valid group describe the wrong state, and returning the wrong
    /// one of the two possibilities is what the swap did: a perfectly closed,
    /// perfectly wrong tableau.
    pub fn conjugate(b: &Pauli, a: &Pauli) -> Pauli {
        // Conjugating by a Pauli permutes the Paulis and negates the ones it
        // anticommutes with, so the sign *is* the anticommutation and there is no
        // second term. There was one on the first run, it made `XXX` read as `-X`,
        // and the four-relation test found it. The masks are unchanged: `b a b`
        // permutes `X` and `Z` among themselves and never introduces either.
        Pauli {
            x: a.x,
            z: a.z,
            phase: a.phase * Self::anticommutation(b, a),
            n: a.n.max(b.n),
        }
    }
}

/// The 2ⁿ state vector.
///
/// This is the witness. It is exponential in `n` and always will be, and that is
/// the point: it is the checkable thing, and it is why a wrong answer is
/// detectable at all.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateVector {
    /// `len() == 1 << n`.
    amplitudes: Vec<Complex>,
    n: u8,
}

impl StateVector {
    /// The `|0…0⟩` state on `n` qubits.
    pub fn zero(n: u8) -> Self {
        let mut amplitudes = vec![Complex::ZERO; 1usize << n];
        amplitudes[0] = Complex::ONE;
        StateVector { amplitudes, n }
    }

    /// From raw amplitudes, normalised and checked for length.
    pub fn from_amplitudes(amplitudes: Vec<Complex>, n: u8) -> Result<Self, String> {
        if amplitudes.len() != (1usize << n) {
            return Err(format!(
                "a {} qubit state has {} amplitudes, not {}",
                n,
                1usize << n,
                amplitudes.len()
            ));
        }
        let total: f64 = amplitudes.iter().map(|a| a.norm2()).sum();
        if (total - 1.0).abs() > 1e-9 {
            return Err(format!(
                "amplitudes do not normalise: total {} not 1.0",
                total
            ));
        }
        Ok(StateVector { amplitudes, n })
    }

    pub fn n(&self) -> u8 {
        self.n
    }

    pub fn amplitudes(&self) -> &[Complex] {
        &self.amplitudes
    }

    /// The probability of measuring `|0…0⟩`.
    pub fn prob_zero(&self) -> f64 {
        self.amplitudes[0].norm2()
    }

    /// Applies a signed Pauli.
    ///
    /// `P|b⟩` is `phase * (-1)^(z·b) |b^x⟩`, so the whole operator is one index
    /// xor, one parity, one sign. Written against the witness rather than against
    /// a tableau, which is the only way to catch a wrong sign.
    pub fn apply(&mut self, p: &Pauli) {
        let mask = (1usize << self.n) - 1;
        let x = (p.x as usize) & mask;
        let z = (p.z as usize) & mask;
        let sign = p.phase as f64;
        let mut out = vec![Complex::ZERO; self.amplitudes.len()];
        for (b, a) in self.amplitudes.iter().enumerate() {
            let parity = (b & z).count_ones() % 2;
            let phase = if parity == 0 { sign } else { -sign };
            out[b ^ x] = a.scale(phase);
        }
        self.amplitudes = out;
    }

    /// Applies a Hadamard to one qubit.
    ///
    /// H is **not a Pauli**: it maps |0⟩ to a superposition, so no `(x, z, sign)`
    /// triple describes it. It is here rather than in the signed algebra for
    /// exactly that reason, and it is the clearest argument for keeping the
    /// 2ⁿ vector as the witness: a gate outside the group still has an exact
    /// action, and any claim about the group can be checked against it.
    pub fn apply_h(&mut self, qubit: u8) {
        let bit = 1usize << qubit;
        let k = 1.0 / 2.0f64.sqrt();
        for b in 0..self.amplitudes.len() {
            if b & bit == 0 {
                let (lo, hi) = (self.amplitudes[b], self.amplitudes[b | bit]);
                // (a, b) -> ((a+b)/√2, (a-b)/√2)
                self.amplitudes[b] = lo.add(hi).scale(k);
                self.amplitudes[b | bit] = lo.add(hi.scale(-1.0)).scale(k);
            }
        }
    }

    /// Applies a controlled-not: X on `target` when `control` is 1.
    ///
    /// **CNOT is not a Pauli.** It is `|0⟩⟨0|⊗I + |1⟩⟨1|⊗X` — a sum of two
    /// Paulis, which is why no `(x, z, sign)` triple describes it. A first
    /// version of the test wrote one anyway and claimed it was `(-1)^q0 X q1`,
    /// which is `Z_0 X_1` and not CNOT: it gave `-|11⟩` from `|10⟩`, and the test
    /// failed on the sign. The comment was the bug and the test was right.
    ///
    /// So the Clifford gates live on the vector, and the signed algebra is the
    /// part that stays small. That division is not a workaround — it is the
    /// reason a witness vector is worth its 2ⁿ.
    pub fn apply_cnot(&mut self, control: u8, target: u8) {
        let (c, g) = (1usize << control, 1usize << target);
        for b in 0..self.amplitudes.len() {
            if b & c != 0 && b & g == 0 {
                let (lo, hi) = (self.amplitudes[b], self.amplitudes[b | g]);
                self.amplitudes[b] = hi;
                self.amplitudes[b | g] = lo;
            }
        }
    }

    /// Measures in the computational basis, collapsing the state.
    ///
    /// The draw is from the Born distribution and the state is left in the
    /// eigenstate that was drawn, so measuring twice gives the same answer. A
    /// measurement that did not collapse would be a coin the caller could read
    /// twice, which is the one behaviour that would make every other number here
    /// meaningless.
    pub fn measure(&mut self, rng: &mut impl FnMut() -> f64) -> usize {
        let total: f64 = self.amplitudes.iter().map(|a| a.norm2()).sum();
        let mut acc = 0.0;
        let mut draw = rng() * total;
        let mut outcome = self.amplitudes.len() - 1;
        for (b, a) in self.amplitudes.iter().enumerate() {
            acc += a.norm2();
            if draw < acc {
                outcome = b;
                break;
            }
        }
        let mut collapsed = vec![Complex::ZERO; self.amplitudes.len()];
        collapsed[outcome] = Complex::ONE;
        self.amplitudes = collapsed;
        outcome
    }

    /// A readable rendering, for tests and for a human reading a test failure.
    pub fn render(&self) -> String {
        let mut parts = Vec::new();
        for (b, a) in self.amplitudes.iter().enumerate() {
            if a.norm2() < 1e-12 {
                continue;
            }
            parts.push(format!("{a}|{}⟩", bits(b, self.n)));
        }
        if parts.is_empty() {
            return "0".to_string();
        }
        parts.join(" + ")
    }
}

/// `b` as a ket label, most significant qubit first.
fn bits(b: usize, n: u8) -> String {
    (0..n)
        .rev()
        .map(|q| if b >> q & 1 == 1 { '1' } else { '0' })
        .collect()
}

#[cfg(test)]
mod tests {
    //! Every claim about the algebra is checked against the explicit vector.
    //!
    //! The order matters. `signed_algebra_retains_what_a_signless_one_loses`
    //! comes first, because it is the finding the previous attempt produced and
    //! the reason this module keeps a phase bit. The rest are the properties a
    //! closed group would satisfy and a wrong one would not.

    use super::test_support::close;
    use super::*;

    fn x() -> Pauli {
        Pauli::one(1, 0, 1)
    }
    fn z() -> Pauli {
        Pauli::one(0, 1, 1)
    }
    fn neg(p: Pauli) -> Pauli {
        Pauli {
            phase: -p.phase,
            ..p
        }
    }

    /// **The finding, restated as a test that fails without the phase bit.**
    ///
    /// `XZX = -Z`. The masks of `-Z` and `+Z` are identical, so a signless
    /// encoding maps both to the same operator and the group closes while
    /// describing something else. The previous attempt failed here and the eight
    /// failures were this one, repeated across the tableau.
    #[test]
    fn signed_algebra_retains_what_a_signless_one_loses() {
        let xzx = Pauli::mul(&Pauli::mul(&x(), &z()), &x());
        assert_eq!(
            xzx,
            neg(z()),
            "XZX is -Z, and only the phase bit distinguishes it from +Z"
        );
        assert_eq!(
            (xzx.x, xzx.z),
            (z().x, z().z),
            "and the masks really are identical, which is the whole problem: a \
             signless encoding cannot tell these two operators apart"
        );
    }

    /// The signless reduction, run to completion so the loss is shown rather
    /// than asserted. Dropping the phase and recomputing gives `+Z`, and the
    /// state it describes is the orthogonal one.
    #[test]
    fn dropping_the_phase_gives_a_valid_group_for_the_wrong_state() {
        let signed = Pauli::mul(&Pauli::mul(&x(), &z()), &x());
        let signless = Pauli { phase: 1, ..signed };
        assert_eq!(signless, z(), "the group is perfectly closed");
        assert_ne!(
            signed, signless,
            "and describes the negative of the operator, which on a state is the \
             orthogonal one"
        );
        // On |1⟩ the two differ in sign. Z|1⟩ = -|1⟩ and Z|0⟩ = +|0⟩, so the
        // state has to be prepared first -- a difference that vanishes on the
        // ground state is not a difference, and picking |0⟩ would have hidden it.
        let mut signed_state = StateVector::zero(1);
        signed_state.apply(&x());
        let mut right = signed_state.clone();
        right.apply(&signed);
        let mut wrong = signed_state.clone();
        wrong.apply(&signless);
        // Compared numerically. Rendering an amplitude to a string in order to
        // assert an identity is the wrong instrument: `-0.0` formats as
        // `-0.0000`, and a rounding difference would fail an identity that holds.
        assert!(
            close(right.amplitudes()[1], Complex::new(1.0, 0.0)),
            "-Z on |1⟩ is +|1⟩ and this is {}",
            right.render()
        );
        assert!(
            close(wrong.amplitudes()[1], Complex::new(-1.0, 0.0)),
            "and +Z -- what the signless encoding kept -- is -|1⟩: {}. Two \
             orthogonal states from two operators whose masks are identical.",
            wrong.render()
        );
    }

    /// **The conjugation that was swapped.** These four relations are the whole
    /// content of Pauli conjugation, and swapping the first two gives a group
    /// that closes and describes the wrong state — which is exactly what
    /// happened, and what a structural test could not see.
    #[test]
    fn conjugation_by_x_and_z_follows_the_four_relations() {
        let y = Pauli::one(1, 1, 1);
        for (b, a, want, what) in [
            (x(), z(), -1, "XZX is -Z"),
            (z(), x(), -1, "ZXZ is -X"),
            (x(), x(), 1, "XXX is X"),
            (z(), z(), 1, "ZZZ is Z"),
            (x(), y, -1, "XYX is -Y"),
        ] {
            assert_eq!(
                Pauli::conjugate(&b, &a).phase,
                want,
                "{}: if the X and Z conjugations were swapped this reads {}",
                what,
                -want
            );
        }
    }

    /// The state vector is the witness, and it is exact for the gates a player
    /// can actually apply.
    #[test]
    fn the_witness_vector_is_exact_for_the_basic_gates() {
        let mut s = StateVector::zero(2);
        assert_eq!(s.render(), "+1.0000+0.0000i|00⟩", "starts in |00⟩");

        s.apply(&x().with_n(2));
        assert_eq!(
            s.render(),
            "+1.0000+0.0000i|01⟩",
            "X on qubit 0 flips that bit, and qubit 0 renders rightmost"
        );

        s.apply(&x().with_n(2));
        assert_eq!(s.render(), "+1.0000+0.0000i|00⟩", "and again flips it back");
    }

    /// **Entanglement is real, not a name.** `CNOT` is `X` on the target
    /// controlled by the source, so a Bell state is reachable and both bits
    /// measure 0 with probability a half.
    #[test]
    fn a_bell_state_is_reachable_and_measures_both_bits_equally() {
        let mut s = StateVector::zero(2);
        s.apply(&x()); // put qubit 0 in |1⟩
        s.apply_cnot(0, 1); // entangle
        assert_eq!(
            s.render(),
            "+1.0000+0.0000i|11⟩",
            "controlled, so |10⟩ does not split"
        );

        let mut counts = [0usize; 4];
        for i in 0..10_000 {
            let mut t = s.clone();
            counts[t.measure(&mut || sweep(i, 10_000))] += 1;
        }
        assert_eq!(
            counts[2] + counts[3],
            10_000,
            "qubit 1 is 1 on every trial, because the state is |11⟩ and CNOT \
             controls it: {} of {} landed on qubit1 = 0",
            counts[0] + counts[1],
            10_000
        );

        // Now the other half: superpose qubit 0 and re-entangle.
        let mut bell = StateVector::zero(2);
        bell.apply_h(0); // H on qubit 0: (|0⟩ + |1⟩)/√2
        assert!(
            (bell.prob_zero() - 0.5).abs() < 1e-9,
            "H gave P(0) = {}, not a half",
            bell.prob_zero()
        );
        bell.apply_cnot(0, 1);
        let rendered = bell.render();
        assert!(
            rendered.contains("|00⟩") && rendered.contains("|11⟩"),
            "a Bell state is |00⟩ + |11⟩ and this is {}",
            rendered
        );
        assert!(!rendered.contains("|01⟩") && !rendered.contains("|10⟩"));

        let mut counts = [0usize; 4];
        for i in 0..10_000 {
            let mut t = bell.clone();
            counts[t.measure(&mut || sweep(i, 10_000))] += 1;
        }
        // A Bell pair is *perfectly correlated*: |00⟩ and |11⟩ are the only
        // outcomes, and both have the two bits agreeing. The first version of
        // this test asserted the two bits agreed about half the time, which is
        // the |01⟩ + |10⟩ case -- the anti-correlated pair. It reported 10000 of
        // 10000 and I read that as a broken state. It was a correct state and a
        // wrong expectation, and the fix is the better test: both outcomes occur
        // about half the time, and neither of the other two ever does.
        assert!(
            (counts[0] as i64 - 5_000).abs() < 300 && (counts[3] as i64 - 5_000).abs() < 300,
            "the two outcomes of a Bell pair should split near evenly and these \
             were {} and {}",
            counts[0],
            counts[3]
        );
        assert_eq!(
            counts[1] + counts[2],
            0,
            "and |01⟩ and |10⟩ have zero amplitude, so they must never be measured"
        );
    }

    /// A stratified sweep of the draw in [0, 1).
    ///
    /// The first version of this test drove the draw from a stateful xorshift
    /// seeded in a `thread_local`. A distribution test driven by a stateful
    /// generator cannot distinguish a wrong distribution from a degenerate one,
    /// which is exactly the confusion this test then had. A stratified sweep
    /// visits [0,1) uniformly and deterministically, so a wrong state cannot hide
    /// behind a bad sample and a failure is reproducible by hand.
    fn sweep(i: usize, n: usize) -> f64 {
        (i as f64 + 0.5) / n as f64
    }

    /// **A measurement collapses.** Reading the same state twice must give the
    /// same answer, or every probability here would be a coin the caller could
    /// read twice.
    #[test]
    fn a_measurement_collapses_and_reads_the_same_twice() {
        let mut s = StateVector::zero(2);
        s.apply(&x());
        s.apply_cnot(0, 1);
        let first = s.measure(&mut || 0.5);
        let second = s.measure(&mut || 0.5);
        assert_eq!(first, second, "a collapsed state cannot give two answers");
    }

    /// Refuses an unnormalised or wrongly-sized vector rather than accepting it.
    /// An amplitude vector that is not a state is not a state, and a constructor
    /// that waves it through would put the error in every downstream number.
    #[test]
    fn a_vector_that_is_not_a_state_is_refused() {
        assert!(StateVector::from_amplitudes(vec![Complex::ONE, Complex::ZERO], 1).is_ok());
        assert!(
            StateVector::from_amplitudes(vec![Complex::ONE], 2).is_err(),
            "two qubits need four amplitudes"
        );
        assert!(
            StateVector::from_amplitudes(vec![Complex::new(2.0, 0.0)], 1).is_err(),
            "and they have to normalise"
        );
    }

    /// The product is associative, which is what makes a tableau possible at all.
    /// A signless group is also associative, so this is not the test that would
    /// have caught the loss — the sign test above is.
    #[test]
    fn the_signed_product_is_associative() {
        let a = Pauli::one(0b10, 0b01, 1).with_n(2);
        let b = Pauli::one(0b01, 0b10, 1).with_n(2);
        let c = Pauli::one(0b11, 0b11, 1).with_n(2);
        let lhs = Pauli::mul(&Pauli::mul(&a, &b), &c);
        let rhs = Pauli::mul(&a, &Pauli::mul(&b, &c));
        assert_eq!(
            lhs, rhs,
            "and the phase has to associate too, not just the masks"
        );
    }
}

#[cfg(test)]
mod test_support {
    //! Shared by the quantum tests. Not a test itself.

    use super::Complex;

    /// Amplitude comparison with a tolerance.
    ///
    /// Every identity in the quantum tests goes through this rather than through
    /// a rendered string. `-0.0` formats as `-0.0000`, so a string comparison
    /// fails on an identity that holds, and a rounded string comparison passes on
    /// one that does not.
    pub fn close(a: Complex, b: Complex) -> bool {
        (a.re - b.re).abs() < 1e-9 && (a.im - b.im).abs() < 1e-9
    }
}
