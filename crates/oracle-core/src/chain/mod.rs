//! Ethereum calldata helpers for on-chain settlement.

use crate::prover::PublicInputs;

/// Groth16 proof components for `PredictionMarketOracle.resolveMarket`.
#[derive(Debug, Clone)]
pub struct EthereumProof {
    pub proof_a: [u128; 2],
    pub proof_b: [[u128; 2]; 2],
    pub proof_c: [u128; 2],
}

/// Encode the two public field elements used by Groth16 verify on-chain.
pub fn public_inputs_u256(inputs: &PublicInputs) -> [u128; 2] {
    [
        u128::from(inputs.outcome),
        u128::from(inputs.source_count),
    ]
}

/// Build mock proof components for dev/test networks using the mock verifier scheme.
pub fn mock_proof_for_public_inputs(inputs: &PublicInputs) -> EthereumProof {
    let public = public_inputs_u256(inputs);
    let c0 = public[0] ^ (public[1] << 1);
    EthereumProof {
        proof_a: [0, 0],
        proof_b: [[0, 0], [0, 0]],
        proof_c: [c0, 0],
    }
}
