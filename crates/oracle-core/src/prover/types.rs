//! Proof artifacts and verifier-facing public inputs.

use serde::{Deserialize, Serialize};

/// Values revealed to verifiers (on-chain or CLI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicInputs {
    /// `true` when aggregated outcome is Yes.
    pub outcome: bool,
    /// Number of sources included after outlier removal.
    pub source_count: u32,
    /// Blake3 commitment over included source hashes (first 32 bytes).
    pub agreement_hash: [u8; 32],
    /// Unix timestamp (seconds) for the resolution.
    pub timestamp: u64,
}

/// Serialized Groth16 proof bytes plus metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleProof {
    pub proof_bytes: Vec<u8>,
    pub generated_at: u64,
}
