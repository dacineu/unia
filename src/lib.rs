pub mod bridge;
pub mod evolution;
pub mod fluid;
pub mod harvester;
pub mod identifiers;
pub mod learner;
pub mod mcp;
pub mod meta_actuators;
pub mod node;
pub mod nucleus;
pub mod orchestrator;
pub mod pipeline;
pub mod primitives;
pub mod profiler;
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
/// `docs/arch_decoupling_strategy.md` for how it relates to the nucleus.
pub mod wmis;

/// Induction of candidate actuators from recorded LLM interactions.
pub mod induce;

/// Degradation and self-collection. Retires artifacts by changing a lifecycle
/// field, never by deleting them.
pub mod gc;

/// Gathering: continuous multidimensional interpenetration between knowledge
/// spheres, converging on common denominators. Ancestry is a relation on
/// content addresses, not a hash chain, so independent derivation of the same
/// capability is recorded as evidence instead of being forced into a tree.
pub mod gather;

// Wasm Entry Point
#[cfg(target_arch = "wasm32")]
pub mod wasm_core;
