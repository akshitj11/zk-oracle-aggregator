//! Chain calldata encoding tests.

use oracle_core::chain::{mock_proof_for_public_inputs, public_inputs_u256};
use oracle_core::prover::PublicInputs;

#[test]
fn mock_proof_matches_public_inputs() {
    let inputs = PublicInputs {
        outcome: true,
        source_count: 3,
        agreement_hash: [0u8; 32],
        timestamp: 1,
    };
    let public = public_inputs_u256(&inputs);
    let proof = mock_proof_for_public_inputs(&inputs);
    assert_eq!(proof.proof_c[0], public[0] ^ (public[1] << 1));
}
