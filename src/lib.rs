pub mod bridge;
pub mod clean;
pub mod doubt;
pub mod edit;
pub mod evolution;
pub mod fluid;
pub mod harvester;
pub mod identifiers;
pub mod learner;
pub mod link;
pub mod mcp;
pub mod meet;
pub mod meta_actuators;
pub mod node;
pub mod nucleus;
pub mod orchestrator;
/// The emulated OS layer: a syscall surface over the actuator nucleus, and
/// a virtual filesystem mapping paths to `.ure` identities.
///
/// **This module was not declared anywhere and was therefore unreachable.**
/// `src/os/` held about six kilobytes -- a kernel, a syscall enum and a VFS
/// -- and nothing in the crate included it. The only file that tried was
/// `src/wasm_core.rs`, which is gated to `wasm32` and imports
/// `crate::os::{UniaKernel, UniaSyscall}`; since it had never been compiled,
/// the dangling import was invisible. Declaring it revives the code and is
/// what makes the `wasm` CI job able to say anything at all.
///
/// It is tokio-free, which is the only reason it can be part of a `wasm`
/// build: the syscall names are emulated, and nothing here calls the host.
pub mod os;
pub mod pipeline;
pub mod primitives;
pub mod profiler;
pub mod quantum;
pub mod registry;
pub mod release_manager;
pub mod router;
pub mod slm;
pub mod transducer;
pub mod weights;
/// Macro-Fabric Integration Standards (WMIS).
///
/// The `wmis` namespace holds the resource accounting and permission layer that
/// sits underneath the actuator mesh: `WmisResource`, the token economy, and the
/// permission engine. WMIS is a project concept in its own right, distinct from
/// the crate name, so the namespace is kept rather than renamed. See
/// `docs/arch-decoupling-strategy.md` for how it relates to the nucleus.
pub mod wmis;

/// Generation of a synthetic `.ure` corpus.
///
/// The shipped corpus had twelve artifacts and zero shared capabilities, so
/// convergence had nothing to converge on and the escalation rate could not be
/// computed. This generates one that shares primitives on purpose, seeded so a
/// regression is a regression. It is a fiction, and every manifest says so.
pub mod corpus;

/// Evaluation of a manifest's declared `constraints`. This is the verifier: with
/// it absent, `constraints` was printed for operator visibility and never
/// checked, which left the learning loop with no reward signal. See
/// `docs/SPEC.md` §2.4 for the formalism this implements and §7 divergence D5.
pub mod constraints;

/// Where synthesised manifests are written. Kept out of the process working
/// directory so that running the tests does not change the corpus a cloner
/// sees. See `docs/SPEC.md` section 6.
pub mod outdir;

/// Induction of candidate actuators from recorded LLM interactions.
pub mod induce;

/// Degradation and self-collection. Retires artifacts by changing a lifecycle
/// field, never by deleting them.
pub mod gc;

/// ca(R)maduci: a digital pet whose care is recorded as traces. Pure logic, with
/// no clock and no I/O, so a terminal example, a browser client, and the tests
/// all agree on how time passes. See `docs/camaduci.md`.
pub mod camaduci;

/// Surface forms held beside identity rather than inside it. Maps a content
/// address to the phrasings known in each language, which is what lets two
/// creatures holding the same act in different vocabularies understand each
/// other.
pub mod lexicon;

/// Turning a session with a language model into evidence a creature can learn
/// from. Extracts corrections and confirmations; does not emit traces, because
/// the primitive vocabulary a trace needs is not yet defined. See D5 and D6.
pub mod session;

/// Gathering: continuous multidimensional interpenetration between knowledge
/// spheres, converging on common denominators. Ancestry is a relation on
/// content addresses, not a hash chain, so independent derivation of the same
/// capability is recorded as evidence instead of being forced into a tree.
pub mod gather;

// Wasm Entry Point
#[cfg(target_arch = "wasm32")]
pub mod wasm_core;
