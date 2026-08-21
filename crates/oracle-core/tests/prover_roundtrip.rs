//! Groth16 round-trip and tamper tests.

use std::collections::HashSet;

use ark_bn254::Fr;

use oracle_core::circuit::{agreement_hash, build_witness};
use oracle_core::fetcher::{Outcome, SourceResponse};
use oracle_core::prover::{OracleProver, PublicInputs};

fn response(id: &str, outcome: Outcome, confidence: f64) -> SourceResponse {
    SourceResponse {
        source_id: id.to_owned(),
        outcome,
        confidence,
        fetched_at: 1,
        raw_hash: {
            let mut hash = [0u8; 32];
            hash[0] = id.as_bytes()[0];
            hash
        },
    }
}

#[test]
fn prove_and_verify_round_trip() {
    let prover = OracleProver::generate_keys().expect("setup");
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.85),
        response("c", Outcome::No, 0.2),
    ];

    let (circuit, result) = build_witness(&responses).expect("witness");
    let filtered_ids: HashSet<_> = ["a", "b", "c"].into_iter().collect();
    let hash = agreement_hash(&responses, &filtered_ids);
    let public = PublicInputs::from_aggregation(&result, hash, 1_700_000_000);

    let proof = prover.prove(circuit).expect("prove");
    let ok = prover
        .verifier()
        .verify(&proof, &public.to_verifier_inputs())
        .expect("verify");
    assert!(ok);
}

#[test]
fn tampered_public_inputs_fail_verify() {
    let prover = OracleProver::generate_keys().expect("setup");
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.85),
    ];

    let (circuit, result) = build_witness(&responses).expect("witness");
    let filtered_ids: HashSet<_> = ["a", "b"].into_iter().collect();
    let hash = agreement_hash(&responses, &filtered_ids);
    let mut public = PublicInputs::from_aggregation(&result, hash, 1);

    let proof = prover.prove(circuit).expect("prove");
    public.outcome = false;
    let ok = prover
        .verifier()
        .verify(&proof, &public.to_verifier_inputs())
        .expect("verify");
    assert!(!ok);
}

#[test]
fn tampered_proof_bytes_fail_verify() {
    let prover = OracleProver::generate_keys().expect("setup");
    let responses = vec![response("a", Outcome::Yes, 0.9)];

    let (circuit, result) = build_witness(&responses).expect("witness");
    let filtered_ids: HashSet<_> = ["a"].into_iter().collect();
    let hash = agreement_hash(&responses, &filtered_ids);
    let public = PublicInputs::from_aggregation(&result, hash, 1);

    let mut proof = prover.prove(circuit).expect("prove");
    if let Some(byte) = proof.proof_bytes.get_mut(10) {
        *byte ^= 0xff;
    }

    let ok = prover
        .verifier()
        .verify(&proof, &public.to_verifier_inputs())
        .unwrap_or(false);
    assert!(!ok);
}

#[test]
fn wrong_source_count_public_input_fails() {
    let prover = OracleProver::generate_keys().expect("setup");
    let responses = vec![
        response("a", Outcome::Yes, 0.9),
        response("b", Outcome::Yes, 0.8),
    ];

    let (circuit, result) = build_witness(&responses).expect("witness");
    let filtered_ids: HashSet<_> = ["a", "b"].into_iter().collect();
    let hash = agreement_hash(&responses, &filtered_ids);
    let public = PublicInputs::from_aggregation(&result, hash, 1);

    let proof = prover.prove(circuit).expect("prove");
    let mut inputs = public.to_verifier_inputs();
    inputs[1] = Fr::from(99u64);

    let ok = prover.verifier().verify(&proof, &inputs).expect("verify");
    assert!(!ok);
}
