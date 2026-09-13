use serde::{Serialize, Deserialize};

/// The dimensionality of the computation, determining the required hardware target.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Dimensionality {
    /// Standard scalar computation (CPU)
    Scalar,
    /// Parallel tensor computation (GPU/TPU)
    Tensor,
    /// Superposition/Entanglement computation (Quantum)
    Superposition,
}

/// The "Floating Assembly" - architecture-agnostic representations of logic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpaOp {
    /// Summation of inputs
    Suma { a: String, b: String },
    /// Multiplication of inputs
    Product { a: String, b: String },
    /// Data transformation (mapping/reduction)
    Transform { data: String, op: String },
    /// Quantum entanglement operation
    Entangle { target_1: String, target_2: String },
    /// State superposition
    Superposition { target: String },
}

/// A packet specialized for the United Processor Architecture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpaPacket {
    pub op: UpaOp,
    pub dim: Dimensionality,
    pub resource_id: String,
    pub request_id: String,
}
