//! Proof artifacts and verifier-facing public inputs.

use ark_bn254::Fr;

use crate::circuit::bool_to_field;
use crate::fetcher::Outcome;
use crate::aggregator::AggregationResult;

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

impl PublicInputs {
    /// Build public inputs from an aggregation result and agreement hash.
    pub fn from_aggregation(
        result: &AggregationResult,
        agreement_hash: [u8; 32],
        timestamp: u64,
    ) -> Self {
        Self {
            outcome: result.outcome == Outcome::Yes,
            source_count: u32::try_from(result.source_count).unwrap_or(u32::MAX),
            agreement_hash,
            timestamp,
        }
    }

    /// Values passed to Groth16 verify (must match circuit public inputs).
    pub fn to_verifier_inputs(&self) -> Vec<Fr> {
        vec![
            bool_to_field(self.outcome),
            Fr::from(u64::from(self.source_count)),
        ]
    }
}

/// Serialized Groth16 proof bytes plus metadata.
///
/// Proof bytes use arkworks uncompressed serialization for local dev;
/// production deployments should pin vk hash and proof format at the API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleProof {
    pub proof_bytes: Vec<u8>,
    pub generated_at: u64,
}
