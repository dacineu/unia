pub mod identifiers;
pub mod registry;
pub mod orchestrator;
pub mod router;
pub mod evolution;
pub mod slm;
pub mod weights;
pub mod profiler;
pub mod primitives;
pub mod harvester;
pub mod meta_actuators;
pub mod learner;
pub mod transducer;
pub mod bridge;
pub mod mcp;
pub mod pipeline;
pub mod release_manager;
pub mod fluid;
/// Macro-Fabric Integration Standards (WMIS).
///
/// The `wmis` namespace holds the resource accounting and permission layer that
/// sits underneath the actuator mesh: `WmisResource`, the token economy, and the
/// permission engine. WMIS is a project concept in its own right, distinct from
/// the crate name, so the namespace is kept rather than renamed. See
/// `docs/arch_decoupling_strategy.md` for how it relates to the nucleus.
pub mod wmis;
pub mod nucleus;
pub mod node;

// Wasm Entry Point
#[cfg(target_arch = "wasm32")]
pub mod wasm_core;
